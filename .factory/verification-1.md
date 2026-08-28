# Verification 1 — FAIL

Date: 2026-08-28  
Work order: `alert-evidence-envelope-verify-1`  
Candidate: `109a9714bddb00ebc26ae28158c709332b1c6352`  
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL.** The local candidate is buildable and its static frontend exactly matches the live frontend, but release acceptance is blocked by three user-visible/release-integrity defects:

1. **P1 — legal routes return HTTP 404.** Both the local release server and live deployment return a 404 status for `/privacy` and `/terms` while serving the SPA shell. The browser renders the legal copy after JavaScript loads, but these required legal pages are not valid HTTP pages for direct links, crawlers, or no-JavaScript clients.
2. **P1 — 390px mobile layout has horizontal overflow.** Fresh Chromium at a 390px CSS viewport measured `document.documentElement.scrollWidth = 414` and `innerWidth = 390`. The Field Kit grid resolves to a 396px column, causing 24px horizontal scrolling.
3. **P1 — deployed backend has no build identity.** `GET /health` on the live URL returned `{"build":"development","status":"ok"}`. The contract requires a build SHA; consequently the backend cannot be confirmed to be candidate `109a971…` even though the static assets match.

## Reproducible evidence

### Clean local quality gates

- Started from a clean checkout at exactly `109a9714bddb00ebc26ae28158c709332b1c6352`; `npm ci` completed with 0 vulnerabilities.
- `npm test` passed. It runs `svelte-check` (0 errors/warnings), `cargo test --locked` (4 passed), the exact Vite production build, and Playwright. A separately rerun `npx playwright test --reporter=line` passed **8/8**: desktop Chromium and 390 × 844 Chromium.
- `npm run build` produced `dist/`: JS 63,161 B raw / 24,550 B gzip, CSS 16,087 B raw / 4,680 B gzip, fonts 115,560 B total, and mobile hero 75,882 B. All stated asset budgets pass.
- `cargo fmt --check` and `git diff --check` passed. `cargo clippy --all-targets --locked` exited 0 with four `field_reassign_with_default` warnings in test code; with `-D warnings` it fails on those warnings.
- Docker/Podman/Buildah are unavailable in this verifier image, so the Dockerfile itself was not built.

### End-to-end relay and boundary checks

Against a fresh local SQLite database with admin/inbound tokens enabled:

- Authorized config returned 200; omitted admin token returned 401. Relay omitted inbound token returned 401; invalid JSON returned 400.
- A representative checkout alert returned 202, preserved the provider-signature flag, recursively redacted `email`, nested `token`, and `cookie`, reported service/error/first-seen, applied a query fingerprint, and its HMAC verified independently using the configured key.
- Invalid caps returned 400 (`max_bytes=1023`, `max_items=101`); an unsafe `http://evil.example` endpoint returned 400; a body exceeding 256 KiB returned 413.
- After 22 seeded relay calls, `/api/v1/history` held exactly 20 metadata-only records; its JSON contained neither seeded evidence nor email values. This validates the no-raw-payload retention boundary for the exercised path.
- 200 health requests at concurrency 10 (approximately 100 requests/s) returned 200/200.

### Browser, privacy, PWA, and transport checks

- On the live UI, desktop and mobile browser checks found no console errors, one `h1`, one `main`, meaningful image alt text, no serious/critical axe WCAG A/AA violations, keyboard-first focus on the visible skip link (`outline: solid`), and reduced-motion animation duration of `0.01s`. Invalid sample JSON showed the recovery state; restoring the sample and rebuilding produced the signed envelope.
- At an exact 390px viewport the page horizontally overflows as noted above. The existing Playwright mobile suite did not detect scroll width.
- With no license present, observed browser requests were first-party only. Source inspection found no analytics, remote font, or third-party script; the only allowed external application endpoint is Sociobot billing verification when a license exists. Live history remained empty after preview use.
- Service worker was active and controlling the live page, `registration.update()` completed without error, cache `envelope-shell-v2` was present, and the repository's offline reload test passed in both browser projects.
- Live response headers: CSP limits default/style/script/image to self (plus data images) and connect to self plus Sociobot; `nosniff` and `no-referrer` are present. API responses use `Cache-Control: no-store`; hashed assets use `public, max-age=31536000, immutable`; HTML and service worker use `no-cache`.
- An independent Lighthouse run could not complete because the supplied Playwright Chromium crashed when invoked by Lighthouse. This is an environment limitation, not a substituted performance score; bundle-budget evidence above is direct.

### Live candidate comparison

- The freshly built candidate generated `assets/index-SCRc4Rzq.js` (SHA-256 `d51123ff599b17399b4a436a447206eb6f2fee0d436eb5743479a51bf9396c8a`) and `assets/index-Cz8MLpGn.css` (SHA-256 `a32738d0fa708022e20fdb7acbcedf65f798cb71717d31f7553a7daf46455028`). The two live files have identical byte counts and hashes.
- Live HTML is timestamped `Fri, 28 Aug 2026 00:16:57 GMT`, shortly after the candidate commit timestamp `00:16:22 UTC`.
- This proves the live static frontend matches; it does **not** prove the live backend matches because health reports `development` rather than an immutable SHA.
- `curl` evidence for both local and live: `/` is 200; `/privacy`, `/terms`, and an arbitrary nonexistent route are 404 with the same 1,150-byte SPA shell body.

## Required fixes before re-verification

1. Serve `/privacy` and `/terms` as HTTP 200 routes (and retain correct SPA fallback behavior only where intended).
2. Remove the 390px Field Kit/grid minimum-width overflow; add a mobile scroll-width assertion to browser coverage.
3. Inject the immutable build SHA at compile/build time and return it from `/health`; deploy it so live backend identity can be compared to the candidate.
4. Optionally clean the four Clippy test-code warnings if Clippy is to be a warning-free gate, then rerun independent Lighthouse in an environment that can launch Chrome.
