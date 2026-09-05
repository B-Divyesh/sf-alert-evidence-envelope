# Verification 16 handoff — PASS

Date: 2026-09-05

Independent QA of implementation `9c741c506d374e71578605ed43593d76f0ab5620`
is **PASS** with zero findings and zero untested claims. The live runtime reports
the report-only documentation SHA `2f47a5aa2464715e8309921d29fc153f1f8755cd`;
the diff is only this handoff, and live topology reports revision
`sf-alert-evidence-envelope--0000039` with one replica and `/data` mounted.

From a clean clone, `npm ci`, all 29 exact declared claim commands, `npm test`,
formatting, strict Clippy, frontend build, and Rust release build passed. Fresh
desktop and 390 × 844 phone checks confirmed the plain first screen, one-click
sample, complete bounded/redacted/signed output, persistent demo label, reset,
exit cleanup, isolation, offline reload, accessibility, legal/404 routes,
links, headers, and live rate limiting. The reset/exit race passed live with
and without an admin token: no protected call, no demo authorization, and no
remaining `demo:` key.

Run locally:

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
npm run build
cargo build --release --locked
npm run verify:live-topology
```

No known product gaps. See `.factory/verification-16.md` for evidence and the
full prior-finding disposition.

## Prior repair handoff

Date: 2026-09-05

Implementation deployed: `9c741c506d374e71578605ed43593d76f0ab5620`
Live revision: `sf-alert-evidence-envelope--0000038`
Live image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:9c741c506d37`

## What changed

Rapid **Reset demo** then **Start for real** could previously let an unfinished
demo start choose `/api/v1/preview` after the page had moved to the real route.
The demo now captures an operation generation, session, route, and abort signal
before asynchronous work. Demo preview requests always use the demo session
endpoint and never attach the loaded admin token. Leaving demo mode cancels and
waits for pending demo work, clears every `demo:` key, and removes any created
demo session before navigating. Reset and exit controls are disabled while a
reset is in progress.

The claim inventory now separates browser isolation from server expiry:

- `isolated-demo` is a browser outcome test. It delays the reset delete, forces
  the queued exit event, runs with and without a loaded admin token, and observes
  no protected request, no demo authorization header, and no remaining demo key.
- `demo-expiry` keeps the server-side temporary SQLite test for 24-hour expiry
  and route-history isolation.

The Privacy page and README now say that demo workspaces expire after 24 hours.

## Earlier review history

The full review and verification history (`review-1` through `review-5`,
`verification-1` through `verification-15`, and `polish-1` through `polish-4`)
was checked before this repair. The earlier mobile result, license transport,
claim coverage, navigation/focus, 404, wording, route-policy, legal chrome,
destination-contract, and mobile-layout findings remain closed as recorded in
the latest verification evidence. Review 5 finding `F-5-1` is closed by this
repair and its new regression coverage. No earlier minor finding was reopened.

## Verification

- Fresh clone `/tmp/aee-repair11-clean-sPBLwn` at implementation SHA:
  `npm ci`, then all **29** exact commands in `.factory/claims.json` passed.
  Log: `/tmp/aee-repair11-claim-commands.log`.
- Repository suite: `npm test` passed — Svelte check had zero diagnostics, all
  25 Rust tests passed, deployment policy and claims manifest passed, and all
  62 Playwright cases passed. `test-results/.last-run.json` records `passed`.
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  `npm run build`, and `cargo build --release --locked` passed. Final frontend
  build is 74.12 KB JavaScript (27.20 KB gzip) and 22.31 KB CSS (5.84 KB gzip).
- `npm run verify:live-topology` passed. The product runs in single revision
  mode with one healthy replica and `alert-evidence-envelope-data` mounted at
  `/data`; it made 20 fresh live demo previews.
- Fresh live desktop (1440 × 900) and phone (390 × 844) contexts first showed:
  **Add redacted evidence to webhook alerts**; the on-call/webhook-consumer
  audience; and **Try it with sample data**. In both contexts the sample showed
  the persistent demo label, `checkout-api`, the timeout, and first-seen time;
  normal reset made a new session; exit left no `demo:` key and requested no
  protected route endpoint.
- Live Axe scans on `/`, `/demo`, `/privacy`, `/terms`, and the intentional
  `/not-a-real-route` 404 found zero serious or critical violations. The worker
  URL audit found one title, `lang=en`, one main landmark, complete image alt
  text, labelled buttons, and no browser errors (694 ms load).
- Live limiter check with a fresh forwarded address returned 43 × 401 and
  57 × 429; every 429 had `Retry-After: 1`.

## Run and deploy

```sh
npm ci
npm test
npm run build
npm run deploy
npm run verify:live-topology
```

The container needs only `PORT`; it generates or persists required server
credentials and SQLite state under `/data` when the durable mount is present.

## Known gaps

No product defects are known. Docker, Podman, and Buildah were not available
in this worker, so a local container image was not assembled. The direct Rust
release build and deployed ACR image/topology were verified instead.
