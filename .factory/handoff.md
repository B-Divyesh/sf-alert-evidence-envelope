# Polish 2 handoff — Alert Evidence Envelope

Date: 2026-09-02  
Work order: `alert-evidence-envelope-polish-2`  
Repair commits: `43f47000fe5ef77ec4e1a9414476314ab575ab82` and `a2234a17b5ac0f7c44a225556ba81c9ae7c70dba`

## Delivered

- Repaired every cumulative F-1 and F-2 review finding; the exact mapping is in `.factory/polish-2.md`.
- Made `/demo` an isolated, one-click two-route comparison: Internal Slack redacts `token`; Customer automation redacts `email` and `token`.
- Made the mobile completion result fully visible at 390 × 844, including `[REDACTED]`, with a full bounding-box regression assertion and screenshot artifact.
- Added real JSON, Slack, and email-webhook delivery contracts and local capture-server coverage.
- Tightened free-core, credential-exposure, 24-hour throttle, rate-limit, legal chrome, metadata, wording, and feedback behavior.

## Verification

- `npm test` — pass: Svelte check, 25 Rust tests, deployment policy, 28-claim manifest, and 56 Playwright desktop/mobile tests.
- `cargo fmt --check` — pass.
- `cargo clippy --all-targets --locked -- -D warnings` — pass.
- `npm run build` — pass; initial JavaScript gzip is 26.42 KB and CSS gzip is 5.49 KB.
- Fresh clone `/tmp/aee-clean-OGLsmX` at `43f47000fe5ef77ec4e1a9414476314ab575ab82`: `npm ci`, then every exact command in all 28 `.factory/claims.json` entries — pass.
- Browser coverage includes keyboard, route focus/back behavior, 390 px layout and touch targets, dark/light axe scans, offline reload, request privacy, legal pages, 404, security headers, and demo isolation.

## Deploy and live recheck

- Deployed `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:a2234a17b5ac` on `sf-alert-evidence-envelope--0000030`.
- `npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot a2234a17b5ac0f7c44a225556ba81c9ae7c70dba alert-evidence-envelope-data` — pass: single active/healthy replica, `/data` Azure File mount, 20 fresh demo previews, live `/health` build `a2234a17b5ac0f7c44a225556ba81c9ae7c70dba`.
- Cold `/opt/fleet/lib/verify-url.sh` evidence is in `/tmp/aee-live-final-YRIRsl`: 200 home page; title, `lang=en`, one h1, main landmark, image alt text, and no console errors. It wrote desktop/mobile screenshots and `verify.json`.
- Cold URL checks: `/`, `/demo`, `/privacy`, and `/terms` returned 200; `/not-a-real-route` returned 404.
- Live Playwright axe scans at 390 × 844 found zero serious/critical violations on home, demo, privacy, terms, and 404. Both legal headers expose Home, Demo, Privacy, and Terms, with the current page marked.
- Live mobile demo check passed: all required completed-result boxes ended within the 844 px viewport; Internal Slack route showed token-only redaction while retaining the sample email; no console errors.
- `npx @axe-core/cli` could not locate its Selenium Chrome binary in this worker. The equivalent Playwright AxeBuilder scan used the installed Playwright browser and passed all five live routes.

## Known gaps

None known locally. Runtime state remains SQLite under `/data`; the product requires no environment variables beyond `PORT`.
