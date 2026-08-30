# Alert Evidence Envelope — repair handoff

Date: 2026-08-30

Work order: `alert-evidence-envelope-repair-5`

Verifier report: `.factory/verification-4.md` at `9eaf31dff930b839b45546ed9ca37b2bb8d0ea55`

Rejected candidate: `c58704a9cb320aa55206e55fdd70442b0fe859a7`

Release status: repaired, tested, and deployed

## What changed

| Verifier finding | Root-cause repair | Regression evidence |
| --- | --- | --- |
| Missing claims contract | Added 12 claims with one source tag each in `.factory/claims.json`. | Every listed command passes; the tag audit reports one source for every ID. |
| No one-click isolated demo | Added `/demo`, automatic sample execution, a 24-hour in-memory session, demo-only browser keys, persistent banner, reset, exit, and offline cached result. | `@claim:demo-envelope`, `@claim:isolated-demo`, `@claim:offline-demo`, reset/exit browser test. |
| First screen failed the user/action/copy rules | The six-word job headline now names on-call engineers and webhook consumers. The primary action is **Try it with sample data**. | Desktop and 390px first-read test; `.factory/copy-audit.md` has no sentence over 22 words or banned term. |
| Public config/history/preview/relay | First boot generates separate 256-bit admin and inbound tokens. They persist with mode 600. Authorization runs before body parsing. | `@claim:protected-real-apis`; generated/persisted-token Rust tests; unauthenticated malformed requests return 401. |
| Replica-local production state | Deployment uses single-revision mode, min/max one replica, and an Azure File mount at `/data`. SQLite runs locally because SMB does not implement its locking contract; every committed config/history change is atomically snapshotted to the mount and restored on replacement. | `durable_snapshot_restores_committed_route_state`, deployment-policy test, and live topology check. |
| Legal-page dark contrast | Raised amber text to `#f0a45b` on `#101815`. | Light/dark axe audits on `/privacy` and `/terms` at 390px report zero serious/critical issues. |
| Dead Field Kit checkout | Registered the $39 one-time product in production and pilot Sociobot billing. | `@claim:field-kit-purchase` verifies production and pilot Dodo redirects. A pilot purchase returned a license; the mocked restore flow stores, verifies, and strips it from the URL. |
| Missing discovery/identity files | Added canonical/OG/Twitter metadata, 1200×630 social art, 180px touch icon, robots, sitemap, designed true 404, Param Factory credit, and build identity. | Browser discovery, 404, dimensions, and `/health`/footer identity tests. |
| Small touch targets | Navigation, wordmark, and footer links now have 44px minimum target height. | Desktop and 390px geometry test. |
| Missing anti-framing response policy | Added CSP `frame-ancestors 'none'` and `X-Frame-Options: DENY`. | Header test covers HTML, legal, health, and protected API responses. |
| Multiplied/unclear limits | API rate limiting is 40 burst then 20 requests/second, keyed by first `X-Forwarded-For`; health is exempt; 429 sends `Retry-After: 1`. README documents it. | `@claim:rate-limit` checks same-client rejection and another-client isolation in both browser projects. |
| Missing sandbox/copy docs and decorative labels | Added `.factory/demo.md` and `.factory/copy-audit.md`; replaced decorative labels with task language. | Copy audit and browser text assertions. |

The database now deletes records beyond the latest 20 instead of only limiting the history response. Demo sessions never touch SQLite. The service worker cache is `envelope-shell-v3` and includes `/demo`, legal pages, and the designed 404.

## Clean local verification

Run from `/work/repo`:

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release --locked
git diff --check
```

Observed results:

- `npm ci`: 56 packages, 0 vulnerabilities.
- `npm test`: Svelte 0 errors/warnings; Rust 14/14; deployment contract pass; Playwright 36/36 across desktop Chromium and 390×844 mobile Chromium.
- Rustfmt, warning-denying Clippy, release build, and whitespace check: pass.
- Vite output: JS 66,444 bytes raw / 25.25 KB gzip; CSS 17,557 bytes raw / 4.97 KB gzip; fonts 115,560 bytes total; mobile hero 40,982 bytes.
- Empty environment except `PATH` and `PORT`: starts successfully; signing/admin/inbound files are 32/64/64 bytes with mode 600. Restart logs all three sources as `persisted`.
- `/opt/fleet/lib/verify-url.sh`: one h1, `lang=en`, main present, no missing alt text, no unlabeled button, no console error.
- `@axe-core/cli` 4.10.3: zero violations on `/`, `/demo`, `/privacy`, and `/terms`.
- Mobile Lighthouse: performance 98, accessibility 100, best practices 100, SEO 100; FCP 1,244 ms, LCP 2,295 ms, TBT 56 ms, CLS 0.
- Keyboard: skip link first; demo action opens and runs; signed JSON opens with Enter, receives focus, and scrolls with PageDown. No trap.
- 200% text and 390px width: no horizontal overflow on product, demo, or legal routes.
- Privacy: the complete demo request log is same-origin. No analytics, CDN script, hosted font, or undisclosed request.
- Offline/update: service-worker update succeeds; a dedicated browser context reloads the last demo result offline without closing the shared browser.
- Billing: production redirects to `checkout.dodopayments.com`; pilot redirects to `test.checkout.dodopayments.com`; a no-charge pilot purchase completed with the factory test card.
- This is not a package, library, or CLI, so package-consumer tests do not apply. It has no sign-in authority.

## Deployment and live verification

Deploy and verify with:

```sh
npm run deploy
npm run verify:live-topology
```

The deployed app uses Azure Container Apps in single-revision mode with min/max one replica. Azure Files share `sf-alert-evidence-envelope-data` is registered as `alert-evidence-envelope-data` and mounted at `/data`. The live image tag and `/health.build` match `git rev-parse HEAD`. Exactly one revision is active and one replica is running.

Live policy checks cover protected endpoints before parsing, 429 plus `Retry-After: 1`, consistent repeated history reads, CSP/HSTS/nosniff/no-referrer, true 404, discovery files, production checkout redirect, desktop/390px demo, console, keyboard, axe, offline reload, and the compiled footer identity.

## Known gaps

No release-blocking product gap remains. The optional Field Kit verification depends on the Sociobot billing API by design; the free relay and demo do not depend on it.
