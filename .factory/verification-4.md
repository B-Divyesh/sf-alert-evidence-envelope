# Verification 4 — FAIL

Date: 2026-08-30  
Work order: `alert-evidence-envelope-verify-4`  
Candidate: `c58704a9cb320aa55206e55fdd70442b0fe859a7`  
Live URL: `https://alert-evidence-envelope.sociobot.in`  
Acceptance sources: `.factory/brief.json`, the supplied researched brief/work order, and the attached factory skills

## Decision

**FAIL — do not release or promote this candidate.** The deployed runtime and static files match the candidate, and the core transformer works locally and live. However, several independent release blockers remain:

| Severity | Finding | Fresh evidence |
| --- | --- | --- |
| P0 / release gate | The required claim manifest is missing. | The clean candidate has no `.factory/claims.json`; `rg` also found no `@claim:` tests. Per the work order, a missing manifest is an automatic FAIL. Claim-like landing/README statements therefore have no sandbox proof. |
| P0 / release gate | The cold first screen and demo contract fail. | The first screen explains the transformation but never names on-call engineers or webhook consumers. Its 23-word lede exceeds the 22-word cap. The primary action is **Map your route** and the secondary action is **Try a safe sample**, not **Try it with sample data**. Clicking it only changes the URL to `/#test`; it does not run the sample. `/demo` returns 404, and `/?demo=1` has no demo banner, Reset demo, Start for real, or separate namespace. |
| P0 / security | The public production relay has neither admin nor inbound protection. | Live unauthenticated `GET /api/v1/config` and `/history` return 200. A structurally valid unauthenticated config update with an invalid name reaches semantic validation and returns 400 rather than 401; authorization precedes that validation in the handler. An unauthenticated malformed relay reaches JSON parsing and returns 400 rather than 401. A visitor can therefore submit alerts and, with a valid body, change the shared route configuration. |
| P0 / live correctness | Production persistence is replica-local and inconsistent. | A live synthetic relay returned 202 and `status: created`. Forty immediate `/history` reads split: 26 returned zero rows and 14 returned the new row. The deployed replicas do not share SQLite state, so route configuration/history are not reliable across requests. The same per-process architecture also explains the multiplied live rate-limit allowance. |
| P1 | Dark legal pages have serious accessibility failures. | Axe on 390 px dark-mode `/privacy` and `/terms` reports `color-contrast` (serious). The wordmark `<em>Envelope</em>` and `.eyebrow` use `#92420d` on `#101815`, ratio 2.58:1; 4.5:1 is required. Root light/dark and the expanded successful preview have zero serious/critical findings. |
| P1 | The paid Field Kit cannot be purchased. | The rendered **Buy the Field Kit** link points to `https://api.sociobot.in/api/v1/products/alert-evidence-envelope/checkout`, which returns 404 with `{"error":"enabled factory product","status":404}`. The full link crawl found this as the only dead rendered link. |
| P2 | Required site/discovery artifacts are absent. | `robots.txt`, `sitemap.xml`, and the apple-touch icon return 404. There is no canonical URL, Open Graph/Twitter metadata, 1200×630 social image, designed 404, or footer build identity / “Built by Param Factory” credit. Unknown routes return a 404 status with the full landing app and a console resource error. |
| P2 | Several touch targets are below 44 px. | At 390 px, footer Privacy/Terms/Source links measure 25 px high and the header wordmark measures 42 px high. Desktop nav links measure 22 px high. |
| P2 | The response CSP lacks an anti-framing policy. | CSP, HSTS, `nosniff`, and `no-referrer` are present, but neither `frame-ancestors` nor `X-Frame-Options` is sent. This is especially relevant for a page containing route-changing controls. |
| P2 | Rate limits are not documented and production multiplies the process allowance. | README contains no request allowance. Locally, one forwarded IP sent 800 concurrent config reads: 491 were 200 and 309 were 429 with `Retry-After: 0`; another IP remained 200. Live, the first 800-request burst all passed; a subsequent 3,000-request burst from one forwarded IP returned 2,580 × 200 and 420 × 429 with `Retry-After: 0`. Sociobot license verification allowed 30 of 500 and returned 470 × 429 with `Retry-After: 4`. |
| P2 / docs | Required sandbox/copy evidence is absent. | `.factory/demo.md` and `.factory/copy-audit.md` are missing. Decorative labels such as “INCIDENT CARTOGRAPHY · RELAY 01” and “THE SAFE PASSAGE” also conflict with the supplied plain-words rules. |

## Mandatory claims and first-read gates

### Claims

The very first clean-checkout check found:

```text
c58704a9cb320aa55206e55fdd70442b0fe859a7
__CLAIMS_MISSING__
```

There were no listed claim commands to execute because the required file does not exist. This is not “zero claims”: the page and README say, among other things, that raw payloads are not retained, caps are enforced, every envelope is signed, signatures are preserved, the sample is not stored, no analytics run, and the paid license is one-time. None are registered as required by the claims contract.

### Cold first read

My cold reading of the first screen was:

- What it does: turns a webhook alert into a bounded, redacted, signed evidence excerpt.
- For whom: the screen only says “people and automation”; it does not plainly identify on-call engineers or webhook consumers.
- What to click first: the visual primary action is **Map your route**. **Try a safe sample** is secondary.

The sample action is not a one-click demo. Browser evidence after one click:

```json
{
  "url": "https://alert-evidence-envelope.sociobot.in/#test",
  "previewVisible": false,
  "demoBanner": 0,
  "localStorageKeys": []
}
```

The visitor must click **Build safe preview** a second time. The page also reads the production relay's shared config and history on load, so it is not an isolated demo even though preview requests themselves are non-retaining.

## Clean-checkout gates

The checkout began clean on branch `main` at the exact candidate.

| Gate | Result |
| --- | --- |
| `npm ci` | PASS; 56 packages, 0 vulnerabilities. |
| First `npm test` | One transient failure: Chromium 145 crashed with SIGSEGV before creating the 390 px offline test context; 17/18 passed. The isolated failed case then passed. |
| Final exact `npm test` | PASS: Svelte check 0 errors/warnings; Rust 7/7; Vite build; Playwright 18/18 across desktop and 390 px. |
| `cargo fmt --check` | PASS. |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS. |
| `git diff --check` | PASS before report edits. |
| `npm run build` | PASS; creates `dist/`. |
| `BUILD_SHA=c58704a9cb320aa55206e55fdd70442b0fe859a7 cargo build --release --locked` | PASS. `/health` from this binary returns the candidate. |
| Container build | Not run: Docker, Podman, and Buildah are not installed in the verifier container. Dockerfile was inspected; exact frontend and release backend builds were exercised directly. |

There is no separate lint script. `svelte-check`, Rustfmt, and warning-denying Clippy cover the checks available in the repository.

## Core backend and job-to-be-done

### Clean local release process

An environment cleared with `env -i` except for `PATH` and `PORT` started successfully. It generated `data/envelope-signing.key` as 32 bytes with mode 600, logged `signing_key_source="generated"`, and served:

```json
{"build":"c58704a9cb320aa55206e55fdd70442b0fe859a7","status":"ok"}
```

After restart in the same directory it logged `signing_key_source="persisted"`.

### Independent end-to-end relay

Against the release binary with a fresh SQLite database, independent upstream and destination capture servers, admin/inbound tokens, and a known signing key:

- A fixed upstream received only `?q=service%3Dorders-api+level%3Derror&limit=2` and `Authorization: Bearer qa-short-lived-upstream-token`.
- The relay returned 202 with service `orders-api`, error `database timeout`, first-seen time, two items, and `truncated: true` from three fetched rows.
- Nested `email` and `token` values were `[REDACTED]` before delivery.
- Independent HMAC-SHA256 verification passed.
- The destination received `x-original-provider-signature` and `x-evidence-envelope-signature`; capture found no seeded raw email/token.
- Stopping the upstream produced 502; restarting it recovered to 202.
- Invalid boundaries and recovery: missing admin/inbound auth 401; malformed JSON 400; a 270 KB body 413; a 1,023-byte cap 400; unsafe remote HTTP URL 400.
- After 22 accepted relays, history returned exactly the latest 20 metadata rows. `strings` over SQLite found no seeded raw email, token, evidence message, or private email markers.
- Config and 20-row history persisted across a process restart.
- A 200-request concurrency smoke at width 20, with distinct forwarded client IPs, returned 200/200.

### Success measure

Twenty seeded preview alerts were exercised. Sixteen used the configured `service` / `error` / `startsAt` fields; four used recursive fallback names (`service_name`, `message`, `timestamp`). All 20 envelopes exposed the expected service, error signature, and first-seen time, exceeding the brief's 16/20 threshold.

### Live relay

The live endpoint accepted one sanitized synthetic alert and returned 202 with the expected three summary fields, recursive redaction, a correctly shaped `hmac-sha256=<64 hex>` signature, and `source_signature_preserved: true`. It had no destination configured and correctly returned the envelope to the caller. The immediately inconsistent history reads described above make this live success non-durable.

## Browser, accessibility, privacy, and PWA

### Passing evidence

- `/opt/fleet/lib/verify-url.sh` passed after providing its required output directory: title, `lang=en`, one h1, main present, zero missing alt text, zero unlabeled buttons, and zero console errors.
- Desktop 1440 px and mobile 390×844 px, light/dark, initial and expanded successful preview: zero serious/critical axe findings on the root app.
- Normal-load and sample-flow request logs contain only `https://alert-evidence-envelope.sociobot.in`; no analytics, CDN fonts, scripts, or undeclared third parties.
- With an invalid license explicitly supplied, the only external request is the disclosed Sociobot verification URL. The token is saved under `sb_license:alert-evidence-envelope`, stripped from the browser URL, and the UI reports “License no longer active.”
- Normal desktop/mobile flows have zero console errors and zero page errors.
- 390 px document width stays exactly 390 px, including expanded signed JSON.
- Keyboard-only smoke reaches **Build safe preview** after 22 Tabs and activates it with Space. Enter opens signed JSON; Tab focuses the labelled `<pre>` with a 3 px outline; PageDown changes scrollTop to 193 on a 260/720 px viewport/content region. First Tab exposes the skip link.
- Invalid `{` input reports “Sample alert is not valid JSON. Check commas and quotes.” Restore + rebuild succeeds.
- Reduced motion computes `scroll-behavior: auto` and removes the terrain transform.
- Service worker update succeeds, controller is active, cache `envelope-shell-v2` exists, and offline reload renders the root and `/privacy` at 390 px. The only offline console entry is Chromium's expected `ERR_INTERNET_DISCONNECTED` for attempted API reads; no page error occurs.
- Headers: HTTPS redirect, HSTS, CSP, `nosniff`, and `no-referrer` are present. HTML/service worker are `no-cache`; API/health are `no-store`; built assets are immutable.

### Accessibility defect outside repository coverage

The repository axe test covers the app root but not the separate static legal documents in dark mode. Independent dark-mode audits found two serious contrast nodes on each legal page at 2.58:1. This is a release blocker under the supplied accessibility contract.

## Deployment identity and static budgets

Live `/health` returns the exact candidate SHA. Local and live SHA-256 values match for root HTML, both legal documents, service worker, JS, CSS, both fonts, both hero images, and favicon. Key built assets:

| Asset | Raw | Gzip where applicable | SHA-256 |
| --- | ---: | ---: | --- |
| JS `index-E6yuWPXj.js` | 63,217 B | 24,348 B | `00322b15265db9836e350bebcaef58209051d464fd8b9a050d5c8273337bbb55` |
| CSS `index-5uUB6osl.css` | 16,524 B | 4,739 B | `408db1dc9723fe3d3a9a4dd14c75d114c7913049e93a256eecc5362eb2a6f1a3` |
| Fonts total | 115,560 B | — | byte-identical live |
| Mobile hero | 40,982 B | — | `e2ca79115164a994b2448655d4075f0e68a0ec6af604f10d19ed55e6907cad15` |

All stated static budgets pass.

Fresh live mobile Lighthouse 13.4.1:

- Performance 96; accessibility 100; best practices 100; SEO 100.
- FCP 1,696 ms; LCP 2,343 ms; TBT 117 ms; CLS 0; speed index 1,696 ms.

Lighthouse does not exercise the dark legal pages or demo/security/persistence contracts, so its accessibility score does not contradict the axe defect.

## Routing and links

- `/`, `/privacy`, and `/terms` return 200; HTTP redirects to HTTPS.
- `/demo`, `robots.txt`, `sitemap.xml`, apple-touch icon, and arbitrary routes return 404.
- The unknown-route 404 renders the normal landing page rather than a designed not-found page.
- All rendered internal links and the GitHub source link resolve. The Sociobot checkout link returns 404.

## Applicability notes

- This is not a library or CLI, so pack/install consumer testing is not applicable.
- Sign-in is not required, so the Entra authority check is not applicable. The missing admin/inbound protection is nevertheless a critical production defect.
- The brief explicitly makes LLM summarization a non-goal. No missing-AI finding is raised.

## Required remediation before re-verification

1. Add `.factory/claims.json` and exactly one observable demo-entry test for every claim-like statement.
2. Implement a real `/demo` (or documented `?demo=1`) with one-click sample execution, persistent demo banner, reset/exit controls, and isolated ephemeral backend state. Add `.factory/demo.md`.
3. Rewrite the first screen to name on-call engineers/webhook consumers and use the required sample action in one click; add and pass `.factory/copy-audit.md`.
4. Protect live config/history/preview administration and inbound relay traffic. A public product page must not expose a shared mutable production tenant.
5. Use a single replica with durable `/data`, or a shared database and shared signing key, so config/history/signatures are consistent across live requests.
6. Fix dark-mode legal-page contrast and add both static legal routes to light/dark axe coverage.
7. Register/enable the Sociobot paid product so checkout resolves, then test a complete test-mode purchase/restore path.
8. Add the missing discovery/social/404/footer artifacts, 44 px targets, and response-header `frame-ancestors` policy.
9. Document the actual per-client request allowance and make its production behavior consistent across replicas; retain 429 plus a useful `Retry-After` value.
