# Verification 16 — PASS

Date: 2026-09-05

## Verdict

**PASS.** There are zero findings at every severity and zero untested declared
claims. The deployed product completes the real job: it builds a bounded,
recursively redacted, HMAC-signed incident evidence envelope and delivers it
through separately configured routes without retaining raw alert bodies.

Implementation reviewed: `9c741c506d374e71578605ed43593d76f0ab5620`  
Documentation commit: `2f47a5aa2464715e8309921d29fc153f1f8755cd`  
Live URL: <https://alert-evidence-envelope.sociobot.in>  
Live revision observed: `sf-alert-evidence-envelope--0000039`

The live health endpoint and image identify the later documentation SHA. Its
only diff from the implementation SHA is `.factory/handoff.md`, so this is the
reviewed implementation with a report-only commit, not a different product
image.

## First screen and sample

Fresh desktop (1440 × 900) and phone (390 × 844) contexts, before scrolling,
showed:

- Job: “Add redacted evidence to webhook alerts”.
- Audience: on-call engineers and webhook consumers who need incident context
  without another dashboard login.
- First action: “Try it with sample data”; it says that it opens a signed,
  redacted envelope in an isolated workspace.

The action opened `/?demo=1` with the persistent “Demo — sample data, nothing
is saved” label. The populated result showed `checkout-api`, “payment
authorization timed out”, first-seen time, redaction, count, byte/truncation
state, fingerprint, and signature. On phone its complete result card was
`x=12`, `y=286.69`, `width=366`, `height=314.78`, fully within 390 × 844.

Reset generated a usable new sample. Start for real removed all `demo:` browser
keys and made no protected API request. The forced reset/exit race was also run
against live with and without a filled admin-token field: both controls were
disabled during reset, the queued exit completed, every demo request had an
empty authorization header, no protected request occurred, and no demo key
remained. Customer automation redacted email and token; Internal Slack kept the
sample email and redacted token. These are isolated demo requests only.

## Claims and clean checkout

A fresh clone at the implementation SHA was prepared with `npm ci`. Every one
of the 29 exact commands recorded in `.factory/claims.json` passed, including
the browser demo, mobile result, route-policy, isolation, offline, licensing,
privacy, provenance, deployment, redaction, signing, relay, persistence, and
rate-limit commands. `npm run test:claims-manifest` confirmed one tagged test
for each of the 29 entries.

The full gate also passed:

- `npm test`: zero Svelte diagnostics, 25 Rust tests, deployment/manifest
  checks, and 62 Playwright tests all passed.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --locked -- -D warnings` passed.
- `npm run build` produced `dist/` (74.10 KB JavaScript, 27.17 KB gzip; 22.31
  KB CSS, 5.84 KB gzip).
- `cargo build --release --locked` passed.

No declared claim is missing, failed, incomplete, or untested.

## Live runtime, accessibility, and recovery

- `/`, `/demo`, `/privacy`, `/terms`, discovery files, and `/health` returned
  200. The deliberate `/not-a-real-route` returned its designed 404; this is
  expected, not a broken link.
- Route titles are correct: home, Demo, Privacy, Terms, and the 404 each have
  their own title and one h1. A fresh keyboard Tab focuses “Skip to main
  content” first. Sample, legal, and 404 flows emitted no browser console
  errors.
- Axe WCAG 2 A/AA scans on home, demo, Privacy, Terms, and the 404 at 390 px
  reported zero serious or critical violations. The live sample also reloaded
  offline after one visit with the offline-ready label and no failed request.
- Internal links returned 200 except the intentional current-page 404 skip-link
  target. External checkout and source links are explicitly external.
- Live headers include HSTS, `nosniff`, `DENY`, CSP `frame-ancestors 'none'`,
  and appropriate no-store/no-cache policy. Unauthenticated protected endpoints
  reject malformed input before parsing.
- A concurrent 100-request live burst with one fresh forwarded address produced
  87 × 401 and 13 × 429; all 429 responses included `Retry-After: 1`.
- `npm run verify:live-topology` passed: single revision mode; one min/max/running
  replica; `/data` mounted from `alert-evidence-envelope-data`; 20 fresh demo
  previews succeeded. The durable SQLite/restart and tenant isolation behavior
  is additionally covered by the passing Rust claims.

## Earlier-finding disposition

All prior findings were rechecked. The earlier deployment identity, Clippy,
manifest, first-screen/demo, authentication, replica/durable storage, offline,
Slack contract, mobile-result/LCP, optional URL, legal-focus, wording, footer,
404, accessibility/headers, and claim-coverage findings remain closed by the
current source and the checks above. Review findings F-1-1 through F-1-21,
F-2-1 through F-2-12, F-3-1 through F-3-5, and F-4-1 are closed. Review
F-5-1 is closed by the live forced-race check and its passing regression claim.
No earlier minor finding reopened.

## Finding summary

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |
| Untested claims | 0 |

