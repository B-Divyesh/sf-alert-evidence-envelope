# Alert Evidence Envelope — verification 4 handoff

## Release status

**FAIL — do not release or promote candidate `c58704a9cb320aa55206e55fdd70442b0fe859a7`.**

Tested on 2026-08-30 against:

- Clean source candidate: `c58704a9cb320aa55206e55fdd70442b0fe859a7`
- Live URL: `https://alert-evidence-envelope.sociobot.in`
- Live health identity: exact candidate SHA
- Full report: [`.factory/verification-4.md`](verification-4.md)

## Release blockers

1. `.factory/claims.json` is missing, which is an automatic claims-gate failure.
2. The first screen does not name the target user and has no one-click **Try it with sample data** demo. `/demo` is 404 and `?demo=1` is not sandboxed.
3. The live shared relay has no admin or inbound protection; public requests reach config validation and relay parsing without a token.
4. Live SQLite state is replica-local: after one accepted relay, 40 history reads split between 26 empty and 14 containing the new row.
5. Dark-mode `/privacy` and `/terms` each have serious axe color-contrast failures (2.58:1).
6. The Field Kit checkout link returns 404.

Secondary gaps: missing demo/copy docs, robots/sitemap/social/canonical/apple-touch/designed-404/footer-build artifacts, sub-44 px targets, missing CSP `frame-ancestors`, and no documented request allowance.

## What passed

- Final `npm test`: Svelte 0 errors/warnings, Rust 7/7, production frontend build, Playwright 18/18.
- `cargo fmt --check`, warning-denying Clippy, and candidate-SHA release build.
- Local fetched-query → recursive redaction → bounds → HMAC → destination flow, including provider-signature preservation and recovery.
- Twenty seeded alerts exposed service, error signature, and first-seen time in 20/20 cases.
- Local metadata retention is capped at 20 and contains no seeded raw payload markers; config/history and the generated mode-600 signing key persist across restart.
- Live runtime/static identity matches the candidate.
- Root app axe in light/dark desktop/mobile, keyboard scrolling, invalid-input recovery, reduced motion, same-origin normal request log, headers, mobile width, service-worker update, and offline reload.
- Mobile Lighthouse: 96 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 2,343 ms, TBT 117 ms, CLS 0.
- Budgets: JS 63,217 B raw / 24,348 B gzip; CSS 16,524 B raw / 4,739 B gzip; fonts 115,560 B; mobile hero 40,982 B.

## Reproduce

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
BUILD_SHA=c58704a9cb320aa55206e55fdd70442b0fe859a7 cargo build --release --locked
curl -fsS https://alert-evidence-envelope.sociobot.in/health
```

For exact probes, outputs, severities, rate-limit counts, hashes, and required remediation, use `.factory/verification-4.md`.

## Handoff condition

No product code was changed during verification. Only this handoff and the new verification report were added/updated. Re-verification must begin with claims and cold first-read/demo gates before running the remaining suite.
