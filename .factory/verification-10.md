# Verification 10 — FAIL

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-10`
Candidate: `eeb7d38b2022ec3ed3079f6da67aa7edfce270fb`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL — do not accept this candidate yet.** All functional, privacy, deployment, accessibility, and identity checks passed. Two independent stable mobile Lighthouse measurements, however, measured LCP at 2.7 s and 2.8 s. The product performance contract sets an LCP target below 2.5 s. This is a P2 release-blocking quality finding.

## First-read check

The cold live first screen passes the plain-language and demo requirements.

- What it does: “Send safe evidence with every alert.”
- Who it is for: “For on-call engineers and webhook consumers who need incident context without another dashboard login.”
- First action: “Try it with sample data.”
- The adjacent text explains that the sample opens a signed, redacted envelope in an isolated workspace.

## Release-blocking finding

| Severity | Finding | Evidence | Required correction |
| --- | --- | --- | --- |
| P2 | Mobile LCP does not meet the below-2.5-second performance target. | Two clean Lighthouse mobile runs using the installed Playwright Chromium completed without runtime errors: performance 92 / LCP 2.7 s and performance 91 / LCP 2.8 s. | Reduce the mobile LCP to below 2.5 s in repeatable Lighthouse mobile measurement, then rerun independent verification. |

The first Lighthouse attempt recorded performance 88 and a Chromium target crash, so it is not used as the basis for the finding. It did show 3.1 s LCP, consistent with the stable reruns being above the target.

## Required claims check

The checkout started at the candidate SHA. `npm ci` installed 56 locked packages with 0 reported vulnerabilities. `.factory/claims.json` exists with 25 claims, and every listed command was run through the product demo entry point before the broader suite. All passed.

| Claim | Result |
| --- | --- |
| `demo-envelope` | PASS |
| `mobile-demo-result` | PASS |
| `bounded-redacted-signed` | PASS |
| `query-fingerprint` | PASS |
| `fixed-query-source` | PASS |
| `provider-signature` | PASS |
| `slack-delivery` | PASS |
| `isolated-demo` | PASS |
| `raw-not-retained` | PASS |
| `history-limit` | PASS |
| `protected-real-apis` | PASS |
| `credential-storage` | PASS |
| `preview-no-history` | PASS |
| `per-route-isolation` | PASS |
| `no-tracking` | PASS |
| `offline-demo` | PASS |
| `license-transport` | PASS |
| `local-policy-presets` | PASS |
| `license-throttle` | PASS |
| `free-core` | PASS |
| `field-kit-purchase` | PASS |
| `license-revocation` | PASS |
| `provenance-license` | PASS |
| `durable-deployment` | PASS |
| `rate-limit` | PASS |

## Local quality gates

| Check | Result |
| --- | --- |
| `npm test` | PASS — Svelte check: 0 errors and 0 warnings; 23 Rust tests passed; deployment and claim-manifest policy checks passed; Playwright reports 52 passed and 0 failed. |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=eeb7d38b2022ec3ed3079f6da67aa7edfce270fb npm run build` | PASS — produced `dist/`. |
| `BUILD_SHA=eeb7d38b2022ec3ed3079f6da67aa7edfce270fb cargo build --release --locked` | PASS |
| Bundle sizes | PASS — JS 69,642 B raw / 26,090 B gzip; CSS 18,791 B raw / 5,210 B gzip; fonts 115,560 B; mobile hero 40,982 B. |

## Live deployment identity and backend checks

- `GET /health` returned `{"build":"eeb7d38b2022ec3ed3079f6da67aa7edfce270fb","status":"ok"}`.
- Every one of the 18 files in the candidate-stamped local `dist/` tree matched the live file byte-for-byte by SHA-256.
- The scoped topology verifier passed: revision `sf-alert-evidence-envelope--0000026`, single revision mode, one active and running replica, image `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:eeb7d38b2022`, and product data mounted at `/data` from `alert-evidence-envelope-data`.
- The topology verifier completed 20 fresh-connection demo previews successfully.
- A separate 20-way concurrent create/preview/delete check completed 20/20 successful, redacted demo previews.
- A normal demo preview returned `checkout-api`, `payment authorization timed out`, `2026-08-27T14:32:08Z`, recursively redacted `email` and nested `token` fields, a valid HMAC-SHA256-format signature, two evidence items, and 225 evidence bytes.
- A one-item boundary request returned status 200 with one item and `truncated: true`.
- Invalid JSON returned 400 with `request body must be valid preview JSON`; a zero item cap returned 400 with the permitted range. In the live UI, “Restore valid sample” returned the preview to a signed `checkout-api` result with no console or page error.
- Deleting an isolated demo session returned 204; its later preview returned 404.
- A 60-request live burst from one forwarded client yielded 18 responses with status 429 and `Retry-After: 1`; an independent forwarded client received the expected 401 credential response. The observed allowance is a 40-request burst with the documented 20 requests/second refill.

## Privacy, browser, accessibility, and headers

- The complete normal demo request log contained only `https://alert-evidence-envelope.sociobot.in`; no analytics, advertising, hosted font, or third-party script request was observed.
- The live service worker cache was `envelope-shell-v5`. A fresh 390 × 844 context reloaded `/demo` offline with the cached sample, visible offline-ready state, no failed requests, and no API or health request.
- `verify-url.sh` passed live in 690 ms with no console/page errors, title present, `lang=en`, one `h1`, a `main`, no missing image alternatives, and no unlabelled buttons.
- Fresh ordinary `/demo` loading produced no console or page errors. The temporary 404 console response observed while deliberately opening the designed missing-page route was not present on ordinary routes.
- Axe found 0 serious or critical findings on desktop `/` and 390 px dark `/privacy` and `/terms`, plus the 390 px designed 404.
- Keyboard Tab reached the visible “Skip to main content” link first with a solid focus outline. On the 390 px demo, `scrollWidth` equalled the 390 px viewport and every measured interactive target was at least 44 × 44 CSS px.
- Reduced-motion mode matched and limited the largest measured transition duration to 0.00001 s.
- `/`, `/demo`, `/privacy`, and `/terms` returned `no-cache`; `/health` and `/api/v1/config` returned `no-store`; the content-hashed JavaScript returned `public, max-age=31536000, immutable`.
- Browser response headers included HSTS, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and a response-header CSP containing `frame-ancestors 'none'`.
- Internal site links returned their intended 200 pages. The deliberate missing-page route returned its designed 404 page. Metadata, canonical URL, social card, robots file, sitemap, privacy page, terms page, MIT license, demo documentation, design record, and copy audit are present.

## Lighthouse mobile measurement

| Run | Performance | Accessibility | Best practices | SEO | FCP | LCP | TBT | CLS |
| --- | ---: | ---: | ---: | ---: | --- | --- | --- | --- |
| Stable run 1 | 92 | 100 | 100 | 100 | 2.7 s | 2.7 s | 0 ms | 0 |
| Stable run 2 | 91 | 100 | 100 | 100 | 2.8 s | 2.8 s | 0 ms | 0 |

No product code was changed during this verification. Only the scoped `sf-alert-evidence-envelope` container application was read for the topology check; no unrelated product, storage, database, secret store, or staging resource was read or changed.

## Next step

Correct the repeatable mobile LCP result, preserve the existing passing behavior, and request a new verification.
