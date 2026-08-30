# Alert Evidence Envelope — repair handoff

Date: 2026-08-30

Work order: `alert-evidence-envelope-repair-6`

Source finding: `.factory/verification-5.md` at report commit `9e2990c931e8dcfb16901e1e834ec0b8ac4fee8d`, verifying candidate `10fd47b26b6de165b432420838c9f4300b5c07c1`.

## Outcome

The release-blocking product defects are repaired. Demo session IDs and expiry times now live in SQLite beside channel state. The production image uses `/data/envelopes.db` directly, and the deployment policy requires one replica with `/data` mounted. A legacy `/data/envelopes.snapshot.db` is copied once when the direct database does not yet exist.

The checkout endpoint’s shared pilot HTTP 500 is environment-gated per the controller. Product coverage now verifies the displayed $39 one-time price and official Sociobot checkout URL without making shared checkout availability a product gate.

## Reproduction and regression

Before the repair, `cargo test demo_create_then_preview_survives_a_fresh_database_connection --locked` reproduced the live failure: create returned 200, then preview through a newly opened state/database connection returned 404 instead of 200.

That exact regression now passes. The test creates a session, closes the first SQLite pool, opens a fresh state against the same database, and successfully previews the sample. A separate port-only runtime smoke repeated create, process stop, process restart, and preview successfully.

Additional claim coverage now proves:

- the demo session expiry is 24 hours and expired rows are rejected and removed;
- the configured upstream receives only `q` and `limit`, even when input contains an attacker-controlled source URL;
- HMAC-SHA256 values are recomputed and verified for generated and forwarded envelopes;
- valid Field Kit users can save, reload, and apply named presets from browser-only storage;
- the demo remains outside protected delivery history and raw evidence remains absent from SQLite.

## Local verification

- `npm ci`: passed; 56 packages installed, 0 vulnerabilities.
- `npm test`: passed; Svelte reported 0 errors/warnings, Rust passed 17 tests, deployment policy passed, Vite built `dist/`, and all 38 desktop/390 px browser cases completed. One mobile Chromium process crashed once under the worker and passed on its configured retry; no product assertion failed.
- Every non-live command in `.factory/claims.json`: passed independently.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `BUILD_SHA=repair-local cargo build --release --locked`: passed.
- Port-only runtime: started with an empty environment except `PATH` and `PORT`; generated credentials, returned `repair-local` from `/health`, and recovered them after restart.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4180 ...`: passed with title, `lang=en`, one h1, main, alt text, labelled buttons, and zero console/page errors.
- Playwright axe: zero serious/critical findings on product, demo, privacy, and terms in light/dark desktop and 390 px states.
- Keyboard, focus, 44 px targets, 200% text, reduced motion, offline reload/update, same-origin demo privacy, security/cache headers, 404, and rate-limit response policy: passed in the browser suite.
- Mobile Lighthouse, three runs: performance 93/98/98 (median 98), accessibility 100/100/100, best practices 100/100/100, SEO 100/100/100. Median LCP 2,404 ms, TBT 53 ms, CLS 0.
- Production assets: initial JS 66,327 bytes raw / 25,220 gzip; CSS 17,557 bytes raw / 4,970 gzip; fonts 115,560 bytes total; mobile hero 40,982 bytes.
- Docker is unavailable in this worker. The deployment uses the Dockerfile through ACR Build.

## Deployment evidence

Pending the committed deployment. After deployment, record the exact build SHA, revision, one-replica `/data` topology, 20 fresh-connection demo previews, live accessibility/browser checks, and live response policy here.

## Known external condition

The shared pilot Sociobot checkout returned HTTP 500 in verification 5. The production checkout redirected correctly. Per the controller, the pilot outage is not a product repair target and no shared billing resource was accessed or changed in this repair.
