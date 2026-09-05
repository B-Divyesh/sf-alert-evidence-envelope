# Add redacted evidence to webhook alerts — strict review 6

Date: 2026-09-05

Work order: `alert-evidence-envelope-review-6`  
Implementation reviewed: `9c741c506d374e71578605ed43593d76f0ab5620`  
Live documentation build: `2f47a5aa2464715e8309921d29fc153f1f8755cd`  
Review baseline: `85dfa02643250f8430adab74002ecf3873112dd4`  
Live URL: <https://alert-evidence-envelope.sociobot.in>  
Live revision: `sf-alert-evidence-envelope--0000039`

## Verdict: FAIL

**FAIL — one high-severity finding and one untested public behavior.** The
default one-click sample, backend, accessibility, privacy, offline, deployment,
and all 29 declared claim commands pass. However, the demo presents an editable
**Sample alert JSON** field and a **Build signed preview** action while always
parsing the built-in constant. Valid edits are silently discarded, and invalid
JSON produces a successful envelope instead of the promised recovery state.

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 1 |
| Medium | 0 |
| Low | 0 |
| Untested public claims | 1 |

## First screen before scrolling

Fresh Chromium contexts with no cookies, storage, or service worker were opened
at 1440 × 900 and 390 × 844. Before scrolling, both answered:

| Question | Answer shown |
| --- | --- |
| Job | **Add redacted evidence to webhook alerts** |
| Audience | On-call engineers and webhook consumers who need incident context without another dashboard login. |
| First action | **Try it with sample data**; the adjacent sentence says it opens a signed, redacted envelope in an isolated workspace. |

The action was visible at `y=436.81–487.61` on the phone. The title names the
job: **Alert Evidence Envelope — add evidence to alerts**. Screenshots are
`/work/.evidence/review6-desktop-cold.png` and
`/work/.evidence/review6-phone-cold.png`.

## Finding

### F-6-1 — High: the demo ignores edited alert JSON and reports success

- **Where:** live `/?demo=1`; `frontend/src/App.svelte:316`.
- **Public behavior:** the demo shows an editable field labelled **Sample alert
  JSON** next to **Build signed preview**. The recovery UI also says **Restore
  valid sample**, and product copy says users may paste a realistic alert.
- **Valid-input reproduction:** change the visible sample service from
  `checkout-api` to `edited-review-service`, then select **Build signed
  preview**.
- **Observed:** the textarea retains `edited-review-service`, but the request
  body and completed envelope both still contain `checkout-api`.
- **Invalid-input reproduction:** replace the textarea with `{`, then select
  **Build signed preview**.
- **Observed:** the app sends the built-in valid sample, receives HTTP 200,
  shows “Envelope signed. Demo data was not stored.”, and never shows the
  **Preview stopped** / **Restore valid sample** recovery state.
- **Cause:** `runDemoPreview()` parses `sampleAlert`, the immutable built-in
  constant, instead of the bound `sample` value. This line was introduced in
  implementation `9c741c5` while repairing the reset/exit race.
- **Coverage gap:** all 29 declared commands pass because
  `@claim:demo-envelope` exercises only the unchanged default sample. No claim
  or browser test proves that the visible demo editor controls the envelope.
- **Required repair:** capture the current textarea value together with the
  demo session, route, generation, and abort signal; parse that captured value
  without consulting mutable route state. Add one claim-backed browser test
  that proves a valid custom service reaches the request and result, invalid
  JSON shows the recovery panel without a request, and restoring the sample
  succeeds. Preserve the current reset/exit cancellation test.

This is not a privacy leak: the ignored text was not sent anywhere. It is a
blocking correctness defect because a primary demo control silently produces a
different result than the input shown to the user.

## One-click sample and sandbox

The unchanged default path passes:

- One click opened `/?demo=1` and retained the persistent **Demo — sample data,
  nothing is saved** label with **Reset demo** and **Start for real**.
- The populated result showed `checkout-api`, “payment authorization timed
  out”, first seen `8/27/2026, 2:32:08 PM`, 2 items, 213 B, no truncation,
  fingerprint `b44f90d5c75de84a`, recursive redaction, and an HMAC signature.
- The full phone result occupied `x=12–378`, `y=286.69–601.47`, inside the
  initial 390 × 844 viewport.
- Customer automation removed email and token. Internal Slack retained the
  sample email and removed the token. No protected endpoint was requested.
- Reset replaced the random demo session. Normal exit cleared every `demo:`
  key and made no protected request.
- The review-5 race was forced live with and without a filled invalid admin
  token. Reset and exit were disabled during deletion; a dispatched queued exit
  completed safely. Demo authorization headers stayed empty, no protected API
  was called, and no demo key remained.

No real route, history, or alert data was changed. Live writes were limited to
ephemeral demo sessions, which were deleted. The restart check used a temporary
local SQLite database.

## Claims and clean checkout

A detached clone at the implementation SHA was installed with `npm ci` (56
packages, zero reported vulnerabilities). Every exact `test` command in
`.factory/claims.json` was invoked independently. All 29 passed:

`demo-envelope`, `mobile-demo-result`, `demo-route-policies`,
`bounded-redacted-signed`, `query-fingerprint`, `fixed-query-source`,
`provider-signature`, `slack-delivery`, `isolated-demo`, `demo-expiry`,
`raw-not-retained`, `history-limit`, `protected-real-apis`,
`credential-storage`, `credential-browser-exposure`, `preview-no-history`,
`per-route-isolation`, `no-tracking`, `offline-demo`, `license-transport`,
`local-policy-presets`, `license-throttle`, `free-core`,
`field-kit-purchase`, `license-revocation`, `provenance-license`,
`durable-deployment`, `rate-limit`, and `destination-contracts`.

`npm run test:claims-manifest` confirms one tagged test for every declared
entry. There are zero unrun declared commands. F-6-1 is an additional unlisted
public behavior, so the review records one untested public claim.

The complete clean-checkout gate also passed:

- `npm test`: zero Svelte errors or warnings; 25 Rust tests passed; deployment
  and claim-manifest checks passed; Playwright finished with 60 passed, one
  intentional desktop skip, and one mobile browser-start crash that passed on
  its configured retry.
- The crashed legal-page accessibility case was rerun independently three
  times without retry; all three passed. Its stack trace was a Chromium
  `SIGSEGV` before a browser context opened, not an application failure.
- `cargo fmt --check` passed.
- `cargo clippy --all-targets --locked -- -D warnings` passed.
- `npm run build` produced `dist/`.
- `BUILD_SHA=9c741c5… cargo build --release --locked` passed.
- `git diff --check` passed; the detached implementation checkout remained
  clean.

The production frontend is 74,122 B JavaScript (27.17 KB gzip), 22,318 B CSS
(5.84 KB gzip), 115,560 B of self-hosted fonts, and a 40,982 B mobile hero.
These remain within the supplied budgets.

## Backend, boundaries, recovery, and persistence

- `/health` returns 200 and build
  `2f47a5aa2464715e8309921d29fc153f1f8755cd`.
- Unauthenticated malformed config, history, config-update, preview, and relay
  requests returned 401 before parsing.
- Live demo boundaries returned 200 at 1/100 records and 1,024/131,072 bytes;
  they returned useful 400 responses at 0/101 records, 1,023/131,073 bytes,
  and for malformed JSON. A valid request immediately after each invalid case
  succeeded.
- Two live demo tenants received different identifiers. Deleting tenant A made
  A return 404 while tenant B still returned its own `review-api` result. A
  made-up tenant returned 404.
- A local release process generated protected credentials at mode 0600. A
  route saved to temporary SQLite remained present after graceful stop and
  restart; `/health` retained the implementation SHA. Startup logs did not
  contain the generated credential value.
- A fresh 100-request live burst from one forwarded client produced 42 × 401
  and 58 × 429. Every 429 had `Retry-After: 1`; a different forwarded client
  immediately received the normal 401.
- `npm run verify:live-topology` passed: single-revision mode, min/max/running
  replicas all 1, and `alert-evidence-envelope-data` mounted at `/data`.
  Twenty fresh create/preview pairs passed.

The local integration claims also exercised fixed-source query fetching,
recursive redaction, item and byte truncation, independent HMAC recomputation,
provider-signature forwarding, JSON/Slack/email delivery contracts, 20-row
metadata history, and absence of raw payload markers in closed SQLite bytes.

## Accessibility, privacy, offline, links, and performance

- Twenty live Axe WCAG 2 A/AA scans covered home, demo, Privacy, Terms, and the
  designed 404 in desktop/phone, light/dark, and reduced-motion contexts. They
  found zero serious or critical violations.
- Every audited page had `lang=en`, one `h1`, `main`, header, nav, footer, the
  correct route title, and no horizontal overflow. First Tab focused **Skip to
  main content** with a solid outline. In-app Demo navigation focused its h1
  and announced **Demo — Alert Evidence Envelope**.
- Reduced-motion matched and no running animation remained. Local tests also
  passed 200% text resizing, 44 × 44 px targets, keyboard scrolling of expanded
  JSON, and route focus/back behavior.
- The normal sample flow contacted only the product origin. It loaded no
  analytics, advertising, third-party script, or hosted font.
- After one online visit, a fresh phone context updated service worker cache
  `envelope-shell-v6`, reloaded offline, showed the offline-ready label and
  sample result, and made no failed, API, or health request.
- All internal links returned 200 except the intentional current-page skip
  target on the 404. The source returned 200. The official checkout returned
  the expected 303 redirect. Discovery files and social/icon assets returned
  200.
- The deliberate `/not-a-real-route` returned HTTP 404 with the designed page,
  correct title, landmarks, and recovery links. Chromium's failed-resource
  line for that requested document is expected and is not a defect.
- HSTS, `nosniff`, `DENY`, `no-referrer`, response-header CSP with
  `frame-ancestors 'none'`, and the intended cache policies are present.
- `/opt/fleet/lib/verify-url.sh` passed in 665 ms with no console errors.
- Mobile Lighthouse: performance 96, accessibility 100, best practices 100,
  SEO 100, LCP 1.76 s, CLS 0, and total blocking time 210 ms.

Privacy and Terms plainly describe SQLite, browser storage, demo expiry,
license transport, deletion, operator responsibility, purchase support, and
the free core. No sign-in exists. AI summarization remains an explicit brief
non-goal; no missed-AI finding is raised.

## Live implementation comparison

The live image and health endpoint carry documentation SHA `2f47a5a`.
`9c741c5..2f47a5a` changes only `.factory/handoff.md`; the later review
baseline `85dfa02` adds only handoff and verification reports. A production
frontend built from implementation `9c741c5` with live build identity
`2f47a5a` matched live `index.html`, JavaScript, and CSS byte for byte:

- HTML SHA-256 `4fa0601a7ffaef0e4591dab745ad11e2104ec9455dde625df364c0c0808fe79b`
- JavaScript SHA-256 `bf86b438bcf5e682f7ad86531083fef02a935eed3d04a0a27f6c33b42b55f32e`
- CSS SHA-256 `04acc2f9d5e116e8225026343099b3636432b14897302a57d37b4d39492ad5f9`

The live runtime is therefore the reviewed implementation with a later
documentation build, not an unreviewed product change.

## Earlier-finding disposition

Every earlier report present in the repository was inspected. The following
closures were re-proved; F-6-1 is a new regression introduced by the final
review-5 repair.

| Earlier finding group | Current evidence |
| --- | --- |
| Verification 1–2: legal HTTP status, mobile overflow, missing/empty build identity, strict Clippy | Privacy and Terms are 200; all audited widths equal their viewports; health has the full documentation SHA; strict Clippy passes. |
| Verification 3: keyboard-inaccessible JSON, dark contrast, missing HSTS | Expanded JSON remains focusable and keyboard-scrollable; all 20 Axe scans are clear; HSTS is present. |
| Verification 4: missing claims/demo/auth/durability; legal contrast; checkout; discovery/404/footer; touch targets; anti-framing; limiter/docs/plain copy | The manifest has 29 commands; default demo and race isolation pass; protected APIs return 401; one replica uses `/data`; legal Axe is clear; checkout is 303; discovery, chrome, 404, 44 px targets, CSP, limiter, demo/copy/design docs all pass. |
| Verification 5: checkout test, durable deployment, cross-replica demo failure, claim/HMAC gaps | Purchase and topology claims pass; 20 fresh live previews pass; HMAC and all current inventory entries have regression coverage. |
| Verification 8: offline state, four omitted claims, cold command timeout, incomplete copy audit, small targets, stale immutable caching | Offline reload is clean; all named claims are listed; first clean claim command passed after documented install; copy audit is complete; target tests pass; only hashed JS/CSS are immutable. |
| Verification 9: Slack omitted the envelope and had no claim | Slack delivery integration and `slack-delivery` claim pass. |
| Verification 10: mobile LCP above 2.5 s | Fresh mobile LCP is 1.76 s with performance 96. |
| Verification 13: blank optional URLs broke route creation | The authenticated browser regression passes with both URLs blank and reloads the new route. |
| Reviews 1–2: F-1-1…F-1-21 and F-2-1…F-2-12 | Current first screen, default mobile result, license transport/throttle, free controls, claim inventory, credentials, titles/focus, 404, wording, route policies, destination contracts, legal chrome, and rate-limit checks pass. |
| Review 3: F-3-1…F-3-5, including minor README grammar, price-fact wording, and 404 destination | Current navigation, narrow free-feature wording, README sentence, split price facts, and both `/#configure` recovery links pass. |
| Review 4: F-4-1 mobile complete-result assertion | The result card and every required field end above 844 px; the strengthened claim passes. |
| Review 5: F-5-1 reset/exit race | Forced live races with and without a filled admin field call no protected endpoint, attach no demo authorization, and leave no demo key. |
| Verification 11–12 and 14–16 | These reports declared no open findings. Their passing areas were rerun as described above. |

One earlier recovery statement does not remain true: verification 15 reported
that invalid demo JSON showed the recovery panel. Implementation `9c741c5`
changed the demo parser to the constant and regressed that path; this is F-6-1.

## Required next step

Repair F-6-1 without weakening the reset/exit isolation guard. Add the missing
claim and exercise valid edit, invalid input, recovery, and the existing race
from a clean demo context. Then rerun all claim commands, the full gate, and
the live phone/desktop demo before requesting another strict review.
