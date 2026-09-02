# Review 4 handoff — FAIL

Date: 2026-09-02

Work order: `alert-evidence-envelope-review-4`

Reviewed repository: `df58f602b019827a3afe29d1df749f9ef2abcc54`

Live build: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

## What was done

- Audited the live site cold at 390 × 844 and 1440 × 900.
- Exercised the one-click demo, both sample policies, Reset, Start for real,
  offline reload, request isolation, storage cleanup, navigation focus, deep
  links, metadata, the designed 404, and all unique links.
- Read all earlier reviews, polish reports, and the prior handoff; rechecked
  every earlier finding in live behavior and source.
- Audited every landing and README sentence in `.factory/review-4.md`.
- Ran all 28 exact claim commands from clean clone
  `/tmp/aee-review4-clean-4IlcYG`.
- Ran the full suite, production build, Rust format, strict Clippy, the fleet
  URL verifier, and 12 live Axe scans.

No product code, deployment, DNS, billing, or infrastructure was modified.

## Result

**FAIL.** See [`.factory/review-4.md`](review-4.md).

`F-4-1 / F-1-1 / F-2-1` is blocking. The mobile demo claim says the complete
result fits in 390 × 844, but first seen begins below the viewport and the
bounds/fingerprint follow it. The tagged test passes because it checks only
signed status, redaction, service, and error.

## Verification summary

- All 28 exact `.factory/claims.json` commands: command PASS.
- Independent `mobile-demo-result` claim check: FAIL; incomplete test and
  false live geometry.
- `npm test`: exit 0; 25 Rust tests and 60 browser cases. One Chromium crash
  retried successfully.
- `npm run build`: PASS; `dist/` produced; JS 26.65 KB gzip.
- `cargo fmt --check`: PASS.
- Strict Clippy: PASS.
- Fleet URL verifier: PASS; no home console errors.
- Live Axe, six routes × light/dark: zero serious or critical findings.
- Demo Reset changed the ephemeral session; Start for real removed `demo:`
  keys; request logging found no protected endpoint or third-party request.

## Next step

Compact the mobile demo until the complete result ends within 844 px, then
extend `@claim:mobile-demo-result` to assert the entire result container,
first seen, item/byte/truncation fields, and fingerprint. Rerun the claim
manifest and live mobile check before another verdict.
