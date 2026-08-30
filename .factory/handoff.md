# Review 1 handoff — Alert Evidence Envelope

Date: 2026-08-30

Work order: `alert-evidence-envelope-review-1`

## Result

**FAIL.** The complete adversarial report is in `.factory/review-1.md`. Product code was not modified.

Blocking findings:

- The 390 × 844 demo's first viewport shows the sample input but not the transformed envelope result.
- The privacy page says license tokens remain in the browser, while runtime code sends the token to Sociobot in a query URL.

The report also records unlisted public claims, route-focus/404 metadata gaps, plain-word copy issues, and the missing ability to create separate per-channel routes.

## Verification performed

- Used fresh live Chromium contexts at 390 × 844 and 1440 × 900.
- Exercised the live demo, Reset, Start for real, same-origin request logging, and offline reload.
- Ran every exact `.factory/claims.json` command from clean clone `/tmp/aee-review-s2cEaR`; all listed claims passed.
- Ran `npm test` in that clone; 18 Rust tests and 38/38 Playwright tests passed.
- Ran the scoped live topology check only for `sf-alert-evidence-envelope`; it passed with one replica and `/data` storage.
- Crawled internal, checkout, and source links; all intended links reached HTTP 200.
- Ran Playwright axe on `/`, `/demo`, `/privacy`, `/terms`, and the designed 404 at mobile/desktop in light/dark; no serious or critical findings.
- Ran `/opt/fleet/lib/verify-url.sh` against the live root; it passed with zero console errors.

## Files changed

- Added `.factory/review-1.md`.
- Replaced `.factory/handoff.md` with this review handoff.

## Next steps

Resolve every `F-1-*` finding, add the missing claim entries/tests, deploy through the factory, and repeat the entire review from a fresh context. Do not treat the passing registered claim suite as sufficient while public claims remain unlisted.
