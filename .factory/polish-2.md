# Polish 2 — cumulative finding closure

Repair candidate based on `5f8c113597af535a582e46263c02ee9cf2c832ef`. Browser evidence is from `npm test`; the 390 × 844 claim screenshot is retained by Playwright as `test-results/**/mobile-demo-complete-result.png` on a claim run.

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 / F-2-1 | Compact mobile result, move redaction before details, require complete bounding boxes. | `@claim:mobile-demo-result`; `/demo`. |
| F-1-2 | Authorization-header verification and disclosed transport retained. | `@claim:license-transport`; `/privacy`. |
| F-1-3 / F-2-2 | Free controls now have an unlicensed edit, signed preview, and clipboard-flow test. | `@claim:free-core`. |
| F-1-4 | Source/query fingerprint claim remains deterministic. | `claim_query_fingerprint_uses_source_and_query`. |
| F-1-5 / F-2-3 | Split protected-file and browser-exposure claims; browser test checks response, DOM, and storage markers. | `@claim:credential-browser-exposure`; `claim_generated_credentials_are_protected`. |
| F-1-6 | Protected preview history remains unchanged. | `claim_preview_does_not_record_history`. |
| F-1-7 / F-2-4 | Removed untestable merchant/refund wording; checkout link remains tested. | `@claim:field-kit-purchase`; `/terms`. |
| F-1-8 | Removed the undocumented public request-size promise. | README audit. |
| F-1-9 | Removed unsupported no-destination copy. | README audit. |
| F-1-10 | Removed unsupported secret-log copy. | README audit. |
| F-1-11 | Build-process prose stays in handoff, not product claims. | README audit. |
| F-1-12 / F-2-7 | Throttle stores attempts and checks 23:59 versus 24:00 boundaries. | `@claim:license-throttle`; `/privacy`. |
| F-1-13 | Removed unsupported runtime identity prose. | README audit. |
| F-1-14 | Dated asset provenance and MIT evidence retained. | `@claim:provenance-license`. |
| F-1-15 | Home metadata says “add evidence to alerts.” | metadata browser test; `/`. |
| F-1-16 | In-app navigation focuses and announces the route heading. | route-navigation browser test. |
| F-1-17 | Designed 404 keeps metadata and shared chrome. | metadata browser test; `/not-a-real-route`. |
| F-1-18 | Plain terms use alert, envelope, route, and delivery metadata. | `.factory/copy-audit.md`. |
| F-1-19 | Descriptive process headings retained. | `.factory/copy-audit.md`. |
| F-1-20 | Relay and preview actions name their result. | browser suite. |
| F-1-21 / F-2-5 | Demo now compares Internal Slack and Customer automation policies without protected-route access. | `@claim:demo-route-policies`; `/demo`. |
| F-2-6 | Production limiter constants are shared with a controlled 40-burst/20-per-second claim test; browser checks 429 and isolated forwarded clients. | `claim_rate_limit_contract_has_a_40_request_burst_and_20_per_second_refill`; browser rate test. |
| F-2-8 | Replaced unbounded “safe” and “verified” copy. | `.factory/copy-audit.md`; `/`. |
| F-2-9 | JSON, Slack, and email-webhook payload contracts are shown in the UI/README and captured locally. | `claim_json_slack_and_email_destination_contracts`. |
| F-2-10 | Rewrote headings, caps, and query-pointer helper text. | `.factory/copy-audit.md`; `/`. |
| F-2-11 | Static legal pages now use Home/Demo/current-route navigation plus the standard one-line footer and source link. | legal browser/a11y test; `/privacy`, `/terms`. |
| F-2-12 | Split the README setup instructions. | README audit. |

All listed claims were run from the committed clean clone before deployment. Live recheck evidence is appended to `.factory/handoff.md` after deploy.
