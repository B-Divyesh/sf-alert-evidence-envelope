# Verification 15 — PASS

Date: 2026-09-02

Work order: `alert-evidence-envelope-verify-15`

Candidate: `34c18ffe0d3d779a07baf2620969cd89636a7a60`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** The live deployment is the candidate, every declared claim passes
after the documented clean install, and the product completes the researched
webhook-transformer job end to end. No product code or infrastructure was
changed during verification.

## Severity summary

| Severity | Count | Findings |
| --- | ---: | --- |
| P0 / critical | 0 | None |
| P1 / high | 0 | None |
| P2 / medium | 0 | None |
| P3 / low | 0 | None |

## Mandatory first checks

### Claims gate

`.factory/claims.json` exists and declares 28 claims. As required, every exact
manifest command was invoked before other repository inspection. The initial
invocation from the dependency-free checkout recorded 14 Rust/policy passes
and 14 browser-command setup failures (`vite: not found`). After the required
clean `npm ci` (56 packages, zero vulnerabilities), all 28 exact claim commands
were run again and passed. The manifest validator also passed and confirmed
one tagged regression test per claim.

Passed claims: `demo-envelope`, `mobile-demo-result`,
`demo-route-policies`, `bounded-redacted-signed`, `query-fingerprint`,
`fixed-query-source`, `provider-signature`, `slack-delivery`,
`isolated-demo`, `raw-not-retained`, `history-limit`,
`protected-real-apis`, `credential-storage`,
`credential-browser-exposure`, `preview-no-history`,
`per-route-isolation`, `no-tracking`, `offline-demo`, `license-transport`,
`local-policy-presets`, `license-throttle`, `free-core`,
`field-kit-purchase`, `license-revocation`, `provenance-license`,
`durable-deployment`, `rate-limit`, and `destination-contracts`.

Landing, application, legal, README, catalog, and demo wording were checked
against the manifest. No unsupported public product claim was found.

### Cold first-read gate

PASS in fresh 1440 × 900 and 390 × 844 browser contexts. The first screen says:

- What it does: **“Add redacted evidence to webhook alerts.”**
- Who it serves: on-call engineers and webhook consumers who need incident
  context without another dashboard login.
- What to select first: **“Try it with sample data.”**

The action is visible without scrolling at 390 px. Its adjacent sentence says
that it opens a signed, redacted envelope in an isolated workspace. One click
opens a completed sample. The complete mobile result occupied x=12–378 and
y=280.69–595.47 inside the 390 × 844 viewport.

## Clean local gates

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 56 packages, 0 vulnerabilities |
| `npm test` | PASS — Svelte 0 errors/warnings; 25 Rust tests; policy and manifest checks; 59 Playwright passes and one intentional desktop skip for the mobile-only geometry case |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=34c18ffe… npm run build` | PASS; `dist/` produced |
| `BUILD_SHA=34c18ffe… cargo build --release --locked` | PASS |
| `git diff --check` | PASS before report creation |

The exact candidate build contains 72,458 bytes JavaScript (26.76 KB gzip),
21,973 bytes CSS (5.78 KB gzip), 115,560 bytes of self-hosted fonts, and a
40,982-byte mobile hero. These are within the supplied budgets. Docker,
Podman, and Buildah are unavailable in this verifier image; the frontend and
optimized Rust Docker build stages passed directly, while live image identity
was verified separately.

## Deployment identity and topology

- `/health` returns HTTP 200, `status: ok`, and the full candidate SHA.
- All 18 candidate-built files in `dist/` match their live responses byte for
  byte by SHA-256.
- Scoped topology verification passed without reading another service.
- Revision: `sf-alert-evidence-envelope--0000037`.
- Image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:34c18ffe0d3d`.
- Single revision mode, min/max replicas 1, one running replica, and the owned
  `alert-evidence-envelope-data` storage mounted at `/data`.
- Twenty fresh-connection live demo previews passed.
- `/opt/fleet/lib/verify-url.sh` passed in 866 ms with no browser errors,
  `lang=en`, one `h1`, one `main`, complete image alternatives, and labelled
  buttons.

## End-to-end product evidence

The live demo met the researched success measure for 20/20 distinct seeded
alerts. Every response exposed the exact service, error signature, and
first-seen time without a dashboard lookup, recursively redacted email/token
fields, and included an HMAC-SHA256 signature.

A live boundary request with two records and a one-record/1,024-byte cap
returned one 42-byte redacted record and `truncated: true`. Invalid zero-item,
1,023-byte, and malformed-JSON inputs returned useful HTTP 400 errors. A
300 KB request returned 413, and a valid request immediately afterward
returned 200. Deleting a demo workspace returned 204; reusing it returned 404.

Twenty simultaneous live create/preview/delete flows and twenty simultaneous
local flows each passed `200/200/204`, with isolated service values and
redaction intact.

An independent local capture server received real deliveries from the release
binary for all destination contracts:

- JSON received the complete envelope and signature header.
- Slack received the envelope plus readable `text`.
- Email webhook received `subject`, `text`, and `envelope`.
- Every delivery retained one recursively redacted record, reported
  truncation, and forwarded the provider signature.
- Independent HMAC-SHA256 recomputation from the persisted signing key matched.
- History contained metadata only; a closed SQLite byte scan found none of the
  unique email, token, or provider-signature markers.

The protected browser workflow also loaded a route, created a second route
with both optional URLs serialized as JSON `null`, built a signed preview,
reloaded the route, and deleted it. The admin credential appeared in neither
rendered HTML nor browser storage. All live protected configuration, route,
history, preview, and relay endpoints returned 401 before parsing malformed
bodies without credentials.

## Runtime, persistence, and rate limiting

The release binary started in a new directory with an empty environment except
`PATH` and `PORT`. It generated SQLite, a signing key, an admin token, and an
inbound token. Credential files were mode 0600, and logs identified generated
versus supplied/persisted configuration without printing values.

After graceful restart, all three credential hashes were unchanged and the
saved route remained present. This verifies the local fallback persistence
boundary; live topology separately confirms `/data` durability.

A single local client issued 100 concurrent protected requests in 258 ms:
40 returned 401 and 60 returned 429. Every 429 included `Retry-After: 1`.
Against production, 100 concurrent requests completed in 539 ms: 45 returned
401 and 55 returned 429; all 429 responses had `Retry-After: 1` and advertised
limit 40. The additional five permits are consistent with the documented
20-per-second refill. A fresh forwarded client immediately received the normal
401. One hundred concurrent health checks returned 200.

Observed allowance: **40-request burst, refilling at 20 requests per second,
keyed by the first forwarded client IP; health exempt.**

## Browser, privacy, accessibility, and PWA

- Twenty independent axe scans covered home, demo, privacy, terms, and the
  designed 404 at desktop/mobile and light/dark: zero serious or critical
  findings.
- All audited pages have `lang=en`, one `h1`, one `main`, route-specific
  titles, no horizontal overflow, and no visible target below 44 × 44 px.
- The first Tab reaches **Skip to main content** with a 3 px amber focus ring;
  the next task control is **Try it with sample data** with the same focus
  treatment. Enter opens the demo. Opening signed JSON by keyboard makes the
  bounded JSON region the next focus target, and Page Down scrolls it.
- Invalid `{` input announces “Sample alert is not valid JSON. Check commas and
  quotes.” through a polite live region. Restoring the sample and rebuilding
  succeeds.
- Reset replaces the demo session; Start for real removes every `demo:` browser
  key.
- A full landing/demo/error/recovery/reset/exit flow requested only the product
  origin and produced no console or page errors.
- At 200% text size, home, demo, privacy, and terms retain their headings,
  landmarks, and primary actions with no 390 px horizontal overflow.
- Reduced-motion emulation matches, changes scrolling to `auto`, caps
  transitions at 0.01 ms, leaves no running animation after settling, and has
  no infinite animation.
- The service worker controls the page, updates successfully, and uses cache
  `envelope-shell-v6`. Offline reload restores the completed sample without an
  API/health request or failed request.

The browser response logs contained no analytics, advertising, third-party
scripts, or hosted font requests. The CSP permits only self-hosted assets plus
the documented Sociobot license API and sends `frame-ancestors 'none'` as a
header. HSTS, `nosniff`, `DENY`, and `no-referrer` are present. HTML and stable
assets use `no-cache`, API/health use `no-store`, and hashed JS/CSS use one-year
immutable caching.

Internal routes, fragment targets, the source link, and the official checkout
link resolve as intended. The checkout endpoint returns the expected 303 to
hosted checkout. The 404 skip link correctly targets its own `main` landmark
while the document retains the required 404 status.

## Performance and assets

Lighthouse mobile, second clean run:

- Performance: 96
- Accessibility: 100
- Best practices: 100
- SEO: 100
- FCP: 1.38 s
- LCP: 1.87 s
- TBT: 215 ms
- CLS: 0
- Total transfer: 138,155 bytes

The 720 × 480 mobile hero is 40,982 bytes; the 1536 × 1024 desktop WebP is
187,278 bytes. The social card is a real 1200 × 630 JPEG and the touch icon is
180 × 180. Asset prompt, generation date, review, and originality provenance
are recorded in `.factory/design.md` and `assets/src/evidence-terrain.json`.

## Applicability

The product requires no sign-in, so Entra External ID does not apply. It is
not a library or CLI, so consumer package installation does not apply. LLM
summarization is explicitly a non-goal in the researched brief; no AI feature
is missing. License verification is a direct browser request to the shared
Sociobot billing API, not a product server endpoint; the owned backend's API
allowance was verified without load-testing infrastructure outside scope.

## Evidence locations

- `/tmp/alert-envelope-claim-results.txt` — mandated pre-install invocation
- `/tmp/alert-envelope-claim-results-installed.txt` — all 28 claims passing
- `/tmp/aee-verification-15-npm-test.log` — full repository suite
- `/tmp/aee-verification-15-{fmt,clippy,build,release}.log` — build gates
- `/tmp/aee-v15-axe.json` — 20 live axe/semantics audits
- `/tmp/aee-v15-lighthouse-second.json` — clean Lighthouse result
- `/tmp/aee-v15-verify-url/` — worker URL audit and screenshots
- `/tmp/live-cold-{desktop,mobile}.png` — cold first-read captures
- `/tmp/aee-v15-live-demo-mobile.png` — completed 390 × 844 demo

## Known gaps

No product defect was found. The verifier environment had no container-engine
executable, so a new local Docker image was not assembled; the exact frontend
and optimized Rust builds passed, and the scoped deployed image, build SHA,
topology, and static contents were verified independently.
