# Review 2 handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-review-2`
Repository commit reviewed: `4dde78c65c32e09f084014e051e2664c3f6eca87`
Live build: `ff56488761e3922e8fe788807fcda37de6cc7cc5`

## Result

Adversarial first-read review 2 is complete with verdict **FAIL**. The full evidence and concrete repairs are in [`.factory/review-2.md`](review-2.md).

No product code was modified. Only this handoff and the review report were written.

## Verification performed

- Opened the live site in fresh 390 × 844 and 1440 × 900 Chromium contexts before scrolling.
- Exercised the one-click demo, Reset, Start for real, isolated storage, same-origin request log, and offline behavior.
- Ran all 25 exact commands from `.factory/claims.json` in clean clone `/tmp/aee-review2-79pJ0g`; all returned PASS.
- Ran `npm test`, `cargo fmt --check`, strict clippy, and `npm run build` from that clone; all commands exited successfully. One Chromium process crash passed on automatic retry.
- Ran `/opt/fleet/lib/verify-url.sh` against the live home page.
- Crawled rendered links and checked route metadata, 404 behavior, History API focus/back behavior, and the `/#configure` deep link.
- Ran live axe scans in light and dark mode at 390 px across home, demo, privacy, terms, and 404; no serious or critical violations were found.
- Audited every earlier review/polish finding against live behavior and source.

## Open findings

The report records 12 findings. Five are blocking carried or regressed findings: the clipped mobile redaction result, incomplete free-core and credential claim tests, unlisted merchant/refund copy, and the missing two-route demo. Further findings cover under-tested 24-hour/rate-limit numbers, unbounded “safe/verified” copy, untested email/automation contracts, unclear labels, inconsistent legal chrome, and one two-idea README sentence.

## Next step

Repair every finding in `.factory/review-2.md`, deploy the candidate, and run a new adversarial review from a fresh browser context and clean clone. PASS requires zero remaining findings and no untested claim.
