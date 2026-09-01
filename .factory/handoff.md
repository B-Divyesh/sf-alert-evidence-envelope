# Verification handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-11`
Candidate: `ff56488761e3922e8fe788807fcda37de6cc7cc5`
Live URL: <https://alert-evidence-envelope.sociobot.in>

## Current verification result: PASS

Independent QA accepted the candidate. The live `/health` identity is exactly
`ff56488761e3922e8fe788807fcda37de6cc7cc5`; locally built candidate JS, CSS,
and service-worker assets match the live files byte-for-byte.

- All 25 declared claim commands passed from a clean checkout.
- `npm test` passed: Svelte/type checks, 23 Rust tests, policy/manifest
  checks, and 54 desktop/mobile browser tests.
- Formatting, strict clippy, and the candidate-stamped production frontend
  build passed.
- Live normal, boundary, malformed-input, and deleted-demo recovery paths
  passed. Privacy request logging, headers, keyboard/focus, 390 px layout,
  reduced motion, and axe serious/critical checks passed.
- Mobile Lighthouse measured 1.9 s and 1.8 s LCP (performance 94 and 99),
  below the 2.5-second release contract.

No P0–P3 defects are open. See
[`.factory/verification-11.md`](verification-11.md) for exact commands,
claim coverage, response evidence, rate-limit observation, and the one
environment limitation: this verifier container does not have `docker`, so
it could not invoke a local `docker build`.

## Historical repair handoff

Date: 2026-09-01
Work order: `alert-evidence-envelope-repair-9`
Base candidate: `eeb7d38b2022ec3ed3079f6da67aa7edfce270fb`
Repair commit: `61a26525deb51ab52e2fc26841a8c714be78a940` (`fix: restore mobile LCP budget`)
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result

Repaired the sole release blocker in `.factory/verification-10.md`: repeatable mobile LCP exceeded 2.5 seconds. The artifact remains the Rust/axum + SQLite container product; no product behavior, deployment class, claims, or data topology changed.

## Root cause and repair

The live candidate was reproduced twice with mobile Lighthouse before editing: 93 performance, 2.6 s FCP/LCP, 0 ms TBT, 0 CLS on both runs. The LCP trace identified the mobile evidence-terrain illustration. Its request began only after the Svelte application booted, while the 48 KB Inter and 67 KB Fraunces preloads competed with the app shell on the constrained network.

- Removed the two mobile-critical font preloads.
- Preloaded the existing 40,982-byte mobile WebP with `fetchpriority="high"`; the rendered image also declares high fetch priority.
- Use installed system/serif faces at 700 px and below so webfonts cannot compete with the mobile LCP path. Desktop retains the self-hosted Fraunces/Inter treatment.
- Tightened the demo bench’s 390 px spacing so the complete signed/redacted sample remains visible above the 844 px viewport.
- Added a browser regression that asserts the mobile LCP image is discovered before the JS bundle, no font is requested on the landing mobile critical path, and axe reports no serious/critical issue. It runs in both browser projects.

## Verification evidence

- Clean install: `npm ci` installed 56 locked packages; audit reported 0 vulnerabilities.
- `npm test`: PASS — Svelte check 0 errors/0 warnings; 23 Rust tests; deployment and 25-claim manifest checks; 54 Playwright tests across desktop and 390 px mobile.
- `cargo fmt --check`: PASS.
- `cargo clippy --all-targets --locked -- -D warnings`: PASS.
- Local production container server (`target/release`, built `dist/`): two stable mobile Lighthouse runs: performance 97/97, FCP/LCP 2.1 s/2.2 s, TBT 0 ms, CLS 0. Both are below the 2.5-second LCP target.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4181/`: PASS — 200, title, `lang=en`, one `h1`, `main`, image alternatives, labelled buttons, and no console/page errors. The standalone axe CLI could not start its ChromeDriver in this worker; the in-repo Playwright axe integration passed for the landing regression, demo themes, and 390 px legal pages.
- Browser suite also covers keyboard skip-link/focus, 44 px mobile targets, 200% text size, reduced motion, offline demo reload/service-worker update, same-origin demo request logging, headers/cache policy, rate limits, and the declared sample/claim flows.

## Deploy and live identity

- Scoped deployment: PASS via `npm run deploy`.
- Live `/health`: `{"build":"61a26525deb51ab52e2fc26841a8c714be78a940","status":"ok"}`.
- Revision: `sf-alert-evidence-envelope--0000027`.
- Image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:61a26525deb5`.
- Topology: single revision mode, one running replica, min/max replicas 1/1, durable `/data` mounted from `alert-evidence-envelope-data`; the deploy verifier completed 20 fresh-connection demo previews.
- Live URL verification: PASS — 200 in 699 ms, expected title/lang/one `h1`/`main`, no image or button labelling issue, and no console or page errors.
- Final live mobile Lighthouse runs against the deployed URL: performance 96/96, FCP/LCP 2.2 s/2.2 s, TBT 0 ms, CLS 0/0. Both repeatable results are below the 2.5-second release target.

## How to verify

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=$(git rev-parse HEAD) npm run build
BUILD_SHA=$(git rev-parse HEAD) cargo build --release --locked
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot $(git rev-parse HEAD) alert-evidence-envelope-data
```

## Known gaps

None. The separate ChromeDriver-based axe CLI is unavailable in this worker image; Playwright’s bundled Chromium + axe integration is the passing accessibility evidence.
