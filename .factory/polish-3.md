# Polish 3 — cumulative finding closure

Date: 2 September 2026

Work order: `alert-evidence-envelope-polish-3`

Repair commit: `d7bc262be19bb0bcf89b71d83120f21308b20778`

Deployed source commit: `a841f3b8a47cc9f252de0e7384376ca221418e97`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Evidence index

- **E1 — full local suite:** `npm test` passed 25 Rust tests and 58 Playwright cases across desktop and 390 × 844 Chromium.
- **E2 — independent clean clone:** `/tmp/aee-polish3-clean-d7bc262.log` records `npm ci`, `npm test`, and all 28 exact `.factory/claims.json` commands. It ends with `ALL_CLEAN_CLONE_CHECKS_PASSED`.
- **E3 — mobile screenshots:** `/tmp/aee-polish3-live/home-mobile-cold.png` and `/tmp/aee-polish3-live/demo-mobile-cold.png`; the claim-run screenshot is `test-results/app--claim-mobile-demo-res-e69c6-med-envelope-above-the-fold-mobile-chromium/mobile-demo-complete-result.png`.
- **E4 — cold live browser audit:** `/?demo=1`, `/demo`, `/privacy`, `/terms`, and the designed 404 had the expected status, title, one `h1`, one `main`, no unexpected console errors, and correct focus announcements. Demo traffic stayed on the product origin.
- **E5 — accessibility/offline:** live Playwright AxeBuilder scans found zero serious or critical findings on five routes in both light and dark modes. A dedicated fresh context reloaded `/?demo=1` offline with no failed, API, or health request.
- **E6 — deployment:** revision `sf-alert-evidence-envelope--0000032`, image `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:a841f3b8a47c`, one healthy replica, single-revision mode, `/data` mount, and 20 fresh-connection previews.
- **E7 — performance/basic verifier:** `/tmp/aee-polish3-live/verify.json` reports a 757 ms cold load and no console errors. Lighthouse: performance 94, accessibility 100, best practices 100, SEO 100, LCP 1,949 ms, CLS 0, TBT 248 ms.

## Review 1 findings

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | Kept the completed signed and redacted result before the sample input on mobile. | `@claim:mobile-demo-result`; E3; live `/?demo=1`. |
| F-1-2 | License verification uses an authorization header; Privacy discloses storage and transport separately. | `@claim:license-transport`; live `/privacy`; E4. |
| F-1-3 | Free copy now matches the tested controls, including the narrowed Terms sentence. | `@claim:free-core`; live `/terms`; E4. |
| F-1-4 | Fingerprints remain deterministic over both source and query. | `claim_query_fingerprint_uses_source_and_query`; E2; live `/`. |
| F-1-5 | Generated secrets stay in protected files and out of route responses, markup, and browser storage. | `claim_generated_credentials_are_protected`, `@claim:credential-browser-exposure`; E2. |
| F-1-6 | Protected previews do not create delivery-history rows. | `claim_preview_does_not_record_history`; E2; live `/`. |
| F-1-7 | Untestable merchant/refund promises remain absent; invalid licenses remove only preset controls. | `@claim:license-revocation`, `@claim:field-kit-purchase`; live `/terms`. |
| F-1-8 | The unsupported 256 KB promise remains absent. | `test:claims-manifest`; README audit in E2. |
| F-1-9 | The unsupported no-destination promise remains absent. | `test:claims-manifest`; README audit in E2. |
| F-1-10 | The unsupported secret-free logging promise remains absent. | `test:claims-manifest`; README audit in E2. |
| F-1-11 | Build-process details remain verification evidence, not public product claims. | `test:claims-manifest`; E1 and E2. |
| F-1-12 | Every license-check attempt suppresses retries until the tested 24-hour boundary. | `@claim:license-throttle`; E2; live `/privacy`. |
| F-1-13 | Public deployment wording stays within the inspected non-root, one-replica, `/data` contract. | `@claim:durable-deployment`; E6. |
| F-1-14 | Dated generated-art provenance and MIT status remain test-backed. | `@claim:provenance-license`; live footer and repository source link. |
| F-1-15 | Home, Open Graph, and Twitter titles say the action. | metadata browser test; live `/`; E4. |
| F-1-16 | Added cross-document focus handoff and polite route titles for static pages and browser Back. | `moves focus and announces static legal routes and browser Back`; live `/` → `/privacy` → Back and `/` → `/terms`; E4. |
| F-1-17 | The 404 retains complete metadata and chrome; its recovery links now reach their named targets. | `serves discovery metadata, icons, and a designed 404`; live `/not-a-real-route`; E4. |
| F-1-18 | Earlier jargon replacements remain in the rendered product and audit. | `.factory/copy-audit.md`; live `/`. |
| F-1-19 | All four process headings continue to name their object. | heading audit; live `/`. |
| F-1-20 | Relay and preview controls continue to name their result. | full browser suite; live `/`. |
| F-1-21 | Protected routes remain independent; the sample compares two distinct redaction policies. | `claim_routes_keep_independent_policies`, `@claim:demo-route-policies`; live `/?demo=1`. |

## Review 2 findings

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-2-1 | The assertion checks each result box’s bottom edge, and the complete result remains above 844 px. | `@claim:mobile-demo-result`; E3; live `/?demo=1`. |
| F-2-2 | The unlicensed test edits settings, previews, signs, and copies; Terms now uses only that wording. | `@claim:free-core`; live `/terms`; E4. |
| F-2-3 | File protection and browser/API credential absence remain separate tested claims. | `claim_generated_credentials_are_protected`, `@claim:credential-browser-exposure`; E2. |
| F-2-4 | Merchant/refund claims remain absent. | `@claim:field-kit-purchase`, `@claim:license-revocation`; live `/` and `/terms`. |
| F-2-5 | `/?demo=1` visibly switches between Internal Slack and Customer automation without protected API access. | `@claim:demo-route-policies`; E3 and E4. |
| F-2-6 | The shared limiter still proves 40 burst and 20/second refill using the first forwarded IP. | `claim_rate_limit_contract_has_a_40_request_burst_and_20_per_second_refill`; live burst returned 43 × 401 and 57 × 429 with `Retry-After: 1`. |
| F-2-7 | The browser test checks 23:59 and 24:00 retry boundaries. | `@claim:license-throttle`; E2. |
| F-2-8 | Unbounded “safe” and false “verified” wording remain absent. | `.factory/copy-audit.md`; live `/`; E3. |
| F-2-9 | JSON, Slack, and email webhook contracts remain explicit and locally captured. | `claim_json_slack_and_email_destination_contracts`; E2; live `/`. |
| F-2-10 | Labels and headings remain descriptive and use one term per concept. | `.factory/copy-audit.md`; live `/`. |
| F-2-11 | Privacy and Terms retain standard navigation, footer, source, build ID, and current-page state. | legal light/dark accessibility test; live `/privacy` and `/terms`; E4/E5. |
| F-2-12 | README retains separate admin-token and provider-token setup sentences. | README copy audit in E2; GitHub source link returned 200. |

## Review 3 findings

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-3-1 / F-1-16 | Static pages now receive a cross-document navigation marker, focus their `h1`, announce the route title, and hand focus back on browser Back. Named listeners are removed correctly. | `moves focus and announces static legal routes and browser Back` passed desktop/mobile; live focus audit passed; E4/E5. |
| F-3-2 / F-1-3 | Replaced the overbroad Terms promise with “Redaction, signing, previews, copying, and route settings remain free.” | `@claim:free-core`; live `/terms`; E4. |
| F-3-3 | Rewrote the README contract as “Sends the signed envelope and signature header to a JSON webhook.” | `.factory/copy-audit.md`; clean-clone README in E2. |
| F-3-4 | Split the first-screen price line into two sentences. | `@claim:field-kit-purchase`; `/tmp/aee-polish3-live/home-mobile-cold.png`; live `/`. |
| F-3-5 | Both 404 builder links target `/#configure`, and the footer says “Source (external)”. | `serves discovery metadata, icons, and a designed 404`; live `/not-a-real-route`; E4. |

All earlier findings were rechecked. None remains open.
