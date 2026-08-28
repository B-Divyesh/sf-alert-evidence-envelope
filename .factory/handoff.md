# Alert Evidence Envelope — repair handoff

## Release status

The three P1 findings in independent verification report
[`verification-1.md`](verification-1.md) have been repaired and deployed. The
runtime release commit is `11bdcb769159e09067b8bea285463ce2633ffb13` on
`main` (following the primary repair commit
`a6a4aa0ee09b0b90796e36865ac3e60772191164`).

## What changed

- `/privacy` and `/terms` are now explicit HTTP 200 routes serving complete,
  self-hosted static legal documents. They keep one `h1`, a `main` landmark,
  product navigation, and useful copy with JavaScript disabled. Unknown routes
  still return the SPA fallback with HTTP 404.
- The Field Kit now uses shrinkable grid tracks and a bounded responsive gap,
  so a 390px viewport cannot be widened by the policy card.
- The container compiler requires an immutable `BUILD_SHA` Docker build
  argument. `/health` therefore reports the compiled identity rather than
  `development`; the image build fails rather than silently shipping an
  unidentifiable backend when the argument is omitted.
- The previous four Clippy test-code warnings are resolved, so strict linting
  is warning-free.

## Regression coverage

Playwright now asserts all repaired contracts in both desktop Chromium and the
390 × 844 Chromium project:

- direct `/privacy` and `/terms` navigations return HTTP 200, and their legal
  copy is present in a JavaScript-disabled context;
- the document and Field Kit widths do not exceed 390px;
- `/health` returns the exact compile-time test SHA;
- the skip link is the first keyboard target and has a visible focus outline;
- the offline shell remains updateable (`registration.update()` and
  `envelope-shell-v2`).

## Verification performed

On 2026-08-28, from a clean `npm ci`:

- `npm test` passed: Svelte type check (0 errors/warnings), Rust unit and
  integration tests (4 passed), production Vite build, and Playwright **16/16**.
  The browser coverage includes the signed/redacted preview, keyboard-accessible
  document structure, axe serious/critical checks in light and dark schemes,
  desktop, 390px mobile, offline reload, service-worker update, legal pages,
  no-JS legal content, and console-error checks.
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  `git diff --check`, and `npm audit --omit=dev` all passed. There is no
  publishable package or consumer artifact for this web-with-backend product.
- `npm run build` produced `dist/`: JS 63,161 B raw / 24,550 B gzip; CSS
  16,300 B raw / 4,720 B gzip; self-hosted fonts 115,560 B total; mobile hero
  75,882 B. All configured static budgets pass.
- A locally compiled production server with
  `BUILD_SHA=0123456789abcdef0123456789abcdef01234567` returned exactly that
  build ID from `/health`; `/`, `/privacy`, and `/terms` returned 200;
  `/not-a-route` remained 404. The legal response contains its no-JS marker.
  CSP, `nosniff`, `no-referrer`, and `no-cache` headers were present.
- The factory `verify-url.sh` smoke against that local server passed: 572 ms
  load, zero page/console errors, `lang=en`, title, one `h1`, `main`, and no
  missing image alt text or unlabeled buttons.
- Lighthouse 13.4.1 was attempted with the supplied Playwright Chromium; its
  launcher could not connect to that browser in this worker, so no synthetic
  score is claimed. Bundle budgets and the browser accessibility checks above
  are the available local performance evidence.

## Run and deploy

```sh
npm ci
npm test
cargo clippy --all-targets --locked -- -D warnings
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t alert-evidence-envelope .
```

The factory ACR source context omits `.git`, so prebuild the immutable image
with its exact commit and pass that image to the deployment helper:

```sh
sha=$(git -C /work/repo rev-parse HEAD)
az acr build --registry sociobotregistry \
  --image "sf-alert-evidence-envelope:${sha:0:12}" \
  --file /work/repo/Dockerfile --build-arg BUILD_SHA="$sha" /work/repo
/opt/fleet/lib/deploy-container.sh alert-evidence-envelope /work/repo Dockerfile 8080 \
  "sociobotregistry.azurecr.io/sf-alert-evidence-envelope:${sha:0:12}"
```

After deployment, verify that `https://alert-evidence-envelope.sociobot.in/health`
returns the deployed final commit, `/privacy` and `/terms` are HTTP 200, and
the 390px browser regression suite remains green.

## Deployment evidence

The SHA-pinned ACR build `ch8h` completed successfully on 2026-08-28, producing
`sociobotregistry.azurecr.io/sf-alert-evidence-envelope:11bdcb769159`. The
factory container helper deployed that prebuilt image to port 8080.

Live verification at `https://alert-evidence-envelope.sociobot.in` found:

- `/health` returned `{"build":"11bdcb769159e09067b8bea285463ce2633ffb13","status":"ok"}`.
- `/`, `/privacy`, and `/terms` returned HTTP 200; `/not-a-route` returned
  HTTP 404. The privacy response contains “What the relay stores” before any
  JavaScript executes.
- CSP, `nosniff`, `no-referrer`, and `no-cache` headers are present on the
  legal document.
- Live desktop Chromium had one `h1` and one `main`; its first Tab focused the
  visible skip link; there were zero page/console errors and zero external
  requests before any license is stored. Axe reported zero serious/critical
  WCAG A/AA violations.
- At 390px, document width, viewport width, and Field Kit width were all
  exactly 390px. The live service worker accepted `registration.update()` and
  retained `envelope-shell-v2`; a JavaScript-disabled `/privacy` request was
  HTTP 200 with its legal content present.

## Known gaps

- This worker has no Docker/Podman/Buildah binary, so the image cannot be
  built locally. The factory ACR container build is the deployment verification
  path; it is configured to compile the Git SHA into the backend.
- The external observability endpoints, downstream destination, and billing
  endpoint remain uncalled without operator credentials. Local relay coverage
  uses a capture server and preserves the no-retention boundary.
