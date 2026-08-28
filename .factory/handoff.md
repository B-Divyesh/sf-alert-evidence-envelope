# Alert Evidence Envelope — verification 3 handoff

## Release status: FAIL

**Do not accept candidate `96f81cbfd91c5e976cdd35c413841895271c0161` at `https://alert-evidence-envelope.sociobot.in`.**

Fresh evidence confirms the deployment now runs the exact candidate:

```json
{"build":"96f81cbfd91c5e976cdd35c413841895271c0161","status":"ok"}
```

The source quality gates and relay behavior are healthy, but the real successful-preview flow fails the required axe serious/critical gate:

- P1: expanded signed JSON is an unfocusable scrollable `<pre>` (`scrollable-region-focusable`, serious), so keyboard-only responders cannot inspect long evidence.
- P1: dark-mode skip link (1.25:1) and `SEALED` label (2.06:1) fail serious color contrast.
- P2: the HTTPS response lacks `Strict-Transport-Security`; HTTP redirects to HTTPS.

See [verification-3.md](verification-3.md) for exact commands, hashes, response policies, and reproduction steps.

## What was verified

From a clean detached checkout at the candidate SHA:

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
BUILD_SHA=96f81cbfd91c5e976cdd35c413841895271c0161 cargo build --release --locked
```

All passed: 7 Rust tests, 16 desktop/390px Playwright tests, Svelte type checking, formatting, clippy, and the exact release build. The live static assets and fonts hash-identically to the local build, and the live health identity is the candidate SHA.

The exact release binary also passed independent relay checks: HMAC signing, recursive redaction, original-provider signature forwarding, delivery to a capture webhook, unauthorized/malformed/oversize/unsafe-input recovery, 20-record metadata retention with no seeded raw data in SQLite, first-boot CSPRNG key creation, and 200 concurrent health requests.

## Next steps

Fix the two P1 accessibility issues, add a post-preview light/dark axe regression test, configure HSTS, deploy a new immutable candidate, and re-run the live verification. No product-code changes were made during this verification.
