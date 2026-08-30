# Polish 1 — review finding closure

Candidate repaired from `5c10f93da6f4e95c64ed9f9cc70b06b81f08df83`.

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | Mobile demo places the completed envelope before editable JSON and prints redaction state. | `@claim:mobile-demo-result`, `tests/browser/app.spec.ts`; live `/demo` after deploy. |
| F-1-2 | License verification uses `Authorization: Bearer`, never a query token; privacy notice states this. | `@claim:license-transport`; live `/privacy`. |
| F-1-3 | Free controls are explicitly exercised without a license. | `@claim:free-core`; live `/demo`. |
| F-1-4 | Fingerprint has source/query change coverage. | `claim_query_fingerprint_uses_source_and_query`; live `/`. |
| F-1-5 | Protected generated credential files are asserted. | `claim_generated_credentials_are_protected`. |
| F-1-6 | Authenticated preview history remains empty. | `claim_preview_does_not_record_history`. |
| F-1-7 | Invalid licenses hide only Field Kit controls; untestable merchant/refund wording was removed. | `@claim:license-revocation`; live `/terms`. |
| F-1-8 | The unverified numeric request-limit sentence was removed from public copy. | README copy audit. |
| F-1-9 | The unverified no-destination public sentence was removed from public copy. | README copy audit. |
| F-1-10 | The unverified secret-log public sentence was removed from public copy. | README copy audit. |
| F-1-11 | Build-process assertions were moved out of public product copy. | README copy audit. |
| F-1-12 | A timestamp is saved before verification so failures are throttled for 24 hours. | `@claim:license-throttle`; live `/privacy`. |
| F-1-13 | The unverified process-identity claim was removed from public copy. | README copy audit. |
| F-1-14 | Footer now gives dated asset provenance; MIT header and asset metadata are tested. | `@claim:provenance-license`; live `/`. |
| F-1-15 | Home, OG, and Twitter titles say “add evidence to alerts.” | metadata browser test; live `/`. |
| F-1-16 | History navigation focuses the new h1 and announces it. | `moves focus and announces the new route`; live `/` → `/demo`. |
| F-1-17 | 404 has canonical, OG URL, theme color, and complete chrome. | `serves discovery metadata, icons, and a designed 404`; live `/not-a-real-route`. |
| F-1-18 | Jargon and inconsistent terms were rewritten. | `.factory/copy-audit.md`. |
| F-1-19 | One-word stage headings were replaced with descriptive headings. | `.factory/copy-audit.md`; live `/`. |
| F-1-20 | Buttons now say “Copy relay URL” and “Build signed preview.” | browser suite; live `/`. |
| F-1-21 | Added protected route list/create/update/delete API and UI; each route has its own relay URL and policy. | `claim_routes_keep_independent_policies`; live `/` after loading admin routes. |

Local evidence: `npm test` passed (22 Rust tests and 52 Playwright tests); `cargo clippy --all-targets --locked -- -D warnings`, `cargo fmt --check`, and `npm run build` passed. Browser trace and failure-free run log are retained at `/tmp/aee-npm-test.log` in this worker.
