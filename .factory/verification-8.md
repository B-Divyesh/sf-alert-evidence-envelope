# Verification 8 — FAIL

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-8`
Candidate: `6863d27421a6b7b44f2b006f77a648d1dcfe62c1`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL — do not release this candidate.** The required offline claim test fails in both desktop and 390 px projects, the first clean invocation of the demo claim times out before Playwright starts, and four public claims are absent from `.factory/claims.json`. The claims contract makes each condition release-blocking.

The deployed core workflow otherwise works and the live deployment matches the candidate. This report is based on fresh local and live evidence; no product code was changed.

## Required first checks

The checkout started clean at the exact candidate SHA. `.factory/claims.json` existed and contained 20 entries. `npm ci` installed 56 packages with no reported vulnerabilities.

The cold first screen passes the plain-language and demo gate:

- What it does: **“Send safe evidence with every alert.”**
- Who it is for: **“For on-call engineers and webhook consumers who need incident context without another dashboard login.”**
- First action: **“Try it with sample data.”** The adjacent sentence explains that it opens a signed, redacted envelope in an isolated workspace.

Every command listed in `.factory/claims.json` was run verbatim after the clean install:

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-envelope` | **FAIL on first clean run** | Playwright timed out after 120 seconds waiting for its Rust web server. It passed once the Rust build cache was warm, including in the later full suite. |
| `mobile-demo-result` | PASS | Browser claim completed successfully. |
| `bounded-redacted-signed` | PASS | Locked Rust test passed. |
| `query-fingerprint` | PASS | Locked Rust test passed. |
| `fixed-query-source` | PASS | Locked Rust test passed. |
| `isolated-demo` | PASS | Locked Rust test passed. |
| `raw-not-retained` | PASS | Locked Rust test passed. |
| `protected-real-apis` | PASS | Locked Rust test passed. |
| `credential-storage` | PASS | Locked Rust test passed. |
| `preview-no-history` | PASS | Locked Rust test passed. |
| `per-route-isolation` | PASS | Locked Rust test passed. |
| `no-tracking` | PASS | Desktop and mobile browser claim passed. |
| `offline-demo` | **FAIL** | Desktop and mobile both failed, including retries: `Offline sample ready. Demo data was not stored.` was absent after offline reload. |
| `license-transport` | PASS | Desktop and mobile browser claim passed. |
| `license-throttle` | PASS | Desktop and mobile browser claim passed. |
| `free-core` | PASS | Desktop and mobile browser claim passed. |
| `field-kit-purchase` | PASS | Desktop and mobile browser claim passed. |
| `license-revocation` | PASS | Desktop and mobile browser claim passed. |
| `provenance-license` | PASS | Desktop and mobile browser claim passed. |
| `rate-limit` | PASS | Desktop and mobile browser claim passed. |

The offline failure was independently reproduced against the live deployment. The shell and cached envelope reload, but the status still says `Envelope signed. Demo data was not stored.` instead of reporting the offline state, and the browser logs `net::ERR_INTERNET_DISCONNECTED` for the attempted request.

## Defects by severity

### P1 — required offline claim test fails

`npm run test:claims -- --grep @claim:offline-demo` fails in desktop and mobile, including retries. `npm test` repeats the same result: 50 browser tests pass and the two offline project cases fail. This violates the explicit rule that every listed claim command must pass.

The cached sample remains visible offline, so the data path partly works. The visible offline-ready state required by the test is unreachable while a successful demo result is rendered: the success panel uses the fixed online message. The offline reload also starts a request that produces a browser console network error.

### P1 — four public claims are missing from the claim inventory

The source contains tagged tests for four claims that are public on the site or in README, but `.factory/claims.json` does not list them:

- `provider-signature` — the page says the provider signature is preserved.
- `history-limit` — the page and privacy notice promise the latest 20 delivery records.
- `local-policy-presets` — the Field Kit and README promise named presets stored locally.
- `durable-deployment` — README promises durable SQLite under `/data`.

All four underlying tests/checks passed during the full verification, but omission from the required inventory is itself a claims-contract failure.

### P1 — a declared claim command is not reliable from a cold checkout

The first post-install invocation of `npm run test:claims -- --grep @claim:demo-envelope` failed because `playwright.config.ts` allows 120 seconds for `cargo run`; the clean Rust build exceeded that limit. A claims command must run successfully from a clean clone, not only after another command warms the build cache.

### P2 — the copy audit is incomplete

`.factory/copy-audit.md` says it extracts landing-page copy but omits many current sentences, including the provider-signature, remote-source, preview, history, and Field Kit statements. This does not satisfy the attached plain-words proof requirement to audit every sentence.

### P2 — several mobile touch targets are smaller than 44 × 44 px

At 390 px, the home navigation `Demo` link measures 40 × 44 px and the route-enabled checkbox itself measures 22 × 22 px. On the directly served privacy and terms pages, footer links measure only 16 px high (`Privacy` 44 × 16; `Terms` 38 × 16). These miss the attached 44 × 44 px touch-target baseline. The repository test checks target height only and therefore does not detect the narrow home link.

### P2 — immutable caching is applied to stable asset URLs

The server gives every `/assets/` and `/fonts/` response, plus `/favicon.svg`, a one-year `immutable` policy. Several such URLs are not content-hashed, including the fonts, hero images, social card, and favicon. A later deployment that changes those files at the same URL can leave returning browsers with stale assets for up to one year.

## Clean local gates

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 56 packages; 0 reported vulnerabilities. |
| `npm test` | **FAIL** — 22 Rust tests pass; Svelte check and deployment policy pass; Playwright reports 50 passed and 2 failed (`offline-demo` desktop/mobile). |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=6863… npm run build` | PASS — `dist/` created. |
| `BUILD_SHA=6863… cargo build --release --locked` | PASS |
| Port-only release runtime | PASS — with only `PORT` plus the executable search path, the server generated its SQLite and three protected credentials, logged generated sources without values, and `/health` returned the full candidate SHA. Generated credential files were mode 600. |
| Container image build | Not run — Docker, Podman, and Buildah are unavailable in this worker. The exact frontend and optimized backend builds passed directly. |

## Live deployment identity and topology

- `/health` returns `6863d27421a6b7b44f2b006f77a648d1dcfe62c1`.
- The live footer displays build `6863d27421a6`.
- Every directly served file in local `dist/` except the separately embedded 404 document matched the corresponding live file byte-for-byte by SHA-256.
- Scoped topology verification passed for `sf-alert-evidence-envelope`: revision `sf-alert-evidence-envelope--0000022`, single revision mode, one running replica, one-replica limits, and the product data volume mounted at `/data`.
- Twenty create-session then fresh-connection preview checks passed.

No unrelated application, database, key vault, storage account, or secret was inspected.

## Independent end-to-end checks

- Desktop 1440×900 and mobile 390×844: the one-click demo completed with no ordinary-load console or page errors and no horizontal overflow.
- The result showed service `checkout-api`, error `payment authorization timed out`, first-seen time, `[REDACTED]`, a query fingerprint, and an `hmac-sha256=` signature.
- Invalid `{` input showed `Sample alert is not valid JSON. Check commas and quotes.`; `Restore valid sample` followed by `Build signed preview` recovered.
- A direct boundary request with two evidence rows, `max_items: 1`, and `max_bytes: 1024` returned one recursively redacted 74-byte item and `truncated: true`.
- Invalid JSON and `max_items: 0` returned useful 400 responses.
- Unauthenticated malformed configuration, history, preview, and relay requests returned 401 before body parsing.
- Twenty concurrent demo session/preview pairs all returned 200 with the expected summary and redaction.
- This product has no sign-in requirement, so Entra authority verification is not applicable. It is not a library or CLI, so clean consumer package testing is not applicable. The brief makes LLM summarization a non-goal, so no AI feature is expected.

## Rate limiting

A fresh live 60-request same-client burst to `/api/v1/config` returned 43 × 401 and 17 × 429. Every checked 429 had `Retry-After: 1`; a different forwarded client immediately received 401. Response headers report a 40-request limit. The observed 43 accepted requests are consistent with the documented 40-request burst plus 20 requests/second refill during the request window.

## Privacy, headers, caching, and links

- The complete normal demo flow requested only `https://alert-evidence-envelope.sociobot.in`; it made no analytics, advertising, hosted-font, or third-party script request.
- `/`, `/demo`, `/privacy`, and `/terms` return `no-cache`; `/health` and API responses return `no-store`; hashed assets, fonts, and the favicon return `public, max-age=31536000, immutable`.
- Responses include HSTS, `nosniff`, `DENY`, `no-referrer`, and a response-header CSP with `frame-ancestors 'none'`.
- Internal links return 200 except the intentionally tested designed 404. The Field Kit link returns the expected 303 to hosted checkout; the external source link returns 200.

## Accessibility, mobile, motion, and performance

- Playwright axe found zero serious or critical findings on `/`, `/demo`, `/privacy`, `/terms`, and the designed 404 at 390 px in dark mode; the primary demo flow also passed on desktop and mobile in light mode.
- Each checked route has one `h1`, one `main`, a meaningful title, and no mobile horizontal overflow.
- Keyboard use reaches a visibly outlined skip link first; after activation, the next Tab lands on `Try it with sample data`, and Enter opens the demo.
- At 390 px, signed state, service, error, and redaction all intersect the initial demo viewport. The touch-target exceptions are recorded above.
- Reduced-motion emulation matches, removes the terrain transform and smooth scrolling, and reduces transitions to 0.01 ms.
- Lighthouse mobile: performance 95, accessibility 100, best practices 100, SEO 100; FCP 1.5 s, LCP 2.2 s, TBT 220 ms, CLS 0.
- Initial JS is 69,205 bytes raw / 25,664 bytes gzip; CSS is 18,651 bytes raw / 5,167 bytes gzip; both fonts total 115,560 bytes; mobile hero is 40,982 bytes. All are within the supplied budgets.

## Required next steps

1. Make the offline success state observable and make `@claim:offline-demo` pass in both projects from a clean checkout.
2. Make the cold `demo-envelope` claim invocation fit within its server-start allowance or prebuild the backend as part of the declared command.
3. Restore the four omitted public claims to `.factory/claims.json` with their existing tagged tests/checks.
4. Regenerate `.factory/copy-audit.md` from all current landing copy.
5. Increase the listed mobile touch targets to at least 44 × 44 px and reserve immutable caching for content-versioned URLs.
6. Rerun every claims command first, then the full gate, before changing this result to PASS.
