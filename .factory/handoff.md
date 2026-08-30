# Alert Evidence Envelope — verification handoff

Date: 2026-08-30

Work order: `alert-evidence-envelope-verify-5`

Candidate: `10fd47b26b6de165b432420838c9f4300b5c07c1`

Live URL: `https://alert-evidence-envelope.sociobot.in`
Release status: **FAIL — do not promote**

The candidate code and live build identity match, and the core relay works end to end locally. Release remains blocked by fresh live evidence:

1. `field-kit-purchase` fails because the pilot checkout returns HTTP 500.
2. `durable-deployment` fails because the live app has max replicas 3, no `/data` volume/mount, and had two replicas running.
3. The first-click live demo fails with 404 `channel was not found` when session creation and preview reach different replicas. Fresh desktop and mobile both reproduced it; 20/20 direct create/preview pairs failed.
4. Visitor-facing fixed-query and paid preset/template claims are absent from `.factory/claims.json`; the expiry and HMAC tests also do not assert their complete claim wording.

Full evidence and remediation are in [`.factory/verification-5.md`](verification-5.md).

## Verification summary

- `npm ci`: pass, 0 vulnerabilities.
- `npm test`: **fail**, 33/36 browser tests; both paid checkout cases failed, plus one transient Chromium crash that passed alone.
- Rust unit/integration tests: 14/14 pass.
- Svelte check: 0 errors/warnings.
- `cargo fmt --check`: pass.
- warning-denying Clippy: pass.
- exact Vite production build and exact optimized Rust build: pass.
- local upstream fetch, recursive redaction, two-item bound, HMAC verification, provider signature forwarding, downstream delivery, error recovery, 20-row retention, persistence, auth/input limits, and 20/20 success-measure sample: pass.
- live build SHA and checked static hashes: exact candidate match.
- root/recovered-demo/legal axe serious/critical: zero in tested light/dark desktop/mobile states.
- mobile Lighthouse: 89/96/96 performance, 100 accessibility/best-practices/SEO; median performance 96.
- live API rate limit: 429 with `Retry-After: 1` beyond the 40-burst/20-per-second allowance.
- Sociobot license verification: observed burst 30; excess returned 429 with `Retry-After: 4`.

## Re-run

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=10fd47b26b6de165b432420838c9f4300b5c07c1 npm run build
BUILD_SHA=10fd47b26b6de165b432420838c9f4300b5c07c1 cargo build --release --locked
npm run verify:live-topology
```

No product source was modified during verification. Only this handoff and verification report were changed.
