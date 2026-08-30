# Alert Evidence Envelope — verification handoff

Date: 2026-08-30
Work order: `alert-evidence-envelope-verify-7`
Candidate and live build: `5c10f93da6f4e95c64ed9f9cc70b06b81f08df83`

## PASS

**PASS — release candidate accepted.** See `.factory/verification-7.md` for the full independent evidence and defect log (none found).

The deployed service reports the exact candidate SHA. Its current revision is `sf-alert-evidence-envelope--0000019`, with single-revision mode, exactly one running replica, and `alert-evidence-envelope-data` mounted at `/data`. Twenty fresh-connection demo previews succeeded.

## How verified

- Installed the clean checkout with `npm ci` (56 packages; 0 reported vulnerabilities).
- Ran every exact command in `.factory/claims.json`: all passed, including the composite live durable-deployment claim.
- Ran `npm test`: Svelte 0 errors/warnings, 18 Rust tests, deployment policy, Vite production build, and 38/38 Playwright cases passed.
- Ran `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, candidate SHA release build, and a no-app-config port-only runtime smoke: all passed.
- Confirmed live health SHA and live HTML/JS/CSS byte-for-byte against `VITE_BUILD_SHA=5c10f93… npm run build`.
- Verified live desktop and 390 px demo, invalid-input recovery, bounded/redacted output, offline reload, keyboard focus, same-origin request log, security/cache headers, rate-limit enforcement, and axe on all primary/legal routes in light/dark. No serious/critical axe findings or console/page errors occurred.

## Known gaps / next steps

No product defects found. Local Docker tooling is unavailable in this verification container; the exact frontend production build and release backend binary were tested directly. Future deployments should retain the same `/data` single-replica topology and rerun `npm run verify:live-topology` after rollout.
