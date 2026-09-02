# Adversarial first-read review 3 — Alert Evidence Envelope

Date: 2026-09-02
Work order: `alert-evidence-envelope-review-3`
Repository commit reviewed: `7be04add5636efe6f12cbbf61148a499539934e0`
Live build: `36fc44438dd299a142ce5fe30fd1a8676e539877`

## Verdict: FAIL

There are five findings, including two blocking regressions/reopens. The
one-click product explanation and sample are clear, and every registered
claim test passed. A pass is not possible while normal navigation to the
static legal routes loses focus and an unlisted broader free-feature claim is
live in Terms.

## Cold first screen

Fresh Chromium contexts (no storage, cookies, or service worker) at 390 × 844
and 1440 × 900 gave the same answers before scrolling:

- **What it does:** adds redacted evidence to webhook alerts.
- **For whom:** on-call engineers and webhook consumers.
- **What to click first:** **Try it with sample data**.

This check passes. The mobile first screen visibly contains “Add redacted
evidence to webhook alerts”, “For on-call engineers and webhook consumers who
need incident context without another dashboard login.”, the sample action,
its outcome (“The sample opens a signed, redacted envelope in an isolated
workspace.”), and all three facts. It had no console errors. The desktop
screen gives the same information.

## Findings

### Blocking

#### F-3-1 / F-1-16 — Legal-route navigation still does not focus or announce the new page

- **Location:** live `/` → header **Privacy** → `/privacy`, then browser Back.
- **Evidence:** after each completed navigation, `document.activeElement` was
  `BODY`; it was not the new route's `<h1>` (“How this relay handles data” or
  “Add redacted evidence to webhook alerts”). No `aria-live` route message is
  present on the static privacy page. `frontend/static/build.js` only obtains
  the build ID. The Svelte focus/announcement logic in
  `frontend/src/App.svelte` covers the in-app home/demo navigation, not the
  static `/privacy`, `/terms`, or 404 documents.
- **Why this fails:** a keyboard or screen-reader visitor following a normal
  header link receives no programmatic indication of the new place. This is
  the actual behavior required by F-1-16, not just the home-to-demo case
  previously tested.
- **Concrete fix:** use one route implementation for these pages, or have the
  static-page script focus the sole `<h1 tabindex="-1">` after a navigation
  and expose a polite “Privacy — Alert Evidence Envelope”/equivalent route
  announcement. Add a browser test that clicks home → Privacy, uses Back, and
  asserts focus and the announcement both times.

#### F-3-2 / F-1-3 — Terms makes a broader free-feature promise than the registered claim

- **Location:** `/terms`, Field Kit license.
- **Exact quote:** “Accessibility, export, redaction, signing, and every
  safety control remain free.”
- **Evidence:** `free-core` registers and tests only “Redaction, signing,
  previews, copying, and route settings work without a Field Kit license.”
  The test edits controls, copies a URL and a demo envelope, and verifies no
  billing request. It does not define or exercise “accessibility”, “export”,
  or “every safety control”. None of those words appears in the registered
  claim.
- **Why this fails:** this is a visitor-facing availability claim that is
  broader than the proof. It reopens the original free-tier claim finding.
- **Concrete fix:** replace it with the exact registered wording: “Redaction,
  signing, previews, copying, and route settings remain free.” Alternatively,
  register each extra feature and add observable unlicensed tests for it.

### Minor

#### F-3-3 — The README JSON-webhook sentence is grammatically incomplete

- **Location:** `README.md`, **What it does**.
- **Exact quote:** “Sends a JSON webhook the signed envelope and signature
  header.”
- **Why this fails:** it omits the relationship between the webhook and the
  payload, so an operator cannot parse the delivery contract on a first read.
- **Concrete fix:** “Sends the signed envelope and signature header to a JSON
  webhook.”

#### F-3-4 — The first-screen price fact combines two independent facts

- **Location:** landing first-screen fact list.
- **Exact quote:** “Self-hosted core is free; Field Kit costs $39 once”.
- **Why this fails:** this is two product facts in one line, contrary to the
  one-idea-per-sentence copy rule. It is especially easy to skim past the
  difference between the free core and optional local presets on a phone.
- **Concrete fix:** “The self-hosted core is free. Field Kit costs $39 once.”

#### F-3-5 — The 404 recovery action does not open the named destination

- **Location:** `/not-a-real-route`.
- **Exact quote/action:** **Open route builder** links to `/`, whose first
  screen is the landing hero; the builder is at `/#configure`.
- **Why this fails:** a visitor who has already hit a bad address needs the
  stated recovery result, not another scroll/search step. The 404 footer's
  GitHub link also reads “Source”, unlike the other routes' “Source
  (external)”.
- **Concrete fix:** point **Open route builder** and the 404 header
  **Configure** link at `/#configure`, and rename the footer link **Source
  (external)**.

## Demo and sandbox check

The required demo is present and passes its product check:

- `/` → **Try it with sample data** opens `/demo` in one click.
- The first 390 × 844 demo viewport already displays a sealed result with
  signed status, `[REDACTED]`, `checkout-api`, and the timeout error. The
  sample alert input remains available for inspection rather than standing in
  for the result.
- The persistent banner reads “Demo — sample data, nothing is saved” and has
  **Reset demo** and **Start for real**. Reset produced a new session and
  envelope. Start for real removed all `demo:` browser-storage keys.
- Internal Slack and Customer automation visibly show different redaction
  policies. The latter redacts email and token; the former retains the sample
  email and redacts token.
- A fresh live request log during the complete demo flow contained only
  `https://alert-evidence-envelope.sociobot.in`. A dedicated claim test also
  passed the cached offline reload. The Rust isolation test passed against
  temporary SQLite state.

## Claims and quality gates

I cloned the reviewed tree fresh to `/tmp/aee-review3-1o3dII`, ran `npm ci`,
then ran every exact command in `.factory/claims.json`. All returned success.

| Claim | Result | Claim | Result |
| --- | --- | --- | --- |
| demo-envelope | PASS | mobile-demo-result | PASS |
| demo-route-policies | PASS | bounded-redacted-signed | PASS |
| query-fingerprint | PASS | fixed-query-source | PASS |
| provider-signature | PASS | slack-delivery | PASS |
| isolated-demo | PASS | raw-not-retained | PASS |
| history-limit | PASS | protected-real-apis | PASS |
| credential-storage | PASS | credential-browser-exposure | PASS |
| preview-no-history | PASS | per-route-isolation | PASS |
| no-tracking | PASS | offline-demo | PASS |
| license-transport | PASS | local-policy-presets | PASS |
| license-throttle | PASS | free-core | PASS |
| field-kit-purchase | PASS | license-revocation | PASS |
| provenance-license | PASS | durable-deployment | PASS |
| rate-limit | PASS | destination-contracts | PASS |

`npm test` also passed from that clone (type check, 25 Rust tests, deployment
policy, claims-manifest validation, and the full Playwright suite), and
`npm run build` produced `dist/`. The deployed app emitted no console errors
on the cold home or demo routes. The claim inventory covers the remaining
claim-like landing and README statements; F-3-2 is the unlisted Terms
exception above.

## Copy audit

Counts use whitespace-delimited words; paths, prices, and hyphenated forms
count as one. Sample JSON and values generated from that sample are data, not
landing copy. Headings and button labels are audited below the sentence lists.

### Landing sentences and factual lines

| Words | Copy |
| ---: | --- |
| 14 | For on-call engineers and webhook consumers who need incident context without another dashboard login. |
| 11 | The sample opens a signed, redacted envelope in an isolated workspace. |
| 8 | Demo data is never added to route history. |
| 5 | No analytics or third-party scripts. |
| 9 | Self-hosted core is free; Field Kit costs $39 once. |
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
| 8 | Path to the query field, for example `/query`. |
| 4 | JSON receives the envelope. |
| 6 | Slack adds a readable `text` field. |
| 7 | Email gateways receive `subject`, `text`, and `envelope`. |
| 7 | Incoming requests must send the server’s `x-envelope-token`. |
| 10 | Preview applies the live route’s bounds, redaction, fingerprint, and signature. |
| 6 | It does not add delivery history. |
| 13 | Use the sample as-is or paste a realistic alert with sensitive values removed. |
| 5 | Delivery history stores metadata only. |
| 6 | Raw alerts and evidence are absent. |
| 8 | Send a live alert to the relay URL. |
| 5 | Preview runs never appear here. |
| 9 | Redaction, signing, previews, copying, and route settings are free. |
| 8 | The $39 Field Kit is a one-time purchase. |
| 8 | It adds named redaction presets on this device. |
| 7 | Apply a policy before saving a route. |
| 5 | Checkout is hosted by Sociobot. |
| 3 | No presets yet. |
| 8 | Name the current policy to keep it locally. |
| 8 | Send bounded incident evidence with a webhook alert. |
| 18 | Built by Param Factory · Build `36fc44438dd2` · Cartography generated for this product on 2026-08-27 · MIT licensed. |

The only >22-word candidate is none. No banned marketing adjective appears.
The prose flags are F-3-4 and F-3-3; all other landing headings name their
section and the action controls name their results.

### README sentences

| Words | Copy |
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
| 10 | Sends a JSON webhook the signed envelope and signature header. |
| 10 | Sends Slack the envelope plus a readable `text` field. |
| 10 | Sends an email gateway webhook `subject`, `text`, and `envelope` fields. |
| 13 | Keeps separate delivery routes with their own inbound URLs, destinations, and redaction lists. |
| 11 | SQLite stores route settings, short-lived demo session IDs, and delivery metadata. |
| 9 | It does not store inbound bodies or evidence excerpts. |
| 7 | Requirements: Node 22+, Rust, and SQLite support. |
| 2 | Open `http://localhost:8080`. |
| 15 | First boot creates protected signing, admin, and inbound credentials in `data/` (or `/data` when mounted). |
| 8 | Set their corresponding environment variables to supply replacements. |
| 8 | Enter the admin token in the route builder. |
| 10 | Configure alert providers to send the inbound token in `x-envelope-token`. |
| 11 | Each `/api/v1` endpoint is rate limited by the first `X-Forwarded-For` address. |
| 6 | `/health` remains available for platform probes. |
| 13 | Each public product claim and its repeatable sandbox command is listed in `.factory/claims.json`. |
| 6 | Demo behavior is documented in `.factory/demo.md`. |
| 11 | The container serves the built frontend and Rust API on `PORT`. |
| 11 | Durable SQLite state lives at `/data` when the platform mounts it. |
| 17 | The optional Field Kit costs $39 USD once and adds named redaction presets stored in this browser. |
| 14 | Redaction, signing, previews, copying envelopes, and route settings stay available without a license. |
| 7 | License tokens are stored in the browser. |
| 14 | Verification sends the token to Sociobot in an authorization header, not in a URL. |
| 8 | Privacy and Terms explain storage and purchase terms. |
| 1 | MIT. |
| 14 | The cartography was generated for this product on 2026-08-27; prompt metadata is in `assets/src`. |
| 7 | Inter and Fraunces notices are in `THIRD_PARTY_NOTICES.md`. |

No README sentence exceeds 22 words. F-3-3 is the only README wording flag.

## Structure, accessibility, and links

- `/`, `/demo`, `/privacy`, and `/terms` returned 200. The designed unknown
  route returned 404 and includes a way back.
- Each checked route had `lang="en"`, exactly one `h1`, a `main` landmark,
  route-specific title, description, canonical URL, OG/Twitter card and image,
  favicon, and Apple touch icon. `robots.txt` and `sitemap.xml` were present.
- The live response uses HSTS, `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and a response-header CSP with
  `frame-ancestors 'none'`.
- All normal internal and external links resolved 200. The in-page skip link
  on the intentionally 404 document correctly stays on that document; it is
  not treated as a dead destination. F-3-5 covers the inaccurate recovery
  target and external-label inconsistency.
- Live axe scans at 390 × 844 in light and dark found zero serious or critical
  WCAG 2 A/AA/2.1 AA violations on home, demo, privacy, terms, and 404.
- The topographic survey-paper identity is distinct and conforms to the
  recorded cartography design direction. It is not a generic SaaS layout.

## Earlier-finding audit

| Earlier finding | Live/code status in this review |
| --- | --- |
| F-1-1 | Fixed: completed redacted result is fully within the mobile claim viewport. |
| F-1-2 | Fixed: license transport uses an authorization header, not a URL token. |
| F-1-3 | Regressed as F-3-2: Terms expands the tested free-feature promise. |
| F-1-4 | Fixed: source and query mutations have a dedicated passing test. |
| F-1-5 | Fixed: protected file and browser/API exposure checks are separate and pass. |
| F-1-6 | Fixed: preview does not add history. |
| F-1-7 | Fixed: merchant/refund wording remains absent. |
| F-1-8 | Fixed: unsupported request-size copy remains absent. |
| F-1-9 | Fixed: unsupported no-destination behavior remains absent. |
| F-1-10 | Fixed: unsupported logging promise remains absent. |
| F-1-11 | Fixed: test-process claims are not public product copy. |
| F-1-12 | Fixed: the 23:59 and 24:00 throttle boundaries are tested. |
| F-1-13 | Fixed: deployment wording matches the scoped policy claim. |
| F-1-14 | Fixed: dated provenance and MIT status are tested. |
| F-1-15 | Fixed: home metadata says what the product does. |
| F-1-16 | Regressed as F-3-1 on navigation to/from static legal pages. |
| F-1-17 | Fixed: 404 has metadata, shared styling, and recovery actions (target detail is F-3-5). |
| F-1-18 | Fixed: the earlier terminology replacements remain present. |
| F-1-19 | Fixed: process headings name their objects. |
| F-1-20 | Fixed: primary controls name their result. |
| F-1-21 | Fixed: the demo compares two isolated routes with different policies. |
| F-2-1 | Fixed: the test asserts complete boxes, not merely top edges. |
| F-2-2 | Fixed for the registered core controls; the new broader Terms statement is F-3-2. |
| F-2-3 | Fixed: response, DOM, and local storage credential markers are checked. |
| F-2-4 | Fixed: merchant/refund claims remain absent. |
| F-2-5 | Fixed: both demo routes are visible and test-covered. |
| F-2-6 | Fixed: the shared limiter test covers the 40/20 contract and first IP. |
| F-2-7 | Fixed: the throttle test covers both sides of 24 hours. |
| F-2-8 | Fixed: unbounded “safe”/“verified” landing copy remains absent. |
| F-2-9 | Fixed: JSON, Slack, and email delivery contracts are captured in a test. |
| F-2-10 | Fixed: current headings and helper labels are descriptive. |
| F-2-11 | Fixed: legal pages now have full header/footer information. |
| F-2-12 | Fixed: README setup actions are separate sentences. |

## Missed leverage

No additional AI feature is implied by this deterministic, security-sensitive
transformation. The valuable import/export and route-comparison behavior is
already present: the demo compares policies and the UI copies the signed
envelope. There is no embedded provider key or decorative AI feature.

## What would make this perfect

Make legal-route and Back navigation focus and announce the destination,
narrow or prove the Terms free-feature statement, correct the README delivery
sentence and split the mobile price fact, and make the 404 recovery and
external-source label match their stated results. Then rerun all claim
commands, the live navigation check, link crawl, and mobile axe scan.
