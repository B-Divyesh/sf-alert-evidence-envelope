# Repair 12 handoff — PASS

Date: 2026-09-05

Implementation and deployment SHA: `db73a281fe9cd1911851a239d2b31d9bbcbedff0`

Live revision: `sf-alert-evidence-envelope--0000040`

Live image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:db73a281fe9c`

## Outcome

Strict review 6 finding F-6-1 is fixed. The editable **Sample alert JSON**
field now controls the demo preview. A preview captures the current JSON with
its demo session, selected route, operation generation, and abort signal. It
never falls back to the built-in sample after the user edits the field.

Malformed JSON now stops in the browser before a preview request. The page
shows **Preview stopped** with the commas-and-quotes recovery message.
**Restore valid sample** restores the shipped alert, and the next preview
succeeds.

The previous reset/exit isolation guard remains intact. Demo requests still use
the session endpoint without an admin token. Leaving the demo cancels pending
work, removes all `demo:` browser keys, deletes the temporary session, and
makes no protected-route request.

## Regression coverage

The new `editable-demo` claim exercises outcomes rather than source text. In a
fresh demo it:

- edits both service and error fields;
- checks the edited values in the outgoing alert and rendered envelope;
- submits `{` and confirms that no preview request is made;
- checks the visible recovery message; and
- restores the shipped sample and builds a successful envelope again.

`.factory/claims.json` now contains 30 claims with exactly one tagged test per
claim. `.factory/demo.md` documents the editor, validation, and recovery path.

## Clean verification

A detached clean worktree at the implementation SHA ran `npm ci`, followed by
all 30 exact commands in `.factory/claims.json`. Every command passed.

The same clean worktree also passed:

```sh
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 npm run build
BUILD_SHA=db73a281fe9cd1911851a239d2b31d9bbcbedff0 cargo build --release --locked
```

`npm test` reported zero Svelte diagnostics, 25 Rust tests, passing deployment
and claim-manifest checks, and 63 passing Playwright cases with one intentional
desktop skip for the mobile-only geometry claim. The production frontend is
74,151 bytes of JavaScript (27.22 KB gzip) and 22,318 bytes of CSS (5.84 KB
gzip). All 18 clean candidate files matched the live files byte for byte.

## Live verification

- The scoped deploy completed through ACR. `/health` reports the full
  implementation SHA.
- The app remains in single-revision mode with min/max/running replicas set to
  one. The existing `alert-evidence-envelope-data` storage is mounted at
  `/data`. Twenty fresh create/preview checks passed.
- Fresh 1440 × 900 and 390 × 844 pages showed the job, audience, and **Try it
  with sample data** action before scrolling.
- Both browsers built the default sample, sent edited service/error values,
  rejected invalid JSON without a request, recovered, reset to a new session,
  and exited with no demo keys or protected API calls.
- The phone result card remained fully inside the first viewport at
  `x=12–378`, `y=286.69–601.47`.
- The complete browser flow contacted only the product origin and emitted no
  console or page errors.
- Twenty live Axe audits covered home, demo, Privacy, Terms, and the designed
  404 in desktop/phone and light/dark modes. There were no serious or critical
  violations.
- Keyboard focus, route announcement, 44 px targets, 200% text checks, and
  response-header security checks remain covered by the passing browser suite.
  A fresh live reduced-motion context had no running animation.
- The service worker used `envelope-shell-v6`. A fresh phone demo reloaded
  offline with its envelope and made no API, health, or failed request.
- Seventeen live links and discovery assets passed. The checkout returned its
  expected 303 redirect. The deliberate missing route returned the designed
  HTTP 404.
- Live demo boundaries accepted 1/100 records and 1,024/131,072 bytes, rejected
  0/101 records, 1,023/131,073 bytes, and malformed JSON, then accepted a valid
  recovery request.
- Two live demo tenants stayed isolated: deleting one returned 404 on reuse
  while the other retained its own result.
- A fresh 100-request protected-API burst returned 43 normal credential
  responses and 57 rate-limited responses. Every 429 included
  `Retry-After: 1`; a different forwarded client immediately received 401.
- `/opt/fleet/lib/verify-url.sh` passed in 692 ms with the correct title,
  `lang=en`, one h1, a main landmark, image alternatives, labelled buttons,
  and no console errors.
- Mobile Lighthouse: performance 95, accessibility 100, best practices 100,
  SEO 100, FCP 1.35 s, LCP 1.94 s, TBT 244 ms, CLS 0, and 140,184 bytes total.

Evidence is under `/work/.evidence/repair12-*`. The catalog description was
copied to `/work/.evidence/catalog-description.txt`. The live $39 USD one-time
Field Kit offer is recorded without credentials in
`/work/.evidence/billing-offer.json`.

## Earlier findings

All earlier verification, review, and polish reports were read before the
repair. Their closures remain intact:

- legal HTTP status, mobile overflow, build identity, strict Clippy, expanded
  JSON keyboard access, dark contrast, HSTS, anti-framing, and 44 px targets;
- the one-click isolated demo, 24-hour expiry, offline state, no tracking,
  protected APIs, durable `/data`, one replica, and rate limiting;
- complete mobile results, Slack/JSON/email delivery contracts, fixed-source
  fetching, recursive redaction, HMAC verification, provider signatures,
  metadata-only history, and route isolation;
- license transport and throttling, free controls, the $39 one-time offer,
  revocation behavior, local presets, and credential non-exposure;
- route titles, focus announcements, legal chrome, 404 recovery, discovery
  metadata, plain wording, catalog wording, optional URL normalization, and
  generated-art provenance; and
- review 5's reset/exit race, re-proved alongside the new editable-input test.

F-6-1 is closed by the implementation and live evidence above. No earlier
minor finding reopened.

## Run and deploy

```sh
npm ci
npm test
npm run build
npm run deploy
npm run verify:live-topology
```

The container starts with only `PORT`. It persists SQLite and generated server
credentials under `/data` when the fleet mount is present.

## Known gaps

No product defect is known. This worker has no local Docker-compatible engine,
so it did not assemble the container locally. The direct release build, remote
ACR build, live image identity, static hashes, topology, and HTTPS behavior all
passed.
