#!/usr/bin/env bash
set -euo pipefail

slug=${1:-alert-evidence-envelope}
repo=${2:-/work/repo}
dockerfile=${3:-Dockerfile}
port=${4:-8080}
resource_group=${AZURE_RESOURCE_GROUP:-sociobot}
environment=${AZURE_CONTAINERAPP_ENV:-factory-env}
storage_account=${AZURE_STORAGE_ACCOUNT:-sociobotblob}
registry=${AZURE_CONTAINER_REGISTRY:-sociobotregistry}
fleet_helper=${FLEET_DEPLOY_CONTAINER_HELPER:-/opt/fleet/lib/deploy-container.sh}

app_name="sf-$slug"
if [ ${#app_name} -gt 32 ]; then
  app_name="sf-${slug:0:22}-$(printf '%s' "$slug" | sha1sum | cut -c1-6)"
  app_name=${app_name//--/-}
fi
storage_name="${slug}-data"
share_name="sf-${slug}-data"
source_sha=$(git -C "$repo" rev-parse HEAD)
image="$registry.azurecr.io/$app_name:${source_sha:0:12}"

if [ -z "${PREBUILT_IMAGE:-}" ]; then
  echo "== acr build $app_name:${source_sha:0:12}"
  az acr build --registry "$registry" --image "$app_name:${source_sha:0:12}" \
    --file "$dockerfile" --build-arg "BUILD_SHA=$source_sha" \
    --build-arg "GIT_SHA=$source_sha" --build-arg "SOURCE_COMMIT=$source_sha" \
    "$repo" 2>&1 | tail -3
else
  image=$PREBUILT_IMAGE
  echo "== using prebuilt image $image"
fi

storage_key=$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' --output tsv)
az storage share create --name "$share_name" --account-name "$storage_account" --account-key "$storage_key" --output none
if storage_config=$(az containerapp env storage show --resource-group "$resource_group" --name "$environment" --storage-name "$storage_name" --output json 2>/dev/null); then
  existing_share=$(jq -r '.properties.azureFile.shareName' <<<"$storage_config")
  [ "$existing_share" = "$share_name" ] || { echo "ERROR: $storage_name points to $existing_share, expected $share_name" >&2; exit 1; }
else
  az containerapp env storage set --resource-group "$resource_group" --name "$environment" \
    --storage-name "$storage_name" --access-mode ReadWrite \
    --azure-file-account-name "$storage_account" --azure-file-share-name "$share_name" \
    --azure-file-account-key "$storage_key" --output none
fi

if ! az containerapp show --resource-group "$resource_group" --name "$app_name" --output none 2>/dev/null; then
  "$fleet_helper" "$slug" "$repo" "$dockerfile" "$port" "$image"
fi

app=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
template=$(jq --arg storage "$storage_name" --arg image "$image" '
  .properties.template
  | .revisionSuffix = null
  | .scale = {minReplicas: 1, maxReplicas: 1}
  | .volumes = [{name: "envelope-data", storageType: "AzureFile", storageName: $storage}]
  | .containers |= map(if .name == "app" then .image = $image | .volumeMounts = [{volumeName: "envelope-data", mountPath: "/data"}] else . end)
' <<<"$app")
payload=$(jq -n --argjson template "$template" '{properties:{configuration:{activeRevisionsMode:"Single"},template:$template}}')
subscription=$(az account show --query id --output tsv)
az rest --method patch \
  --url "https://management.azure.com/subscriptions/$subscription/resourceGroups/$resource_group/providers/Microsoft.App/containerApps/$app_name?api-version=2024-03-01" \
  --body "$payload" --output none

for _ in $(seq 1 "${DEPLOY_VERIFY_ATTEMPTS:-30}"); do
  effective=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
  if jq -e --arg storage "$storage_name" --arg image "$image" '
    .properties.latestRevisionName == .properties.latestReadyRevisionName
    and .properties.template.scale.minReplicas == 1
    and .properties.template.scale.maxReplicas == 1
    and any(.properties.template.volumes[]?; .name == "envelope-data" and .storageType == "AzureFile" and .storageName == $storage)
    and any(.properties.template.containers[]?; .name == "app" and any(.volumeMounts[]?; .volumeName == "envelope-data" and .mountPath == "/data"))
    and any(.properties.template.containers[]?; .name == "app" and .image == $image)
  ' >/dev/null <<<"$effective"; then
    break
  fi
  sleep "${DEPLOY_VERIFY_INTERVAL_SECONDS:-10}"
done

if ! jq -e --arg storage "$storage_name" --arg image "$image" '
  .properties.latestRevisionName == .properties.latestReadyRevisionName
  and .properties.template.scale.minReplicas == 1
  and .properties.template.scale.maxReplicas == 1
  and any(.properties.template.volumes[]?; .storageName == $storage)
  and any(.properties.template.containers[]?; .name == "app" and .image == $image)
' >/dev/null <<<"$effective"; then
  echo "ERROR: deployment did not reach one ready replica with durable /data" >&2
  exit 1
fi

latest_revision=$(jq -r '.properties.latestRevisionName' <<<"$effective")
while IFS= read -r stale_revision; do
  [ -z "$stale_revision" ] && continue
  [ "$stale_revision" = "$latest_revision" ] && continue
  az containerapp revision deactivate --resource-group "$resource_group" --name "$app_name" --revision "$stale_revision" --output none || true
done < <(az containerapp revision list --resource-group "$resource_group" --name "$app_name" --query '[?properties.active].name' --output tsv)

"$repo/scripts/verify-live-topology.sh" \
  "https://$slug.sociobot.in" "$app_name" "$resource_group" "$(git -C "$repo" rev-parse HEAD)" "$storage_name" "$share_name"
