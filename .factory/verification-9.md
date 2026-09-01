# Verification 9 — FAIL

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-9`
Candidate: `e47c898b7d069a182943390b07f1c3459e4c7673`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL — do not release this candidate.** The live deployment matches the candidate and the declared checks pass, but the advertised Slack destination does not receive the signed evidence excerpt required by the researched brief. It receives summary text only. Slack delivery is also absent from `.factory/claims.json`, despite being offered as a destination in the route builder.

No product code was changed during verification.

## Release-blocking finding

### P1 — Slack delivery omits the evidence envelope

The acceptance contract requires the relay to attach a bounded, redacted, signed evidence excerpt and forward it to Slack, email, or automation. The route builder offers **Slack incoming webhook** as a destination.

A local capture endpoint received deliveries from the exact candidate release binary:

- The Slack request body contained one key, `text`, with service, error, first-seen time, item count, and fingerprint.
- The Slack request body contained no `evidence` array and no `signature` field.
- The same alert sent through the automation destination contained the complete envelope, including recursively redacted evidence and the HMAC signature.
- The relay did send signature headers, but those headers do not attach the signed excerpt to the Slack message body.

This means the Slack option does not deliver the portable evidence artifact that defines the smallest useful product. There is no `@claim` check for Slack delivery in the 24-entry claim inventory, so the public capability is also unlisted and unverified under the claims contract.

## Required first checks

The checkout started clean at the exact candidate commit. `npm ci` installed 56 packages from the lockfile and reported no known vulnerabilities.

The cold first screen passes the required first-read check:

- What it does: **“Send safe evidence with every alert.”**
- Who it is for: **“For on-call engineers and webhook consumers who need incident context without another dashboard login.”**
- What to select first: **“Try it with sample data.”**
- The adjacent sentence explains that the sample opens a signed, redacted envelope in an isolated workspace.

`.factory/claims.json` exists. After the clean lockfile install, every listed command was run exactly and passed:

| Claim | Result | Confirmed behavior |
| --- | --- | --- |
| `demo-envelope` | PASS | One selection opened the demo and built a bounded, recursively redacted, signed envelope. |
| `mobile-demo-result` | PASS | Summary, redaction, and signed state intersected the initial 390 × 844 viewport. |
| `bounded-redacted-signed` | PASS | Nested fields were removed, item and byte caps applied, and the HMAC recomputed. |
| `query-fingerprint` | PASS | Source and query both changed the deterministic fingerprint. |
| `fixed-query-source` | PASS | The configured source received only the alert query and fixed item limit. |
| `provider-signature` | PASS | The incoming provider signature remained present on the delivered request. |
| `isolated-demo` | PASS | Demo state could not reach protected history and had a 24-hour expiry. |
| `raw-not-retained` | PASS | Closed SQLite bytes did not contain unique inbound or evidence markers. |
| `history-limit` | PASS | Delivery history retained the latest 20 metadata rows. |
| `protected-real-apis` | PASS | Configuration, preview, history, and relay routes required credentials before parsing bodies. |
| `credential-storage` | PASS | Generated credentials used protected files and were absent from browser configuration. |
| `preview-no-history` | PASS | A protected preview did not create a delivery-history entry. |
| `per-route-isolation` | PASS | Routes retained independent URLs, destinations, and redaction lists. |
| `no-tracking` | PASS | The normal demo requested only the product origin. |
| `offline-demo` | PASS | The last sample envelope reloaded offline in a dedicated browser context. |
| `license-transport` | PASS | License verification used an authorization header; presets remained local. |
| `local-policy-presets` | PASS | A named policy persisted locally and restored after reload. |
| `license-throttle` | PASS | A failed verification attempt was not repeated within 24 hours. |
| `free-core` | PASS | Preview, redaction, signing, and copy remained available without a license. |
| `field-kit-purchase` | PASS | The page showed $39 USD once and the official Sociobot checkout URL. |
| `license-revocation` | PASS | An invalid license removed Field Kit controls while leaving the free relay available. |
| `provenance-license` | PASS | Asset provenance and the MIT license were present. |
| `durable-deployment` | PASS | Deployment policy requires a non-root container, one replica, and SQLite under `/data`. |
| `rate-limit` | PASS | The declared 40-request burst, refill, client separation, 429 status, and `Retry-After` were observed. |

## Clean local gates

| Check | Result |
| --- | --- |
| `npm test` | PASS — Svelte reported 0 errors and 0 warnings; 22 Rust tests passed; policy checks passed; all 52 browser cases completed. One mobile case passed on its configured retry after the browser process closed, then passed 3/3 independent repeats. |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=e47c898… npm run build` | PASS — `dist/` produced. |
| `BUILD_SHA=e47c898… cargo build --release --locked` | PASS |
| Port-only runtime | PASS — with only `PORT` and the executable path, the server created SQLite and three credentials, served `/`, and returned the full candidate SHA from `/health`. Credential files were mode 600. Logs reported generated configuration without values. |
| Container build | Not run — Docker, Podman, and Buildah are unavailable in this worker. The matching live container and exact optimized build were verified separately. |

The final frontend contains 69,642 bytes of JavaScript (25,695 bytes gzip), 18,791 bytes of CSS (5,194 bytes gzip), 115,560 bytes of fonts, and a 40,982-byte mobile hero image. All supplied budgets pass.

## Live deployment and product checks

- `/health` returned build `e47c898b7d069a182943390b07f1c3459e4c7673`; the footer showed `e47c898b7d06`.
- All 18 files in the candidate-stamped local `dist/` matched their live counterparts byte-for-byte by SHA-256.
- The scoped topology check passed for `sf-alert-evidence-envelope`: revision `sf-alert-evidence-envelope--0000024`, single revision mode, one running replica, one-replica limits, and `/data` mounted.
- Twenty fresh-connection demo previews passed in the topology check.
- Twenty independently seeded alerts produced the correct service, error signature, and first-seen time in 20/20 cases; all 20 private email fields were redacted.
- A normal preview returned a signed envelope with the expected summary and nested redaction.
- A one-item boundary returned one evidence row, 74 evidence bytes, and `truncated: true`.
- Invalid JSON returned 400 with `request body must be valid preview JSON`. A zero item cap returned 400 with the allowed range. The browser’s **Restore valid sample** action recovered successfully.
- Deleting a demo workspace returned 204; a later preview for it returned 404.
- Twenty concurrent session and preview pairs returned 20/20 successful, correctly redacted envelopes.
- A 60-request live burst from one client returned 42 credential responses and 18 rate-limit responses. Every checked limited response included `Retry-After: 1`; a different client remained available. The count reflects the documented 40-request burst plus refill during the request window.

## Privacy, PWA, accessibility, and performance

- A complete live demo and invalid-input recovery flow made 16 requests, all to the product origin. No analytics, advertising, hosted-font, or third-party script request occurred. No console or page error occurred.
- The live service worker controlled the page, updated successfully, and used cache `envelope-shell-v5`. A fresh 390 px demo reloaded offline with the cached envelope, visible offline-ready state, no failed request, and no API or health request.
- `/`, `/demo`, `/privacy`, and `/terms` return `no-cache`; API and health responses return `no-store`; the content-hashed JavaScript and CSS return one-year immutable caching; stable fonts, images, and the service worker revalidate.
- Live responses include HSTS, `nosniff`, `DENY`, `no-referrer`, and a response-header CSP with `frame-ancestors 'none'`.
- Desktop and 390 px checks found one `h1`, one `main`, `lang=en`, route-specific titles, no missing image text alternatives, no horizontal overflow, and no ordinary-load console errors.
- Keyboard use reached the visible skip link first with a 3 px amber focus outline; the next focus after activation was **Try it with sample data**, which worked with Enter.
- Every measured 390 px control was at least 44 × 44 CSS px. The initial mobile demo showed service, error, redaction, and signed state within the 844 px viewport.
- Reduced-motion mode matched, removed the terrain transform, used automatic scrolling, and reduced transitions to 0.01 ms.
- Axe found zero serious or critical findings on `/`, `/demo`, `/privacy`, `/terms`, and the designed 404, including mobile dark mode.
- Lighthouse mobile scored 97 performance, 100 accessibility, 100 best practices, and 100 SEO. FCP was 1.4 s, LCP 2.2 s, TBT 120 ms, and CLS 0.
- Internal links returned their intended pages; the designed missing-page route returned 404, the source link returned 200, and the official checkout returned the expected 303 hosted-checkout redirect.

This product has no sign-in requirement, library or CLI distribution, or runtime AI feature, so Entra, clean-consumer installation, and model-gateway checks do not apply.

Only the scoped `sf-alert-evidence-envelope` application and its revisions were read for deployment topology. No unrelated application, database, secret store, storage account, or service was read or changed.

## Required next steps

1. Make the Slack destination include the bounded, redacted evidence excerpt and a verifiable signature in the delivered Slack payload.
2. Add the public Slack-delivery capability to `.factory/claims.json` with an end-to-end capture check that confirms evidence, redaction, bounds, and signature.
3. Run every claim command first, then the full local and live verification suite before changing the result to PASS.
