# Verification 7 — PASS

Date: 2026-08-30
Work order: `alert-evidence-envelope-verify-7`
Candidate: `5c10f93da6f4e95c64ed9f9cc70b06b81f08df83`  
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**PASS — candidate is suitable for release.** No P0, P1, P2, or P3 product defects were found in this independent verification. The live deployment reports the candidate SHA, uses one running replica with the product's Azure File mount at `/data`, and its production HTML/CSS/JS exactly matches a build of this commit with `VITE_BUILD_SHA` set to the candidate.

The previous deployment-only failure is repaired: the exact durable-deployment claim passed, including 20 successful create-session → fresh-connection preview pairs. Cold desktop and 390 px mobile first clicks both opened a working isolated sample envelope without console or page errors.

## Required first checks

The clean checkout was at the candidate SHA with no pre-existing worktree changes. After `npm ci` (56 packages; 0 reported vulnerabilities), every command in `.factory/claims.json` was run verbatim before other QA.

| Claim | Result | Fresh evidence |
| --- | --- | --- |
| `demo-envelope` | PASS | `npm run test:claims -- --grep @claim:demo-envelope`: 2/2 desktop/mobile. |
| `bounded-redacted-signed` | PASS | Locked Rust claim test passed. |
| `fixed-query-source` | PASS | Locked Rust claim test passed. |
| `isolated-demo` | PASS | Locked Rust claim test passed. |
| `raw-not-retained` | PASS | Locked Rust claim test passed. |
| `provider-signature` | PASS | Locked Rust claim test passed. |
| `history-limit` | PASS | Locked Rust claim test passed. |
| `protected-real-apis` | PASS | Locked Rust claim test passed. |
| `no-tracking` | PASS | `npm run test:claims -- --grep @claim:no-tracking`: 2/2. |
| `offline-demo` | PASS | `npm run test:claims -- --grep @claim:offline-demo`: 2/2. |
| `rate-limit` | PASS | `npm run test:claims -- --grep @claim:rate-limit`: 2/2. |
| `local-policy-presets` | PASS | `npm run test:claims -- --grep @claim:local-policy-presets`: 2/2. |
| `field-kit-purchase` | PASS | `npm run test:claims -- --grep @claim:field-kit-purchase`: 2/2. |
| `durable-deployment` | PASS | All three locked Rust tests, deployment-policy test, and live topology verification passed. |

The cold first screen passes the plain-words/demo gate:

- Does: **“Send safe evidence with every alert.”**
- For: **“For on-call engineers and webhook consumers who need incident context without another dashboard login.”**
- First action: **“Try it with sample data.”** The adjacent sentence says the click opens a signed, redacted envelope in an isolated workspace.

## Clean-checkout quality gates

| Gate | Result |
| --- | --- |
| `npm test` | PASS — Svelte check: 0 errors/warnings; Rust: 18 passed; deployment policy passed; production Vite build passed; Playwright: 38/38 passed. |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `npm run build` | PASS — `dist/` created; JS 66,356 B raw / 24,940 B gzip; CSS 17,557 B raw / 4,952 B gzip. |
| `VITE_BUILD_SHA=<candidate> npm run build` | PASS — live root HTML and JS/CSS SHA-256 values matched byte-for-byte. |
| `BUILD_SHA=<candidate> cargo build --release --locked` | PASS |
| Port-only runtime | PASS — with no app configuration other than `PORT` (and shell `PATH`), the release binary generated 600-mode signing/admin/inbound files, logged generated sources without secret values, and returned the candidate SHA from `/health`. |

Docker tooling is unavailable in this verifier container, so a local image build was not attempted. The release backend and exact frontend production build were exercised directly.

## Independent end-to-end evidence

### Product job

The live demo produced a signed envelope containing the brief's required service, error signature, and first-seen time. It recursively redacted the sample email/token, stayed within its item and byte caps, and exposed a 64-hex `hmac-sha256=` signature. Resetting the demo changed its session ID. A deliberately invalid `{` sample displayed **“Sample alert is not valid JSON. Check commas and quotes.”**; Restore valid sample followed by Build safe preview recovered successfully.

Direct live demo API evidence covered boundaries and invalid input: a two-row alert sent with `max_items: 1` and `max_bytes: 1024` returned service `orders`, error `timeout`, first seen `2026-08-30T01:00:00Z`, one 38-byte redacted item, and `truncated: true`. Malformed preview JSON returned 400 with `request body must be valid preview JSON`.

Malformed unauthenticated requests to config, history, config update, preview, and inbound relay returned 401 before body parsing. This product does not require sign-in; Entra validation is therefore not applicable.

### Live deployment and persistence

`npm run verify:live-topology` reported:

```json
{
  "build": "5c10f93da6f4e95c64ed9f9cc70b06b81f08df83",
  "revision": "sf-alert-evidence-envelope--0000019",
  "topology": {
    "revisionMode": "Single",
    "minReplicas": 1,
    "maxReplicas": 1,
    "runningReplicas": 1,
    "mountPath": "/data",
    "storage": "alert-evidence-envelope-data"
  },
  "freshConnectionPreviews": 20
}
```

`GET /health` returned the exact candidate SHA. The live root page, `index-VV1uz3W_.js`, and CSS matched the candidate's SHA-aware production build byte-for-byte.

### Privacy, transport, rate limit, and performance

- A fresh 390 px demo context made requests only to `https://alert-evidence-envelope.sociobot.in`: document, self-hosted fonts/assets, `/health`, demo-session creation, and demo preview. No analytics, advertising, hosted font, or third-party script request occurred.
- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with `no-cache`; `/health` and API responses used `no-store`; hashed assets used `public, max-age=31536000, immutable`.
- Responses sent HSTS, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and a response-header CSP with `frame-ancestors 'none'`.
- The initial JS budget is 24,940 B gzip (under 200 KB); CSS is 4,952 B gzip; self-hosted fonts total 115,560 B; the mobile hero is 40,982 B.
- A live 60-request same-client burst to `/api/v1/config` yielded 43 × 401 and 17 × 429. Every 429 included `Retry-After: 1`; a different forwarded IP immediately received 401, not 429. The observed 43 accepted requests are consistent with the documented 40 burst plus 20/s refill during the 0.5-second request window.

### Browser, accessibility, and PWA

- Fresh desktop 1440×900 and mobile 390×844 first-click demos succeeded. Both had one `h1`, a `main`, no horizontal overflow, and no console/page errors.
- Playwright axe found zero serious/critical violations on `/`, `/demo`, `/privacy`, and `/terms`, in light/dark desktop and 390 px states.
- Keyboard smoke: the skip link is the first focus target with a solid focus outline. The signed JSON is focusable, has a solid outline, and at 390 px scrolled from 0 to 163 with PageDown.
- A dedicated fresh context loaded `/demo`, awaited service-worker readiness, went offline, reloaded, and displayed the cached banner, **“Offline sample ready. Demo data was not stored.”**, and `checkout-api`.

## Defects by severity

None found.

## Scope notes

This is a web-with-backend, not a package/CLI, so clean consumer pack/install testing does not apply. The researched brief expressly excludes LLM summarization, so no AI-feature omission is raised.
