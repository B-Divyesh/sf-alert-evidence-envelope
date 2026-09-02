# Polish 4 handoff — complete

Date: 2 September 2026

Work order: `alert-evidence-envelope-polish-4`

Product: <https://alert-evidence-envelope.sociobot.in>

Released candidate repaired: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

Repair commit: `14f8cf03eddd457ed1a837e9cbfaffed238b5147`

Deployed revision: `sf-alert-evidence-envelope--0000036`

## Outcome

All findings in reviews 1–4 are closed. The round-4 mobile demo blocker is
fixed on the deployed site: the full result card is visible without scrolling
at 390 × 844, including signed and redacted state, service, error, first-seen
time, item count, byte count, truncation state, fingerprint, and result
actions. The isolated demo, production routes, legal routes, 404, offline
fallback, privacy behavior, and distinct topographic visual system remain
intact.

## What changed

- Compacted only the phone demo: shorter banner rhythm, tighter route heading,
  two-column summary rows, three-column bounds, and paired result actions.
- Marked the whole completed result and each required field for direct geometry
  assertions.
- Strengthened `@claim:mobile-demo-result` to use the real mobile project and
  assert 390 × 844, scroll y=0, every card edge, exact first-seen and bound
  values, the fingerprint, and each field's bottom edge.
- Clarified the short demo introduction without changing the landing headline,
  audience sentence, one-click action, or three facts.
- Updated `.factory/claims.json`, `.factory/catalog-description.txt`, the copy
  audit, visual thesis, and cumulative finding map.

## Exact mobile evidence

Cold live `/?demo=1` at 390 × 844 and scroll y=0:

| Element | Top | Bottom |
| --- | ---: | ---: |
| Complete result card | 286.69 | 601.47 |
| First seen | 457.30 | 477.89 |
| Item / byte / truncation row | 481.89 | 518.69 |
| Query fingerprint | 522.69 | 535.88 |

The card ends 242.53 px above the viewport bottom. Evidence files:

- `/tmp/aee-polish4-live/demo-mobile-cold.png`
- `/tmp/aee-polish4-live/live-browser-audit.json`
- `/tmp/aee-polish4-clean-NrxuZj/test-results/app--claim-mobile-demo-res-e69c6-med-envelope-above-the-fold-mobile-chromium/mobile-demo-complete-result.png`

## Verification performed

From clean clone `/tmp/aee-polish4-clean-NrxuZj` at the repair commit:

- `npm ci`: passed, zero audit vulnerabilities.
- `npm test`: passed; 25 Rust tests and 59 browser passes. The only skip is
  the desktop copy of the explicitly mobile-only geometry claim.
- Every one of the 28 exact `.factory/claims.json` commands: passed.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `npm run build`: passed; `dist/` produced, JavaScript 26.73 KB gzip and CSS
  5.78 KB gzip.
- Full log: `/tmp/aee-polish4-clean-14f8cf0.log` ends with
  `ALL_CLEAN_CLONE_CHECKS_PASSED`.

Against the deployed repair commit:

- `/health` returned the full repair SHA and `status: ok`.
- Scoped topology verification passed: single revision, one running replica,
  durable `/data`, and 20 successful fresh-connection demo previews.
- `/opt/fleet/lib/verify-url.sh` passed in 767 ms with no console errors,
  one h1, one main, `lang=en`, complete image alt text, and labelled buttons.
- Playwright Axe scans covered `/`, `/?demo=1`, `/demo`, `/privacy`, `/terms`,
  and the 404 in light and dark mode: zero serious or critical findings in all
  12 runs.
- Route audit confirmed route-specific titles, metadata, one h1, one main,
  focus/announcements, and a real 404 response.
- Link audit confirmed every internal, Privacy, Terms, checkout, source,
  robots, sitemap, and fragment target resolves.
- Demo audit confirmed same-origin traffic only, no protected API access,
  Reset session replacement, Start for real cleanup, and offline reload with
  no API/health request or failed request.
- Lighthouse mobile: performance 92, accessibility 100, best practices 100,
  SEO 100, LCP 2,027 ms, CLS 0, TBT 326 ms.

Live evidence is under `/tmp/aee-polish4-live/`.

## Run and verify

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
npm run build
```

Open <https://alert-evidence-envelope.sociobot.in/?demo=1> for the isolated
sample. Use **Reset demo** for a new ephemeral session and **Start for real**
to clear the demo namespace.

## Known gaps and next steps

None for the reviewed scope. No infrastructure, DNS, billing registration,
shared service, or out-of-scope resource was changed.
