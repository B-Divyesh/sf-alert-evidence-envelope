# Repair handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-repair-7`
Verifier report: `1da3ccd5ced6b99cabb40198c753ffd204242e89` / `.factory/verification-8.md`
Repair implementation: `8fbfd321480f96e7ef35158ca57d5ec31b087d74`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result

All release-blocking findings from verification 8 and the controller evidence review are repaired and covered by regressions. The repaired implementation was pushed and deployed as revision `sf-alert-evidence-envelope--0000023`. Live `/health` reports `8fbfd321480f96e7ef35158ca57d5ec31b087d74`.

## Repairs

- Offline demo reload now renders `Offline sample ready. Demo data was not stored.` from the cached envelope. Offline startup skips `/health`, demo API, and license calls. The service worker uses its cached navigation response when the browser is offline.
- The offline claim now fails on any disconnected request and on any offline `/health` or `/api/` request.
- Browser tests explicitly prebuild Rust, then start `target/debug/alert-evidence-envelope`. A cold compile no longer consumes Playwright’s 120-second server allowance.
- Registered `provider-signature`, `history-limit`, `local-policy-presets`, and `durable-deployment` in `.factory/claims.json`.
- Added a manifest regression that requires every public claim to have exactly one tagged test. The inventory now has 24 unique claims and 24 unique tags.
- Rebuilt `.factory/copy-audit.md` from the rendered landing route, including provider signature, remote source, preview, history, Field Kit, statuses, controls, and recovery copy. All audited entries are at most 22 words and use the agreed terms.
- Enlarged every interactive target on the 390 px home, demo, privacy, and terms routes to at least 44 by 44 CSS pixels, including Demo, the enabled checkbox, and legal footer links.
- Limited one-year immutable caching to Vite’s content-hashed `index-*` JavaScript and CSS. Fonts, illustrations, social art, icons, the service worker, and HTML now revalidate with `no-cache`.

## Reproduction and regression evidence

- Before the fix, the exact command `npm run test:claims -- --grep @claim:offline-demo --reporter=line` failed in both projects because the offline status was absent. It also exposed the cold-build delay before Playwright began.
- After `cargo clean`, `npm run test:claims -- --grep @claim:demo-envelope --reporter=line` compiled Rust in 1m00s outside Playwright and then passed 2/2 browser cases in 4.3s.
- Every command in `.factory/claims.json` was run verbatim after the fix. All 24 claim entries passed.
- `npm test` passed: Svelte reported 0 errors and 0 warnings; 22 Rust tests passed; deployment and claim-manifest policies passed; Playwright passed 52/52 desktop and 390 px cases.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --locked -- -D warnings` passed.
- `VITE_BUILD_SHA=8fbfd321480f96e7ef35158ca57d5ec31b087d74 npm run build` passed and produced `dist/`.
- `BUILD_SHA=8fbfd321480f96e7ef35158ca57d5ec31b087d74 cargo build --release --locked` passed.
- Release assets: JavaScript 69,642 bytes raw / 25,727 bytes gzip; CSS 18,791 bytes raw / 5,194 bytes gzip; fonts 115,560 bytes; mobile hero 40,982 bytes.
- Port-only startup with only `PATH` and `PORT=4191` generated the SQLite configuration and three credentials without secrets in logs. Credential file modes were 600 and `/health` returned the full repair SHA.
- The worker `verify-url.sh` passed locally in 596 ms and live in 635 ms: title present, `lang=en`, one `h1`, `main` present, no missing alt text, no unlabeled buttons, and no console errors.
- Playwright axe found zero serious or critical WCAG A/AA findings in light and dark modes. Keyboard skip/focus, route announcements, reduced motion, 200% text, and horizontal overflow checks passed.
- Local Lighthouse: performance 97, accessibility 100, best practices 100, SEO 100; FCP 1.3s, LCP 2.3s, TBT 110ms, CLS 0. Lighthouse wrote the complete report and then printed a post-audit browser-tab crash from the supplied Chromium.

## Live evidence

- Scoped deployment verification passed: revision `sf-alert-evidence-envelope--0000023`, single revision mode, min/max replicas 1/1, one running replica, `/data` mounted from `alert-evidence-envelope-data`, and 20/20 fresh-connection previews passed.
- A fresh live 390 px context loaded `/demo` online, switched offline, reloaded the cached envelope, showed the offline-ready status and `checkout-api`, and recorded 0 failed requests.
- Live 390 px checks on `/`, `/demo`, `/privacy`, and `/terms` found 0 serious/critical axe findings, 0 console errors, no horizontal overflow, minimum target width 44 px, and minimum target height 44 px.
- Live stable font and image URLs return `Cache-Control: no-cache`; the content-hashed JavaScript returns `public, max-age=31536000, immutable`; `/sw.js` returns `no-cache`.
- Live responses retain HSTS, CSP with response-header `frame-ancestors 'none'`, `nosniff`, `DENY`, and `no-referrer`.

## Commands

```sh
npm ci
npm test
# Run each command listed in .factory/claims.json.
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=$(git rev-parse HEAD) npm run build
BUILD_SHA=$(git rev-parse HEAD) cargo build --release --locked
npm run deploy
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot $(git rev-parse HEAD)
```

## Known gaps

- Docker, Podman, and Buildah are unavailable in this worker, so no local image build ran. Azure Container Registry built the multi-stage Dockerfile successfully (`ch1p8`).
- This product has no package consumer, sign-in flow, or AI feature; consumer-install, Entra identity, and model-gateway checks do not apply.
