# Verification 14 — PASS

Date: 2026-09-02

Work order: `alert-evidence-envelope-verify-14`

Candidate: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

Live URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** The candidate is the live deployment, the prior optional-URL route
creation blocker is repaired, all 28 declared claims pass, and the product
completes the researched job end to end. No product code or infrastructure was
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

`.factory/claims.json` exists and contains 28 claims. The initial invocation
before dependency installation reached the first browser command and reported
`vite: not found`, as expected in a clean Node checkout without `node_modules`.
After the repository's required clean `npm ci`, every manifest command was run
individually and passed. The install reported 56 packages and 0 vulnerabilities.

| Claim | Result |
| --- | --- |
| `demo-envelope` | PASS — desktop and mobile |
| `mobile-demo-result` | PASS — desktop and mobile |
| `demo-route-policies` | PASS — desktop and mobile |
| `bounded-redacted-signed` | PASS |
| `query-fingerprint` | PASS |
| `fixed-query-source` | PASS |
| `provider-signature` | PASS |
| `slack-delivery` | PASS |
| `isolated-demo` | PASS |
| `raw-not-retained` | PASS |
| `history-limit` | PASS |
| `protected-real-apis` | PASS |
| `credential-storage` | PASS |
| `credential-browser-exposure` | PASS — desktop and mobile |
| `preview-no-history` | PASS |
| `per-route-isolation` | PASS |
| `no-tracking` | PASS — desktop and mobile |
| `offline-demo` | PASS — desktop and mobile |
| `license-transport` | PASS — desktop and mobile |
| `local-policy-presets` | PASS — desktop and mobile |
| `license-throttle` | PASS — desktop and mobile |
| `free-core` | PASS — desktop and mobile |
| `field-kit-purchase` | PASS — desktop and mobile |
| `license-revocation` | PASS — desktop and mobile |
| `provenance-license` | PASS — desktop and mobile |
| `durable-deployment` | PASS |
| `rate-limit` | PASS |
| `destination-contracts` | PASS |

The manifest validator also passed: every claim has exactly one tagged
regression test. Landing, demo, legal, README, design, demo documentation, and
copy audit were cross-checked; no unsupported public claim was found.

### Cold first-read gate

PASS at 1440 × 900 and 390 × 844 in fresh browser contexts. Before any action,
the first screen answers all three required questions:

- What it does: **“Add redacted evidence to webhook alerts.”**
- Who it serves: on-call engineers and webhook consumers who need incident
  context without another dashboard login.
- What to select first: **“Try it with sample data.”**

The adjacent sentence says the sample opens a signed, redacted envelope in an
isolated workspace. The action is visible without scrolling and opens the
completed demo in one click.

## Clean local gates

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 56 packages, 0 vulnerabilities |
| `npm test` | PASS — Svelte 0 errors/warnings; 25 Rust tests; deployment and manifest checks; 60 Playwright cases |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `VITE_BUILD_SHA=f4bf8ae… npm run build` | PASS; `dist/` produced |
| `BUILD_SHA=f4bf8ae… cargo build --release --locked` | PASS |
| `git diff --check` | PASS |

The exact candidate frontend contains 72,063 bytes JavaScript (26.68 KB gzip),
20,049 bytes CSS (5.49 KB gzip), 40,982 bytes for the mobile hero, and 115,560
bytes of self-hosted fonts. These are within all supplied budgets. Docker,
Podman, and Buildah are unavailable in this verifier image; the exact frontend
and optimized Rust stages passed directly, and the live scoped image and build
identity were verified.

## Live identity and deployment

- `/health` returns HTTP 200 and full build
  `f4bf8ae31eb1c8be548508341d75d7fed251977c`.
- All 18 candidate-built `dist/` files match their live counterparts byte for
  byte by SHA-256.
- `npm run verify:live-topology` passes for the owned product only.
- Revision: `sf-alert-evidence-envelope--0000035`.
- Image: `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:f4bf8ae31eb1`.
- Single revision mode; min/max replicas 1; one running replica; product data
  storage is mounted at `/data`.
- The topology check completed 20/20 fresh-connection demo previews.
- `/opt/fleet/lib/verify-url.sh` passes in 954 ms with one `h1`, `lang=en`, a
  main landmark, complete image alternatives, labelled controls, and no errors.

No unrelated app, database, key vault, storage account, setting, secret, DNS
record, or infrastructure resource was read or changed.

## End-to-end product evidence

The live demo returned a signed envelope for a representative orders incident:

- service `orders-api`, error signature `database connection timeout`, and the
  supplied first-seen timestamp;
- two evidence records and a 152-byte evidence excerpt;
- recursive email and nested-token redaction;
- a 16-character query fingerprint and 64-hex-digit HMAC-SHA256 signature.

A one-record, 1,024-byte boundary returned one 101-byte record,
`truncated: true`, recursive redaction, and a valid signature. Invalid JSON,
`max_items: 0`, and `max_bytes: 1023` returned useful HTTP 400 responses.
Deleting the demo workspace returned 204; using it afterward returned 404.

The researched success measure passed: 20/20 seeded alerts exposed the exact
service, error signature, and first-seen time without a dashboard lookup, and
20/20 redacted both private fields. Twenty simultaneous create/preview/delete
flows from isolated clients passed `200/200/204` with the expected output.

Every protected configuration, route, history, preview, and inbound relay
request returned 401 before malformed body parsing when no credential was
provided.

### Real relay and repaired route workflow

An independent local capture server received real deliveries from the
candidate release binary:

- JSON delivery received the complete bounded envelope and signature header.
- Slack received the complete envelope at top level plus readable `text`.
- Email webhook received `subject`, `text`, and the complete `envelope`.
- Every format contained one recursively redacted row, reported truncation,
  and carried a verifiable HMAC-SHA256 signature.
- A supported `x-signature` provider header was forwarded as
  `x-original-provider-signature` and reflected by
  `source_signature_preserved: true`.
- Independent HMAC recomputation using the generated key matched exactly.
- Delivery history contained metadata only. A byte scan of the closed SQLite
  file found none of the unique inbound body or provider-signature markers.

The previous blocker is fixed in the shipped UI. In a real Chromium session,
both optional URL fields were left blank. **Create route** sent both values as
JSON `null`, returned 200, and the new route remained selectable after reload
with both inputs blank. The temporary route was deleted afterward.

## Runtime, persistence, and rate limiting

The release binary started in an empty directory with an empty environment
except `PATH` and `PORT`. It generated SQLite, its signing key, admin token, and
inbound token without additional configuration. The three credential files had
mode 0600, and startup logs reported generated versus persisted sources without
printing secret values.

After graceful restart, all credential hashes were unchanged, the demo session
remained usable from a fresh connection, and `/health` still returned the full
candidate SHA. This confirms the fallback data directory and restart boundary;
the live topology separately confirms the durable `/data` mount.

A single live client issued 100 concurrent protected requests in 475 ms:
45 returned the expected 401 and 55 returned 429. Every 429 had
`Retry-After: 1`; responses advertised a 40-request limit. The observed five
additional permits match the documented 20-per-second refill during the burst.
A different forwarded client immediately received the normal 401. Sixty
concurrent health requests all returned 200. A separate 100-request health load
smoke completed 100/100 in 418 ms (about 239 requests/second).

Observed allowance: **40-request burst, refilling at 20 requests per second,
keyed by the first forwarded client IP; health exempt.**

## Browser, privacy, accessibility, and PWA

- A complete landing → keyboard demo → invalid-input recovery → reset → exit
  flow requested only `https://alert-evidence-envelope.sociobot.in`.
- Supported pages produced no console or page errors. The deliberate 404
  produced only Chromium's expected failed-resource message for the 404
  document itself.
- Axe found zero serious or critical issues in 10 independent audits covering
  home, demo, privacy, terms, and designed 404 at desktop light and 390 px dark.
- Every checked page has `lang=en`, one `h1`, a main landmark, a route-specific
  title, no horizontal overflow, and a minimum visible target size of 44 px.
- The first Tab reaches **Skip to main content** with a 3 px amber outline.
  After activation, the next Tab reaches **Try it with sample data** with the
  same visible focus; Enter opens the demo.
- Invalid `{` input announces “Sample alert is not valid JSON. Check commas and
  quotes.” Restoring the valid sample and rebuilding succeeds.
- Reset changes the demo session. **Start for real** clears all `demo:` browser
  storage keys.
- Reduced-motion emulation matches, uses automatic scrolling, has no active
  animation longer than 0.01 ms, and reduces transitions to 0.01 ms.
- The service worker is controlling and updateable with cache
  `envelope-shell-v6`. A dedicated offline context reloads the completed sample
  with no failed request and no API or health request.

The response-header CSP includes `frame-ancestors 'none'`; HSTS, `nosniff`,
`DENY`, and `no-referrer` are present. HTML and stable assets revalidate,
API/health responses use `no-store`, and only hashed JS/CSS use one-year
immutable caching. All internal links return their intended status. The source
link returns 200, and the $39 Field Kit link returns the expected 303 from the
Sociobot billing endpoint to hosted checkout.

## Performance

Lighthouse mobile scores:

- Performance: 98
- Accessibility: 100
- Best practices: 100
- SEO: 100
- FCP: 1.4 s
- LCP: 1.7 s
- TBT: 150 ms
- CLS: 0
- Total transfer: 136,199 bytes

## Applicability notes

This product does not require sign-in, so Entra External ID verification does
not apply. It is not a library or CLI, so consumer package installation does
not apply. The brief explicitly makes LLM summarization a non-goal; no runtime
AI feature is expected.

## Re-run

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA="$(git rev-parse HEAD)" npm run build
BUILD_SHA="$(git rev-parse HEAD)" cargo build --release --locked
npm run verify:live-topology -- \
  https://alert-evidence-envelope.sociobot.in \
  sf-alert-evidence-envelope sociobot "$(git rev-parse HEAD)" \
  alert-evidence-envelope-data
```
