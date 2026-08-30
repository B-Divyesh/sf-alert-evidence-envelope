# Polish 1 handoff — Alert Evidence Envelope

Date: 2026-08-30

## Result

All `F-1-1` through `F-1-21` findings are closed. The detailed mapping is in `.factory/polish-1.md`.

Deployed commit: `c43bb3677e782fe5d080ad2a9c220e58a9b5aa79`  
Live URL: `https://alert-evidence-envelope.sociobot.in`

The deployed product reports that exact SHA from `/health`. Deployment verification passed with revision `sf-alert-evidence-envelope--0000021`, one active/running replica, the scoped `alert-evidence-envelope-data` Azure File mount at `/data`, and 20 fresh demo-session previews.

## What changed

- Made the 390px demo result-first, including service, error, redaction state, and signed status above the fold.
- Removed license tokens from verification URLs; verification now uses an authorization header and records failed attempts before throttling them for 24 hours.
- Added route list/create/update/delete support with independent route IDs, policies, destinations, and relay URLs.
- Completed the claim inventory and observable tests; removed public assertions that could not be proved in the sandbox.
- Reworked route titles/metadata, History API focus announcements, mobile header navigation, 404 metadata/chrome, legal wording, and plain-language copy.

## Verification

- Clean dependency install: `npm ci` — PASS.
- `npm test` — PASS: 22 Rust tests and 52 desktop/mobile Playwright tests.
- `cargo fmt --check` — PASS.
- `cargo clippy --all-targets --locked -- -D warnings` — PASS.
- `npm run build` — PASS; initial JavaScript 26.00 KB gzip and CSS 5.18 KB gzip.
- Live cold checks — PASS: `/health` SHA, root title/metadata, `/privacy` transport wording, designed 404 metadata/chrome, and 390×844 `/demo` result-first test with zero console errors.
- Live accessibility — PASS: Playwright axe found zero serious/critical violations on `/`, `/demo`, `/privacy`, `/terms`, and the 404 at 390px.
- `verify-url.sh` live root — PASS; its screenshots and report are in `/tmp/aee-live-evidence/` for this worker. The result-first mobile capture is `/tmp/aee-live-evidence/demo-390.png`.

## Run and deploy

```sh
npm ci
npm test
npm run build
PORT=8080 cargo run
npm run deploy
```

## Known gaps

None. The optional standalone `@axe-core/cli` could not locate a Selenium Chrome binary in this worker; equivalent Playwright axe checks ran locally and against the live routes.
