# Review 3 handoff — FAIL

Date: 2026-09-02
Work order: `alert-evidence-envelope-review-3`

## What was done

- Performed a cold, live review of `https://alert-evidence-envelope.sociobot.in`
  at 390 × 844 and 1440 × 900 without modifying product code.
- Rechecked the one-click isolated demo, reset/exit behavior, request privacy,
  offline claim, metadata, routes, link targets, browser console, and light/
  dark live axe scans.
- Cloned the reviewed tree cleanly to `/tmp/aee-review3-1o3dII`, installed
  dependencies, ran `npm test`, `npm run build`, and every exact command in
  `.factory/claims.json`. All quality and registered-claim commands passed.
- Wrote the full findings, copy audit, claim table, and historical-finding
  audit in `.factory/review-3.md`.

## Known gaps

The review verdict is **FAIL**. Product code remains unchanged. The blocking
items are F-3-1/F-1-16 (focus and route announcement are lost when navigating
to or back from static legal pages) and F-3-2/F-1-3 (Terms makes a broader
unlisted free-feature promise). Three minor copy/recovery findings are also
open: F-3-3 through F-3-5.

## Next steps

Implement the concrete fixes in `.factory/review-3.md`, add the legal-route
focus/back regression test and any required claim coverage, then rerun the
full clean-clone claim list and live review checks.

---

# Verification 12 handoff — PASS

Candidate `36fc44438dd299a142ce5fe30fd1a8676e539877` at
<https://alert-evidence-envelope.sociobot.in> **PASSes independent QA**.
The live health build and JS/CSS/service-worker checksums match the candidate.
The one-click demo, signing/redaction routes, isolated storage, offline PWA
reload, request privacy, headers, rate limiter, keyboard/mobile behavior, and
live axe scans passed. The observed API allowance is a 40-request burst plus
20 requests/second refill; a 100-request live burst returned 55 `429`s with
`Retry-After: 1`.

Run `npm ci && npm test && npm run build`; open `/demo` for the no-setup
acceptance path. One healthy non-root replica persists SQLite state at `/data`.
No known P0–P3 defects. Complete evidence: `.factory/verification-12.md`.

---

# Historical Polish 2 handoff — Alert Evidence Envelope

Date: 2026-09-02  
Work order: `alert-evidence-envelope-polish-2`  
Repair commits: `43f47000fe5ef77ec4e1a9414476314ab575ab82` and `a2234a17b5ac0f7c44a225556ba81c9ae7c70dba`

## Delivered

- Repaired every cumulative F-1 and F-2 review finding; the exact mapping is in `.factory/polish-2.md`.
- Made `/demo` an isolated, one-click two-route comparison: Internal Slack redacts `token`; Customer automation redacts `email` and `token`.
- Made the mobile completion result fully visible at 390 × 844, including `[REDACTED]`, with a full bounding-box regression assertion and screenshot artifact.
- Added real JSON, Slack, and email-webhook delivery contracts and local capture-server coverage.
- Tightened free-core, credential-exposure, 24-hour throttle, rate-limit, legal chrome, metadata, wording, and feedback behavior.

## Verification

- `npm test` — pass: Svelte check, 25 Rust tests, deployment policy, 28-claim manifest, and 56 Playwright desktop/mobile tests.
- `cargo fmt --check` — pass.
- `cargo clippy --all-targets --locked -- -D warnings` — pass.
- `npm run build` — pass; initial JavaScript gzip is 26.42 KB and CSS gzip is 5.49 KB.
- Fresh clone `/tmp/aee-clean-OGLsmX` at `43f47000fe5ef77ec4e1a9414476314ab575ab82`, and the final pushed tree at `/tmp/aee-final-clean-6a4Vtt`: `npm ci`, then every exact command in all 28 `.factory/claims.json` entries — pass.
- Browser coverage includes keyboard, route focus/back behavior, 390 px layout and touch targets, dark/light axe scans, offline reload, request privacy, legal pages, 404, security headers, and demo isolation.

## Deploy and live recheck

- Deployed `sociobotregistry.azurecr.io/sf-alert-evidence-envelope:a2234a17b5ac` on `sf-alert-evidence-envelope--0000030`.
- `npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot a2234a17b5ac0f7c44a225556ba81c9ae7c70dba alert-evidence-envelope-data` — pass: single active/healthy replica, `/data` Azure File mount, 20 fresh demo previews, live `/health` build `a2234a17b5ac0f7c44a225556ba81c9ae7c70dba`.
- Cold `/opt/fleet/lib/verify-url.sh` evidence is in `/tmp/aee-live-final-YRIRsl`: 200 home page; title, `lang=en`, one h1, main landmark, image alt text, and no console errors. It wrote desktop/mobile screenshots and `verify.json`.
- Cold URL checks: `/`, `/demo`, `/privacy`, and `/terms` returned 200; `/not-a-real-route` returned 404.
- Live Playwright axe scans at 390 × 844 found zero serious/critical violations on home, demo, privacy, terms, and 404. Both legal headers expose Home, Demo, Privacy, and Terms, with the current page marked.
- Live mobile demo check passed: all required completed-result boxes ended within the 844 px viewport; Internal Slack route showed token-only redaction while retaining the sample email; no console errors.
- `npx @axe-core/cli` could not locate its Selenium Chrome binary in this worker. The equivalent Playwright AxeBuilder scan used the installed Playwright browser and passed all five live routes.

## Known gaps

None known locally. Runtime state remains SQLite under `/data`; the product requires no environment variables beyond `PORT`.
