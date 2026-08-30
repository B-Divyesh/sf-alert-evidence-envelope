import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const deploy = readFileSync(new URL('./deploy-durable-container.sh', import.meta.url), 'utf8');
const dockerfile = readFileSync(new URL('../Dockerfile', import.meta.url), 'utf8');

assert.match(deploy, /minReplicas: 1, maxReplicas: 1/);
assert.match(deploy, /scale\.minReplicas == 1/);
assert.match(deploy, /scale\.maxReplicas == 1/);
assert.match(deploy, /activeRevisionsMode:\"Single\"/);
assert.match(deploy, /\.revisionSuffix = null/);
assert.match(deploy, /storageType: "AzureFile"/);
assert.match(deploy, /mountPath: "\/data"/);
assert.match(deploy, /verify-live-topology\.sh/);
assert.match(dockerfile, /USER envelope/);
assert.match(dockerfile, /DATABASE_URL=sqlite:\/tmp\/envelopes\.db\?mode=rwc/);
assert.match(dockerfile, /DATABASE_SNAPSHOT_FILE=\/data\/envelopes\.snapshot\.db/);
assert.match(dockerfile, /COPY frontend\/static\/404\.html \.\/frontend\/static\/404\.html/);
console.log('PASS @claim:durable-deployment: one replica, Azure File /data mount, non-root runtime');
