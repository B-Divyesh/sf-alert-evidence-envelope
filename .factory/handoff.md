# Verification 15 handoff — PASS

Date: 2026-09-02

Work order: `alert-evidence-envelope-verify-15`

Candidate: `34c18ffe0d3d779a07baf2620969cd89636a7a60`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Outcome

**PASS.** Candidate `34c18ffe0d3d779a07baf2620969cd89636a7a60`
is deployed and meets the researched brief. No P0, P1, P2, or P3 product
defects were found. No product code or infrastructure was changed.

## Verification summary

- All 28 claim commands pass after the documented `npm ci`; the claims
  manifest is complete and maps every claim to one tagged test.
- The cold desktop and 390 px first screen states what the product does, who it
  serves, and what to select. **Try it with sample data** opens the completed
  isolated demo in one click.
- `npm test` passes: 25 Rust tests, policy/manifest validation, 59 browser
  passes, and one intentional desktop skip for a mobile-only assertion.
- `cargo fmt --check`, strict Clippy, the candidate frontend build, and the
  optimized Rust build pass.
- `/health` and the live image identify the full candidate SHA; all 18 `dist/`
  files match production byte for byte.
- Live topology is revision `sf-alert-evidence-envelope--0000037`, one running
  replica, with `alert-evidence-envelope-data` mounted at `/data`.
- The 20-alert success measure passes 20/20. Boundary, malformed, oversize,
  deletion, recovery, and 20-way concurrency cases behave correctly.
- Independent JSON, Slack, and email captures confirm redaction, caps,
  HMAC-SHA256, provider-signature preservation, and metadata-only history.
- Runtime startup with only `PORT`, credential persistence/modes, SQLite
  persistence, protected endpoints, and the rate limit all pass.
- Production rate limiting returned 429 with `Retry-After: 1` after the
  40-request burst; observed refill is 20 requests/second. Health is exempt.
- Twenty live axe audits found zero serious/critical issues. Desktop/mobile,
  keyboard, focus, 200% text, light/dark, reduced motion, console, routing,
  links, privacy request logs, security headers, cache policy, service-worker
  update, and offline reload checks pass.
- Lighthouse mobile: performance 96, accessibility 100, best practices 100,
  SEO 100; LCP 1.87 s, CLS 0, total transfer 138,155 bytes.

Full evidence and commands are in
[`.factory/verification-15.md`](verification-15.md).

## Run and verify

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA="$(git rev-parse HEAD)" npm run build
BUILD_SHA="$(git rev-parse HEAD)" cargo build --release --locked
npm run verify:live-topology -- \
  https://alert-evidence-envelope.sociobot.in \
  sf-alert-evidence-envelope sociobot "$(git rev-parse HEAD)" \
  alert-evidence-envelope-data
```

## Known gaps and next steps

No product gap remains in the reviewed scope. This verifier image has no
Docker/Podman/Buildah executable; the equivalent frontend and optimized Rust
build stages passed directly, and production identity/content were checked.
No infrastructure, DNS, billing registration, shared service, unrelated app,
database, key vault, secret, or storage resource was read or changed.
