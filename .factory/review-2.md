# Adversarial first-read review 2 — Alert Evidence Envelope

Date: 2026-09-01

Work order: `alert-evidence-envelope-review-2`

Repository commit reviewed: `4dde78c65c32e09f084014e051e2664c3f6eca87`

Live build reported by `/health`: `ff56488761e3922e8fe788807fcda37de6cc7cc5`

## Verdict: FAIL

Five findings are blocking because earlier findings remain only partly fixed or have regressed. Seven further findings cover incomplete claim tests, unsupported destination copy, unclear words, and inconsistent legal-page chrome. A PASS requires zero findings and no untested claim.

## Cold first screen, before scrolling

Fresh Chromium contexts were used at 390 × 844 and 1440 × 900. No storage, cookies, or prior service worker state was supplied.

My answers at both sizes were:

- What it does: adds redacted, signed evidence to webhook alerts.
- For whom: on-call engineers and webhook consumers who need incident context.
- What to click first: **Try it with sample data**.

This check passes. The exact first-screen copy was “Send safe evidence with every alert”, “For on-call engineers and webhook consumers who need incident context without another dashboard login.”, and “Try it with sample data”. The sample-result note and all three facts were visible without scrolling at both sizes. No console error appeared on `/`.

## Findings

### Blocking

#### F-2-1 / F-1-1 — The redaction result is still outside the readable mobile demo viewport

- Location: `/demo`, 390 × 844, immediately after selecting **Try it with sample data**.
- Exact visible result: “Envelope signed. Demo data was not stored.”, `checkout-api`, “payment authorization timed out”, and the first-seen time.
- Evidence: the `[REDACTED]` row starts at `y=841.53` and ends at `y=866.33`. The 844 px screenshot cuts it off before the value is readable. The claim test passes because it checks only `box.y < 844`; it does not check the element's visible area or bottom edge.
- Why this fails: F-1-1 explicitly required the redaction result as well as the summary and signed status in the initial mobile viewport. The transform is visible now, but the safety outcome still requires scrolling.
- Concrete fix: remove at least 24 px above the result or place “Sensitive fields: [REDACTED]” before the first-seen row. Change `@claim:mobile-demo-result` to require the complete bounding box, including `box.y + box.height <= 844`, and attach the 390 × 844 screenshot.

#### F-2-2 / F-1-3 — The free-core claim test does not exercise the controls it claims are free

- Locations/quotes: landing, “The self-hosted relay and its safety controls are free” and “No subscription or hosted data dependency”; README, “Redaction, signing, previews, copying envelopes, and route safety controls stay available without a license.”
- Evidence: `@claim:free-core` opens `/demo`, waits for a completed sample, confirms that **Copy envelope JSON** is visible, and checks that no billing request occurred. It does not click the copy action, visit the route controls, exercise a safety control, or test the no-subscription/hosted-dependency statement.
- Why this fails: the claim was added for F-1-3, but its assigned test does not prove most of the words. The claim remains partly untested.
- Concrete fix: in a fresh unlicensed context, exercise redaction, signing, preview, clipboard export, each route safety control, and verify that none is gated or calls billing. Remove “No subscription or hosted data dependency” unless the same test can define and prove it.

#### F-2-3 / F-1-5 — The credential-storage test omits the browser/API half of its claim

- Location: `.factory/claims.json`, `credential-storage`: “Generated signing, admin, and inbound credentials use protected server files; browser route settings never include them.”
- Evidence: `claim_generated_credentials_are_protected` checks generated file modes and one supplied override. It never requests a configuration response and never inspects browser storage. This also contradicts its own sandbox description, which says to assert “no token in a configuration response.”
- Why this fails: F-1-5 required proof that credentials do not appear in HTML, API responses, or browser storage. Only server-file handling is tested.
- Concrete fix: extend the tagged test through the authenticated configuration API and a browser context. Seed unique marker credentials, then assert that response JSON, rendered HTML, and persisted browser storage contain none of them.

#### F-2-4 / F-1-7 — Untested merchant and refund claims remain on the live landing page

- Location: `/`, Field Kit section.
- Exact quotes: “Sociobot/Dodo is the merchant of record.” and “Refunds are handled there.”
- Evidence: `field-kit-purchase` checks the displayed price and official checkout URL. `license-revocation` supplies an invalid fixture; it does not test a refund or merchant-of-record status. No claim entry lists either live sentence. Polish 1 says this wording was removed, but it is present in the deployed page and `frontend/src/App.svelte`.
- Why this fails: a buyer is asked to rely on purchase and refund terms that the claims inventory cannot verify.
- Concrete fix: remove both sentences, or list them in `claims.json` and test a recorded checkout contract plus a refunded-license fixture that revokes only Field Kit controls.

#### F-2-5 / F-1-21 — Separate routes exist, but the demo still does not demonstrate per-destination policies

- Location: `/demo` and the earlier F-1-21 repair requirement.
- Evidence: protected create/update/delete APIs and the production UI now support separate routes, and `per-route-isolation` passes. The one-click demo still contains one hard-coded `primary` envelope and no route list or comparison. A visitor without the server admin token cannot try the repaired feature.
- Why this fails: the concrete F-1-21 fix required at least two seeded demo routes with visibly different redaction policies. The production capability exists, but the required tryable proof does not.
- Concrete fix: seed two isolated demo routes, such as Internal Slack and Customer Automation, show their different redaction lists and results, and add a browser claim test that switches between them without reading or writing protected routes.

### High

#### F-2-6 — The quantitative rate-limit claim is not tested at its stated boundaries

- Locations: `.factory/claims.json` `rate-limit`; README request-limit sentence.
- Exact claim: “API clients receive a 40-request burst, then 20 requests per second, keyed by the first forwarded client IP.”
- Evidence: `@claim:rate-limit` sends 60 requests concurrently, checks only that at least one receives 429, then checks that a different single IP receives 401. It does not assert exactly 40 initial admissions, a 20-per-second refill, or selection of the first address in a multi-address `X-Forwarded-For` value.
- Why this fails: all three quantitative/selection details can pass while the actual values differ.
- Concrete fix: run an isolated limiter with controlled time. Assert 40 admissions before 429, 20 new admissions after one second, and independent keys for headers whose first forwarded addresses differ.

#### F-2-7 — The 24-hour license throttle test proves only an immediate retry delay

- Locations: `/privacy`; `.factory/claims.json` `license-throttle`.
- Exact claim: “The browser waits 24 hours after each license verification attempt.”
- Evidence: `@claim:license-throttle` aborts one request and immediately reloads once. It never advances the clock to 23 hours 59 minutes or 24 hours.
- Why this fails: the test would pass for any retry delay longer than the few milliseconds between its two page loads.
- Concrete fix: control `Date.now()`, confirm no second request just before 24 hours, then confirm exactly one request at or after 24 hours.

#### F-2-8 — “Safe” and “verified” are unbounded, unlisted claims

- Locations/quotes: `/` h1, “Send safe evidence with every alert”; landing preview label, “Safe preview”; hero art label, “ROUTE VERIFIED”.
- Why this fails: “safe” is broader than bounded/redacted/signed, and the fresh page says the route is verified before an admin token has loaded any route. None of these statements has a matching claim entry.
- Concrete fix: use “Add redacted evidence to webhook alerts”, “Envelope preview”, and a factual art label such as “REDACT → SIGN”. Keep the tested bounded/redacted/signed facts nearby.

#### F-2-9 — Email and automation destinations are presented without delivery-contract tests

- Locations: `/`, Destination type options “Automation webhook” and “Email gateway webhook”; README, “delivers it to a configured destination.”
- Evidence: `slack-delivery` verifies only the Slack request body. No claim test defines or captures the request contract for `json` or `email-webhook`; the implementation sends the same envelope JSON for both.
- Why this fails: an operator cannot tell what an email gateway receives or verify that the advertised destination types are usable.
- Concrete fix: either expose one accurately named **JSON webhook** destination, or document subject/body/header contracts for email and automation and add local capture-server tests for each. No AI feature is warranted for deterministic delivery.

### Medium

#### F-2-10 — Several headings and labels require surrounding context or use untested adjectives

| Exact copy | Problem | Proposed rewrite |
| --- | --- | --- |
| “Safe preview” | “Safe” is unmeasured and does not name the output. | “Envelope preview” |
| “Name and state” | A heading list does not reveal what is named or what “state” means. | “Name and enable the route” |
| “Locate evidence” | Does not say whether this selects a source or finds an existing record. | “Choose the evidence source” |
| “Set the boundary” | Metaphorical product language rather than the settings in the section. | “Limit and redact evidence” |
| “Address the envelope” | Metaphor; the section configures delivery. | “Choose the delivery destination” |
| “Item cap” / “Byte cap” | Operator shorthand is not explained. | “Maximum records” / “Maximum envelope bytes” |
| “Query JSON pointer” | Standards jargon has no helper text. | Keep the label, then add “Path to the query field, for example `/query`.” |

No landing or README sentence exceeds 22 words, and no banned marketing word appears. The issue is meaning, not length.

#### F-2-11 — Privacy and Terms do not use the complete site footer

- Locations: `/privacy` and `/terms`.
- Exact footer: “Alert Evidence Envelope · Privacy · Terms · Built by Param Factory · Build …”.
- Evidence: `/`, `/demo`, and the designed 404 include the product one-line description and Source link. Privacy and Terms omit both, and their headers replace the normal navigation with only “Back to product”.
- Why this fails: the site-structure contract requires consistent chrome and a product one-liner, Privacy, Terms, factory credit, and build ID on every route.
- Concrete fix: render the same shared header/footer component on legal routes, with the current route identified in navigation.

#### F-2-12 — One README sentence combines two setup instructions

- Location: README, local-run instructions.
- Exact quote: “Enter the admin token in the route builder; incoming alerts send the inbound token in `x-envelope-token`.”
- Why this fails: the admin action and provider configuration are different tasks, contrary to the one-idea-per-sentence rule.
- Concrete rewrite: “Enter the admin token in the route builder. Configure alert providers to send the inbound token in `x-envelope-token`.”

## Demo and sandbox evidence

- `/` → **Try it with sample data** opened `/demo` in one click.
- The first demo screen contained a checkout timeout for `checkout-api`, the error signature, first-seen time, and signed status. The redaction result was clipped as described in F-2-1.
- The persistent banner said “Demo — sample data, nothing is saved” and offered **Reset demo** and **Start for real**.
- Reset changed both the session ID and envelope ID. Start for real removed every `demo:` local-storage key.
- The complete live sample flow requested only `https://alert-evidence-envelope.sociobot.in`. No analytics, hosted font, advertising, or other third-party request occurred.
- The separate offline claim run loaded the cached sample in a dedicated offline context.
- The server isolation test used temporary SQLite and passed. Demo endpoints did not access protected route history.

## Claim test results

Every exact command in `.factory/claims.json` ran from clean clone `/tmp/aee-review2-79pJ0g`.

| Claim | Exact command result |
| --- | --- |
| `demo-envelope` | PASS |
| `mobile-demo-result` | PASS; assertion weakness is F-2-1 |
| `bounded-redacted-signed` | PASS |
| `query-fingerprint` | PASS |
| `fixed-query-source` | PASS |
| `provider-signature` | PASS |
| `slack-delivery` | PASS |
| `isolated-demo` | PASS |
| `raw-not-retained` | PASS |
| `history-limit` | PASS |
| `protected-real-apis` | PASS |
| `credential-storage` | PASS; incomplete coverage is F-2-3 |
| `preview-no-history` | PASS |
| `per-route-isolation` | PASS |
| `no-tracking` | PASS |
| `offline-demo` | PASS |
| `license-transport` | PASS |
| `local-policy-presets` | PASS |
| `license-throttle` | PASS; boundary omission is F-2-7 |
| `free-core` | PASS; incomplete coverage is F-2-2 |
| `field-kit-purchase` | PASS |
| `license-revocation` | PASS |
| `provenance-license` | PASS |
| `durable-deployment` | PASS |
| `rate-limit` | PASS; quantitative omissions are F-2-6 |

No command returned a failing exit code. Passing a command does not close a finding when the assertion does not prove the registered wording.

## Copy audit

Counts treat a hyphenated term, path, or displayed code token as one word.

### Landing-page sentences and factual lines

| Words | Copy |
| ---: | --- |
| 14 | For on-call engineers and webhook consumers who need incident context without another dashboard login. |
| 11 | The sample opens a signed, redacted envelope in an isolated workspace. |
| 8 | Demo data is never added to route history |
| 5 | No analytics or third-party scripts |
| 9 | Self-hosted core is free; Field Kit costs $39 once |
| 20 | An amber alert path crosses a topographic incident map, passes a redaction mark, and arrives at a sealed green envelope. |
| 4 | Use one fixed source. |
| 7 | Limit the record count and envelope size. |
| 11 | Remove sensitive keys recursively with this route’s redaction list before forwarding. |
| 13 | Hash the configured query and source so responders know what shaped the excerpt. |
| 14 | Sign the final JSON envelope and preserve the provider signature in transit when present. |
| 9 | Each route stores its own redaction list and destination. |
| 6 | Server credentials stay outside the browser. |
| 9 | Enter the server admin token to load this route. |
| 11 | Leave the source blank when evidence already arrives inside the alert. |
| 6 | A remote source receives only `?q=…&limit=…`. |
| 8 | Set in the server environment; never entered here. |
| 7 | Incoming requests must send the server’s `x-envelope-token`. |
| 10 | Preview applies the live route’s bounds, redaction, fingerprint, and signature. |
| 6 | It does not add delivery history. |
| 13 | Use the sample as-is or paste a realistic alert with sensitive values removed. |
| 5 | Delivery history stores metadata only. |
| 6 | Raw alerts and evidence are absent. |
| 8 | Send a live alert to the relay URL. |
| 5 | Preview runs never appear here. |
| 9 | The self-hosted relay and its safety controls are free. |
| 8 | The $39 Field Kit is a one-time purchase. |
| 8 | It adds named redaction presets on this device. |
| 7 | Named policies for Slack, customers, and automation |
| 7 | Apply a policy before saving a route |
| 6 | No subscription or hosted data dependency |
| 6 | Sociobot/Dodo is the merchant of record. |
| 4 | Refunds are handled there. |
| 3 | No presets yet. |
| 8 | Name the current policy to keep it locally. |
| 8 | Send bounded incident evidence with a webhook alert. |
| 18 | Built by Param Factory · Build `ff56488761e3` · Cartography generated for this product on 2026-08-27 · MIT licensed |

### README sentences

| Words | Sentence |
| ---: | --- |
| 8 | Add bounded, redacted, signed evidence to webhook alerts. |
| 6 | For on-call engineers and webhook consumers. |
| 15 | It builds an evidence envelope from alert JSON, then delivers it to a configured destination. |
| 5 | Try it with sample data. |
| 10 | The isolated sample shows a checkout timeout with redacted values. |
| 8 | Limits evidence by record count and byte size. |
| 7 | Removes configured sensitive fields, including nested fields. |
| 10 | Records a fingerprint from the fixed source and alert query. |
| 5 | Signs the envelope with HMAC-SHA256. |
| 14 | Sends Slack the readable summary, bounded redacted evidence, and signature in one request body. |
| 13 | Keeps separate delivery routes with their own inbound URLs, destinations, and redaction lists. |
| 11 | SQLite stores route settings, short-lived demo session IDs, and delivery metadata. |
| 9 | It does not store inbound bodies or evidence excerpts. |
| 7 | Requirements: Node 22+, Rust, and SQLite support. |
| 2 | Open `http://localhost:8080`. |
| 15 | First boot creates protected signing, admin, and inbound credentials in `data/` (or `/data` when mounted). |
| 8 | Set their corresponding environment variables to supply replacements. |
| 16 | Enter the admin token in the route builder; incoming alerts send the inbound token in `x-envelope-token`. |
| 11 | Each `/api/v1` endpoint is rate limited by the first `X-Forwarded-For` address. |
| 6 | `/health` remains available for platform probes. |
| 13 | Each public product claim and its repeatable sandbox command is listed in `.factory/claims.json`. |
| 6 | Demo behavior is documented in `.factory/demo.md`. |
| 11 | The container serves the built frontend and Rust API on `PORT`. |
| 11 | Durable SQLite state lives at `/data` when the platform mounts it. |
| 17 | The optional Field Kit costs $39 USD once and adds named redaction presets stored in this browser. |
| 14 | Redaction, signing, previews, copying envelopes, and route safety controls stay available without a license. |
| 7 | License tokens are stored in the browser. |
| 14 | Verification sends the token to Sociobot in an authorization header, not in a URL. |
| 8 | Privacy and Terms explain storage and purchase terms. |
| 1 | MIT. |
| 14 | The cartography was generated for this product on 2026-08-27; prompt metadata is in `assets/src`. |
| 7 | Inter and Fraunces notices are in `THIRD_PARTY_NOTICES.md`. |

No sentence exceeds 22 words. Headline/heading/label flags are F-2-8 and F-2-10. The only sentence-structure flag is F-2-12. Buttons name results or use the required demo actions; no button finding remains.

## Structure, accessibility, and link evidence

- `/`, `/demo`, `/privacy`, and `/terms` returned 200. The designed unknown route returned 404.
- Every checked route had `lang=en`, one `h1`, one `main`, a route-specific title, description, canonical URL, Open Graph URL/title/image, favicon, and touch icon.
- `robots.txt`, `sitemap.xml`, and the four sitemap routes returned 200.
- All rendered internal links returned 200. The source link returned 200. The Field Kit action reached the official Sociobot endpoint and then a checkout page with 200.
- In-app navigation to Demo focused and announced “Inspect a sample evidence envelope”. Back navigation restored `/` and focused its h1. A cold `/#configure` deep link settled with the configuration section at the viewport top.
- Live light/dark axe scans at 390 px found zero serious or critical violations on `/`, `/demo`, `/privacy`, `/terms`, and the designed 404.
- `/opt/fleet/lib/verify-url.sh` passed for `/`: correct title, language, h1, main, image alternatives, button names, and no console/page errors.
- Live security headers included CSP, HSTS, `X-Content-Type-Options`, `X-Frame-Options`, and `Referrer-Policy`.
- The generated production bundle was 69.64 KB JavaScript and 26.07 KB gzip, below the product budget.
- The topographic survey-paper identity matches `.factory/design.md` and is distinct from a generic SaaS template.
- The remaining chrome inconsistency is F-2-11.

## Full quality gates

From the same clean clone:

- `npm test`: exit 0; 23 Rust tests passed and all 54 Playwright cases completed. One mobile legal-page Chromium process crashed once and passed automatically on retry, so the run reported one flaky case rather than a product assertion failure.
- `cargo fmt --check`: PASS.
- `cargo clippy --all-targets --locked -- -D warnings`: PASS.
- `npm run build`: PASS; `dist/` produced.

## Earlier-finding audit

| Earlier finding | Review 2 status | Evidence |
| --- | --- | --- |
| F-1-1 mobile demo result | PARTLY FIXED; repeated as F-2-1 | Summary and signature are visible; redaction is clipped. |
| F-1-2 license-token locality | FIXED | Live privacy wording and intercepted request show an authorization header with no token in the URL. |
| F-1-3 free-tier claims | PARTLY FIXED; repeated as F-2-2 | Claim exists, but its test does not exercise the full claim. |
| F-1-4 query fingerprint | FIXED | Dedicated source/query mutation test passes. |
| F-1-5 credential storage | PARTLY FIXED; repeated as F-2-3 | File protections pass; API/browser absence is not tested. |
| F-1-6 preview history | FIXED | Protected preview leaves history unchanged. |
| F-1-7 merchant/refund claims | REGRESSED; repeated as F-2-4 | Unlisted sentences are live again. |
| F-1-8 256 KB request limit | FIXED | Public numeric sentence was removed. |
| F-1-9 no-destination behavior | FIXED | Public sentence was removed. |
| F-1-10 secret-free logging | FIXED | Public sentence was removed. |
| F-1-11 README suite claims | FIXED | Process assertions were removed from public product copy. |
| F-1-12 failed license retry | FIXED | Timestamp is saved before the request; immediate retry is suppressed. F-2-7 separately covers the untested 24-hour boundary. |
| F-1-13 deployment runtime wording | FIXED | Overbroad runtime sentences were removed; remaining deployment claim matches the policy inspection. |
| F-1-14 provenance/font wording | FIXED | Dated asset provenance and MIT status are tested; README now points to the existing font notices without making an originality/license assertion. |
| F-1-15 home title | FIXED | Title, OG title, and Twitter title say “add evidence to alerts”. |
| F-1-16 route focus | FIXED | Demo navigation and browser back focus/announce the new h1. |
| F-1-17 404 metadata/chrome | FIXED | Canonical, OG URL, theme color, recovery links, and complete 404 footer are present. |
| F-1-18 inconsistent jargon | FIXED | The specific six phrases from review 1 were replaced. F-2-10 lists separate remaining labels. |
| F-1-19 one-word stage headings | FIXED | All four now name the affected object. |
| F-1-20 ambiguous buttons | FIXED | Buttons read “Copy relay URL” and “Build signed preview”. |
| F-1-21 per-channel routes | PARTLY FIXED; repeated as F-2-5 | Protected routes exist and isolate policy, but the required two-route demo is absent. |

## Missed leverage

F-2-5 is the still-missing tryable route comparison implied by strict per-channel redaction. F-2-9 covers the unproven email/automation delivery promise. An AI-assisted step would not improve this deterministic security boundary: redaction, caps, signing, and routing should remain explicit and reproducible, and there is no decorative AI feature or embedded provider key to remove.

## What would make this perfect

Bring the redaction result fully into the initial 390 × 844 demo viewport; make the free-core, credential, throttle, and rate-limit tests prove every word; remove or test merchant/refund language; seed two visibly different demo routes; define and test email/automation delivery contracts; replace vague headings and unbounded “safe/verified” copy; unify legal-page chrome; and split the two-action README sentence. Then rerun all 25 claim commands, the full quality gate, live request/offline logs, link crawl, light/dark axe scans, and cold mobile screenshot. A follow-up review must report zero findings.
