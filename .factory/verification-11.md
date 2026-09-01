# Verification 11 — PASS

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-11`
Candidate: `ff56488761e3922e8fe788807fcda37de6cc7cc5`
Live URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** The candidate meets the researched brief: it produces a bounded,
recursively redacted, HMAC-signed incident evidence envelope, with a
one-click isolated sample and the protected real-route workflow. No release
blocking defect was found.

## Cold first read

Opened the live URL in a fresh desktop browser context. The first screen says:

- **What it does:** “Send safe evidence with every alert.”
- **Who it is for:** “For on-call engineers and webhook consumers who need
  incident context without another dashboard login.”
- **What to do first:** the visible primary link is “Try it with sample data,”
  with adjacent text explaining that it opens a signed, redacted envelope in
  an isolated workspace.

This passes the plain-words and one-click sample requirements. The live demo
completed without setup.

## Required claims check

From this clean checkout, `npm ci` installed the locked dependencies (56
packages, 0 audit vulnerabilities). I then ran every command in
`.factory/claims.json` through the shipped demo/test entry point. All 25
claims passed (the individual command log recorded exit status 0 for each):

`demo-envelope`, `mobile-demo-result`, `bounded-redacted-signed`,
`query-fingerprint`, `fixed-query-source`, `provider-signature`,
`slack-delivery`, `isolated-demo`, `raw-not-retained`, `history-limit`,
`protected-real-apis`, `credential-storage`, `preview-no-history`,
`per-route-isolation`, `no-tracking`, `offline-demo`, `license-transport`,
`local-policy-presets`, `license-throttle`, `free-core`,
`field-kit-purchase`, `license-revocation`, `provenance-license`,
`durable-deployment`, and `rate-limit`.

## Local build and tests

| Check | Result |
| --- | --- |
| `npm test` | PASS — Svelte check 0 errors/0 warnings; 23 Rust tests; deployment/claims-manifest policy checks; 54 Playwright tests (desktop and 390 px mobile). |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=ff56488761e3922e8fe788807fcda37de6cc7cc5 npm run build` | PASS — produced `dist/`. |
| Bundle budget | PASS — initial JS 72 KB raw / 26.1 KB gzip; CSS 19.2 KB raw / 5.33 KB gzip; self-hosted fonts total 116 KB; mobile hero 44 KB. |

The verifier container has no `docker` executable, so a local `docker build`
could not be run. This is an environment limitation, not a product failure:
the deployed backend reports the candidate identity and the locally produced
candidate JS, CSS, and service-worker files matched the corresponding live
files byte-for-byte by SHA-256.

## Live end-to-end, privacy, and backend evidence

- `GET /health` returned
  `{"build":"ff56488761e3922e8fe788807fcda37de6cc7cc5","status":"ok"}`.
- A real isolated live demo session returned 200 with service
  `qa-checkout-api`, error `timeout`, first-seen
  `2026-09-01T00:00:00Z`, a recursively redacted `token`, and an
  `hmac-sha256=` signature. A zero item cap returned 400 with the stated
  range error; malformed JSON returned 400; deleting the demo returned 204
  and subsequent preview returned 404.
- The live frontend JS, CSS, and service worker matched the local candidate
  SHA-256 values. The live health build identity matches the tested commit.
- A 60-request burst to `/api/v1/config` using one forwarded IP produced 47
  401 responses and 13 rate-limited 429 responses. Every observed 429 carried
  `Retry-After: 1`; a different forwarded IP received the expected 401. The
  documented allowance is a 40-request burst with a 20 requests/second refill.
- Complete desktop and 390 × 844 sample flows made requests only to
  `https://alert-evidence-envelope.sociobot.in`; no analytics, advertising,
  external scripts, or hosted-font origin was requested. This confirms the
  normal-demo privacy promise.
- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with `no-cache`;
  `/health` returned `no-store`; the content-hashed JS returned
  `public, max-age=31536000, immutable`. Responses included HSTS, nosniff,
  DENY framing, no-referrer, and a response-header CSP with
  `frame-ancestors 'none'`.

## Accessibility, mobile, and performance

- Fresh live desktop and 390 px demo flows had no console or page errors.
- Playwright axe found **0 serious/critical** violations on both live demo
  sizes. The committed full suite also covers light/dark legal pages,
  skip-link focus, keyboard JSON scrolling, 44 px controls, 200% text size,
  offline reload, service-worker update, and reduced motion.
- On live mobile, document width equalled the 390 px viewport (no horizontal
  overflow). `prefers-reduced-motion: reduce` was active during both live
  flows.
- Two independent mobile Lighthouse runs (Chrome with simulated throttling)
  passed the performance target:

| Run | Performance | Accessibility | Best practices | SEO | FCP | LCP | TBT | CLS |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- | --- |
| 1 | 94 | 100 | 100 | 100 | 1.4 s | 1.9 s | 260 ms | 0 |
| 2 | 99 | 100 | 100 | 100 | 1.4 s | 1.8 s | 110 ms | 0 |

The passing 1.8–1.9 s LCP results resolve the previous verification's
sub-2.5-second release blocker.

## Defects

None found. No severity P0–P3 defects are open.
