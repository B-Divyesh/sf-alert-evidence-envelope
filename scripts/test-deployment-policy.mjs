import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const deploy = readFileSync(new URL('./deploy-durable-container.sh', import.meta.url), 'utf8');
const dockerfile = readFileSync(new URL('../Dockerfile', import.meta.url), 'utf8');

assert.match(deploy, /minReplicas: 1, maxReplicas: 1/);
assert.match(deploy, /storageType: "AzureFile"/);
assert.match(deploy, /mountPath: "\/data"/);
assert.match(deploy, /verify-live-topology\.sh/);
assert.match(dockerfile, /USER envelope/);
assert.match(dockerfile, /DATABASE_URL=sqlite:\/data\/envelopes\.db\?mode=rwc/);
console.log('PASS @claim:durable-deployment: one replica, Azure File /data mount, non-root runtime');
