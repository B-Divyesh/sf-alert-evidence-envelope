# Alert Evidence Envelope — repair handoff

Date: 2026-08-30

Work order: `alert-evidence-envelope-repair-6`

Source finding: `.factory/verification-5.md` at report commit `9e2990c931e8dcfb16901e1e834ec0b8ac4fee8d`, verifying candidate `10fd47b26b6de165b432420838c9f4300b5c07c1`.

## Outcome

The release-blocking product defects are repaired. Demo session IDs and expiry times now live in SQLite beside channel state. The production image uses `/data/envelopes.db` directly, and the deployment policy requires one replica with `/data` mounted. A legacy `/data/envelopes.snapshot.db` is copied once when the direct database does not yet exist.

Azure Files rejected SQLite's POSIX byte-range locking during the first repaired activation. The final runtime selects SQLite's `unix-none` VFS only for `/data`, keeps one database connection, and the deploy helper drains old revisions before replacement. Local SQLite paths retain normal locking. This combination enforces one writer without moving state outside the required durable SQLite file.

The checkout endpoint’s shared pilot HTTP 500 is environment-gated per the controller. Product coverage now verifies the displayed $39 one-time price and official Sociobot checkout URL without making shared checkout availability a product gate.

## Reproduction and regression

Before the repair, `cargo test demo_create_then_preview_survives_a_fresh_database_connection --locked` reproduced the live failure: create returned 200, then preview through a newly opened state/database connection returned 404 instead of 200.

That exact regression now passes. The test creates a session, closes the first SQLite pool, opens a fresh state against the same database, and successfully previews the sample. A separate port-only runtime smoke repeated create, process stop, process restart, and preview successfully. The three generated credential files remained mode 600.

Additional claim coverage now proves:

- the demo session expiry is 24 hours and expired rows are rejected and removed;
- the configured upstream receives only `q` and `limit`, even when input contains an attacker-controlled source URL;
- HMAC-SHA256 values are recomputed and verified for generated and forwarded envelopes;
- valid Field Kit users can save, reload, and apply named presets from browser-only storage;
- the demo remains outside protected delivery history and raw evidence remains absent from SQLite.

## Local verification

- `npm ci`: passed; 56 packages installed, 0 vulnerabilities.
- `npm test`: passed; Svelte reported 0 errors/warnings, Rust passed 18 tests, deployment policy passed, Vite built `dist/`, and all 38 desktop/390 px browser cases passed without a retry.
- Every command in `.factory/claims.json`: passed independently, including the live `durable-deployment` command.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `BUILD_SHA=repair-local cargo build --release --locked`: passed.
- Port-only runtime: started with an empty environment except `PATH` and `PORT`; generated credentials, returned `repair-local` from `/health`, and recovered them after restart.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4180 ...`: passed with title, `lang=en`, one h1, main, alt text, labelled buttons, and zero console/page errors.
- Playwright axe: zero serious/critical findings on product, demo, privacy, and terms in light/dark desktop and 390 px states.
- Keyboard, focus, 44 px targets, 200% text, reduced motion, offline reload/update, same-origin demo privacy, security/cache headers, 404, and rate-limit response policy: passed in the browser suite.
- Live mobile Lighthouse, three runs: performance 98/98/98, accessibility 100/100/100, best practices 100/100/100, SEO 100/100/100. LCP was 2,250/2,130/2,250 ms, TBT 82/62/58 ms, and CLS 0/0/0.
- Production assets: initial JS 66,327 bytes raw / 25,220 gzip; CSS 17,557 bytes raw / 4,970 gzip; fonts 115,560 bytes total; mobile hero 40,982 bytes.
- Docker is unavailable in this worker. The deployment uses the Dockerfile through ACR Build.

## Deployment evidence

- Verified repair build: `39cac7f36811a8b68cc005a1f7b19535e1f29aff`.
- Verified revision: `sf-alert-evidence-envelope--0000017`.
- Verified image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:39cac7f36811`.
- Topology: single revision; min 1; max 1; one running replica; `alert-evidence-envelope-data` mounted at `/data`.
- Durability: 20/20 live session-create requests were followed by successful previews over fresh HTTP connections.
- `/opt/fleet/lib/verify-url.sh`: HTTPS 200, 588 ms load, one h1/main, complete alt/button labels, and zero console errors.
- Live browser: first-click demo passed at 1440 px and 390 px with no overflow or console errors; all demo requests were same-origin.
- Live axe: zero serious/critical findings for demo on desktop/mobile and for home, privacy, and terms in light/dark at 390 px.
- Live offline/update, security headers, cache policy, 404 route, HTTP-to-HTTPS redirect, and immutable health identity passed.
- Live rate limit: 60 concurrent requests from one forwarded IP returned 42 x 401 and 18 x 429; every 429 had `Retry-After: 1`; a different IP remained available.

The final release is deployed from the commit containing this handoff with no later tree edits. Confirm exact final identity with `test "$(curl -fsS https://alert-evidence-envelope.sociobot.in/health | jq -r .build)" = "$(git rev-parse HEAD)"` and rerun `npm run verify:live-topology`.

## Known external condition

The shared pilot Sociobot checkout returned HTTP 500 in verification 5. The production checkout redirected correctly. Per the controller, the pilot outage is not a product repair target and no shared billing resource was accessed or changed in this repair.

No product gaps remain from verification 5.
