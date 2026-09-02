# Polish 3 handoff — PASS

Date: 2 September 2026

Work order: `alert-evidence-envelope-polish-3`

Repair commit: `d7bc262be19bb0bcf89b71d83120f21308b20778`

Deployed source commit: `a841f3b8a47cc9f252de0e7384376ca221418e97`

## Delivered

- Closed every F-1, F-2, and F-3 finding; `.factory/polish-3.md` maps each ID to its repair and evidence.
- Restored focus and polite route-title announcements into Privacy and Terms and back to the app.
- Added the direct isolated `/?demo=1` path while preserving `/demo`; reset creates a new session and exit clears every `demo:` key.
- Narrowed the Terms free-feature promise to the tested controls.
- Split the first-screen price fact, corrected the README JSON-webhook sentence, and repaired both 404 builder links and its external-source label.
- Updated `.factory/claims.json`, `.factory/demo.md`, `.factory/copy-audit.md`, and the 86-character verb-first catalog description.

## Verification

- `npm test` — pass: Svelte check, 25 Rust tests, deployment/claim manifest checks, and 58 Playwright cases across desktop and 390 × 844 Chromium.
- `cargo fmt --check` — pass.
- `cargo clippy --all-targets --locked -- -D warnings` — pass.
- `npm run build` — pass; `dist/` produced. Initial JS is 71.94 KB / 26.64 KB gzip; CSS is 20.04 KB / 5.49 KB gzip.
- Clean clone `/tmp/aee-polish3-clean-d7bc262` — `npm ci`, `npm test`, then all 28 exact claim commands passed. Log: `/tmp/aee-polish3-clean-d7bc262.log`.
- Mobile evidence: `/tmp/aee-polish3-live/home-mobile-cold.png`, `/tmp/aee-polish3-live/demo-mobile-cold.png`, and the Playwright `mobile-demo-complete-result.png` artifact.
- `/opt/fleet/lib/verify-url.sh` — pass. `/tmp/aee-polish3-live/verify.json` reports title, `lang=en`, one `h1`, `main`, alt text, 757 ms cold load, and no console errors.
- Live Playwright AxeBuilder — zero serious/critical findings on `/`, `/?demo=1`, `/privacy`, `/terms`, and the 404 in both light and dark modes.
- Live offline check — cached sample reloaded twice in a fresh offline context with no failed, API, or health requests.
- Live link crawl — all product, checkout, and source links returned 200; the unknown route correctly returned 404.
- Lighthouse mobile — performance 94, accessibility 100, best practices 100, SEO 100; LCP 1,949 ms, CLS 0, TBT 248 ms.
- Live limiter — 100-request burst returned 43 unauthorized responses and 57 rate-limited responses; every 429 included `Retry-After: 1`.

## Deployment

- URL: <https://alert-evidence-envelope.sociobot.in>
- Health build: `a841f3b8a47cc9f252de0e7384376ca221418e97`
- Revision: `sf-alert-evidence-envelope--0000032`
- Image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:a841f3b8a47c`
- Topology: single revision, one healthy replica, SQLite and generated credentials under the fleet-mounted `/data` share `alert-evidence-envelope-data`.
- Post-deploy topology check opened 20 fresh demo previews successfully.

## Known gaps

None known. No finding of any severity remains open.
