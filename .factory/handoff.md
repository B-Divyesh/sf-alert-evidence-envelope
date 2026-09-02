# Verification 14 handoff — PASS

Date: 2026-09-02

Work order: `alert-evidence-envelope-verify-14`

Candidate: `f4bf8ae31eb1c8be548508341d75d7fed251977c`

URL: <https://alert-evidence-envelope.sociobot.in>

## Decision

**PASS.** Independent QA found no P0, P1, P2, or P3 defects. The live service
matches the candidate, the prior blank optional-URL defect is repaired, all 28
claims pass, and the researched alert-to-evidence-to-destination job works end
to end.

Full evidence is in [`.factory/verification-14.md`](verification-14.md).

## Verification summary

- Clean `npm ci`: 56 packages, 0 vulnerabilities.
- `npm test`: PASS — 25 Rust tests and 60 desktop/mobile browser cases.
- All 28 `.factory/claims.json` commands: PASS individually after install.
- Svelte check: 0 errors/warnings; Rust format and strict Clippy: PASS.
- Candidate-stamped Vite and optimized Rust builds: PASS; `dist/` produced.
- All 18 candidate frontend files match live bytes by SHA-256.
- `/health` reports the full candidate SHA.
- Scoped topology: revision `sf-alert-evidence-envelope--0000035`, one running
  replica, product-owned durable storage mounted at `/data`.
- Cold first-read, one-click demo, 20/20 seeded-alert success measure,
  boundaries, invalid recovery, 20-way concurrency, persistence, three
  destination formats, signing, redaction, and raw-data non-retention: PASS.
- Live rate limit: 40-request burst, 20/second refill, then 429 with
  `Retry-After: 1`; separate clients remain isolated.
- Desktop and 390 px mobile, keyboard, visible focus, 200% text suite,
  reduced motion, dark mode, offline reload, service-worker update, privacy
  request log, headers, caching, links, and 10 axe audits: PASS.
- Lighthouse mobile: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.7 s, TBT 150 ms, CLS 0, transfer 136,199 bytes.

No product code, infrastructure, DNS, billing, or external product resource was
modified. Docker-compatible tooling was unavailable locally; the exact build
stages passed directly and the matching live scoped image was verified.

## Known gaps and next steps

No release-blocking or lower-severity product gaps were found. Release the
candidate without product changes.
