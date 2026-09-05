# Add redacted evidence to webhook alerts — Strict review 7

Date: 2026-09-05

## Verdict: PASS

**PASS.** There are zero findings at every severity and zero untested public claims.

- Implementation reviewed: `db73a281fe9cd1911851a239d2b31d9bbcbedff0`
- Live documentation build: `08231a06b7efab7bf12ef51a85f55367c132b468`
- Live URL: <https://alert-evidence-envelope.sociobot.in>

`db73a28..08231a0` changes only `.factory/copy-audit.md` and
`.factory/handoff.md`. A fresh production frontend built from `db73a28` with
the live documentation build identity matched the live HTML, JavaScript, CSS,
and hero asset byte for byte. The deployed product therefore matches the
implementation candidate.

## First screen and sample

Fresh desktop (1440 × 900) and phone (390 × 844) browser contexts, before
scrolling, showed:

| Check | Observed text |
| --- | --- |
| Job | “Add redacted evidence to webhook alerts” |
| Audience | “For on-call engineers and webhook consumers who need incident context without another dashboard login.” |
| First action | “Try it with sample data” |

The action opened a populated sample with service, error signature, first-seen
time, recursive redaction, item and byte bounds, query fingerprint, and HMAC
signature. Both contexts showed the persistent “Demo — sample data, nothing is
saved” label, Reset demo, and Start for real.

In both fresh contexts, changing `checkout-api` to `review-seven-api` changed
the result. Invalid JSON showed “Preview stopped”, made zero preview requests,
and recovered after Restore valid sample. Reset produced a usable fresh sample;
Start for real returned to the normal workspace. No console or page errors were
observed. Screenshots are in `/work/.evidence/review-7/`.

## Claims and clean checkout

A fresh clone was checked out detached at `db73a28` and installed with `npm ci`:
56 packages installed and npm reported zero vulnerabilities. Every exact command
in `.factory/claims.json` passed. The manifest contains 30 claims and the
manifest test confirmed exactly one tagged regression test for each.

The passing claims cover demo construction and editing, mobile output, route
policies, redaction and HMAC signing, fingerprinting, fixed-source fetching,
provider signatures, Slack/email/JSON delivery contracts, demo isolation and
expiry, raw-data retention, history limits, credentials, route isolation,
offline behavior, privacy, licensing, local presets, provenance, deployment,
and rate limiting. No command was missing, skipped, failed, incomplete, or
untested.

The clean checkout also passed:

```sh
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 npm run build
BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 cargo build --release --locked
```

`npm test` exited 0 with zero Svelte diagnostics, 25 Rust tests, the deployment
and claim-manifest gates, and 64 browser tests. The production build produced
74.15 KB JavaScript (27.22 KB gzip) and 22.31 KB CSS (5.84 KB gzip).

## Backend, privacy, and recovery

- `/health` returned HTTP 200 with build `08231a0`.
- Two live demo tenants both previewed successfully. Deleting one returned 404
  only for that tenant; the other continued to return 200.
- A valid one-item request returned 200 with recursively redacted fields.
  Zero and 101 item limits, plus malformed JSON, returned 400; a following
  valid request returned 200.
- A 100-request protected-endpoint burst from one forwarded IP returned 43 ×
  401 and 57 × 429. Every 429 had `Retry-After: 1`; another forwarded IP
  immediately received the normal 401 response.
- A locally started release binary kept a demo session usable after graceful
  restart with the same temporary SQLite data directory. Both health checks
  returned `ok` and preview after restart returned 200.
- The normal demo browser flow used only the product origin. The passing claim
  suite covers raw-payload non-retention, browser credential absence, offline
  reload, reset/exit isolation, and licensing transport.

## Accessibility, routes, and performance

`verify-url.sh` passed against the live root: title, `lang=en`, one h1, main,
image alternatives, labelled buttons, and no console errors. Fresh mobile dark
mode Axe WCAG 2 A/AA scans found zero violations on home, demo, Privacy, Terms,
and the designed 404. The first Tab focused Skip to main content.

`/`, `/?demo=1`, `/privacy`, and `/terms` returned 200. The deliberate
missing route returned the designed HTTP 404 with a recovery path, which is
expected. Each route has its appropriate title and one h1. Headers include
HSTS, `nosniff`, `DENY`, `no-referrer`, and response-header CSP with
`frame-ancestors 'none'`.

A completed mobile Lighthouse run measured performance 96, accessibility 100,
best practices 100, SEO 100, LCP 2.2 s, and CLS 0.

## Earlier findings

All earlier review, verification, and polish records were inspected. Their
current dispositions are proved by the checks above and the current claim suite:

| Earlier finding group | Current disposition and evidence |
| --- | --- |
| Verification 1–8 | Closed: build identity, strict Clippy, claims, plain first screen, isolated demo, legal contrast, protected APIs, durable storage, offline reload, copy audit, target size, and cache behavior pass. |
| Verification 9–10 | Closed: Slack carries the signed envelope under `slack-delivery`; the fresh mobile Lighthouse LCP is 2.2 s. |
| Verification 13 | Closed: the browser regression for blank optional URLs is included in the passing browser suite. |
| Reviews F-1-1–F-1-21 and F-2-1–F-2-12 | Closed: mobile complete result, privacy/credential boundaries, narrow free-core wording, claim coverage, route policies, limiter boundary, legal chrome, headings, README, and 404 recovery all remain covered. |
| Reviews F-3-1–F-3-5 and F-4-1 | Closed: cross-document focus, Terms wording, README/price wording, 404 targets, and complete mobile result are covered by passing browser and copy checks. |
| Review F-5-1 | Closed by the passing isolated-demo reset/exit race claim; no protected request or demo storage survives exit. |
| Review F-6-1 | Closed by the passing `editable-demo` claim and this fresh live edit, invalid-input/no-request, restore, and rebuild exercise. |
| Verification 14–17 | No open finding; their current source, live, claim, accessibility, deployment-policy, and identity evidence was rerun here. |

## Finding summary

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| Untested claims | 0 |
