import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const deploy = readFileSync(new URL('./deploy-durable-container.sh', import.meta.url), 'utf8');
const dockerfile = readFileSync(new URL('../Dockerfile', import.meta.url), 'utf8');

assert.match(deploy, /minReplicas: 1, maxReplicas: 1/);
assert.match(deploy, /scale\.minReplicas == 1/);
assert.match(deploy, /scale\.maxReplicas == 1/);
assert.match(deploy, /activeRevisionsMode:"Single"/);
assert.match(deploy, /data_dir=\$\{WO_DATA_DIR:-\/data\}/);
assert.match(deploy, /deploy\.data_dir must be \/data/);
assert.match(deploy, /\.revisionSuffix = null/);
assert.match(deploy, /\.image == \$image/);
assert.match(deploy, /storageType: "AzureFile"/);
assert.match(deploy, /mountPath: "\/data"/);
assert.match(deploy, /verify-live-topology\.sh/);
assert.doesNotMatch(deploy, /storage account keys list|az storage share create|containerapp env storage show/);
const drain = deploy.indexOf('revision deactivate');
const replace = deploy.indexOf('az rest --method patch');
assert.ok(drain >= 0 && replace >= 0 && drain < replace, 'old revisions must drain before replacement');
assert.match(dockerfile, /USER envelope/);
assert.match(dockerfile, /DATA_DIR=\/data/);
assert.match(dockerfile, /DATABASE_URL=sqlite:\/data\/envelopes\.db\?mode=rwc/);
assert.doesNotMatch(dockerfile, /DATABASE_SNAPSHOT_FILE|sqlite:\/tmp\/envelopes\.db/);
assert.match(dockerfile, /COPY frontend\/static\/404\.html \.\/frontend\/static\/404\.html/);
console.log('PASS @claim:durable-deployment: one replica, durable SQLite at /data, non-root runtime');
