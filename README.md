# Alert Evidence Envelope

Add bounded, redacted, signed evidence to webhook alerts.

For on-call engineers and webhook consumers. It builds an evidence envelope from alert JSON, then delivers it to a configured destination.

[Try it with sample data](https://alert-evidence-envelope.sociobot.in/demo). The isolated sample shows a checkout timeout with redacted values.

## What it does

- Limits evidence by record count and byte size.
- Removes configured sensitive fields, including nested fields.
- Records a fingerprint from the fixed source and alert query.
- Signs the envelope with HMAC-SHA256.
- Sends Slack the readable summary, bounded redacted evidence, and signature in one request body.
- Keeps separate delivery routes with their own inbound URLs, destinations, and redaction lists.

SQLite stores route settings, short-lived demo session IDs, and delivery metadata. It does not store inbound bodies or evidence excerpts.

## Run locally

Requirements: Node 22+, Rust, and SQLite support.

```sh
npm ci
npm run build
PORT=8080 cargo run
```

Open `http://localhost:8080`. First boot creates protected signing, admin, and inbound credentials in `data/` (or `/data` when mounted). Set their corresponding environment variables to supply replacements. Enter the admin token in the route builder; incoming alerts send the inbound token in `x-envelope-token`.

Each `/api/v1` endpoint is rate limited by the first `X-Forwarded-For` address. `/health` remains available for platform probes.

## Verify

```sh
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
npm run build
```

Each public product claim and its repeatable sandbox command is listed in [`.factory/claims.json`](.factory/claims.json). Demo behavior is documented in [`.factory/demo.md`](.factory/demo.md).

## Deploy

```sh
npm run deploy
npm run verify:live-topology
```

The container serves the built frontend and Rust API on `PORT`. Durable SQLite state lives at `/data` when the platform mounts it.

## Field Kit

The optional Field Kit costs $39 USD once and adds named redaction presets stored in this browser. Redaction, signing, previews, copying envelopes, and route safety controls stay available without a license.

License tokens are stored in the browser. Verification sends the token to Sociobot in an authorization header, not in a URL. [Privacy](https://alert-evidence-envelope.sociobot.in/privacy) and [Terms](https://alert-evidence-envelope.sociobot.in/terms) explain storage and purchase terms.

## License and assets

MIT. The cartography was generated for this product on 2026-08-27; prompt metadata is in `assets/src`. Inter and Fraunces notices are in `THIRD_PARTY_NOTICES.md`.
