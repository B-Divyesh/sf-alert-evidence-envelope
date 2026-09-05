# Add redacted evidence to webhook alerts — Verification 17

Date: 2026-09-05

## Verdict: PASS

**PASS.** There are zero findings at every severity and zero untested public claims. The deployed product completes the brief's job: it turns an alert into a bounded, recursively redacted, HMAC-signed evidence envelope for a configured delivery route, without retaining raw alert bodies.

- Implementation reviewed: `db73a281fe9cd1911851a239d2b31d9bbcbedff0`
- Documentation commit: `08231a06b7efab7bf12ef51a85f55367c132b468`
- Live URL: <https://alert-evidence-envelope.sociobot.in>
- Assigned live revision: `sf-alert-evidence-envelope--0000040`
- Health observed: HTTP 200, build `08231a06b7efab7bf12ef51a85f55367c132b468`

`db73a28..08231a0` changes only `.factory/handoff.md` and `.factory/copy-audit.md`. A fresh production frontend built from the implementation source with the observed documentation build identity matched the live HTML, JavaScript, and CSS byte for byte. The later documentation commit therefore does not represent a different product image.

## First screen and demo

Fresh Chromium contexts at 1440 × 900 and 390 × 844, before scrolling, both showed:

| Check | Observed text |
| --- | --- |
| Job | “Add redacted evidence to webhook alerts” |
| Audience | On-call engineers and webhook consumers who need incident context without another dashboard login. |
| First action | “Try it with sample data”; it says a signed, redacted envelope opens in an isolated workspace. |

In both contexts the action opened a populated sample result with service, error signature, first-seen time, redaction state, item and byte bounds, fingerprint, and signature. The persistent label was “Demo — sample data, nothing is saved”, with Reset demo and Start for real.

The edited-input regression was exercised live in each context:

- Edited `checkout-api` to `billing-api` and the rendered envelope changed; exactly one demo preview request was sent.
- Entered `{`; the recovery panel said “Preview stopped” and told the user to check commas and quotes. It sent zero preview requests.
- Restore valid sample followed by Build signed preview recovered to a populated envelope.
- Reset demo created a fresh demo request and retained only `demo:` storage keys. These are the demo namespace, not production route data.

Screenshots: `/work/.evidence/verification-17/desktop-home.png`, `desktop-demo.png`, `phone-home.png`, and `phone-demo.png`.

## Claims and clean checkout

A detached clean worktree at `db73a28` ran `npm ci` (56 packages; 0 reported vulnerabilities), then every exact command declared in `.factory/claims.json`. All 30 passed:

| Claim IDs with PASS exact command |
| --- |
| `demo-envelope`, `editable-demo`, `mobile-demo-result`, `demo-route-policies` |
| `bounded-redacted-signed`, `query-fingerprint`, `fixed-query-source`, `provider-signature`, `slack-delivery` |
| `isolated-demo`, `demo-expiry`, `raw-not-retained`, `history-limit`, `protected-real-apis`, `credential-storage`, `credential-browser-exposure`, `preview-no-history`, `per-route-isolation` |
| `no-tracking`, `offline-demo`, `license-transport`, `local-policy-presets`, `license-throttle`, `free-core`, `field-kit-purchase`, `license-revocation`, `provenance-license` |
| `durable-deployment`, `rate-limit`, `destination-contracts` |

`npm run test:claims-manifest` passed and confirmed exactly one tagged regression test for every declared claim. No command was skipped, missing, or failed.

The clean candidate also passed:

```sh
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 npm run build
BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 cargo build --release --locked
```

`npm test` reported zero Svelte diagnostics, 25 Rust tests, policy and claims manifest checks, and 64 Playwright tests. The final production bundle was 74.15 KB JavaScript (27.22 KB gzip) and 22.31 KB CSS (5.84 KB gzip).

## Live backend, safety, and recovery

- `/health` returned 200. A temporary local `DATA_DIR` process created a demo session, was stopped and restarted, and successfully previewed that same session after restart. No credential value was read or reported.
- Live isolated-demo check: two sessions each previewed successfully; deleting one returned 404 for that session while the other continued to return 200. The returned sample token was `[REDACTED]`.
- Live boundaries: 1/100 records and 1,024/131,072 bytes returned 200; 0/101 records and 1,023/131,073 bytes returned 400. Malformed JSON returned 400, followed by a successful valid recovery request. The temporary demo session was deleted (204).
- A fresh 100-request burst to a protected endpoint from one forwarded IP returned 96 × 401 and 4 × 429. Every 429 carried `Retry-After: 1`; a second forwarded IP immediately received the normal 401 response.
- The normal live demo uses only the product origin. Its browser flow had no console or page errors. The passing privacy/offline browser claims cover no tracking, self-hosted fonts, offline reload, license transport, and the demo reset/exit race.

## Accessibility, routes, and links

- Fresh phone reduced-motion inspection found no running animations after the sample settled, no horizontal overflow (390 px scroll/client width), and first Tab focused “Skip to main content”.
- The clean browser suite includes the Axe audits, keyboard, focus handoff, 200% text, target-size, offline, and legal-page checks; all passed.
- Fresh live checks found no console errors and verified one h1 and one main landmark on every route. Titles were: home “Alert Evidence Envelope — add evidence to alerts”; Demo, Privacy, Terms, and the designed 404 each had the corresponding route title.
- `/`, `/?demo=1`, `/privacy`, and `/terms` returned 200. The intentionally missing route returned its designed HTTP 404 with a recovery path; this is expected, not a defect.
- All extracted product links resolved: internal routes returned 200, the official checkout returned its expected 303, and the product source link returned 200.
- Live response headers include HSTS, `nosniff`, `DENY`, `no-referrer`, and a response-header CSP with `frame-ancestors 'none'`.

## Earlier findings

All earlier verification, review, and polish records were inspected. Their closures remain effective: the build identity and strict-Clippy gates, claim-manifest and copy evidence, plain first screen and one-click isolated sample, mobile result, legal dark contrast and focus handoff, durable single-replica storage, protection and rate limits, recursive redaction and delivery contracts, offline/update behavior, route isolation, metadata-only retention, licence transport/revocation, generated-art provenance, titles, links, and designed 404 all remain covered by the passing current checks.

Strict review 6 finding **F-6-1** is specifically closed by the fresh live edited-JSON, invalid-JSON/no-request, restore, and successful-rebuild checks above, and by the passing `editable-demo` claim. No earlier minor finding reopened.

## Finding summary

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| Untested claims | 0 |
