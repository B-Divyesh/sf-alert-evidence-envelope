# Alert Evidence Envelope — repair handoff

## Release status: deployed

The repair runtime is live at `https://alert-evidence-envelope.sociobot.in`.
Its immutable backend identity is:

```json
{"build":"4fc1ed28934163703d2fde081bafe25aeb719705","status":"ok"}
```

This is the source commit built into and deployed as
`sociobotregistry.azurecr.io/sf-alert-evidence-envelope:4fc1ed289341`. Commit
`ebb3d218d5d08ec3406bbfb5552d1f8165847e51` follows with Docker-context
hygiene only; it excludes local artifacts from future ACR submissions and does
not change the shipped application.

## What was repaired

- The build-provenance regression described by `verification-2.md` was
  reproduced from its evidence: the prior live `/health` had an empty build
  field. The release build now receives `BUILD_SHA` and the live service
  returns the exact non-empty immutable SHA above.
- The backend no longer falls back to a known development HMAC key. With only
  `PORT`, first boot CSPRNG-generates a 256-bit key, persists it as a mode-600
  file under `/data`, and logs only whether it was generated, persisted, or
  supplied. `ENVELOPE_SIGNING_KEY` remains an optional 32-byte-minimum
  override.
- Added regression coverage for generated/persisted/overridden signing keys,
  rejected short keys, and the compiled health identity. The browser identity
  assertion now uses the configured test identity rather than a stale commit.
- The legal-document test checks the actual static HTTP bodies without
  creating a second Chromium context; this preserves no-JavaScript coverage
  and avoids the runner-only headless Chromium segfault observed during the
  targeted repair.
- Added `.dockerignore` entries for local build/test artifacts. The clean
  deployment worktree was 3.5 MB rather than carrying the local `target/` and
  dependency directories into future ACR submissions.

## Verification evidence

All commands were run from a clean dependency install for this repair:

```sh
npm ci
npm test
npx playwright test --reporter=line
cargo clippy --all-targets --locked -- -D warnings
BUILD_SHA=4fc1ed28934163703d2fde081bafe25aeb719705 cargo build --release --locked
```

- `npm test` passed: Svelte check had 0 diagnostics; Rust unit/integration
  suite passed 7/7; production Vite build passed; Playwright passed 16/16.
- Independent browser rerun passed 16/16 across desktop Chromium and the
  390 × 844 mobile project, including preview/redaction, keyboard skip link,
  light/dark Axe serious/critical checks, legal no-JS HTTP bodies, mobile
  width, service-worker offline shell, and updateability.
- `cargo clippy --all-targets --locked -- -D warnings` and `git diff --check`
  passed. The production frontend is 63.16 KB JS raw (24.55 KB gzip) and
  16.30 KB CSS raw (4.72 KB gzip), within the stated budgets.
- A release binary compiled with the deployed SHA was started in an empty
  temporary working directory with only `PORT=4181`. It created a 32-byte,
  mode-600 key, logged `signing_key_source="generated"`, returned the exact
  SHA from `/health`, and completed 200 concurrent health requests at
  concurrency 10.
- Factory container deployment used the configured Azure Container Apps path
  and the prebuilt immutable ACR image after the shared ACR queue completed.
  The public `verify-url.sh` check passed: HTTPS 200, 652 ms load, no console
  errors, title/lang/one h1/main present, all images have alt attributes, and
  all buttons are labelled.
- Fresh public checks: `/privacy` = 200, `/terms` = 200; `/` has `no-cache`,
  CSP, `nosniff`, and `no-referrer` headers; public `/health` returns the
  immutable runtime identity shown above.

## Run and operate

```sh
npm ci
npm test
npm run build
PORT=8080 cargo run
```

For a production image, build with `--build-arg BUILD_SHA=<40-character
commit>`. The container needs only `PORT`; mount `/data` so generated signing
keys and SQLite metadata survive restarts. See `README.md` for endpoint and
configuration details.

## Known gaps / next steps

No release-blocking gaps remain. The product continues to store only channel
configuration and delivery metadata, not inbound alert bodies or evidence.
