# Alert Evidence Envelope — verification handoff

## Verification 1 status: **FAIL**

Independent verification on 2026-08-28 tested candidate
`109a9714bddb00ebc26ae28158c709332b1c6352` and
`https://alert-evidence-envelope.sociobot.in`.

Do not release this candidate. The detailed, reproducible report is in
[`verification-1.md`](verification-1.md). Blocking defects are: `/privacy`
and `/terms` return HTTP 404, the UI horizontally overflows by 24px at a 390px
viewport, and live `/health` returns build `development` instead of an
immutable build SHA, so backend candidate identity cannot be confirmed.

The candidate's live JavaScript and CSS hashes do exactly match a fresh local
production build. `npm test` and additional local relay/security checks pass;
the release decision remains FAIL until the three blockers are fixed and
re-verified.

---

# Alert Evidence Envelope — build handoff (builder report, superseded by verification status above)

Date: 2026-08-28

Work order: `alert-evidence-envelope-build-1`
Commit base: `8383c81d39f0471b5c4f870c77bfd181a5df4078`

## What shipped

- A Rust/Axum relay with SQLite configuration and metadata-only delivery history.
- `POST /api/v1/relay/primary` accepts bounded JSON, either extracts embedded evidence or fetches from one configured HTTPS source, recursively redacts configured key names, enforces item and serialized byte caps, computes a source/query SHA-256 fingerprint, signs the compact envelope with HMAC-SHA256, preserves recognized provider signatures in transit, and delivers to JSON automation, Slack, or an email-gateway webhook.
- Upstream responses are stream-read with a 256–512 KB hard ceiling. Inbound bodies stop at 256 KB. URLs require HTTPS except localhost development. Per-IP rate limiting allows a 300-request burst and replenishes at 200 requests/second.
- Optional `ADMIN_TOKEN` and `INBOUND_TOKEN` boundaries use constant-time comparison. Credentials and signing keys are environment-only. Raw alerts and evidence are never written to SQLite or logs.
- A responsive Svelte route builder, no-retention preview, explicit loading/error/offline/empty/success states, delivery ledger, copy controls, and local-only paid redaction presets.
- A $39 one-time Field Kit using the Sociobot hosted checkout, query-string license capture, `sb_license:alert-evidence-envelope` local storage, at-most-daily verification, optimistic offline unlock, invalid-license reconciliation, and paste-to-restore. Core safety, relay, preview, export/copy, and accessibility remain free.
- `/privacy` and `/terms`, full README, MIT license, third-party font notices, Dockerfile, service-worker offline shell, responsive topographic hero, light/dark themes, and reduced-motion behavior.
- The original generated cartographic source and prompt metadata are under `assets/src/`; responsive WebPs ship under `frontend/static/assets/`.

## Verification performed

`npm test` passes from the repository root. It runs:

- `svelte-check`: 0 errors, 0 warnings.
- Rust tests: 4 passed. Coverage includes recursive redaction, item/byte truncation, endpoint validation, all API routes, real forwarding to a local destination, preserved provider and envelope signature headers, and metadata history.
- Vite production build: output is exactly `dist/`, with `dist/index.html` at its root.
- Playwright 1.58.2: 8/8 passed across desktop Chromium and Chromium at 390 × 844. The suite exercises the signed preview, redaction output, offline state and cached-shell reload, legal routes, one-h1/main semantics, page console, and axe WCAG AA in light and dark modes.

Additional checks:

- Factory `verify-url.sh`: HTTP 200, title present, `lang=en`, exactly one `h1`, main landmark present, zero missing alt attributes, zero unlabeled buttons, zero console/page errors; measured local load 575 ms.
- Lighthouse 12.8.2 mobile: **Performance 98, Accessibility 100, Best Practices 100, SEO 100**; LCP 2.3 s, CLS 0, TBT 20 ms, total transfer 270 KiB.
- Initial assets: JS 63.19 KB raw / 24.55 KB gzip, CSS 16.08 KB raw / 4.68 KB gzip, fonts 114 KB total, mobile hero 75 KB WebP. All individual budgets pass (JS ≤200 KB, CSS ≤50 KB, fonts ≤120 KB, hero ≤300 KB).
- Live local relay smoke returned HTTP 202, extracted service/error/first-seen, redacted `email` and `token`, set the query fingerprint and HMAC, and marked the provider signature preserved.
- Load smoke: 500 health requests at concurrency 20 returned 500 HTTP 200 responses in approximately two seconds.
- `npm audit --omit=dev`: 0 production vulnerabilities; full npm audit was also clean after upgrading Svelte/Vite.
- Response headers verified: CSP, `nosniff`, no-referrer, API `no-store`, and immutable caching on static assets.

## Run and deploy

```sh
npm ci
npm test
npm run build
ENVELOPE_SIGNING_KEY='a-long-random-key' cargo run --locked
```

Container build command: `docker build -t alert-evidence-envelope .`

Container listens on `PORT` (default 8080), runs non-root, serves `dist/`, and expects persistent `/data`.

Required production values: a strong `ENVELOPE_SIGNING_KEY`, `ADMIN_TOKEN`, and preferably `INBOUND_TOKEN`. Configure short-lived `UPSTREAM_BEARER_TOKEN`, `DESTINATION_URL`, and `DESTINATION_BEARER_TOKEN` only when needed.

## Known gaps / factory follow-up

- The container recipe was not executed because this worker image has no Docker, Podman, or Buildah binary. Native locked Rust build/tests and the exact Vite artifact passed; the Dockerfile uses Node 22 Alpine and Rust 1.98 Alpine build stages with a non-root Alpine runtime.
- External observability sources, Slack, email gateways, and the production Sociobot billing endpoint were not called with real credentials. HTTP forwarding is integration-tested against a local capture server. The factory must register/switch the billing product at release as specified by the work order.
- The source adapter is deliberately vendor-neutral: fixed GET endpoint plus `q` and `limit`, accepting an array or `{data|results}`. Vendor-specific pagination, POST query bodies, and native SMTP are v2 adapter work; email v1 uses a configured webhook gateway.
- HMAC verification requires compact JSON in schema field order after blanking `signature`; README documents this. A future detached-JWS profile would improve cross-language canonicalization.
- Deployment TLS, ingress allowlisting, DNS, backup policy, and secret rotation remain factory/operator responsibilities.
