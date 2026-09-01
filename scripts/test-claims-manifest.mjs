import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const claims = JSON.parse(readFileSync(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
const taggedSources = [
  new URL('../src/lib.rs', import.meta.url),
  new URL('../tests/browser/app.spec.ts', import.meta.url),
  new URL('./test-deployment-policy.mjs', import.meta.url),
];

const manifestIds = claims.map((claim) => claim.id).sort();
assert.equal(new Set(manifestIds).size, manifestIds.length, 'claim IDs must be unique');

const tags = taggedSources.flatMap((file) =>
  [...readFileSync(file, 'utf8').matchAll(/@claim:([a-z0-9-]+)/g)].map((match) => match[1]),
).sort();
assert.deepEqual(tags, manifestIds, 'every public claim must have exactly one tagged regression test');

for (const claim of claims) {
  assert.equal(typeof claim.claim, 'string');
  assert.ok(claim.claim.length > 0, `${claim.id} needs public copy`);
  assert.equal(typeof claim.test, 'string');
  assert.ok(claim.test.length > 0, `${claim.id} needs a repeatable test command`);
  assert.equal(typeof claim.sandbox, 'string');
  assert.ok(claim.sandbox.length > 0, `${claim.id} needs a sandbox description`);
}

console.log(`PASS claims manifest: ${claims.length} claims each have one tagged regression test`);
