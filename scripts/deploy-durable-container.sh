#!/usr/bin/env bash
set -euo pipefail

slug=${1:-alert-evidence-envelope}
repo=${2:-/work/repo}
dockerfile=${3:-Dockerfile}
port=${4:-8080}
resource_group=${AZURE_RESOURCE_GROUP:-sociobot}
registry=${AZURE_CONTAINER_REGISTRY:-sociobotregistry}
data_dir=${WO_DATA_DIR:-/data}
storage_name=${DURABLE_STORAGE_NAME:-alert-evidence-envelope-data}

if [ "$slug" != "alert-evidence-envelope" ]; then
  echo "ERROR: this deploy helper is scoped only to alert-evidence-envelope" >&2
  exit 2
fi
if [ "$data_dir" != "/data" ]; then
  echo "ERROR: deploy.data_dir must be /data" >&2
  exit 2
fi

app_name="sf-$slug"
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

# Storage provisioning belongs to the work-order runner. This script updates
# only its scoped Container App and attaches the registered product storage.
app=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
app_id=$(jq -er '.id' <<<"$app")
template=$(jq --arg storage "$storage_name" --arg image "$image" '
  .properties.template
  | .revisionSuffix = null
  | .scale = {minReplicas: 1, maxReplicas: 1}
  | .volumes = [{name: "envelope-data", storageType: "AzureFile", storageName: $storage}]
  | .containers |= map(
      if .name == "app" then
        .image = $image
        | .volumeMounts = [{volumeName: "envelope-data", mountPath: "/data"}]
      else . end
    )
' <<<"$app")
payload=$(jq -n --argjson template "$template" \
  '{properties:{configuration:{activeRevisionsMode:"Single"},template:$template}}')

# SQLite uses the no-lock VFS on the single-replica Azure Files mount. Drain
# every old product revision before starting the new one so two processes can
# never write the database during a rolling replacement.
while IFS= read -r active_revision; do
  [ -z "$active_revision" ] && continue
  az containerapp revision deactivate --resource-group "$resource_group" \
    --name "$app_name" --revision "$active_revision" --output none
done < <(az containerapp revision list --resource-group "$resource_group" \
  --name "$app_name" --query '[?properties.active].name' --output tsv)

az rest --method patch \
  --url "https://management.azure.com${app_id}?api-version=2024-03-01" \
  --body "$payload" --output none

effective=''
for _ in $(seq 1 "${DEPLOY_VERIFY_ATTEMPTS:-30}"); do
  effective=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
  if jq -e --arg storage "$storage_name" --arg image "$image" '
    .properties.latestRevisionName == .properties.latestReadyRevisionName
    and .properties.configuration.activeRevisionsMode == "Single"
    and .properties.template.scale.minReplicas == 1
    and .properties.template.scale.maxReplicas == 1
    and any(.properties.template.volumes[]?; .name == "envelope-data" and .storageType == "AzureFile" and .storageName == $storage)
    and any(.properties.template.containers[]?; .name == "app" and .image == $image and any(.volumeMounts[]?; .volumeName == "envelope-data" and .mountPath == "/data"))
  ' >/dev/null <<<"$effective"; then
    break
  fi
  sleep "${DEPLOY_VERIFY_INTERVAL_SECONDS:-10}"
done

if ! jq -e --arg storage "$storage_name" --arg image "$image" '
  .properties.latestRevisionName == .properties.latestReadyRevisionName
  and .properties.configuration.activeRevisionsMode == "Single"
  and .properties.template.scale.minReplicas == 1
  and .properties.template.scale.maxReplicas == 1
  and any(.properties.template.volumes[]?; .storageName == $storage)
  and any(.properties.template.containers[]?; .name == "app" and .image == $image and any(.volumeMounts[]?; .mountPath == "/data"))
' >/dev/null <<<"$effective"; then
  echo "ERROR: deployment did not reach one ready replica with durable /data" >&2
  exit 1
fi

latest_revision=$(jq -r '.properties.latestRevisionName' <<<"$effective")
while IFS= read -r stale_revision; do
  [ -z "$stale_revision" ] && continue
  [ "$stale_revision" = "$latest_revision" ] && continue
  az containerapp revision deactivate --resource-group "$resource_group" \
    --name "$app_name" --revision "$stale_revision" --output none || true
done < <(az containerapp revision list --resource-group "$resource_group" \
  --name "$app_name" --query '[?properties.active].name' --output tsv)

"$repo/scripts/verify-live-topology.sh" \
  "https://$slug.sociobot.in" "$app_name" "$resource_group" "$source_sha" "$storage_name"
