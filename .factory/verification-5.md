# Verification 5 — FAIL

Date: 2026-08-30

Work order: `alert-evidence-envelope-verify-5`

Candidate: `10fd47b26b6de165b432420838c9f4300b5c07c1`

Live URL: `https://alert-evidence-envelope.sociobot.in`
Acceptance sources: `.factory/brief.json`, the supplied researched brief/work order, and the attached factory skills

## Decision

**FAIL — do not release or promote this candidate.** The live backend reports the exact candidate SHA and all checked live static files match a production build of that candidate. The local transformer also completes the real upstream-fetch → redact/bound/sign → downstream-delivery job. Release is nevertheless blocked by two failing declared claim commands and a broken live first-click demo.

| Severity | Finding | Fresh evidence |
| --- | --- | --- |
| P0 / release gate | The declared `field-kit-purchase` claim test fails. | `npm run test:claims -- --grep @claim:field-kit-purchase` failed in desktop and mobile. Production checkout returned 303 to `checkout.dodopayments.com`; the registered pilot endpoint returned HTTP 500 instead of a redirect to `test.checkout.dodopayments.com`. |
| P0 / release gate | The declared `durable-deployment` claim test fails, and live state is neither single-replica-capped nor mounted durably. | The exact composite claim command exited 1 in `npm run verify:live-topology`. Azure reports `maxReplicas: 3`, `volumes: null`, and `volumeMounts: null`. Two replicas were running during QA. The Azure File registration exists, but it is not attached to the app. |
| P1 / mandatory demo gate | The advertised one-click sample is unreliable and failed in both fresh desktop and 390 px browser runs. | Clicking **Try it with sample data** opened `/demo`, but the automatic preview returned 404 and showed `Preview stopped — channel was not found`, with a browser console error. Twenty direct create-session → preview pairs returned 20 × 404. A later manual **Build safe preview** retry could succeed when routed back to the replica that created the in-memory session, so this is a multi-replica deployment failure, not a valid one-click demo. |
| P1 / claim coverage | The manifest does not cover all visitor-facing claims, and two declared tests do not prove their whole wording. | The landing page/README claim fixed query-only upstream access and paid named presets/templates stored on-device, but no claim entries test those outcomes. `isolated-demo` does not assert the stated 24-hour expiry. `bounded-redacted-signed` checks only an `hmac-sha256=` prefix rather than recomputing the MAC. Independent QA did verify the current HMAC implementation, but the release contract requires the claims themselves to remain gated. |

The missing mount explains the live behavior. Demo sessions are stored in process memory; with two replicas, session creation and preview can reach different processes. The same deployment also gives each replica its own `/tmp` SQLite database and ephemeral signing/admin/inbound files, so route state, history, credentials, and signing identity are not durable or consistent.

## Mandatory claims and first-read gates

### Declared claims

The clean checkout contains `.factory/claims.json`. After the required `npm ci`, every listed command was run exactly as declared.

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-envelope` | PASS locally; **FAIL live behavior** | Browser claim passed 2/2 locally. Fresh live desktop and mobile first clicks both received a 404 preview and displayed `channel was not found`. |
| `bounded-redacted-signed` | PASS | Rust test passed; independent end-to-end HMAC recomputation also matched. The test itself only asserts signature shape. |
| `isolated-demo` | PASS with coverage gap | Rust test passed and history remained empty; it does not assert the promised 24-hour expiry. |
| `raw-not-retained` | PASS | Rust test passed; independent SQLite byte search found no seeded email, token, or evidence marker. |
| `provider-signature` | PASS | Rust test passed; independent downstream capture received both original and envelope signature headers. |
| `history-limit` | PASS | Rust test passed; independent 25-relay run retained exactly services 5–24 (20 rows). |
| `protected-real-apis` | PASS | Rust test passed; malformed unauthenticated config and relay bodies returned 401 before parsing. |
| `no-tracking` | PASS | Browser claim passed 2/2; the full live demo flow contacted only the product origin. |
| `offline-demo` | PASS | Browser claim passed 2/2; a dedicated live context reloaded the last recovered sample offline. |
| `rate-limit` | PASS locally and live | Browser claim passed 2/2. Live 60-request run produced 44 × 401 and 16 × 429 while capacity refilled, with `Retry-After: 1`; a different forwarded IP remained available. |
| `field-kit-purchase` | **FAIL** | Both browser projects failed because pilot checkout returned 500. Production returned 303 correctly. |
| `durable-deployment` | **FAIL** | Snapshot Rust test and source policy test passed; live topology verification failed because the replica cap/mount are absent. |

The first browser claim initially could not start before dependencies were installed (`vite: not found`), as expected in a clean clone. `npm ci` installed the locked dependencies with zero reported vulnerabilities, and all browser claim commands were then rerun. The failures above are the post-install results.

### Cold first read

The words on the first live screen pass the plain-language portion:

- What it does: **“Send safe evidence with every alert.”**
- Who it is for: **“For on-call engineers and webhook consumers who need incident context without another dashboard login.”**
- What to click first: **“Try it with sample data.”** The next sentence says it opens a signed, redacted envelope in an isolated workspace.
- The three visible facts cover demo retention, tracking, and price.

The required one-click result fails. On fresh desktop and mobile contexts, that click reached `/demo` but the automatic sample stopped on a 404. This independently makes the first-read/demo gate fail.

## Clean-checkout gates

| Gate | Result |
| --- | --- |
| Checkout identity | PASS: clean `main`, exact SHA `10fd47b26b6de165b432420838c9f4300b5c07c1`. |
| `npm ci` | PASS: 56 packages added, 57 audited, 0 vulnerabilities. |
| `npm test` | **FAIL**: 33/36 Playwright tests passed. Both paid-checkout tests failed deterministically; Chromium also crashed once before the mobile demo-reset test. The crashed test passed 1/1 when rerun alone. Svelte check was 0 errors/warnings, Rust was 14/14, deployment-policy source check passed, and Vite built before the browser failures. |
| `cargo fmt --check` | PASS. |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS. |
| `npm run build` | PASS and produced `dist/`. |
| `VITE_BUILD_SHA=10fd47b… npm run build` | PASS; this reproduces the Docker frontend build and matches live files byte-for-byte. |
| `BUILD_SHA=10fd47b… cargo build --release --locked` | PASS. Local `/health` returned the exact candidate SHA. |
| Docker image build | Not run: Docker, Podman, and Buildah are unavailable in this verifier container. The exact frontend and optimized backend builds were exercised directly. |
| `git diff --check` | PASS before report edits. |

## Independent backend and job-to-be-done evidence

A fresh optimized server started with an environment containing only `PATH` and `PORT`. It generated a 32-byte signing key and two 64-character tokens, all mode 600, and logged only whether each value was generated. Restarting in the same directory logged all three as persisted. `/health` returned the candidate SHA.

Against that server:

- A configured local evidence source received only `q=service=orders-api level=error`, `limit=2`, and the configured bearer credential.
- Three source rows became two bounded rows with `truncated: true`; nested email and token values became `[REDACTED]`.
- The envelope exposed service `orders-api`, error `database timeout`, and first seen `2026-08-30T05:05:00Z`.
- An independent HMAC-SHA256 recomputation using the generated key exactly matched the envelope signature.
- The downstream capture received the two-row redacted envelope, destination bearer token, `x-original-provider-signature`, and `x-evidence-envelope-signature`.
- Stopping the source returned 502; restarting it recovered to 202.
- Malformed authorized preview returned 400; unsafe remote HTTP configuration returned 400; item cap 0 returned 400; a 270 KB body returned 413.
- Twenty-five accepted relays left exactly 20 metadata rows after restart. Seeded raw email, token, and evidence markers were absent from SQLite bytes.
- Twenty seeded demo previews—16 direct fields and four recursive fallback shapes—exposed service, error, and first-seen fields in 20/20 cases, exceeding the brief's 16/20 target.
- Two hundred concurrent health probes returned 200/200.

Local API rate-limit evidence was exactly 40 × 401 followed by 20 × 429 for 60 concurrent requests from one forwarded IP. Every rejection included `Retry-After: 1`; another IP was not limited.

## Live deployment identity and behavior

- `GET /health` returned `{"build":"10fd47b26b6de165b432420838c9f4300b5c07c1","status":"ok"}`.
- The running image is `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:10fd47b26b6d` and its latest revision is ready.
- Local candidate and live SHA-256 values match for root HTML, JS, CSS, service worker, privacy, terms, fonts, social art, and both hero images. Key hashes: JS `cb8f3c3e82615607ff45df687a059d63a072acfb36b5f88da1da3bc1b6ea71e5`; CSS `8f8803ebe15f18d36e8dbbf857fb3fe926a5ed28359439449cd14f1ece2b5520`; root HTML `f389f1fc1c3c38caa3f5b6e4877f30115390568f8ff5ff3d57560212cddefc8f`.
- Azure topology at test time: single revision mode, min 1, **max 3**, **no volumes**, **no volume mounts**, and two running replicas. The correctly named read-write Azure File share exists in the environment but is detached.
- Twenty session-create/preview API pairs returned 20 × 404. Fresh desktop and mobile clicks reproduced the user-facing failure. Manual retries could eventually succeed, demonstrating replica-local session state.
- The production Field Kit checkout returned 303 to Dodo. Pilot returned 500 with no redirect.

## Browser, accessibility, privacy, PWA, and transport

Passing evidence, which does not override the release blockers:

- `/opt/fleet/lib/verify-url.sh` passed: HTTPS 200, title, `lang=en`, one h1, main present, no missing image alt, no unlabeled button, and no root-load console error.
- Fresh desktop 1440×900 and mobile 390×844 runs had exactly one h1/main, no horizontal overflow, a visible 3 px first-focus skip link, and keyboard access to the sample action.
- After a manual retry recovered the demo, signed JSON was keyboard focusable with a 3 px outline and scrolled from 0 to 227 with PageDown at 390 px. Invalid `{` input showed the specific commas/quotes recovery message.
- Playwright axe found zero serious/critical findings on root, recovered expanded demo, privacy, and terms in light/dark at 390 px. All visible controls checked on the recovered demo met 44 px dimensions.
- At 200% text sizing, root, demo, privacy, and terms remained within the 390 px viewport with h1/main present.
- Reduced motion computed to `scroll-behavior: auto` and near-instant transitions.
- The complete demo request log was same-origin. The only console error in the failed first-click flow was the product's 404 preview request. No page errors occurred.
- The service worker controlled the page, `registration.update()` completed, cache `envelope-shell-v3` existed, and the recovered sample reloaded offline at 390 px. Chromium logged only the expected `ERR_INTERNET_DISCONNECTED` request failure while offline.
- `/`, `/demo`, `/privacy`, and `/terms` return 200; a designed unknown route returns 404. All rendered links resolve; production checkout redirects. HTTP redirects to HTTPS.
- HTTPS responses send HSTS, CSP with response-header `frame-ancestors 'none'`, `X-Frame-Options: DENY`, `nosniff`, and `no-referrer`. HTML/legal/service worker are `no-cache`; API/health are `no-store`; hashed assets are `public, max-age=31536000, immutable`.
- Static budgets pass: JS 66,477 B raw / 24,970 B gzip; CSS 17,557 B raw / 4,939 B gzip; fonts 115,560 B total; mobile hero 40,982 B.
- Three fresh mobile Lighthouse runs scored performance **89 / 96 / 96** (median 96), accessibility **100 / 100 / 100**, best practices **100 / 100 / 100**, and SEO **100 / 100 / 100**. Median metrics: FCP 1,380 ms, LCP 2,195 ms, TBT 170 ms, CLS 0.

The product API documents a 40-request burst refilling at 20 requests/second. The live 60-request test produced 429s with `Retry-After: 1`; the observed 44 accepted responses reflect refill during the request window. The external Sociobot license verification endpoint allowed 30 of 100 concurrent invalid-token checks and returned 70 × 429 with `Retry-After: 4`.

## Applicability

- This is not a package, library, or CLI, so clean-consumer pack/install testing does not apply.
- The product does not require sign-in, so the Entra tenant check does not apply.
- LLM summarization is an explicit non-goal in the brief; no missing-AI finding is raised.

## Required remediation before re-verification

1. Apply the intended single-replica min/max policy and mount the registered Azure File storage at `/data`; confirm exactly one running replica and rerun the complete `durable-deployment` command.
2. Re-test the cold one-click demo repeatedly after topology repair. Session creation and its first preview must succeed without a retry on desktop and mobile.
3. Repair/re-register the pilot Field Kit product so its checkout redirects to the Dodo test host, then rerun `field-kit-purchase` and full `npm test`.
4. Add claim entries and observable sandbox tests for the advertised fixed-query upstream restriction and paid on-device preset/template behavior. Strengthen the 24-hour demo and HMAC claim tests to assert duration and MAC validity.
