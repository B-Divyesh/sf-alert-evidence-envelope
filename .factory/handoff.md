# Verification 13 handoff — FAIL

Date: 2 September 2026

Work order: `alert-evidence-envelope-verify-13`

Candidate and live build: `73e4e089b195dfc1460e4735967d95765f3914a7`

URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**FAIL.** The one-click demo, relay behavior, deployment identity, privacy,
accessibility, PWA, rate limiting, persistence, and performance checks pass.
There is one P1/high release blocker in the authenticated route-builder flow.

## Blocking defect

**F-1 — Create route rejects blank optional URLs.** After loading a protected
route, select **Create route** while leaving **Fixed evidence source URL**
blank, as instructed for evidence embedded in an alert. The UI reports
`endpoint URL is invalid` and creates no route. The same happens when the
source is valid but **Destination URL**, labelled optional when supplied by the
server environment, is blank.

The browser's `createRoute()` submits empty strings; the backend correctly
accepts these modes only as JSON `null`. The normal save path already performs
that normalization. Fix `createRoute()` the same way and add an authenticated
browser test for creating and reloading a route with both optional fields
blank. Full evidence is in [verification-13.md](verification-13.md).

## Verification summary

- All 28 commands in `.factory/claims.json`: PASS.
- `npm test`: PASS — 25 Rust tests, 58 browser cases; one Chromium process
  crash passed on Playwright retry and the exact claim passed independently.
- `npm run check`, `cargo fmt --check`, and strict Cargo clippy: PASS.
- Candidate-stamped Vite build and optimized Rust build: PASS; `dist/` exists.
- Docker CLI was unavailable; the Dockerfile's two build stages passed
  directly, and live file hashes matched the candidate output.
- `/health` and footer report `73e4e089b195...`; scoped topology check passed
  for revision `sf-alert-evidence-envelope--0000033`, one replica, durable
  `/data` mount.
- Local full relay: HTTP 202, item cap applied, nested secrets redacted,
  signature present, provider-signature state preserved, destination called,
  metadata persisted, and raw markers absent from SQLite.
- Restart retained route/history and all generated credentials.
- Live 20-way concurrent demo preview: 20/20 passed.
- Live rate limit: 100 requests in 500 ms produced 45×401 and 55×429; every
  429 had `Retry-After: 1`; another forwarded IP was unaffected. Contract is a
  40-request burst with 20 requests/second refill.
- Live request log remained same-origin. Security/cache headers passed.
- Axe serious/critical: 0 across home, demo, privacy, terms, and 404 in light
  and dark. Keyboard, focus, 390 px layout, 44 px targets, 200% text, and
  reduced motion passed.
- Service-worker update and offline demo reload passed.
- Lighthouse mobile: 94 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.8 s, CLS 0, total transfer 132 KiB.

## Re-run

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=73e4e089b195dfc1460e4735967d95765f3914a7 npm run build
BUILD_SHA=73e4e089b195dfc1460e4735967d95765f3914a7 cargo build --release --locked
npm run verify:live-topology
```

Do not release this candidate until F-1 is repaired and the browser regression
test passes. No product code or infrastructure was changed by verification 13.
