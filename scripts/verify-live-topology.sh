#!/usr/bin/env bash
set -euo pipefail

base_url=${1:-https://alert-evidence-envelope.sociobot.in}
app_name=${2:-sf-alert-evidence-envelope}
resource_group=${3:-sociobot}
expected_sha=${4:-}
storage_name=${5:-alert-evidence-envelope-data}
share_name=${6:-sf-alert-evidence-envelope-data}
environment=${AZURE_CONTAINERAPP_ENV:-factory-env}

fail() { echo "ERROR: live topology check failed: $*" >&2; exit 1; }

topology=$(az containerapp show --resource-group "$resource_group" --name "$app_name" --output json)
revision=$(jq -er '.properties.latestRevisionName' <<<"$topology")
ready_revision=$(jq -er '.properties.latestReadyRevisionName' <<<"$topology")
[ "$revision" = "$ready_revision" ] || fail "latest revision is not ready"
jq -e --arg storage "$storage_name" '
  .properties.template.scale == {minReplicas: 1, maxReplicas: 1}
  and any(.properties.template.volumes[]?; .name == "envelope-data" and .storageType == "AzureFile" and .storageName == $storage)
  and any(.properties.template.containers[]?; .name == "app" and any(.volumeMounts[]?; .volumeName == "envelope-data" and .mountPath == "/data"))
' >/dev/null <<<"$topology" || fail "expected one replica with the Azure File volume mounted at /data"

image=$(jq -er '.properties.template.containers[] | select(.name == "app") | .image' <<<"$topology")
[ -z "$expected_sha" ] || [[ "$image" == *":${expected_sha:0:12}" ]] || fail "image does not match $expected_sha"

storage=$(az containerapp env storage show --resource-group "$resource_group" --name "$environment" --storage-name "$storage_name" --output json)
jq -e --arg share "$share_name" '.properties.azureFile.accessMode == "ReadWrite" and .properties.azureFile.shareName == $share' >/dev/null <<<"$storage" || fail "Azure File registration is incorrect"

revisions=$(az containerapp revision list --resource-group "$resource_group" --name "$app_name" --output json)
[ "$(jq '[.[] | select(.properties.active == true)] | length' <<<"$revisions")" = 1 ] || fail "expected one active revision"
replicas=$(az containerapp replica list --resource-group "$resource_group" --name "$app_name" --revision "$revision" --output json)
[ "$(jq '[.[] | select(.properties.runningState == "Running")] | length' <<<"$replicas")" = 1 ] || fail "expected one running replica"

health=$(curl --fail --silent --show-error --no-keepalive --header 'Cache-Control: no-cache' "$base_url/health")
live_sha=$(jq -er '.build' <<<"$health")
[ -z "$expected_sha" ] || [ "$live_sha" = "$expected_sha" ] || fail "live build $live_sha does not match $expected_sha"

jq -n --arg result PASS --arg build "$live_sha" --arg revision "$revision" --arg image "$image" --arg storage "$storage_name" --arg share "$share_name" \
  '{result:$result,build:$build,revision:$revision,image:$image,topology:{minReplicas:1,maxReplicas:1,runningReplicas:1,mountPath:"/data",storage:$storage,share:$share}}'

