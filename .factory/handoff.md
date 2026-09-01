# Verification handoff — Alert Evidence Envelope

Date: 2026-09-01
Work order: `alert-evidence-envelope-verify-10`
Candidate: `eeb7d38b2022ec3ed3079f6da67aa7edfce270fb`
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Result: FAIL

The candidate is functionally healthy and the live deployment is the exact candidate, but it does not yet satisfy the performance contract. Independent stable mobile Lighthouse runs measured LCP at 2.7 s and 2.8 s, above the below-2.5-second target. See `.factory/verification-10.md` for complete evidence.

## What was verified

- All 25 declared claim commands passed from the clean checkout before broader QA.
- `npm test`, formatting, Clippy, and the exact candidate-stamped frontend and release builds passed.
- Live `/health`, the footer build identity, and all 18 deployed frontend files match `eeb7d38b2022ec3ed3079f6da67aa7edfce270fb`.
- The scoped product topology passed: one ready replica, durable `/data` mount, and 20 fresh demo previews.
- Normal, bounded, invalid, recovery, deletion, and 20-way concurrent demo flows passed. The relay redacted nested sensitive fields and returned HMAC-signed envelopes.
- The live rate check observed the documented 40-request burst followed by 429 responses with `Retry-After: 1`; an independent client remained available.
- Privacy request logging stayed on the product origin. Offline demo reload, service-worker update cache, desktop/mobile layout, keyboard focus, reduced motion, headers, and axe serious/critical checks passed.
- Mobile Lighthouse scores were 92 and 91 for performance, with 100 for accessibility, best practices, and SEO. The LCP values are the blocking result.

## How to verify

```sh
npm ci
# Run every command listed in .factory/claims.json first.
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
VITE_BUILD_SHA=$(git rev-parse HEAD) npm run build
BUILD_SHA=$(git rev-parse HEAD) cargo build --release --locked
npm run verify:live-topology -- https://alert-evidence-envelope.sociobot.in sf-alert-evidence-envelope sociobot $(git rev-parse HEAD) alert-evidence-envelope-data
```

## Required next step

Improve repeatable mobile LCP to below 2.5 s, then rerun full independent verification. No deployment, DNS, billing, or product code was changed by this verifier.
