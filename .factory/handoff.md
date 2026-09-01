# Repair handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-repair-8`
Verifier report: `9f57283cf0eb978f237c93df5acb6f70546afb94` / `.factory/verification-9.md`
Rejected candidate: `e47c898b7d069a182943390b07f1c3459e4c7673`
Repair implementation: `bc40d6c09ce55470a269418332f239dac97c6978`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result

The release blocker is repaired, covered by an exact outbound-capture claim, pushed, and deployed. Slack requests retain their readable `text` and now carry the complete bounded, recursively redacted evidence envelope at the top level, including `evidence`, bounds metadata, and the verifiable `signature`.

The live repair is revision `sf-alert-evidence-envelope--0000025`. Live `/health` reports `bc40d6c09ce55470a269418332f239dac97c6978`.

## Failure reproduction

Before editing the candidate, a local Slack capture endpoint received a two-row alert configured with a one-item cap and nested `email` and `token` secrets. The relay response contained one redacted evidence row and `hmac-sha256=c1900374…`, but the captured Slack body was exactly one `text` field:

```json
{"text":"Evidence sealed · checkout-api\ntimeout\nFirst seen 2026-09-01T00:00:00Z · 1 items · fingerprint cb07e6ce1c8154bb"}
```

There was no `evidence` array or `signature` field in that destination request, matching verification 9.

## Repair and regression coverage

- Slack serialization now starts from the same signed `EvidenceEnvelope` sent to automation and email-webhook destinations, then adds Slack's readable `text` field. The body therefore carries the exact excerpt that was bounded, recursively redacted, fingerprinted, and signed.
- Added `@claim:slack-delivery` to `.factory/claims.json`. The manifest now has 25 unique claims and exactly 25 tagged regressions.
- `claim_slack_destination_carries_bounded_redacted_signed_evidence` configures a Slack route through the protected API, relays two evidence rows to a real local HTTP capture server, and inspects the captured request.
- The regression confirms one retained row, `truncated: true`, no private values, recursive `[REDACTED]` replacements, at most 1,024 evidence bytes, a body signature matching the signature header, and a successful HMAC-SHA256 recomputation from the captured envelope.
- README now states the tested Slack body behavior. The landing copy and `.factory/copy-audit.md` did not change.

## Local verification

- `npm ci` installed 56 packages and reported 0 vulnerabilities.
- Every command in `.factory/claims.json` was run verbatim before the full suite. All 25 claims passed.
- `npm test` passed: Svelte found 0 errors and 0 warnings; 23 Rust tests passed; deployment and claims-manifest policies passed; all 52 Playwright cases passed on desktop Chromium and a 390 × 844 mobile viewport.
- The browser suite covers keyboard skip/focus, route announcements, 44 px targets, 200% text, light/dark axe checks, no console errors, privacy request capture, offline reload, service-worker update, response/cache headers, invalid-input recovery, and rate limiting.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --locked -- -D warnings` passed.
- `VITE_BUILD_SHA=bc40d6c09ce55470a269418332f239dac97c6978 npm run build` produced `dist/`.
- `BUILD_SHA=bc40d6c09ce55470a269418332f239dac97c6978 cargo build --release --locked` passed.
- Release assets: JavaScript 69,642 bytes raw / 25,729 bytes gzip; CSS 18,791 bytes raw / 5,194 bytes gzip; fonts 115,560 bytes; mobile hero 40,982 bytes.
- A release binary launched from a fresh directory with only `PATH` and `PORT=4191`. It generated SQLite and three credentials, reported generated configuration without values, set every credential file to mode 600, and returned the full repair SHA from `/health`.
- Local `verify-url.sh` loaded in 590 ms with title, `lang=en`, one `h1`, a `main`, complete image alternatives, labelled buttons, and zero console errors.
- Local Lighthouse mobile scored 97 performance, 100 accessibility, 100 best practices, and 100 SEO. FCP was 1.26 s, LCP 2.46 s, TBT 74 ms, and CLS 0.

## Deployment and live evidence

- Azure Container Registry build `ch1q8` completed successfully for `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:bc40d6c09ce5`.
- The first container-app patch met a transient Azure `CustomDomainLockConflict`. Retrying the same scoped update with the completed image succeeded; no build or data change was lost.
- Scoped topology verification passed for revision `sf-alert-evidence-envelope--0000025`: single revision mode, min/max replicas 1/1, one running replica, and the factory-managed `alert-evidence-envelope-data` storage mounted at `/data`.
- The deployment verifier completed 20/20 fresh-connection demo previews.
- Live `/health` returns the full repair SHA. All 18 production files match the candidate-stamped local `dist/` files byte-for-byte by SHA-256.
- Live `verify-url.sh` returned HTTP 200, loaded in 636 ms, found zero console errors, and confirmed the title, language, one `h1`, `main`, alt text, and labelled buttons.
- Live HTML returns `no-cache`; hashed JS returns one-year immutable caching. HSTS, `nosniff`, `DENY`, `no-referrer`, and the response-header CSP with `frame-ancestors 'none'` are present.

## Commands

```sh
npm ci
# Run every test command listed in .factory/claims.json.
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=$(git rev-parse HEAD) npm run build
BUILD_SHA=$(git rev-parse HEAD) cargo build --release --locked
npm run deploy
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot $(git rev-parse HEAD) alert-evidence-envelope-data
```

## Known gaps

- Docker, Podman, and Buildah are unavailable in this worker, so no local image build ran. Azure Container Registry built the repository Dockerfile successfully.
- This product has no package consumer, sign-in flow, or runtime AI feature. Consumer-install, Entra identity, and model-gateway checks do not apply.
