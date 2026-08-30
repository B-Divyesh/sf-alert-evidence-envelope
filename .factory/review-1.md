# Adversarial first-read review 1 — Alert Evidence Envelope

Date: 2026-08-30

Work order: `alert-evidence-envelope-review-1`

Repository commit reviewed: `7aacde3d99a4de553ccfbef786ef1474f16e69c1`

Live build reported by `/health`: `5c10f93da6f4e95c64ed9f9cc70b06b81f08df83`

## Verdict: FAIL

Two findings are blocking. The one-click demo does not show its transformed result in the first 390 px viewport, and the privacy page says license tokens remain in the browser even though the code sends them to Sociobot in a URL query. The claims inventory also omits multiple statements that visitors are asked to rely on. A PASS requires zero findings and no untested claim.

## Cold first screen, before scrolling

Fresh Chromium contexts were used at 390 × 844 and 1440 × 900.

My answers from the first screen were the same at both sizes:

- What it does: adds a bounded, redacted, signed evidence envelope to webhook alerts.
- For whom: on-call engineers and systems that consume webhooks.
- What to click first: **Try it with sample data**.

This check passes. The mobile first screen showed the headline, audience sentence, sample action, result note, and all three facts without scrolling. Exact supporting copy was “Send safe evidence with every alert”, “For on-call engineers and webhook consumers who need incident context without another dashboard login.”, and “Try it with sample data”.

## Findings

### Blocking

#### F-1-1 — The mobile demo shows input before the transformed result

- Location: `/demo`, 390 × 844, immediately after selecting **Try it with sample data**.
- Exact copy in view: “Inspect a sample evidence envelope”, “The sample runs automatically in an isolated workspace. It never changes the protected route.”, then “Sample alert JSON”.
- Evidence: the first result container began at pixel 968 in an 844 px viewport. The sealed result, redacted fields, summary, and “Envelope signed” status were not visible before scrolling.
- Why this fails: the job is transforming evidence, but the first demo screen shows only the raw input. A 30-second visitor cannot see the product's result without discovering that more content is below the fold. This is a weak demo under the required one-click demo contract.
- Concrete fix: on mobile, place the completed envelope summary before the editable JSON. Keep the banner and heading compact enough to show `checkout-api`, the error, first-seen time, `[REDACTED]`, and signed status in the first 844 px. Add a 390 × 844 test that asserts those result fields intersect the initial viewport.

#### F-1-2 — The privacy page makes a false locality claim about license tokens

- Location: `/privacy`, “Where secrets live”.
- Exact quote: “License tokens and paid presets remain in your browser.”
- Code evidence: `verifyLicense()` sends the license in `GET https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify?license=…`.
- Why this fails: the token is stored in the browser, but it does not remain there. It is transmitted to Sociobot and placed in a URL, where URLs may be logged. The next section says verification contacts Sociobot but does not say that the token is sent.
- Concrete fix: use “License tokens are stored in this browser and sent to Sociobot for verification. Paid presets stay in this browser.” Prefer a POST body or authorization header so the token is not in a URL. Add a claim entry and a request-interception test that verifies the disclosed destination, method, and payload and confirms presets never leave the browser.

### High

#### F-1-3 — Free-tier and gating claims are unlisted

- Locations/quotes: landing, “Self-hosted core is free; Field Kit costs $39 once”; landing/README, “The self-hosted relay and every safety control are free.”; landing, “No subscription or hosted data dependency”; terms, “Accessibility, export, redaction, signing, and every safety control remain free.”
- Why this fails: `field-kit-purchase` verifies the price and checkout URL, not which controls remain free or whether a subscription or hosted dependency exists.
- Concrete fix: add one claim entry with a fresh unlicensed-browser test that exercises redaction, signing, copy/export, and every safety control and asserts no subscription gate or hosted runtime call. Otherwise narrow the copy to the tested $39 price and local-preset entitlement.

#### F-1-4 — Query-fingerprint behavior is unlisted

- Location: landing, “Hash the configured query and source so responders know what shaped the excerpt.”
- Why this fails: no claim entry promises or independently tests that the displayed fingerprint is derived from both the configured query and source. The existing bounded/signing and fixed-source entries do not state this outcome.
- Concrete fix: add a claim entry and a Rust test with fixed query/source inputs, a known digest, and separate assertions that changing either input changes the fingerprint.

#### F-1-5 — Credential-storage claims are unlisted

- Locations/quotes: landing, “The server keeps credentials outside the browser.”; README, “The first boot creates three protected files”, “Each file has mode 600.”, and “Environment variables can supply values instead.”
- Why this fails: `protected-real-apis` proves authentication is required but does not prove where credentials are kept. Relevant unit coverage exists, but these claims have no `claims.json` entry.
- Concrete fix: add one credential-storage claim whose test starts in a temporary data directory, checks all three file modes and environment overrides, and confirms secrets do not appear in HTML, API responses, or browser storage.

#### F-1-6 — The production-preview history claim is unlisted

- Locations/quotes: landing, “It does not add delivery history.” and “Preview runs never appear here.”
- Why this fails: `isolated-demo` tests demo history isolation. It does not state or test the protected `/api/v1/preview` path promised by these sentences.
- Concrete fix: add a claim that runs an authenticated protected preview against temporary SQLite and confirms the history count and database bytes are unchanged.

#### F-1-7 — Billing and refund behavior is unlisted

- Locations/quotes: landing/README/terms, “Sociobot/Dodo is the merchant of record.”, “Refunds are handled there.”, and “A refunded or invalid license removes paid controls without blocking the free relay.”
- Why this fails: `field-kit-purchase` checks only displayed price and checkout URL. The valid-license browser test does not exercise an invalid or refunded verdict.
- Concrete fix: add recorded billing fixtures for valid, invalid, and refunded responses. Assert that invalid/refunded responses hide only preset controls, preserve the free relay controls, and show the merchant/refund copy. If merchant status cannot be tested in the sandbox, remove the untestable sentence.

#### F-1-8 — The 256 KB request-limit claim is unlisted

- Location: README, “`POST /api/v1/relay/primary` accepts vendor-neutral JSON up to 256 KB.”
- Why this fails: no claim entry sends payloads at and above the stated number.
- Concrete fix: add a claim test that accepts a valid payload at the documented boundary and rejects one above it with a useful status and error.

#### F-1-9 — No-destination behavior is unlisted

- Location: README, “With no destination, the relay returns the signed envelope and records delivery metadata.”
- Why this fails: no claim entry covers both the response and history side effect for this configuration.
- Concrete fix: add an authenticated temporary-database test with no destination, then assert the returned signature and the exact metadata row while confirming no raw body is stored.

#### F-1-10 — Secret-free logging is unlisted

- Location: README, “It never logs secret values.”
- Why this fails: this is a security claim with no claim entry or captured-log assertion.
- Concrete fix: start the server with unique marker secrets, exercise boot and failing requests, capture stdout/stderr, and assert that no marker appears.

#### F-1-11 — README test-coverage claims are absent from the claims inventory

- Location: README, “`npm test` runs Svelte checks, Rust unit and route tests, deployment-policy checks, a production build, and Playwright 1.58.2.” and “It covers keyboard access, axe, legal pages, demo isolation, privacy, offline reload, metadata, security headers, and rate limits.”
- Why this fails: these statements are verifiable and passed in this review, but they are still public claims without entries in `claims.json`.
- Concrete fix: either move build-process descriptions to the handoff or add a static suite-composition claim that confirms the invoked commands and named coverage without relying on prose alone.

#### F-1-12 — The once-per-day verification claim is false after a failed request and is unlisted

- Location: `/privacy`, “The browser checks at most once each day.”
- Code evidence: `checkedAt` is written only after a successful JSON response. A network or server failure leaves no fresh verdict, so every reload calls the verification endpoint again.
- Why this fails: a visitor may rely on the stated network frequency when deciding whether to enter a license token.
- Concrete fix: write a last-attempt timestamp before the request and throttle retries for 24 hours, or rewrite the sentence as “After a successful check, the browser waits 24 hours before checking again.” Add a clock-controlled failed-request test.

### Medium

#### F-1-13 — Deployment-runtime statements are not fully represented by their claim

- Location: README, “The image runs as the non-root `envelope` user. It starts with only `PORT` supplied by the platform.”
- Why this fails: the durable-deployment test checks the Dockerfile's non-root directive, but the claim text does not include either public statement and does not test the live process identity or effective environment.
- Concrete fix: expand the claim wording and sandbox to inspect the built container's user and startup environment, or remove the runtime assertions from public copy.

#### F-1-14 — Public provenance and font-license claims are unlisted

- Locations/quotes: footer, “Original generated cartography · MIT licensed”; README, “The generated cartography is original to this product. Inter and Fraunces are distributed under the SIL Open Font License.”
- Why this fails: these are provenance and license statements a user can rely on, but no claim entry checks the source metadata, font notices, or repository license.
- Concrete fix: replace “original” with the verifiable provenance “Generated for this product on 2026-08-27; prompt metadata is in `assets/src`.” Add a static claim test for the asset metadata, MIT `LICENSE`, font files, and `THIRD_PARTY_NOTICES.md`.

#### F-1-15 — The home title does not say what the product does

- Location: `/`, `<title>` and OG/Twitter title.
- Exact quote: “Alert Evidence Envelope — safe incident evidence”.
- Why this fails: “safe incident evidence” is a vague noun phrase, not the required “Product — what it does” pattern.
- Concrete fix: use “Alert Evidence Envelope — add evidence to alerts” everywhere the route title appears.

#### F-1-16 — Route changes do not move focus or announce the new page

- Location: navigating from `/` to `/demo`.
- Evidence: after the demo loaded, `document.activeElement` was `BODY`; the new `h1` had no `tabindex`; there was no route-title live region. Browser back returned to `/`, but focus again remained on `BODY`.
- Why this fails: keyboard and screen-reader users receive no explicit route-change focus or announcement, contrary to the route contract.
- Concrete fix: use one routing path with History API navigation, make the route `h1` programmatically focusable, focus it after navigation, and announce its text in a dedicated polite live region. Add click, back, forward, focus, and scroll-restoration tests.

#### F-1-17 — The designed 404 is missing required metadata and the standard site chrome

- Location: any unknown route, for example `/not-a-real-route`.
- Evidence: the route correctly returns 404 and has a title, description, one `h1`, favicon, and styled recovery actions. It has no canonical URL, no `og:url`, and no theme color. Its header offers only “Try sample data”; its footer omits the product one-liner and source link used by the product routes.
- Why this fails: route metadata and the consistent header/footer skeleton are incomplete.
- Concrete fix: add a canonical for the stable 404 page (or deliberately document a no-canonical policy), `og:url`, theme color, and the same bounded navigation/footer content as the other routes. Extend the metadata test to assert these fields on the 404.

### Copy findings

#### F-1-18 — The landing uses unexplained or inconsistent technical shorthand

Each flagged phrase needs its own plain rewrite:

| Exact copy | Issue | Proposed rewrite |
| --- | --- | --- |
| “Webhook evidence transformer” | Stacked jargon; it does not name the action. | “Add evidence to webhook alerts” |
| “Use a fixed source and item/byte caps.” | “item/byte caps” is compressed operator jargon. | “Use one fixed source. Limit the record count and envelope size.” |
| “channel-specific policy” | The rest of the product calls this a route, not a channel. | “this route’s redaction list” |
| “real-shaped, sanitized alert” | “real-shaped” is not ordinary English. | “realistic alert with sensitive values removed” |
| “The ledger stores metadata only.” | “Ledger” conflicts with the established term “delivery metadata”. | “Delivery history stores metadata only.” |
| README: “vendor-neutral JSON” | Abstract product jargon. | “JSON from any alert provider” |

#### F-1-19 — Four one-word headings do not make sense out of context

- Location: landing, “Bound”, “Redact”, “Fingerprint”, and “Seal”.
- Why this fails: a screen-reader heading list does not explain what is bounded, redacted, fingerprinted, or sealed.
- Concrete fixes: “Limit the evidence”, “Remove sensitive fields”, “Record the source and query”, and “Sign the envelope”.

#### F-1-20 — Two buttons do not name the result precisely

- Locations/quotes: “Copy URL” and “Build safe preview”.
- Why this fails: the first does not say which URL; the second uses the unmeasured adjective “safe” instead of the observable result.
- Concrete fixes: “Copy relay URL” and “Build signed preview”.

### Missed leverage

#### F-1-21 — The product cannot create separate per-channel routes

- Brief requirement: “Strict per-channel redaction” and delivery to “Slack/email/automation”.
- Current behavior: the UI edits one hard-coded `primary` route and chooses exactly one destination type and URL.
- Why this matters: a normal operator needs different fields for an internal Slack channel, a customer email gateway, and an automation webhook. One destination cannot apply per-channel disclosure rules.
- Concrete feature: add route list/create/duplicate/delete support. Give each route its own inbound URL, destination, source, caps, and redaction fields; preserve isolated history and credentials per route. Seed the demo with at least two routes that visibly redact different fields, and add claim tests for isolation. An AI feature is not warranted here: the deterministic redaction and signing job benefits more from explicit routes than model-generated output.

## Demo and sandbox evidence

- `/` → **Try it with sample data** opened `/demo` in one click.
- The sample was a checkout timeout with two evidence records, a customer email, and a token.
- The completed envelope showed service `checkout-api`, error `payment authorization timed out`, first-seen time, two `[REDACTED]` values, a query fingerprint, and a 64-hex HMAC signature.
- The persistent banner contained “Demo — sample data, nothing is saved”, **Reset demo**, and **Start for real**.
- Reset deleted the original server session (204), created a different session ID, and rebuilt the sample.
- Start for real removed all `demo:` localStorage keys.
- The complete live online/offline request log used only `https://alert-evidence-envelope.sociobot.in`.
- An offline reload retained the sample and displayed “Offline sample ready. Demo data was not stored.”
- The protected history endpoint returned 401 without credentials. The temporary-SQLite isolation test confirmed that demo operations cannot enter protected route history.

## Claim test results

Every exact command in `.factory/claims.json` ran from clean clone `/tmp/aee-review-s2cEaR`.

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-envelope` | PASS | 2/2 Playwright projects |
| `bounded-redacted-signed` | PASS | 1 Rust test |
| `fixed-query-source` | PASS | 1 Rust test |
| `isolated-demo` | PASS | 1 Rust test |
| `raw-not-retained` | PASS | 1 Rust test |
| `provider-signature` | PASS | 1 Rust test |
| `history-limit` | PASS | 1 Rust test |
| `protected-real-apis` | PASS | 1 Rust test |
| `no-tracking` | PASS | 2/2 Playwright projects |
| `offline-demo` | PASS | 2/2 Playwright projects, each with its own context |
| `rate-limit` | PASS | 2/2 Playwright projects |
| `local-policy-presets` | PASS | 2/2 Playwright projects |
| `field-kit-purchase` | PASS | 2/2 Playwright projects; live checkout reached HTTP 200 after redirect |
| `durable-deployment` | PASS | all three Rust tests, deployment policy, and the scoped live topology check; one replica, `/data`, 20 fresh previews |

No listed claim test failed. Findings F-1-2 through F-1-14 concern statements missing from the inventory or wording that exceeds what the listed test proves.

## Copy audit

Counting treats a hyphenated term, path, or displayed code token as one word. No landing or README sentence exceeds 22 words. No banned marketing word appears. Flags are F-1-18 through F-1-20; claim-like copy is handled separately above.

### Landing sentences

| Words | Sentence |
| ---: | --- |
| 6 | Send safe evidence with every alert |
| 14 | For on-call engineers and webhook consumers who need incident context without another dashboard login. |
| 11 | The sample opens a signed, redacted envelope in an isolated workspace. |
| 8 | Demo data is never added to route history |
| 5 | No analytics or third-party scripts |
| 9 | Self-hosted core is free; Field Kit costs $39 once |
| 7 | Use a fixed source and item/byte caps. |
| 7 | Alert data cannot choose an arbitrary endpoint. |
| 10 | Remove sensitive keys recursively with a channel-specific policy before forwarding. |
| 13 | Hash the configured query and source so responders know what shaped the excerpt. |
| 14 | Sign the final JSON envelope and preserve the provider signature in transit when present. |
| 5 | Route settings live in SQLite. |
| 7 | The server keeps credentials outside the browser. |
| 11 | Leave the source blank when evidence already arrives inside the alert. |
| 6 | A remote source receives only `?q=…&limit=…`. |
| 9 | Enter the server admin token to load this route. |
| 7 | Incoming requests must send the server’s `x-envelope-token`. |
| 10 | Preview applies the live route’s bounds, redaction, fingerprint, and signature. |
| 6 | It does not add delivery history. |
| 10 | Use the sample as-is or paste a real-shaped, sanitized alert. |
| 5 | The ledger stores metadata only. |
| 6 | Raw alerts and evidence are absent. |
| 8 | Send a live alert to the relay URL. |
| 5 | Preview runs never appear here. |
| 9 | The self-hosted relay and its safety controls are free. |
| 8 | The $39 Field Kit is a one-time purchase. |
| 8 | It adds named redaction presets on this device. |
| 6 | Sociobot/Dodo is the merchant of record. |
| 4 | Refunds are handled there. |
| 8 | Send bounded incident evidence with a webhook alert. |

Landing headings/actions were also checked: “Webhook evidence transformer” (3), “How it works” (3), “Four checks before delivery” (4), “Bound” (1), “Redact” (1), “Fingerprint” (1), “Seal” (1), “Protected route settings” (3), “Configure the alert route” (4), “Name and state” (3), “Locate evidence” (2), “Set the boundary” (3), “Address the envelope” (3), “Load protected route” (3), “Save route” (2), “Copy URL” (2), “Safe preview” (2), “Inspect an envelope before delivery” (5), “Build safe preview” (3), “No envelope yet” (3), “Last 20 deliveries” (3), “Recent delivery metadata” (3), “No delivery metadata yet” (4), “Optional local presets” (3), “Reuse redaction policies” (3), “Buy the Field Kit” (4), “Verify license” (2), and “Free core active” (3).

### README sentences and headings

| Words | Copy |
| ---: | --- |
| 3 | Alert Evidence Envelope |
| 8 | Send bounded incident evidence with a webhook alert. |
| 10 | This self-hosted transformer is for on-call engineers and webhook consumers. |
| 11 | It redacts evidence, applies caps, signs the envelope, and forwards it. |
| 5 | Try it with sample data. |
| 15 | The demo runs in an isolated, 24-hour workspace and does not change the protected route. |
| 17 | The relay does not evaluate alerts, manage incidents, retain raw payloads, or summarize with a language model. |
| 4 | How the route works |
| 9 | `POST /api/v1/relay/primary` accepts vendor-neutral JSON up to 256 KB. |
| 11 | The relay reads embedded evidence or queries one fixed HTTPS source. |
| 10 | It replaces configured sensitive keys with `[REDACTED]`, including nested keys. |
| 8 | It enforces item and byte caps before delivery. |
| 8 | It adds a query fingerprint and HMAC-SHA256 signature. |
| 7 | It forwards supported provider signatures as `x-original-provider-signature`. |
| 15 | SQLite stores route settings, expiring demo session IDs, and the latest 20 delivery metadata rows. |
| 9 | It does not store inbound bodies or evidence excerpts. |
| 2 | Run locally |
| 9 | Requirements: Node 22+, Rust 1.98+, and SQLite support. |
| 4 | Open `http://localhost:8080`. |
| 7 | The first boot creates three protected files: |
| 4 | `data/envelope-signing.key` signs envelopes. |
| 8 | `data/admin.token` authorizes route settings, preview, and history. |
| 6 | `data/inbound.token` authorizes incoming alert traffic. |
| 5 | Each file has mode 600. |
| 6 | Environment variables can supply values instead. |
| 13 | Enter the admin token in the route builder before loading or saving settings. |
| 9 | Alert providers must send `x-envelope-token` with the inbound token. |
| 13 | With no destination, the relay returns the signed envelope and records delivery metadata. |
| 1 | Configuration |
| 11 | The server logs whether each secret was generated, persisted, or supplied. |
| 5 | It never logs secret values. |
| 2 | Request limits |
| 12 | Every `/api/v1` endpoint uses the first `X-Forwarded-For` address as its client key. |
| 7 | It falls back to the socket address. |
| 6 | Each client receives a 40-request burst. |
| 7 | Capacity refills at 20 requests per second. |
| 7 | Rejected requests return 429 with `Retry-After: 1`. |
| 11 | `/health` is exempt so the deployment platform can probe the container. |
| 3 | Test and build |
| 20 | `npm test` runs Svelte checks, Rust unit and route tests, deployment-policy checks, a production build, and Playwright 1.58.2. |
| 9 | Browser coverage runs on desktop and 390 px Chromium. |
| 18 | It covers keyboard access, axe, legal pages, demo isolation, privacy, offline reload, metadata, security headers, and rate limits. |
| 12 | Every product claim and its sandbox command is listed in `.factory/claims.json`. |
| 6 | Demo details are in `.factory/demo.md`. |
| 2 | Container deployment |
| 8 | The deployment command builds in the factory registry. |
| 19 | It then mounts a product-specific Azure File share at `/data`, selects single-revision mode, and fixes scaling at one replica. |
| 8 | The work order sets `deploy.data_dir=/data`. |
| 14 | The container stores SQLite, signing identity, and access tokens directly on that durable mount. |
| 7 | The deployment fixes scaling at one replica. |
| 15 | Route settings and demo sessions therefore remain consistent across fresh HTTP connections and revision restarts. |
| 8 | The image runs as the non-root `envelope` user. |
| 9 | It starts with only `PORT` supplied by the platform. |
| 3 | Paid Field Kit |
| 9 | The self-hosted relay and every safety control are free. |
| 10 | The optional Field Kit is a $39 USD one-time purchase. |
| 9 | It adds named redaction presets stored in this browser. |
| 9 | Checkout and license verification use the Sociobot billing API. |
| 6 | Sociobot/Dodo is the merchant of record. |
| 13 | A refunded or invalid license removes paid controls without blocking the free relay. |
| 7 | See Privacy, Terms, and the visual rationale. |
| 1 | License |
| 1 | MIT. |
| 8 | The generated cartography is original to this product. |
| 11 | Inter and Fraunces are distributed under the SIL Open Font License. |

The README configuration table and shell blocks contain labels, values, and commands rather than sentences; they were checked for terminology but are not counted as sentences.

## Structure, accessibility, and link evidence

- `/`, `/demo`, `/privacy`, and `/terms` returned 200; the designed unknown route returned 404.
- All five checked pages had `lang=en`, one `h1`, and one `main`.
- The 1200 × 630 social image, SVG favicon, 180 px touch icon, `robots.txt`, and sitemap all returned 200.
- Sitemap entries covered `/`, `/demo`, `/privacy`, and `/terms`.
- Internal links returned 200. The Field Kit link reached a Dodo checkout with 200 after redirect; the source repository returned 200.
- The fresh `/#configure` deep link reached the configuration section after load. Browser back returned to `/`.
- Playwright axe checks found no serious or critical issue on all product, demo, legal, and 404 routes at mobile/desktop in light/dark modes.
- No route overflowed horizontally. Primary routes emitted no console or page errors.
- `/opt/fleet/lib/verify-url.sh` passed: title, language, one `h1`, main landmark, image alt text, button labels, and zero console errors.
- First-load JS was 66.32 KB uncompressed and 25.22 KB gzip.
- `npm test` passed: Svelte 0 errors/warnings, 18 Rust tests, deployment policy, production build, and 38/38 Playwright tests.
- The topographic field-instrument identity is distinct and follows `.factory/design.md`; it does not look like a generic gradient/card SaaS template.

## History

No earlier `.factory/review-*.md` or `.factory/polish-*.md` existed. The prior `.factory/handoff.md` contained no finding IDs and declared verification 7 a pass. Its listed checks were re-run from scratch where applicable: every registered claim passed, `npm test` passed, live build/topology matched the handoff, links and primary routes worked, and axe found no serious/critical issues. This review's findings are new first-read, inventory, privacy-wording, and structure findings rather than carried regressions.

## What would make this perfect

Show the completed redacted envelope in the initial mobile demo viewport; correct and test the license-token disclosure; register or remove every unlisted claim; replace the flagged jargon and ambiguous controls; use an action title; implement route-change focus and announcements; complete 404 metadata/chrome; and support separate routes for different destination policies. Then rerun the full claims list, `npm test`, live request/offline logs, mobile first-screen capture, link crawl, and light/dark axe checks. A follow-up review should have zero findings, not merely zero blockers.
