# Verification handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-8`
Candidate: `6863d27421a6b7b44f2b006f77a648d1dcfe62c1`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result

**FAIL — do not release this candidate.**

The full evidence is in `.factory/verification-8.md`. No product code was changed.

## Release-blocking findings

1. `npm run test:claims -- --grep @claim:offline-demo` fails on desktop and 390 px, including retries. `npm test` repeats the same two failures (50 browser tests pass, 2 fail). The cached envelope is present offline, but the required offline-ready state is not rendered and the reload logs a disconnected network request.
2. The first clean invocation of `npm run test:claims -- --grep @claim:demo-envelope` times out after 120 seconds while Playwright waits for the cold Rust server build. It passes only after the Rust cache is warm.
3. `.factory/claims.json` omits four public, tagged claims: `provider-signature`, `history-limit`, `local-policy-presets`, and `durable-deployment`.
4. `.factory/copy-audit.md` does not contain all current landing-page sentences.
5. Several 390 px touch targets miss the required 44 × 44 px size, including legal footer links; long-lived immutable caching also covers stable, non-versioned font and image URLs.

## What passed

- First screen clearly states the job, audience, and first action; the one-click sample demo works online.
- Svelte check, all 22 Rust tests, deployment policy, formatting, strict Clippy, candidate-stamped Vite build, and optimized Rust build.
- Core demo, recursive redaction, item/byte bounds, fingerprint, signature, invalid-input recovery, protected API boundaries, and 20 concurrent previews.
- Port-only startup with generated 600-mode credentials and correct health identity.
- Live build identity and all served `dist/` file hashes match the candidate. Scoped topology: revision `sf-alert-evidence-envelope--0000022`, one running replica, `/data` mounted.
- Live rate limit: a 60-request same-client burst produced 43 × 401 and 17 × 429; `Retry-After: 1`; another client remained available. Documented allowance is 40 burst with 20 requests/second refill.
- Normal demo requests are same-origin only. Security headers and cache policies are present.
- Zero serious/critical axe findings on primary, demo, legal, and 404 pages; desktop/mobile keyboard, focus, reduced-motion, and overflow checks pass.
- Lighthouse mobile: 95 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 2.2 s.
- Budgets: JS 25,664 bytes gzip; CSS 5,167 bytes gzip; fonts 115,560 bytes; mobile hero 40,982 bytes.

## Verification commands

```sh
npm ci
# Then run every command in .factory/claims.json verbatim.
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=6863d27421a6b7b44f2b006f77a648d1dcfe62c1 npm run build
BUILD_SHA=6863d27421a6b7b44f2b006f77a648d1dcfe62c1 cargo build --release --locked
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot 6863d27421a6b7b44f2b006f77a648d1dcfe62c1
```

Docker, Podman, and Buildah are unavailable in this worker, so no local container image build was run. The exact frontend and optimized backend builds completed directly.
