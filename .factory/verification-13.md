# Verification 13 — FAIL

Date: 2026-09-02

Work order: `alert-evidence-envelope-verify-13`

Candidate: `73e4e089b195dfc1460e4735967d95765f3914a7`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**FAIL.** The live deployment is the requested candidate and the core relay,
demo, privacy, accessibility, rate-limit, persistence, and performance checks
pass. However, the browser route builder cannot create a route when either URL
that the interface labels optional is left blank. This blocks a documented
production workflow and the per-channel redaction use case in the brief.

## Release-blocking defect

### F-1 — P1/high: Create route rejects documented optional URL fields

The route builder says:

- “Leave the source blank when evidence already arrives inside the alert.”
- Destination URL is “optional if set by environment.”

After loading a protected route with a valid generated admin token, selecting
**Create route** with an empty source URL returns `endpoint URL is invalid`.
Setting a valid HTTPS source but leaving the destination URL blank produces the
same failure. No route is created. Supplying both URLs allows creation.

This is reproducible against a candidate-stamped production binary started
with only `PORT`. Direct API creation succeeds when both values are JSON
`null`, confirming that the server supports the documented modes and the
browser request is the failure point.

Evidence:

- `frontend/src/App.svelte:227` sends `{ ...config }` from `createRoute()`, so
  the two empty form values remain `""`.
- `frontend/src/App.svelte:244` normalizes the same fields to `null` only in
  the separate save path.
- `src/lib.rs:662` validates every `Some(...)` URL; `Some("")` fails parsing.
- Local UI result with blank source: `endpoint URL is invalid`.
- Local UI result with blank destination: `endpoint URL is invalid`.
- Direct `POST /api/v1/channels` with `source_url: null` and
  `destination_url: null`: HTTP 200; cleanup delete: HTTP 204.
- Screenshot captured during verification:
  `/tmp/aee-create-route-blank-source-failure.png`.

Impact: operators using embedded alert evidence cannot create the additional
routes needed for separate Slack/customer redaction policies. Operators using
an environment-supplied destination cannot create a route without entering a
URL the UI says is optional. Calling the API manually is not an acceptable UI
workaround.

Required repair: normalize blank optional URLs to `null` in `createRoute()`, as
`saveConfig()` already does, and add a browser regression test that loads the
protected route, leaves both optional fields blank, creates a second route,
and verifies the new route is selectable after reload.

## Cold first-read gate

**PASS.** A fresh 1440×900 visit showed, without scrolling:

- What it does: “Add redacted evidence to webhook alerts.”
- Who it is for: “For on-call engineers and webhook consumers who need
  incident context without another dashboard login.”
- What to do first: **Try it with sample data**.
- What happens next: “The sample opens a signed, redacted envelope in an
  isolated workspace.”

The same primary action was visible at 390×844 (`y=436.8`, bottom `487.6`). It
opened `/?demo=1` in one click and immediately showed the completed sample.

## Claims gate

`.factory/claims.json` exists and contains 28 claims. After `npm ci` (56
packages, 0 vulnerabilities), every listed command was run separately before
the wider QA. All 28 passed.

| Claim | Result |
| --- | --- |
| `demo-envelope` | PASS — desktop and mobile |
| `mobile-demo-result` | PASS — desktop and mobile |
| `demo-route-policies` | PASS — desktop and mobile |
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
| `credential-browser-exposure` | PASS — desktop and mobile |
| `preview-no-history` | PASS |
| `per-route-isolation` | PASS at the backend; F-1 is the missing browser-path coverage |
| `no-tracking` | PASS — desktop and mobile |
| `offline-demo` | PASS — desktop and mobile |
| `license-transport` | PASS — desktop and mobile |
| `local-policy-presets` | PASS — desktop and mobile |
| `license-throttle` | PASS — desktop and mobile |
| `free-core` | PASS — desktop and mobile |
| `field-kit-purchase` | PASS — desktop and mobile |
| `license-revocation` | PASS — desktop and mobile |
| `provenance-license` | PASS — desktop and mobile |
| `durable-deployment` | PASS |
| `rate-limit` | PASS |
| `destination-contracts` | PASS |

The claim-manifest test also confirmed that all 28 entries have exactly one
tagged regression test. The public copy and README were cross-checked; no
additional unsupported claim was found. F-1 shows that the backend-only
`per-route-isolation` claim test does not cover creation through the shipped
route-builder UI.

## Clean local verification

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 56 packages, 0 vulnerabilities |
| `npm test` | PASS — 25 Rust tests and 58 browser cases; one Chromium process crash retried successfully |
| Exact `local-policy-presets` claim rerun | PASS without retry |
| `npm run check` | PASS — 0 errors, 0 warnings |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=73e4... npm run build` | PASS; `dist/` produced |
| `BUILD_SHA=73e4... cargo build --release --locked` | PASS |
| `npm run test:deployment-policy` | PASS |

The container CLI is not installed in the verifier image, so `docker build`
could not be invoked. The exact frontend and optimized Rust build stages from
the Dockerfile passed directly. The live image identity, file hashes, and
scoped topology check provide deployment evidence.

Production bundle sizes:

- JavaScript: 71,967 bytes raw / 26.67 KB gzip.
- CSS: 20,049 bytes raw / 5.49 KB gzip.
- Mobile hero: 40,982 bytes.
- Self-hosted fonts: 115,560 bytes total.

These meet the stated 200 KB JS, 50 KB CSS, 300 KB mobile hero, and 120 KB font
budgets.

## Local backend and end-to-end evidence

The release binary started successfully from a new temporary directory with
an empty environment except `PATH` and `PORT=18088`. It generated SQLite and
three credentials under the local `data/` fallback, logging generated versus
supplied state without printing secret values. Credential/key modes were 0600.

An authenticated local relay test configured a JSON destination capture and
submitted two evidence rows containing unique nested secret markers. Result:

- HTTP 202 and `status: delivered`.
- Service `orders-api`, error `database timeout`, and exact first-seen time.
- A one-item cap produced `evidence_items: 1` and `truncated: true`.
- Nested email and token became `[REDACTED]`.
- Provider signature presence was recorded.
- The destination received the signed envelope.
- History contained metadata only.
- A byte scan of the closed SQLite file found none of the raw secret markers.

After graceful stop and restart with the same data directory, the route,
history record, admin token, inbound token, and signing key were unchanged.
Startup correctly reported all three credentials as persisted.

Boundary validation returned 200 at `max_items` 1 and 100, `max_bytes` 1,024
and 131,072, and route-name length 80. It returned 400 at 0/101 items,
1,023/131,073 bytes, route-name length 81, and a JSON pointer without `/`.
Malformed authenticated preview JSON returned 400; the same unauthenticated
request returned 401 before parsing. The browser recovered from a wrong admin
token after the correct token was entered. Invalid sample JSON produced
“Sample alert is not valid JSON. Check commas and quotes.”; **Restore valid
sample** followed by **Build signed preview** recovered successfully.

## Live deployment identity and topology

- `/health`: HTTP 200,
  `build=73e4e089b195dfc1460e4735967d95765f3914a7`.
- Footer: `Build 73e4e089b195`.
- `npm run verify:live-topology`: PASS.
- Revision: `sf-alert-evidence-envelope--0000033`.
- Image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:73e4e089b195`.
- Single revision; min/max/running replicas all 1.
- `/data` mount: `alert-evidence-envelope-data`.
- The topology script completed 20 fresh-connection previews.

Candidate-built hashes matched live bytes:

| File | SHA-256 |
| --- | --- |
| `/` / `dist/index.html` | `e8aa67907f5c73fb0241ae6908899963ffc8159e2e7d1b2a97afe950d5b023c6` |
| JS | `858f069b268e5dd4d5380b0f8fa0ea1b08e734a83edfc26f759421e48069b41e` |
| CSS | `ac972e8a9be7b6c808e46262d130cf54f81391dc79948db38c10966101ce50` |
| `sw.js` | `c155455a4e03bf953b36c2d86c85758fbf7d45b2f370e036c0cfeca97aec50a4` |
| Mobile hero | `e2ca79115164a994b2448655d4075f0e68a0ec6af604f10d19ed55e6907cad15` |

## Live product behavior

- The demo returned `checkout-api`, `payment authorization timed out`, and the
  expected first-seen time without a dashboard lookup.
- Customer automation redacted `email` and nested `token`; Internal Slack kept
  the email and redacted the nested token.
- The envelope contained two items, a 213-byte evidence excerpt, query
  fingerprint, and an `hmac-sha256=` signature.
- Reset replaced the session UUID. Start for real cleared every `demo:` browser
  key. Navigation-related asynchronous session deletes were aborted by page
  transition in one diagnostic run, but the server rows expire after 24 hours
  and no user data is stored in them.
- Twenty concurrent live demo sessions each created, previewed, exposed the
  supplied service/error/time, recursively redacted secrets, and deleted:
  20/20 passed with HTTP `200/200/204`.
- The official Field Kit link showed `$39` once and resolved through the
  Sociobot checkout endpoint to hosted checkout.
- No sign-in is required, so the Entra External ID condition is not applicable.

## Rate limiting

One live client sent 100 concurrent unauthorized protected-API requests in
500 ms. Results were 45 ordinary 401 responses and 55 rate-limited 429
responses. Every 429 contained `Retry-After: 1`. A different forwarded client
IP immediately received the normal 401 response.

Observed/documented allowance: a 40-request burst with one permit every 50 ms
(20 requests per second). The five additional non-429 responses are consistent
with refill during the 500 ms request window. `/health` is exempt as documented.

## Privacy, headers, PWA, accessibility, and performance

- A complete cold landing → demo → policy switch → reset → exit request log
  used only `https://alert-evidence-envelope.sociobot.in`. No analytics,
  advertising, hosted-font, or third-party-script request occurred.
- Core routes produced no console or page errors. The intentional 404 produced
  only the browser's expected failed-resource console message for its own 404.
- `/`, `/demo`, `/privacy`, `/terms`, discovery files, icons, social card,
  checkout link, and source-repository link resolved. The designed unknown
  route returned a real 404.
- HTML routes use `no-cache`; API and health use `no-store`; hashed JS/CSS use
  `public, max-age=31536000, immutable`.
- Responses include HSTS, `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and response-header CSP with
  `frame-ancestors 'none'`.
- `/opt/fleet/lib/verify-url.sh`: PASS; 822 ms load, title, `lang=en`, one
  `<h1>`, `<main>`, alt text, labelled buttons, and zero console errors.
- Live Axe on `/`, demo, privacy, terms, and 404 in light and dark modes: zero
  serious/critical WCAG 2 A/AA and 2.1 AA findings.
- At 390×844 every tested route had `scrollWidth=390`; minimum interactive
  target dimension was 44 px. The first Tab selected the skip link with a
  3 px orange focus outline.
- Reduced-motion mode used `scroll-behavior: auto` and had zero elements with
  an effective animation over 0.01 ms.
- Service worker registration/update succeeded with cache
  `envelope-shell-v6`. A fresh context reloaded the completed demo offline;
  all shell requests were served without failure and no API/health request was
  made.
- Lighthouse mobile: performance 94, accessibility 100, best practices 100,
  SEO 100; FCP 1.4 s, LCP 1.8 s, CLS 0, TBT 260 ms, total 132 KiB.

## Severity summary

| Severity | Count | Findings |
| --- | ---: | --- |
| P0 / critical | 0 | None |
| P1 / high | 1 | F-1: additional route creation fails for documented optional URL modes |
| P2 / medium | 0 | None |
| P3 / low | 0 | None |

Product code and infrastructure were not modified. Only this verification and
the handoff were changed.
