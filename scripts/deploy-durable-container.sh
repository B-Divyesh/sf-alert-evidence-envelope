#!/usr/bin/env bash
set -euo pipefail

slug=${1:-alert-evidence-envelope}
repo=${2:-/work/repo}
dockerfile=${3:-Dockerfile}
port=${4:-8080}
resource_group=${AZURE_RESOURCE_GROUP:-sociobot}
environment=${AZURE_CONTAINERAPP_ENV:-factory-env}
storage_account=${AZURE_STORAGE_ACCOUNT:-sociobotblob}
fleet_helper=${FLEET_DEPLOY_CONTAINER_HELPER:-/opt/fleet/lib/deploy-container.sh}

app_name="sf-$slug"
if [ ${#app_name} -gt 32 ]; then
  app_name="sf-${slug:0:22}-$(printf '%s' "$slug" | sha1sum | cut -c1-6)"
  app_name=${app_name//--/-}
fi
storage_name="${slug}-data"
share_name="sf-${slug}-data"

"$fleet_helper" "$slug" "$repo" "$dockerfile" "$port"

storage_key=$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' --output tsv)
az storage share create --name "$share_name" --account-name "$storage_account" --account-key "$storage_key" --output none
az containerapp env storage set --resource-group "$resource_group" --name "$environment" \
  --storage-name "$storage_name" --access-mode ReadWrite \
  --azure-file-account-name "$storage_account" --azure-file-share-name "$share_name" \
  --azure-file-account-key "$storage_key" --output none

app=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
template=$(jq --arg storage "$storage_name" '
  .properties.template
  | .scale = {minReplicas: 1, maxReplicas: 1}
  | .volumes = [{name: "envelope-data", storageType: "AzureFile", storageName: $storage}]
  | .containers |= map(if .name == "app" then .volumeMounts = [{volumeName: "envelope-data", mountPath: "/data"}] else . end)
' <<<"$app")
payload=$(jq -n --argjson template "$template" '{properties:{template:$template}}')
subscription=$(az account show --query id --output tsv)
az rest --method patch \
  --url "https://management.azure.com/subscriptions/$subscription/resourceGroups/$resource_group/providers/Microsoft.App/containerApps/$app_name?api-version=2024-03-01" \
  --body "$payload" --output none

for _ in $(seq 1 "${DEPLOY_VERIFY_ATTEMPTS:-30}"); do
  effective=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
  if jq -e --arg storage "$storage_name" '
    .properties.latestRevisionName == .properties.latestReadyRevisionName
    and .properties.template.scale == {minReplicas: 1, maxReplicas: 1}
    and any(.properties.template.volumes[]?; .name == "envelope-data" and .storageType == "AzureFile" and .storageName == $storage)
    and any(.properties.template.containers[]?; .name == "app" and any(.volumeMounts[]?; .volumeName == "envelope-data" and .mountPath == "/data"))
  ' >/dev/null <<<"$effective"; then
    break
  fi
  sleep "${DEPLOY_VERIFY_INTERVAL_SECONDS:-10}"
done

if ! jq -e --arg storage "$storage_name" '
  .properties.latestRevisionName == .properties.latestReadyRevisionName
  and .properties.template.scale == {minReplicas: 1, maxReplicas: 1}
  and any(.properties.template.volumes[]?; .storageName == $storage)
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

