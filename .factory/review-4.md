# Adversarial first-read review 4 — Alert Evidence Envelope

Date: 2026-09-02

Work order: `alert-evidence-envelope-review-4`

Repository commit reviewed: `df58f602b019827a3afe29d1df749f9ef2abcc54`

Live build from `/health`: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

## Verdict: FAIL

One blocking finding remains. The registered mobile-demo claim says the
completed result is fully visible at 390 × 844, but the live first-seen value,
item/byte/truncation summary, and query fingerprint are below the first
viewport. The tagged test passes because it does not inspect those parts of
the result. This reopens the original mobile-demo finding.

No other finding was identified. A PASS requires zero findings and no
untested claim.

## Cold first screen

Fresh Chromium contexts with empty storage were opened at 390 × 844 and
1440 × 900. Before scrolling, the answers were the same:

- What it does: adds redacted evidence to webhook alerts.
- For whom: on-call engineers and webhook consumers who need incident context.
- What to click first: **Try it with sample data**.

The exact copy supporting those answers was “Add redacted evidence to webhook
alerts”, “For on-call engineers and webhook consumers who need incident
context without another dashboard login.”, and “Try it with sample data”. The
next-result sentence and all three facts were also visible in both first
screens. This check passes.

## Findings

### Blocking

#### F-4-1 / F-1-1 / F-2-1 — The claimed complete mobile demo result is not in the first viewport

- Location: live `/?demo=1`, fresh 390 × 844 context, before scrolling;
  `.factory/claims.json` entry `mobile-demo-result`; and
  `tests/browser/app.spec.ts:367`.
- Exact claim: “The completed sample result, including its redaction state,
  is fully visible in the first 390 by 844 pixel demo viewport.”
- Exact result copy affected: “First seen”, “8/27/2026, 2:32:08 PM”, “2
  items”, “213 B evidence”, “No truncated”, and “Query fingerprint”.
- Evidence: the signed status ended at y=606, redaction state at y=648,
  service at y=771, and error at y=830. The first-seen value occupied
  y=864–889; the fingerprint began at y=1125. The viewport ended at y=844.
  The first screen screenshot therefore ends after the error value.
- Test gap: `@claim:mobile-demo-result` checks only `checkout-api`, the error,
  `[REDACTED]`, and signed status. It does not assert the first-seen value,
  the item/byte/truncation summary, the fingerprint, or the complete result
  container. The exact command passes while leaving the registered claim
  untested.
- Why this blocks: the brief's success measure requires service, error
  signature, and first-seen time. A phone visitor cannot see all three in the
  promised first screen. This is a half-fix of the earlier F-1-1/F-2-1 issue.
- Concrete fix: compact the demo banner and heading or the result card until
  the whole completed card, through the fingerprint, ends at or above y=844.
  Extend the claim test to assert the bottom edge of the complete result
  container and individually assert first seen, all three bounds fields, and
  the fingerprint. Run it in the mobile project only or assert against the
  actual project viewport.

## Demo and sandbox behavior

- The landing action opens `/?demo=1` in one click and immediately builds a
  checkout-timeout envelope with realistic evidence.
- The persistent banner says “Demo — sample data, nothing is saved” and
  provides **Reset demo** and **Start for real**.
- Reset sent `DELETE` for the old demo session, created a different session
  ID, and rebuilt the sample. Start for real cleared every `demo:` browser key.
- Customer automation redacts `email` and `token`; Internal Slack retains the
  sample email and redacts `token`.
- The demo request log used only the product origin. It called only demo
  session endpoints and never called protected config, route, history,
  preview, or relay endpoints.
- A dedicated offline context reloaded the cached demo with the signed sample.
  It made no API request and had no failed request while offline.
- The temporary-SQLite isolation claim passed. No demo payload entered route
  history or protected route storage.

The demo is realistic and isolated. F-4-1 concerns the false first-viewport
claim, not sandbox separation.

## Claim audit

I cloned the reviewed commit without shared working files to
`/tmp/aee-review4-clean-4IlcYG`, ran `npm ci`, and ran every exact `test`
command in `.factory/claims.json`. All 28 commands returned success.

| Claim | Command result | Claim | Command result |
| --- | --- | --- | --- |
| `demo-envelope` | PASS | `mobile-demo-result` | PASS, but incomplete assertion; F-4-1 |
| `demo-route-policies` | PASS | `bounded-redacted-signed` | PASS |
| `query-fingerprint` | PASS | `fixed-query-source` | PASS |
| `provider-signature` | PASS | `slack-delivery` | PASS |
| `isolated-demo` | PASS | `raw-not-retained` | PASS |
| `history-limit` | PASS | `protected-real-apis` | PASS |
| `credential-storage` | PASS | `credential-browser-exposure` | PASS |
| `preview-no-history` | PASS | `per-route-isolation` | PASS |
| `no-tracking` | PASS | `offline-demo` | PASS |
| `license-transport` | PASS | `local-policy-presets` | PASS |
| `license-throttle` | PASS | `free-core` | PASS |
| `field-kit-purchase` | PASS | `license-revocation` | PASS |
| `provenance-license` | PASS | `durable-deployment` | PASS |
| `rate-limit` | PASS | `destination-contracts` | PASS |

The live landing page and README were cross-checked against the manifest. No
additional unlisted claim-like sentence was found. `mobile-demo-result` is
listed but not fully tested and is false on the live viewport, so there is
still one untested claim.

## Copy audit

Counts use whitespace-delimited words. Hyphenated terms, prices, paths, and
code tokens count as one word. Sample JSON and values produced from it are
data, not landing copy.

No landing or README sentence exceeds 22 words. No banned marketing adjective,
inconsistent product term, empty slogan, metaphor heading, or unclear action
label was found. The tables below list every sentence and factual line.

### Landing-page sentences and factual lines

| Words | Copy |
| ---: | --- |
| 6 | Add redacted evidence to webhook alerts |
| 14 | For on-call engineers and webhook consumers who need incident context without another dashboard login. |
| 11 | The sample opens a signed, redacted envelope in an isolated workspace. |
| 8 | Demo data is never added to route history. |
| 5 | No analytics or third-party scripts. |
| 5 | The self-hosted core is free. |
| 5 | Field Kit costs $39 once. |
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
| 15 | Built by Param Factory · Build `f4bf8ae31eb1` · Cartography generated for this product on 2026-08-27 · MIT licensed. |

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
| 11 | Sends the signed envelope and signature header to a JSON webhook. |
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

### Headings, terms, and actions

The first-screen headline has six words and names the job. Section headings
such as “Four checks before delivery”, “Configure delivery routes”, “Inspect
an envelope before delivery”, “Recent delivery metadata”, and “Reuse redaction
policies” make sense outside their sections. Action controls name their result:
**Try it with sample data**, **Configure your route**, **Load protected route**,
**Save route**, **Copy relay URL**, **Build signed preview**, **Copy envelope
JSON**, **Reset demo**, **Start for real**, **Buy the Field Kit**, and **Verify
license**. No copy rewrite finding is required.

Terminology is consistent: incoming message = alert; transformed output =
envelope; production settings = route; isolated walkthrough = demo; stored
facts = delivery metadata; optional presets = Field Kit.

## Structure, navigation, accessibility, and links

- `/`, `/?demo=1`, `/demo`, `/privacy`, and `/terms` returned 200. The
  designed unknown route returned 404 and supplied working recovery actions.
- Every checked route had `lang="en"`, one `<h1>`, one `<main>`, a plain
  route-specific title, description, canonical URL, OG/Twitter metadata,
  product social image, favicon, Apple touch icon, and palette theme color.
- `robots.txt` and `sitemap.xml` returned 200. The sitemap lists all four
  public routes.
- The response CSP is a header and includes `frame-ancestors 'none'`.
  HSTS, `nosniff`, `DENY`, and `no-referrer` were present.
- Home → Privacy focused “How this relay handles data” and announced
  “Privacy — Alert Evidence Envelope”. Browser Back focused the home h1 and
  announced its title.
- All unique internal links, the Sociobot checkout, and the external source
  link resolved successfully. Fragment targets were present.
- Twelve live Axe runs covering six routes in light and dark modes found zero
  serious or critical violations. The factory URL verifier found one h1, a
  main landmark, no missing image alt, no unlabeled button, and no console
  error on home.
- The CSS provides visible focus, 44 px controls, mobile reflow, dark colors,
  and a reduced-motion rule. No 390 px horizontal overflow was observed.
- The topographic survey-paper palette, clipped cartographic image, route
  line, serif/sans pairing, and square field-instrument controls match
  `.factory/design.md`. The identity is distinct rather than a generic SaaS
  template.

## Earlier-finding audit

Every earlier finding was checked against live behavior and current source.

| Earlier finding | Current status |
| --- | --- |
| F-1-1 | **Half-fixed and reopened as F-4-1:** signed/redacted service and error fit, but first seen and the rest of the completed result do not. |
| F-1-2 | Fixed: verification sends the license only in an authorization header; Privacy states the transport. |
| F-1-3 | Fixed: landing and Terms now match the tested free controls. |
| F-1-4 | Fixed: the fingerprint test changes source and query independently. |
| F-1-5 | Fixed: protected files and browser/API non-exposure have separate passing tests. |
| F-1-6 | Fixed: protected previews leave history unchanged. |
| F-1-7 | Fixed: untestable merchant/refund wording is absent; revocation preserves free controls. |
| F-1-8 | Fixed: the unsupported 256 KB public promise is absent. |
| F-1-9 | Fixed: unsupported no-destination behavior is absent from public copy. |
| F-1-10 | Fixed: unsupported secret-log wording is absent from public copy. |
| F-1-11 | Fixed: build-process prose is not presented as product behavior. |
| F-1-12 | Fixed: failed attempts are timestamped and the 23:59/24:00 boundaries pass. |
| F-1-13 | Fixed: deployment wording matches the inspected non-root, one-replica, `/data` policy. |
| F-1-14 | Fixed: dated asset provenance, MIT license, and font notices are present and tested. |
| F-1-15 | Fixed: home title says “add evidence to alerts”. |
| F-1-16 | Fixed: home/demo/legal navigation and Back focus and announce the destination h1. |
| F-1-17 | Fixed: the 404 has complete metadata, shared chrome, and working recovery links. |
| F-1-18 | Fixed: earlier jargon and inconsistent terms remain absent. |
| F-1-19 | Fixed: process headings name their subject. |
| F-1-20 | Fixed: relay and preview actions name their result. |
| F-1-21 | Fixed: protected routes are independent and the demo compares two policies. |
| F-2-1 | **Half-fixed and reopened as F-4-1:** the assertion still does not cover the complete result promised by the claim. |
| F-2-2 | Fixed: the unlicensed test edits route settings, previews, signs, and copies. |
| F-2-3 | Fixed: API response, DOM, storage, and protected-file checks cover credential exposure. |
| F-2-4 | Fixed: merchant/refund promises remain absent. |
| F-2-5 | Fixed: Internal Slack and Customer automation visibly use different redaction policies. |
| F-2-6 | Fixed: shared limiter constants and first-forwarded-IP behavior are tested at 40/20. |
| F-2-7 | Fixed: the throttle test covers both sides of 24 hours. |
| F-2-8 | Fixed: unbounded “safe” and “verified” product claims remain absent. |
| F-2-9 | Fixed: JSON, Slack, and email webhook contracts have captured-payload tests. |
| F-2-10 | Fixed: headings and helper labels remain descriptive. |
| F-2-11 | Fixed: Privacy and Terms have standard navigation, footer, source, and build ID. |
| F-2-12 | Fixed: README setup actions remain separate sentences. |
| F-3-1 | Fixed: static legal navigation and browser Back focus and announce the route. |
| F-3-2 | Fixed: Terms uses the registered free-core wording. |
| F-3-3 | Fixed: README now says the envelope and signature header are sent “to” a JSON webhook. |
| F-3-4 | Fixed: the first-screen price line is two sentences. |
| F-3-5 | Fixed: both 404 builder links target `/#configure`, and the source link says “Source (external)”. |

## Missed leverage

No additional AI feature is warranted for deterministic, security-sensitive
redaction and signing. The brief's obvious leverage is isolated per-route
policies and delivery to Slack, email gateways, and JSON automation; those
features exist and have tests. Copy/export is present for envelopes, and no
decorative AI control or provider key appears in the product.

## Additional verification

- `npm test`: exit 0; 25 Rust tests and 60 browser cases completed. One
  browser case retried successfully after the Chromium process itself
  terminated with `SIGSEGV`; the same claim had already passed independently.
- `npm run build`: PASS; `dist/` produced; initial JS was 26.65 KB gzip.
- `cargo fmt --check`: PASS.
- `cargo clippy --all-targets --locked -- -D warnings`: PASS.
- `/opt/fleet/lib/verify-url.sh`: PASS; 722 ms cold load and no home console
  errors.
- Live light/dark Axe checks: zero serious or critical findings.

## What would make this perfect

Make the entire completed mobile result—including first seen, bounds, and
fingerprint—visible before y=844. Make the claim test measure the whole result
instead of four selected strings. After that change, rerun all 28 claim
commands and repeat the cold 390 × 844 screenshot check. Nothing else remains
from this review.
