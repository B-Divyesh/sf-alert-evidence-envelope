# Adversarial first-read review 5 — Alert Evidence Envelope

Date: 2026-09-02

Work order: `alert-evidence-envelope-review-5`  
Repository reviewed: `15b487808b03ccddb62b1ef6992658159cc3d7ac`  
Live build: `34c18ffe0d3d779a07baf2620969cd89636a7a60`

## Verdict: FAIL

One blocking demo-isolation race remains. All registered claim commands passed,
and the ordinary one-click demo is clear and functional. A pass is not
possible because a visitor can begin **Reset demo** and immediately select
**Start for real**, causing the still-running demo task to call a protected
production preview endpoint with sample data.

## Cold first screen

Fresh Chromium contexts with no prior storage, cookies, or service worker were
opened at 390 × 844 and 1440 × 900. Before scrolling, the answers were:

| Question | Answer from the first screen |
| --- | --- |
| What does it do? | It adds redacted evidence to webhook alerts. |
| For whom? | On-call engineers and webhook consumers who need incident context. |
| What should I select? | **Try it with sample data**. |

This check passes. The mobile screen visibly contained “Add redacted evidence
to webhook alerts”, “For on-call engineers and webhook consumers who need
incident context without another dashboard login.”, **Try it with sample
data**, “The sample opens a signed, redacted envelope in an isolated
workspace.”, and the three product facts. No console error appeared on the
landing page. The two viewport screenshots are
`/tmp/aee-review5-mobile-cold.png` and `/tmp/aee-review5-desktop-cold.png`.

## Findings

### Blocking

#### F-5-1 — Reset and exit can send demo data to the protected production preview

- **Location:** `/?demo=1`; `frontend/src/App.svelte`, `startDemo()`,
  `runPreview()`, and `leaveDemo()`.
- **Exact affected promise:** “Demo — sample data, nothing is saved”; “The
  sample opens a signed, redacted envelope in an isolated workspace.”
- **Reproduction:** open `/?demo=1` in a fresh 390 × 844 context. When the
  result is shown, select **Reset demo** and immediately select **Start for
  real**, before the asynchronous reset creates its replacement session.
- **Observed request log:** after the expected demo-session `DELETE` and
  replacement-session `POST`, the live browser made `POST /api/v1/preview`.
  That is a protected production endpoint, not
  `/api/v1/demo/sessions/<id>/preview`. The attempted exit also left the
  `demo:alert-evidence-envelope:session` key in browser storage in this race.
- **Code confirmation:** `startDemo()` awaits the new demo session and then
  calls `runPreview()`. `runPreview()` chooses its endpoint from the mutable
  global `path`. `Start for real` runs `leaveDemo()` and changes `path` to
  `/` without cancelling or awaiting the reset. If the session request
  resolves afterwards, `runPreview()` selects `/api/v1/preview` instead of
  the demo endpoint. An admin token already loaded in the route builder would
  also be retained in memory and attached by `api()`.
- **Why this fails:** the required sandbox boundary is not merely a normal-path
  convention. An ordinary fast pair of visible controls can direct a demo
  payload into the real API. The current unauthenticated reproduction receives
  the production authentication response, but it still crosses the forbidden
  boundary; an authenticated visitor could run a protected preview. The demo
  promise is therefore not reliable.
- **Concrete fix:** give each demo start a cancellation token or `AbortController`.
  Capture the demo session ID and route mode before every await, and return
  without previewing when that token is no longer current or the page has
  left demo mode. Disable **Reset demo** and **Start for real** while reset is
  pending. Ensure `leaveDemo()` awaits/cancels pending work before changing
  route. Expand the existing `@claim:isolated-demo` browser test to click
  Reset and Start for real without waiting, both with and without an admin
  token, and assert no request reaches `/api/v1/config`, `/channels`,
  `/history`, `/preview`, or `/relay`, and no `demo:` key remains.

## Demo and sandbox checks

The normal path passes and is distinct from F-5-1:

- `/` → **Try it with sample data** reached `/?demo=1` in one click.
- The initial 390 × 844 demo result was completely visible at x=12,
  y=286.69, width=366, height=314.78. It showed signed state, redaction,
  the Customer automation policy, `checkout-api`, the timeout, first-seen
  time, 2 items, 213 B, no truncation, and fingerprint `b44f90d5c75de84a`.
- The persistent banner said “Demo — sample data, nothing is saved” and had
  **Reset demo** and **Start for real**.
- When the reset was allowed to finish, it replaced
  `e7e16713-a40c-4ee8-a508-b5ba7a26d857` with
  `aa1b2d79-78a9-4d52-b3d1-a9f1ad0a474b`. A subsequent normal exit cleared all
  `demo:` keys and made no protected request.
- A fresh normal demo logged only the product origin. It made only the shell,
  `/health`, and demo-session calls; no analytics, advertising, hosted-font,
  or third-party request was observed.
- The separate clean-clone `@claim:offline-demo` command passed with its own
  context. It reloads the cached sample offline without health or API calls.

## Claims

From clean clone `/tmp/aee-review5-1dF5G6`, `npm ci` completed without
vulnerabilities. Every exact `test` command in `.factory/claims.json` was
run; all 28 passed. The command log is
`/tmp/aee-review5-claim-commands.log`.

| Registered claim IDs with passing commands |
| --- |
| `demo-envelope`, `mobile-demo-result`, `demo-route-policies`, `bounded-redacted-signed`, `query-fingerprint`, `fixed-query-source`, `provider-signature` |
| `slack-delivery`, `isolated-demo`, `raw-not-retained`, `history-limit`, `protected-real-apis`, `credential-storage`, `credential-browser-exposure` |
| `preview-no-history`, `per-route-isolation`, `no-tracking`, `offline-demo`, `license-transport`, `local-policy-presets`, `license-throttle` |
| `free-core`, `field-kit-purchase`, `license-revocation`, `provenance-license`, `durable-deployment`, `rate-limit`, `destination-contracts` |

The complete clean-clone suites also passed: `cargo test --locked` (25
tests), the full browser suite (60 project cases, including the intentional
desktop skip for the mobile-only geometry assertion), and `npm run build`.
No listed test failed. F-5-1 exposes a scenario the current isolated-demo
test does not cover.

Cross-checking the live landing and README against `claims.json` found no
additional unlisted product claim. The bounded/redacted/signed demo,
privacy/network statements, price, free controls, delivery contracts,
credentials, history, and provenance all map to a listed claim. The visual
“BOUNDARY 32 KB” is also observed by the demo-envelope browser assertion,
which requires evidence bytes at or below 32,768.

## Copy audit

Counts treat hyphenated terms, URLs, code tokens, and prices as one word.
Sample JSON is editable data, not landing copy. No landing or README sentence
exceeds 22 words. No banned marketing adjective, metaphor heading, or
non-result-naming button was found. Buttons name their outcome: **Try it with
sample data**, **Configure your route**, **Copy relay URL**, **Build signed
preview**, **Copy envelope JSON**, **Reset demo**, and **Start for real**.

### Landing sentences and facts

| Words | Copy |
| ---: | --- |
| 6 | Add redacted evidence to webhook alerts |
| 14 | For on-call engineers and webhook consumers who need incident context without another dashboard login. |
| 11 | The sample opens a signed, redacted envelope in an isolated workspace. |
| 8 | Demo data is never added to route history |
| 5 | No analytics or third-party scripts |
| 5 | The self-hosted core is free. |
| 5 | Field Kit costs $39 once. |
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
| 4 | Built by Param Factory. |

Landing headings are descriptive in isolation: **Four checks before
delivery**, **Limit the evidence**, **Remove sensitive fields**, **Record the
source and query**, **Sign the envelope**, **Configure delivery routes**,
**Envelope preview**, **Recent delivery metadata**, and **Reuse redaction
policies**. The product terminology remains consistent: alert, envelope,
route, demo, delivery metadata, admin token, inbound token, and Field Kit.

### README sentences and facts

| Words | Copy |
| ---: | --- |
| 8 | Add bounded, redacted, signed evidence to webhook alerts. |
| 6 | For on-call engineers and webhook consumers. |
| 15 | It builds an evidence envelope from alert JSON, then delivers it to a configured destination. |
| 5 | Try it with sample data. |
| 9 | The isolated sample shows a checkout timeout with redacted values. |
| 8 | Limits evidence by record count and byte size. |
| 8 | Removes configured sensitive fields, including nested fields. |
| 10 | Records a fingerprint from the fixed source and alert query. |
| 5 | Signs the envelope with HMAC-SHA256. |
| 11 | Sends the signed envelope and signature header to a JSON webhook. |
| 9 | Sends Slack the envelope plus a readable `text` field. |
| 9 | Sends an email gateway webhook `subject`, `text`, and `envelope` fields. |
| 12 | Keeps separate delivery routes with their own inbound URLs, destinations, and redaction lists. |
| 10 | SQLite stores route settings, short-lived demo session IDs, and delivery metadata. |
| 10 | It does not store inbound bodies or evidence excerpts. |
| 5 | Requirements: Node 22+, Rust, and SQLite support. |
| 5 | Open `http://localhost:8080`. |
| 16 | First boot creates protected signing, admin, and inbound credentials in `data/` (or `/data` when mounted). |
| 9 | Set their corresponding environment variables to supply replacements. |
| 9 | Enter the admin token in the route builder. |
| 11 | Configure alert providers to send the inbound token in `x-envelope-token`. |
| 14 | Each `/api/v1` endpoint is rate limited by the first `X-Forwarded-For` address. |
| 6 | `/health` remains available for platform probes. |
| 12 | Each public product claim and its repeatable sandbox command is listed in `.factory/claims.json`. |
| 5 | Demo behavior is documented in `.factory/demo.md`. |
| 11 | The container serves the built frontend and Rust API on `PORT`. |
| 11 | Durable SQLite state lives at `/data` when the platform mounts it. |
| 16 | The optional Field Kit costs $39 USD once and adds named redaction presets stored in this browser. |
| 11 | Redaction, signing, previews, copying envelopes, and route settings stay available without a license. |
| 7 | License tokens are stored in the browser. |
| 13 | Verification sends the token to Sociobot in an authorization header, not in a URL. |
| 10 | Privacy and Terms explain storage and purchase terms. |
| 1 | MIT. |
| 12 | The cartography was generated for this product on 2026-08-27; prompt metadata is in `assets/src`. |
| 9 | Inter and Fraunces notices are in `THIRD_PARTY_NOTICES.md`. |

No copy finding is opened: every entry is at or below the cap, uses the
consistent terminology table above, and tells the reader an action or fact.

## Structure, accessibility, and links

- Live `/`, `/demo`, `/privacy`, and `/terms` return 200. Unknown routes and
  `/404` return the designed 404 response. `robots.txt`, `sitemap.xml`, icon,
  apple icon, and social card return 200.
- Titles follow the required pattern, including **Demo — Alert Evidence
  Envelope**, **Privacy — Alert Evidence Envelope**, **Terms — Alert Evidence
  Envelope**, and **Page not found — Alert Evidence Envelope**. Canonical,
  description, Open Graph/Twitter, favicon, `lang`, one h1, and `main` are
  present.
- Keyboard focus and route announcements passed in the clean-clone browser
  tests for in-app navigation, static legal pages, and browser Back. The
  header/footer route chrome is consistent and the 404 recovery link names
  and reaches `/#configure`.
- Live Axe scans at 390 px in light and dark across `/`, demo, Privacy, Terms,
  and the 404 found zero serious or critical violations. The browser logs a
  document-level 404 load message only for the intentionally 404 route; no
  product-script error was emitted.
- The cartographic field-instrument surface, original contour art, engraved
  typography, paper/pine/amber palette, and 4-step route line are distinct
  from a generic SaaS template and match `.factory/design.md`.

## Earlier findings rechecked

Every earlier finding was checked again on the live build and against the
current source. These are fixed; F-5-1 is new and is not a regression of any
item below.

| Earlier finding | Verification result |
| --- | --- |
| F-1-1, F-2-1, F-4-1 | Normal mobile demo result is fully above 844 px; current geometry is y=286.69–601.47. |
| F-1-2 | License verification uses an authorization header; Privacy states storage and transport separately. |
| F-1-3, F-2-2, F-3-2 | Free-control wording matches `free-core` and Terms uses the narrow, tested list. |
| F-1-4 | Source-and-query fingerprint test passes. |
| F-1-5, F-2-3 | File protection and browser/API credential exposure have separate passing claims. |
| F-1-6 | Protected preview no-history claim passes. |
| F-1-7, F-2-4 | Untestable merchant/refund wording remains absent; revocation is tested. |
| F-1-8, F-1-9, F-1-10, F-1-11 | Unsupported size, no-destination, secret-log, and suite-coverage promises remain absent from public copy. |
| F-1-12, F-2-7 | 24-hour throttle test covers attempt timestamp and boundary. |
| F-1-13 | Deployment copy matches the tested non-root, one-replica, `/data` contract. |
| F-1-14 | Generated-art provenance and MIT/font notices are test-backed. |
| F-1-15 | Home/OG/Twitter title says “add evidence to alerts”. |
| F-1-16, F-3-1 | In-app, legal, Back, and forward navigation focus and announce the new h1. |
| F-1-17, F-3-5 | 404 has complete metadata/chrome and both builder links target `/#configure`. |
| F-1-18, F-1-19, F-1-20, F-2-8, F-2-10, F-3-3, F-3-4 | Current copy uses descriptive headings, plain terms, result-naming controls, a correct webhook sentence, and separate price facts. |
| F-1-21, F-2-5 | Protected routes are independent and demo compares Internal Slack with Customer automation policies. |
| F-2-6 | 40-burst/20-per-second first-forwarded-IP limiter contract is covered by a deterministic Rust test. |
| F-2-9 | JSON, Slack, and email webhook contracts have local-capture tests and explicit UI copy. |
| F-2-11 | Legal pages now have shared navigation, product line, legal links, source, factory credit, and build ID. |
| F-2-12 | README separates admin-token and inbound-token setup. |

## What would make this perfect

Cancel or serialize demo work at every exit/reset boundary and prove that
rapid action changes cannot call any protected endpoint or retain demo keys.
After that fix and its regression test, this review has no remaining product,
copy, structure, claim, accessibility, or missed-leverage finding. An AI step
is not warranted: bounded extraction, redaction, signing, and deterministic
delivery are the core job and are better kept explicit.
