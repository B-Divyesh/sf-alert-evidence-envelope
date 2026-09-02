# Verification 12 — PASS

Date: 2026-09-02  
Work order: `alert-evidence-envelope-verify-12`  
Candidate: `36fc44438dd299a142ce5fe30fd1a8676e539877`  
Live URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** The candidate is deployed and meets the researched job: it turns an
alert into a bounded, recursively redacted, HMAC-signed evidence envelope
without granting dashboard access. No P0–P3 defects were found.

## Cold first read

- **What it does:** “Add redacted evidence to webhook alerts.”
- **Who it is for:** “For on-call engineers and webhook consumers who need incident context without another dashboard login.”
- **First action:** the visible one-click **Try it with sample data** link says it opens a signed, redacted envelope in an isolated workspace.

The link opened `/demo` without setup. It displayed the persistent “Demo —
sample data, nothing is saved” banner and a completed envelope with
`checkout-api`, `payment authorization timed out`, a first-seen time,
`[REDACTED]` evidence, and an `hmac-sha256=` signature.

## Required claims and local checks

`npm ci` completed from this clean candidate (56 packages, 0 vulnerabilities).
Before other QA, the first listed claim command ran:
`npm run test:claims -- --grep @claim:demo-envelope`. The full `npm test` run
then executed every tagged browser claim and Rust claim from
`.factory/claims.json`; Playwright reported
`{"status":"passed","failedTests":[]}`. `npm run test:claims-manifest`
confirmed all 28 claims have exactly one tagged regression test.

| Check | Result |
| --- | --- |
| `npm test` | PASS — all claim, browser, Rust, policy, and type checks. |
| `npm run check` | PASS — 0 errors and 0 warnings. |
| `cargo test --locked` | PASS — 25 tests. |
| `cargo fmt --check` / `cargo clippy --all-targets --locked -- -D warnings` | PASS. |
| `npm run build` | PASS — produced `dist/`. |
| `npm run test:deployment-policy` | PASS — non-root, single-replica, durable `/data` policy. |

Production build: JS 71,264 bytes raw / 26.45 KB gzip; CSS 20,049 bytes raw
/ 5.49 KB gzip; mobile hero 40,982 bytes.

## Live evidence

- `/health` returned build `36fc44438dd299a142ce5fe30fd1a8676e539877`.
  Candidate-built JS, CSS, and `sw.js` SHA-256 values matched their live files.
- Live topology passed: revision `sf-alert-evidence-envelope--0000031`, image
  `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:36fc44438dd2`, one
  ready/running replica, `/data` mounted, and 20 fresh demo-session previews
  were redacted correctly.
- Internal Slack retained the sample email and redacted `token`; Customer
  automation redacted `email` and `token`. The demo made no protected-route
  API call. Malformed protected `/api/v1/preview` returned 401 before parsing.
- The PWA registered `envelope-shell-v5`; a fresh 390×844 browser context
  reloaded the completed sample offline with no failed request.

## Privacy, security, accessibility, and performance

- Cold landing and demo request logs used only the product origin; no analytics,
  advertising, hosted font, third-party script, console, or page error.
- `/`, `/demo`, `/privacy`, and `/terms` returned 200; designed 404 returned
  404. Internal landing links all returned 200.
- Responses carry HSTS, `nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, and a response-header CSP with
  `frame-ancestors 'none'`; API/health are `no-store` and hashed assets are
  one-year immutable.
- Runtime rate check: 100 concurrent protected requests from one forwarded IP
  in 491 ms returned 45 normal 401s and 55 **429** responses, each with
  `Retry-After: 1`; a different forwarded IP immediately received the normal
  401. Observed contract: 40-request burst and 20 requests/second refill.
- At 390×844, `scrollWidth = 390`; reduced-motion demo worked, first Tab
  focused the visible skip link, in-app demo navigation focused/announced its
  `<h1>`, and expanded JSON had a solid focus outline and keyboard scrolling.
- Live Playwright Axe scans in light and dark found 0 serious/critical WCAG
  2 A/AA/2.1 AA issues. The local suite also passed legal-page, 44 px target,
  200% text, and reduced-motion coverage.
- Lighthouse mobile: 89 and 90 performance (one-point TBT variation), 100
  accessibility, 100 best practices, 100 SEO; both runs had 1.4 s FCP,
  2.1 s LCP, and CLS 0. The second run meets the >=90 target.

## Handoff

No release-blocking defects. This verification changes documentation only;
product code and infrastructure were not modified. Re-run with `npm ci && npm
test`, then use `/demo` for the no-setup acceptance path.
