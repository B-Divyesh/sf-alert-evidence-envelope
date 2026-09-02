# Review 5 handoff — FAIL

Date: 2026-09-02

Work order: `alert-evidence-envelope-review-5`

## Outcome

Reviewed live build `34c18ffe0d3d779a07baf2620969cd89636a7a60` and repository
commit `15b487808b03ccddb62b1ef6992658159cc3d7ac`. No product code or cloud
resource was changed.

The review is **FAIL** with one blocking finding: rapid **Reset demo** then
**Start for real** allows an unfinished demo task to call protected
`/api/v1/preview`. See `.factory/review-5.md` (`F-5-1`) for reproduction,
code path, and required regression test.

## Verified

- Fresh 390 × 844 and desktop first screens are clear, with the one-click
  sample action visible.
- Normal demo behavior works: complete result above fold, normal reset makes a
  new session, normal exit clears demo keys, and normal demo traffic stays on
  the product origin.
- Clean clone `/tmp/aee-review5-1dF5G6`: `npm ci`, all 28 exact claim commands,
  `cargo test --locked`, full Playwright suite, and `npm run build` passed.
- Live light/dark Axe scans found no serious or critical issues. Route,
  metadata, link, privacy, and prior-finding checks passed.

## Next step

Cancel or serialize pending demo work on reset/exit, then add the rapid
reset-and-exit isolation test described in F-5-1. Re-run the complete review
checklist after the repair.
