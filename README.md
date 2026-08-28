# Alert Evidence Envelope

Alert Evidence Envelope is a self-hosted webhook transformer for on-call engineers and automation consumers. It turns an alert into a bounded, recursively redacted evidence excerpt, adds an HMAC signature and query fingerprint, and forwards it to Slack, an email gateway, or a JSON webhook. Responders can identify the service, error signature, and first-seen time without another authenticated dashboard lookup.

It does **not** evaluate alerts, manage incidents, retain raw payloads, or summarize with an LLM.

## How the route works

1. `POST /api/v1/relay/primary` receives vendor-neutral JSON (maximum 256 KB).
2. Evidence is read from a configured JSON pointer, or fetched from one fixed HTTPS source with `q` and `limit` query parameters. The alert cannot select the source host.
3. Key names in the channel redaction policy are recursively replaced with `[REDACTED]`.
4. Item and serialized byte caps are enforced. The envelope exposes whether it was truncated.
5. The source/query fingerprint and envelope HMAC are attached, then the result is forwarded. A supported provider signature header is passed to the destination as `x-original-provider-signature`.

SQLite stores channel configuration and delivery metadata. It never stores the inbound body or evidence. Upstream/destination bearer credentials and the envelope signing key are environment-only.

## Run locally

Requirements: Node 22+, Rust 1.98+, and SQLite support.

```sh
npm ci
npm run build
ENVELOPE_SIGNING_KEY='replace-with-at-least-32-random-bytes' cargo run
```

Open `http://localhost:8080`. For split frontend/backend development, run `cargo run` and `npm run dev` in separate terminals; Vite proxies API requests to port 8080.

### Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP listener |
| `DATABASE_URL` | `sqlite:data/envelopes.db?mode=rwc` | Local metadata/config database |
| `STATIC_DIR` | `dist` | Built frontend directory |
| `ENVELOPE_SIGNING_KEY` | unset | Optional 32-byte-minimum HMAC-SHA256 key override; otherwise a CSPRNG key is generated once and persisted locally |
| `SIGNING_KEY_FILE` | `data/envelope-signing.key` | Persistent generated-key path (container default: `/data/envelope-signing.key`) |
| `ADMIN_TOKEN` | unset | Optional bearer token protecting config, preview, and history routes |
| `INBOUND_TOKEN` | unset | Optional `x-envelope-token` required on incoming relay requests |
| `UPSTREAM_BEARER_TOKEN` | unset | Short-lived bearer token for the fixed evidence source |
| `DESTINATION_URL` | unset | Environment override for the channel destination URL |
| `DESTINATION_BEARER_TOKEN` | unset | Optional destination bearer token |
| `RUST_LOG` | service/tower info | Structured log filter |

If `ADMIN_TOKEN` is enabled, enter it in the route builder; it remains in memory only. In automation, send `Authorization: Bearer …` to config/history/preview. Set `INBOUND_TOKEN` when the alert provider can send a custom `x-envelope-token` header, and use an ingress allowlist as an additional production boundary.

## Alert shape

Every relevant location is configurable with RFC 6901 JSON pointers. The default accepts:

```json
{
  "service": "checkout-api",
  "error": "payment authorization timed out",
  "startsAt": "2026-08-27T14:32:08Z",
  "query": "service=checkout-api level=error",
  "evidence": [
    { "timestamp": "2026-08-27T14:31:41Z", "message": "gateway timeout", "email": "redact-me@example.com" }
  ]
}
```

```sh
curl -fsS http://localhost:8080/api/v1/relay/primary \
  -H 'content-type: application/json' \
  -H 'x-signature: original-provider-signature' \
  --data @alert.json
```

With no destination configured, the relay returns the signed envelope to the caller and records metadata as `created`. This makes the free self-hosted core useful without an outbound service.

To verify a signature, save its `hmac-sha256=…` value, set the envelope's `signature` field to an empty string, serialize the fields in schema order as compact UTF-8 JSON, and compare an HMAC-SHA256 using `ENVELOPE_SIGNING_KEY` in constant time.

## Test and build

```sh
npm test          # type check, Rust tests, build, desktop + 390 px browser/axe tests
npm run build     # reproducible frontend output in dist/
cargo test --locked
```

Playwright is pinned to 1.58.2 and uses Chromium. The Rust suite covers recursive redaction, bounds, endpoint validation, and an HTTP workflow across health/config/preview/relay/history. Browser tests cover the preview path, legal routes, semantics, console errors, mobile layout, and serious/critical axe findings.

Load smoke for a running release:

```sh
seq 1 500 | xargs -P 20 -I{} curl -fsS -o /dev/null http://localhost:8080/health
```

This is a 100+ requests/second health-route smoke on ordinary development hardware, not a capacity claim. Load-test configured upstreams and destinations separately without real incident data.

## Container deployment

```sh
docker build --build-arg BUILD_SHA="$(git rev-parse HEAD)" -t alert-evidence-envelope .
docker run --read-only --tmpfs /tmp -p 8080:8080 \
  -v envelope-data:/data \
  alert-evidence-envelope
```

The multi-stage image runs as a non-root user and serves the Vite build and Axum API from one process. It starts with only `PORT`: on first boot it creates a random 256-bit signing key in `/data`, retains it for stable envelope verification, and logs only whether the key was generated, persisted, or supplied. `ENVELOPE_SIGNING_KEY` remains an optional 32-byte-minimum override. Pass the immutable 40-character commit with `--build-arg BUILD_SHA=<commit>`; it is compiled into `/health` as `build` and recorded in the image revision label. A local build without that argument reports the explicit `development` identity—never an empty value. The factory owns production deployment, DNS, TLS, and billing registration.

## Paid Field Kit

The free core includes relay, redaction, size caps, signing, preview, export, and delivery. The optional $39 one-time Field Kit stores unlimited named redaction policies locally. Checkout and license verification use the Sociobot billing API; no payment provider is embedded and no provider product ID is hardcoded.

See the deployed [Privacy](https://alert-evidence-envelope.sociobot.in/privacy) and [Terms](https://alert-evidence-envelope.sociobot.in/terms), the visual rationale in [`.factory/design.md`](.factory/design.md), and release verification in [`.factory/handoff.md`](.factory/handoff.md).

## License

MIT. Generated cartography is original to this product. Self-hosted Inter and Fraunces font files are distributed under the SIL Open Font License; see `THIRD_PARTY_NOTICES.md`.
