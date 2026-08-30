# Alert Evidence Envelope — repair 4 handoff

## Release status

All findings in `verification-3.md` are repaired. The container release is deployed at `https://alert-evidence-envelope.sociobot.in`; `/health` must identify the running image as the current Git commit. The factory deployment uses the immutable tag `sf-alert-evidence-envelope:<first 12 commit characters>` and passes the full SHA through `BUILD_SHA`.

## Reproduction and repairs

The reported failure was reproduced before changes at a 390 × 844 viewport in dark mode:

- The expanded JSON had `clientHeight=260`, `scrollHeight=720`, `overflow-y=auto`, and no `tabindex`. Axe reported `scrollable-region-focusable` as serious.
- The focused skip link computed to white on `rgb(220, 233, 223)`; `SEALED` computed to white on `rgb(89, 201, 145)`. Axe reported both under serious `color-contrast`.
- The live HTTPS response did not include `Strict-Transport-Security`.

Root-cause repairs:

- The signed JSON is a labelled `tabindex="0"` scroll region with a visible focus ring. The regression test focuses it through the keyboard and proves `PageDown` changes its scroll position.
- Dark-mode skip and sealed-state labels now use `#101815` ink. The successful expanded state is audited by axe in light and dark modes on desktop and 390 px.
- All application responses include `Strict-Transport-Security: max-age=63072000; includeSubDomains`; browser coverage asserts it on both `/` and `/health`.
- The successful mobile state now constrains grid min-content width. Its measured document width is exactly 390 px after the long JSON is expanded.
- Proxy-aware rate limiting now keys the first `X-Forwarded-For` hop and falls back to the socket peer. The Docker Rust builder follows the current stable `rust:1-alpine` contract.

## Verification evidence

Clean install and complete local gates:

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
git diff --check
BUILD_SHA=repair-validation-20260830 cargo build --release --locked
```

- `npm ci`: 0 vulnerabilities.
- Svelte check: 0 errors and 0 warnings.
- Rust: 7/7 unit and route-integration tests passed.
- Playwright 1.58.2: 18/18 passed across desktop Chromium and 390 × 844 Chromium.
- Expanded preview smoke in both themes and viewports: JSON `260/720` px viewport/content height, keyboard scroll position `227`, visible focus outline, zero serious/critical axe findings, zero console/page errors, and same-origin requests only.
- Invalid JSON showed `Preview stopped`; restoring the sample produced the signed, redacted envelope. Desktop remained `1366/1366` and mobile `390/390` document/viewport width.
- Rate-limit smoke: 400 requests from distinct forwarded IPs returned 200; a single forwarded IP received 497 × 200 and 303 × 429 with `Retry-After`; a separate IP still returned 200.
- Response policy: `/`, `/privacy`, `/terms`, `/health`, API, service worker, and hashed assets all include HSTS, CSP, `nosniff`, and `no-referrer`. Legal routes return 200, unknown routes 404, API/health use `no-store`, shell/service worker use `no-cache`, and hashed assets are immutable.
- First boot with an empty environment except `PORT` generated a 32-byte mode-600 signing key and served `/health` with the compiled build identity.

Mobile Lighthouse 13.4.1 against the local production build served by Axum:

- Performance 98; accessibility 100; best practices 100; SEO 100.
- LCP 2,408 ms; TBT 48 ms; CLS 0; FCP 1,355 ms.
- Initial JS is 63,217 bytes raw / 24.56 kB gzip; CSS is 16,524 bytes raw / 4.76 kB gzip; fonts total 115,560 bytes; mobile hero is 40,982 bytes.

## Run and deploy

```sh
npm ci
npm test
PORT=8080 cargo run

/opt/fleet/lib/deploy-container.sh alert-evidence-envelope /work/repo Dockerfile 8080
/opt/fleet/lib/verify-url.sh https://alert-evidence-envelope.sociobot.in <evidence-directory>
```

The deployment requires only `PORT`; persist `/data` for the generated signing key and SQLite metadata. No secrets are committed.

## Known gaps

No known release-blocking or minor verifier findings remain.
