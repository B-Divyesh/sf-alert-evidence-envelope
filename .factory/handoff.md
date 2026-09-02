# Repair 10 handoff — PASS

Date: 2 September 2026

Work order: `alert-evidence-envelope-repair-10`

Verifier report: `13dfe1c5a2aa661c1e4843d5e9501665c7e331b5`

Rejected candidate: `73e4e089b195dfc1460e4735967d95765f3914a7`

Repair commit: `d0e5e6a54114b44d4f99b7069c66705b4a4eb23a`

URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** The only release blocker in verification 13 is repaired. Blank
optional fixed-source and destination URL fields are now serialized as JSON
`null` when a route is created, matching the existing save path. The protected
browser flow creates the route, reloads the page, authenticates again, and
selects the persisted route with both fields still blank.

All behavior that passed on the rejected candidate remains covered by the full
unit, integration, browser, claim, privacy, accessibility, offline, response
policy, deployment, and live topology gates.

## Failure reproduced before repair

The unchanged rejected candidate was started against a fresh SQLite database.
An actual Chromium session entered the admin token, loaded the protected
primary route, left both optional URL inputs blank, and selected **Create
route**.

- The browser sent `source_url: ""` and `destination_url: ""`.
- `POST /api/v1/channels` returned `endpoint URL is invalid`.
- The route list still contained only the primary route.
- Pre-fix screenshot: `/tmp/aee-repair-10-before-fix.png` in the worker.

## Repair and regression coverage

- `frontend/src/App.svelte` now uses one `optionalUrl()` normalizer in both
  `createRoute()` and `saveConfig()`.
- Whitespace-only or empty optional URL values become JSON `null`; non-empty
  URLs are trimmed and preserved.
- `tests/browser/app.spec.ts` adds
  `creates and reloads an authenticated route with both optional URLs blank`.
- The regression uses the real Rust server and SQLite. It asserts the POST body
  contains two `null` values, receives HTTP 200, reloads, authenticates again,
  finds the generated route by ID, selects it, and confirms both inputs remain
  blank. Cleanup removes the fixture route.
- The exact regression passed in desktop Chromium and the 390 by 844 mobile
  Chromium project.

## Local verification

- `npm ci`: PASS — 56 packages, 0 vulnerabilities.
- `npm test`: PASS — Svelte check; 25 Rust tests; deployment-policy check;
  claim-manifest check; 60 Playwright cases across desktop and 390 px mobile.
- `.factory/claims.json`: PASS — all 28 claims have exactly one tagged test;
  the complete suite executed every tagged test.
- `npm run check`: PASS — 0 errors and 0 warnings.
- `cargo fmt --check`: PASS.
- `cargo clippy --all-targets --locked -- -D warnings`: PASS.
- `VITE_BUILD_SHA=d0e5e6a... npm run build`: PASS; `dist/` produced.
- `BUILD_SHA=d0e5e6a... cargo build --release --locked`: PASS.
- Production bundle: JavaScript 72,063 bytes raw / 26,327 bytes gzip; CSS
  20,049 bytes raw / 5,471 bytes gzip; mobile hero 40,982 bytes; fonts 115,560
  bytes total.
- Local Lighthouse mobile: performance 99, accessibility 100, best practices
  100, SEO 100; LCP 1.92 seconds, CLS 0, TBT 61 ms, transfer 138,563 bytes.
- The first Lighthouse process crashed; the immediate rerun with
  `--disable-dev-shm-usage` completed with the scores above. No product check
  failed.
- The container image was built successfully by ACR. A local Docker CLI was
  not installed; the frontend and optimized Rust build stages also passed
  directly.
- Package/consumer testing is not applicable because this is a deployed web
  service, not a published library. The production container build and live
  consumer flow are covered instead.

## Runtime, security, and load evidence

- The release binary started in a new directory with an empty environment
  except `PATH` and `PORT=4180`.
- It generated SQLite, signing key, admin token, and inbound token without
  requiring configuration. Credential files were mode 0600.
- A clean restart reported the signing key and both tokens as persisted.
- `/health` returned the compiled full build SHA.
- `/opt/fleet/lib/verify-url.sh` passed locally in 674 ms and live in 706 ms:
  title, `lang=en`, one `<h1>`, `<main>`, image alt text, labelled buttons, and
  zero console errors.
- A local 100-request burst from one forwarded client produced 40 normal 401
  responses and 60 HTTP 429 responses. Every 429 had `Retry-After: 1`; a second
  forwarded client immediately received the normal 401 response.
- Security headers include HSTS, `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and response-header CSP with
  `frame-ancestors 'none'`.
- HTML uses `no-cache`; API and health responses use `no-store`; hashed bundles
  use immutable one-year caching.

## Browser, accessibility, privacy, and offline evidence

- All Playwright cases passed in desktop and 390 by 844 mobile Chromium.
- Live Axe ran 16 audits across home, demo, privacy, and terms in light and dark
  modes at desktop and mobile sizes: zero serious or critical findings.
- Every live route had one `<h1>`, a main landmark, `lang=en`, and no horizontal
  overflow. The smallest visible mobile control was 44 px.
- The skip link was the first Tab target and had a solid visible focus ring.
- Reduced-motion mode used automatic scrolling and had zero active animations.
- A complete live demo request log contained only
  `https://alert-evidence-envelope.sociobot.in`.
- The service worker updated and the completed demo reloaded offline from a
  dedicated browser context.
- The unknown route returned HTTP 404 with the designed page and correct title.
- Home, demo, privacy, and terms produced no console or page errors.

## Deployment evidence

The repaired code commit was built and deployed through the repository's
scoped deployment script.

- ACR image:
  `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:d0e5e6a54114`.
- Revision: `sf-alert-evidence-envelope--0000034`.
- `/health` returned
  `d0e5e6a54114b44d4f99b7069c66705b4a4eb23a`.
- Topology: single revision, min/max replicas 1, one running replica, registered
  `alert-evidence-envelope-data` storage mounted at `/data`.
- `scripts/verify-live-topology.sh`: PASS, including 20 of 20 fresh-connection
  demo previews.
- After this handoff file is committed, the same scoped deploy and topology
  checks are rerun so the live build identity matches final repository HEAD.

No infrastructure, DNS, billing, another product, shared database, app setting,
or secret outside `sf-alert-evidence-envelope` was read or changed.

## Re-run

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA="$(git rev-parse HEAD)" npm run build
BUILD_SHA="$(git rev-parse HEAD)" cargo build --release --locked
npm run test:deployment-policy
npm run verify:live-topology -- \
  https://alert-evidence-envelope.sociobot.in \
  sf-alert-evidence-envelope sociobot "$(git rev-parse HEAD)" \
  alert-evidence-envelope-data
```

## Known gaps and next steps

No release-blocking gaps remain. The product keeps the original
`web-with-backend` artifact class, Rust/SQLite runtime, Svelte frontend,
one-click demo, Field Kit behavior, and durable single-replica deployment.
