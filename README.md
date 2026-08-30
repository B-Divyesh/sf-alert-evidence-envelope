# Alert Evidence Envelope

Send bounded incident evidence with a webhook alert.

This self-hosted transformer is for on-call engineers and webhook consumers. It redacts evidence, applies caps, signs the envelope, and forwards it.

[Try it with sample data](https://alert-evidence-envelope.sociobot.in/demo). The demo runs in an isolated, 24-hour workspace and does not change the protected route.

The relay does not evaluate alerts, manage incidents, retain raw payloads, or summarize with a language model.

## How the route works

1. `POST /api/v1/relay/primary` accepts vendor-neutral JSON up to 256 KB.
2. The relay reads embedded evidence or queries one fixed HTTPS source.
3. It replaces configured sensitive keys with `[REDACTED]`, including nested keys.
4. It enforces item and byte caps before delivery.
5. It adds a query fingerprint and HMAC-SHA256 signature.
6. It forwards supported provider signatures as `x-original-provider-signature`.

SQLite stores route settings and the latest 20 delivery metadata rows. It does not store inbound bodies or evidence excerpts.

## Run locally

Requirements: Node 22+, Rust 1.98+, and SQLite support.

```sh
npm ci
npm run build
PORT=8080 cargo run
```

Open `http://localhost:8080`. The first boot creates three protected files:

- `data/envelope-signing.key` signs envelopes.
- `data/admin.token` authorizes route settings, preview, and history.
- `data/inbound.token` authorizes incoming alert traffic.

Each file has mode 600. Environment variables can supply values instead.

Enter the admin token in the route builder before loading or saving settings. Alert providers must send `x-envelope-token` with the inbound token.

```sh
curl -fsS http://localhost:8080/api/v1/relay/primary \
  -H 'content-type: application/json' \
  -H "x-envelope-token: $(<data/inbound.token)" \
  -H 'x-signature: original-provider-signature' \
  --data @alert.json
```

With no destination, the relay returns the signed envelope and records delivery metadata.

### Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP listener |
| `DATABASE_URL` | `sqlite:data/envelopes.db?mode=rwc` | Live metadata and route database |
| `DATABASE_SNAPSHOT_FILE` | unset | Durable snapshot path for mounted storage |
| `STATIC_DIR` | `dist` | Built frontend directory |
| `ENVELOPE_SIGNING_KEY` | generated | Optional HMAC key override of at least 32 bytes |
| `SIGNING_KEY_FILE` | `data/envelope-signing.key` | Persisted generated signing key |
| `ADMIN_TOKEN` | generated | Optional admin token override of at least 32 characters |
| `ADMIN_TOKEN_FILE` | `data/admin.token` | Persisted generated admin token |
| `INBOUND_TOKEN` | generated | Optional inbound token override of at least 32 characters |
| `INBOUND_TOKEN_FILE` | `data/inbound.token` | Persisted generated inbound token |
| `UPSTREAM_BEARER_TOKEN` | unset | Credential for the fixed evidence source |
| `DESTINATION_URL` | unset | Destination URL override |
| `DESTINATION_BEARER_TOKEN` | unset | Destination bearer token |
| `RUST_LOG` | service/tower info | Structured log filter |

The server logs whether each secret was generated, persisted, or supplied. It never logs secret values.

## Request limits

Every `/api/v1` endpoint uses the first `X-Forwarded-For` address as its client key. It falls back to the socket address.

Each client receives a 40-request burst. Capacity refills at 20 requests per second. Rejected requests return 429 with `Retry-After: 1`.

`/health` is exempt so the deployment platform can probe the container.

## Test and build

```sh
npm ci
npm test
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
npm run build
```

`npm test` runs Svelte checks, Rust unit and route tests, deployment-policy checks, a production build, and Playwright 1.58.2.

Browser coverage runs on desktop and 390 px Chromium. It covers keyboard access, axe, legal pages, demo isolation, privacy, offline reload, metadata, security headers, and rate limits.

Every product claim and its sandbox command is listed in [`.factory/claims.json`](.factory/claims.json). Demo details are in [`.factory/demo.md`](.factory/demo.md).

## Container deployment

```sh
npm run deploy
npm run verify:live-topology
```

The deployment command builds in the factory registry. It then mounts a product-specific Azure File share at `/data`, selects single-revision mode, and fixes scaling at one replica.

One replica prevents SQLite state and in-process rate limits from splitting across workers. The container runs SQLite on its local filesystem and atomically snapshots each committed change to the mounted share. This avoids SQLite locking on SMB while retaining settings and metadata across revisions. The share also retains signing identity and access tokens.

The image runs as the non-root `envelope` user. It starts with only `PORT` supplied by the platform.

## Paid Field Kit

The self-hosted relay and every safety control are free. The optional Field Kit is a $39 USD one-time purchase.

It adds named redaction presets stored in this browser. Checkout and license verification use the Sociobot billing API.

Sociobot/Dodo is the merchant of record. A refunded or invalid license removes paid controls without blocking the free relay.

See [Privacy](https://alert-evidence-envelope.sociobot.in/privacy), [Terms](https://alert-evidence-envelope.sociobot.in/terms), and [the visual rationale](.factory/design.md).

## License

MIT. The generated cartography is original to this product. Inter and Fraunces are distributed under the SIL Open Font License.
