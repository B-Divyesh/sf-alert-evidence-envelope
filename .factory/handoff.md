# Verification 17 handoff — PASS

Date: 2026-09-05

## Verdict

**PASS.** Independent QA found zero product findings and zero untested public claims.

- Implementation reviewed: `db73a281fe9cd1911851a239d2b31d9bbcbedff0`
- Documentation build observed live: `08231a06b7efab7bf12ef51a85f55367c132b468`
- Live URL: <https://alert-evidence-envelope.sociobot.in>
- Assigned live revision: `sf-alert-evidence-envelope--0000040`

The health endpoint reports the later documentation SHA. Its diff from the implementation contains only factory documentation, and a fresh build stamped with that SHA matched live HTML, JS, and CSS byte for byte.

## What was verified

- Fresh desktop and phone first screens name the job, audience, and “Try it with sample data” before scrolling.
- The one-click isolated demo produced a populated signed and redacted envelope, retained its sample label, reset successfully, and did not change real route data.
- Edited sample JSON changed the envelope. Invalid JSON sent no request, showed recovery UI, restored the shipped sample, and then rebuilt.
- Every one of 30 exact claim commands passed from a detached clean checkout.
- `npm test`, strict Clippy, formatting, production Vite build, and release Rust build all passed. `npm test` had 25 Rust tests and 64 browser tests.
- Live isolation, demo boundaries, invalid-to-valid recovery, protected API rate limiting with `Retry-After: 1`, local restart persistence, route titles, links, legal pages, 404, keyboard skip link, reduced motion, privacy, and headers were checked.

## How to run and verify

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
npm run build
PORT=8080 cargo run
```

The service needs only `PORT`; it generates protected credentials on first boot and persists SQLite under `/data` when mounted. The one-click demo is available at `/?demo=1` and is documented in `.factory/demo.md`.

## Evidence and known gaps

The detailed report is `.factory/verification-17.md`; external handoff copies are `/work/.evidence/qa-report.md` and `/work/.evidence/qa-result.json`. Screenshots are under `/work/.evidence/verification-17/`.

No product defect or untested claim is known. No deployment, infrastructure, or product-code change was made by this verification.
