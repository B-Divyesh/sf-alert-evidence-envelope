# Verification handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-9`
Candidate: `e47c898b7d069a182943390b07f1c3459e4c7673`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result

**FAIL — do not release this candidate.** Full evidence is in `.factory/verification-9.md`. No product code was changed.

The advertised Slack destination sends summary text only. Its request body has no bounded evidence excerpt and no signature field, while the automation destination sends the complete redacted, signed envelope. This misses the researched brief’s core Slack handoff requirement. Slack delivery is also absent from the required claim inventory.

## What passed

- All 24 commands in `.factory/claims.json` passed after `npm ci`.
- The cold first screen clearly states the job, audience, first action, and one-click sample outcome.
- `npm test`, formatting, strict Clippy, candidate-stamped frontend build, and optimized backend build passed.
- The release binary starts with only `PORT`, generates protected credentials, and returns the full build SHA.
- The live deployment reports the candidate SHA; all 18 production files match local hashes.
- Scoped topology passed for revision `sf-alert-evidence-envelope--0000024`, one replica, and `/data`.
- Normal, boundary, invalid-input recovery, offline reload, service-worker update, concurrency, and rate-limit checks passed.
- The brief’s seeded-alert measure passed 20/20.
- Live demo traffic stayed on the product origin. Response headers and cache policies matched the documented design.
- Live desktop, 390 px, keyboard, focus, reduced-motion, 200% text, and axe checks passed.
- Lighthouse mobile: 97 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 2.2 s and CLS 0.
- JavaScript, CSS, fonts, and mobile hero remain within the supplied budgets.

## Release blocker

Fix Slack delivery so the Slack payload contains the bounded, redacted evidence and a verifiable signature. Register that capability in `.factory/claims.json` with an end-to-end destination-capture check.

## Verification commands

```sh
npm ci
# Run each command in .factory/claims.json exactly.
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=e47c898b7d069a182943390b07f1c3459e4c7673 npm run build
BUILD_SHA=e47c898b7d069a182943390b07f1c3459e4c7673 cargo build --release --locked
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot e47c898b7d069a182943390b07f1c3459e4c7673 alert-evidence-envelope-data
```

## Known verification limits

Docker, Podman, and Buildah are unavailable in this worker, so no local image build ran. The exact production build, matching live files, candidate health identity, and scoped live topology were verified independently.
