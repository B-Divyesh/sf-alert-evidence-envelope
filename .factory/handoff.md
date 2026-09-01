# Repair handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-repair-9`
Base candidate: `eeb7d38b2022ec3ed3079f6da67aa7edfce270fb`
Repair commit: `fix: restore mobile LCP budget`
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

Deployment is performed after this commit with the scoped `sf-alert-evidence-envelope` helper. Add the revision, image, `/health` SHA, topology, and live verification results here after deployment.

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
