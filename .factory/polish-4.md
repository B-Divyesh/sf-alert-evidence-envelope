# Polish 4 — cumulative finding closure

Date: 2 September 2026

Work order: `alert-evidence-envelope-polish-4`

Released candidate repaired: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

Repair commit: `14f8cf03eddd457ed1a837e9cbfaffed238b5147`

Live revision: `sf-alert-evidence-envelope--0000036`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Round 4 result

The complete mobile sample card now spans y=286.69–601.47 in a cold 390 ×
844 viewport at scroll y=0. First seen ends at y=477.89; item, byte, and
truncation fields end at y=518.69; the fingerprint ends at y=535.88. The card
also includes the signed state, redaction state, service, error, and both
result actions.

`@claim:mobile-demo-result` now runs only in the configured mobile project. It
asserts the viewport, zero scroll, all four edges of the whole result card,
the exact first-seen/item/byte/truncation content, the fingerprint shape, and
every required field's bottom edge. It saves a viewport screenshot.

Evidence:

- Clean-clone screenshot: `/tmp/aee-polish4-clean-NrxuZj/test-results/app--claim-mobile-demo-res-e69c6-med-envelope-above-the-fold-mobile-chromium/mobile-demo-complete-result.png`
- Live screenshot: `/tmp/aee-polish4-live/demo-mobile-cold.png`
- Live geometry and route audit: `/tmp/aee-polish4-live/live-browser-audit.json`
- Clean-clone log: `/tmp/aee-polish4-clean-14f8cf0.log`

## Every review finding

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 / F-2-1 / F-4-1 | Put the completed demo result before the editor and compacted mobile banner, heading, readout rows, bounds, fingerprint, and actions. | `@claim:mobile-demo-result`; live card y=286.69–601.47; screenshots above. |
| F-1-2 | License verification sends the token only in an authorization header; Privacy discloses storage and transport. | `@claim:license-transport`; live `/privacy`. |
| F-1-3 / F-2-2 / F-3-2 | Narrowed free-core wording to the controls exercised without a license. | `@claim:free-core`; live `/terms`. |
| F-1-4 | Fingerprints are deterministic over both source and query and change when either changes. | `claim_query_fingerprint_uses_source_and_query`. |
| F-1-5 / F-2-3 | Generated credentials use protected files and never appear in route JSON, markup, or browser storage. | `claim_generated_credentials_are_protected`; `@claim:credential-browser-exposure`. |
| F-1-6 | Protected previews leave delivery history unchanged. | `claim_preview_does_not_record_history`. |
| F-1-7 / F-2-4 | Removed untestable merchant/refund promises; revoked licenses remove only local preset controls. | `@claim:field-kit-purchase`; `@claim:license-revocation`; live `/terms`. |
| F-1-8 | Removed the unsupported public 256 KB boundary promise. | README audit; `test:claims-manifest`. |
| F-1-9 | Removed unsupported no-destination behavior from public copy. | README audit; `test:claims-manifest`. |
| F-1-10 | Removed the unsupported secret-free logging promise from public copy. | README audit; `test:claims-manifest`. |
| F-1-11 | Kept build-process details as handoff evidence instead of public behavior claims. | README audit; clean-clone `npm test`. |
| F-1-12 / F-2-7 | Timestamp every license attempt and prove both sides of the 24-hour retry boundary. | `@claim:license-throttle`; live `/privacy`. |
| F-1-13 | Public deployment wording matches the inspected non-root, one-replica, durable `/data` policy. | `@claim:durable-deployment`; revision `sf-alert-evidence-envelope--0000036`. |
| F-1-14 | Use dated generated-art provenance and test the MIT/font notices. | `@claim:provenance-license`; live footer. |
| F-1-15 | Home, Open Graph, and Twitter titles say “add evidence to alerts”. | Metadata browser test; live `/`. |
| F-1-16 / F-3-1 | In-app, static legal, Back, and forward navigation focus and announce the route h1. | Navigation browser tests; live focus audit in `live-browser-audit.json`. |
| F-1-17 | The real 404 has complete metadata, standard chrome, and working recovery links. | Metadata/404 browser test; live `/not-a-real-route` returned 404. |
| F-1-18 | Replaced technical shorthand with alert, envelope, route, and delivery metadata. | `.factory/copy-audit.md`; live `/`. |
| F-1-19 | Process headings name their objects. | `.factory/copy-audit.md`; live `/`. |
| F-1-20 | Actions say “Copy relay URL” and “Build signed preview”. | Browser suite; live `/`. |
| F-1-21 / F-2-5 | Added independent protected routes and two isolated sample policies with different redaction. | `claim_routes_keep_independent_policies`; `@claim:demo-route-policies`. |
| F-2-6 | Share the production 40-burst/20-per-second limiter constants with a boundary and first-forwarded-IP test. | `claim_rate_limit_contract_has_a_40_request_burst_and_20_per_second_refill`. |
| F-2-8 | Replaced unbounded “safe” and false “verified” language with observable actions. | `.factory/copy-audit.md`; live `/`. |
| F-2-9 | Define and capture JSON, Slack, and email gateway request contracts. | `claim_json_slack_and_email_destination_contracts`. |
| F-2-10 | Rewrote contextual headings, caps, and query-pointer help in plain words. | `.factory/copy-audit.md`; live `/`. |
| F-2-11 | Privacy and Terms use the shared navigation, product line, legal links, source, and build ID. | Legal browser/Axe tests; live `/privacy` and `/terms`. |
| F-2-12 | Split admin-token and inbound-token setup into separate README sentences. | README audit. |
| F-3-3 | Corrected the JSON webhook sentence to say the envelope is sent “to” the webhook. | README audit. |
| F-3-4 | Split the first-screen core-price facts into two sentences. | `@claim:field-kit-purchase`; live `/`. |
| F-3-5 | Both 404 builder links target `/#configure`; external source is labelled. | 404 browser test; live link audit. |

## Verification

- Clean clone `/tmp/aee-polish4-clean-NrxuZj` at the repair commit: `npm ci`,
  `npm test`, all 28 exact claim commands, `cargo fmt --check`, strict Clippy,
  and `npm run build` passed. The full suite had 25 Rust tests and 59 browser
  passes with one intentional desktop skip for the mobile-only geometry claim.
- Build output: 26.73 KB gzip JavaScript and 5.78 KB gzip CSS.
- Fleet verifier: 767 ms load, one h1, one main, complete alt/labels, and no
  console errors.
- Live Axe: six routes in light and dark mode, zero serious or critical
  findings in all 12 runs.
- Live Lighthouse mobile: performance 92, accessibility 100, best practices
  100, SEO 100, LCP 2,027 ms, CLS 0.
- Live link crawl: every internal, legal, checkout, source, discovery, and
  fragment target resolved; the designed unknown route alone returned 404.
- Demo traffic stayed on the product origin and touched no protected API.
  Reset changed the ephemeral session, Start for real removed all `demo:`
  keys, and an offline reload made no API or health request.

No review finding remains open.
