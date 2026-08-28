# Verification 3 — FAIL

Date: 2026-08-28  
Work order: `alert-evidence-envelope-verify-3`  
Candidate: `96f81cbfd91c5e976cdd35c413841895271c0161`  
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL — do not accept this candidate.** The previous deployment-provenance failure is repaired: the live runtime identifies itself as exactly the candidate and all checked live static resources are byte-identical to a clean local production build. However, independently exercising the normal signed-preview flow exposes serious accessibility defects which violate the acceptance gate of zero serious/critical axe findings and prevent keyboard-only review of the generated evidence.

| Severity | Defect | Fresh, reproducible evidence |
| --- | --- | --- |
| P1 | The revealed signed JSON evidence is a scrollable `<pre>` that cannot receive keyboard focus. A keyboard-only responder cannot scroll a long evidence envelope to inspect it. | On the live site: Build safe preview → Inspect signed JSON. axe WCAG A/AA reports `scrollable-region-focusable` (serious) for `<pre>`. At 390 px, the element has no `tabindex`, `clientHeight: 260`, `scrollHeight: 720`, and `overflow-y: auto`. |
| P1 | Dark mode has serious contrast failures on active UI states. | On the same live preview, axe reports `color-contrast` (serious): the focused skip link is white on `#dce9df` (1.25:1) and the `SEALED` label is white on `#59c991` (2.06:1); both require 4.5:1. |
| P2 | HTTPS responses do not send `Strict-Transport-Security`. | Fresh `curl -I https://alert-evidence-envelope.sociobot.in/` had no `strict-transport-security` header. HTTP does redirect to HTTPS. This is a defense-in-depth deployment/header-policy gap, not the reason for the FAIL. |

The repository browser suite passes because its axe test audits the initial screen only. It does not audit the successful preview state or the expanded signed-JSON state where these defects occur.

## Clean-checkout quality gates

All source tests ran in a new detached worktree at exactly the candidate SHA, with a fresh `npm ci`.

- `npm ci` passed; npm reported 0 vulnerabilities.
- `npm test` passed: `svelte-check` reported 0 errors/warnings; Rust tests passed **7/7**; Vite production build passed; Playwright passed **16/16** (desktop Chromium and 390 × 844 Chromium).
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `git diff --check` passed.
- Exact release build passed: `BUILD_SHA=96f81cbfd91c5e976cdd35c413841895271c0161 cargo build --release --locked`.
- Docker, Podman, and Buildah are not installed in this verifier image, so an image build was not possible here. The release binary and the deployed runtime were tested instead.
- Lighthouse could not be collected in this container: the supplied Chromium launches Playwright successfully but crashes under Lighthouse's Chrome launcher. This does not supersede the direct axe failures above.

## End-to-end backend evidence

The exact release binary was run against a fresh temporary SQLite database and a local capture destination.

- A representative `checkout-api` alert returned **202**, with `card declined 42` and `2026-08-28T02:15:00Z` in the signed envelope. Recursive `email`, `token`, and `cookie` values were `[REDACTED]`; independent HMAC-SHA256 verification succeeded.
- With a configured local JSON destination, relay result was `delivered`; the capture received service `payments`, redacted email/token, `x-original-provider-signature: original-provider-proof`, and an `x-evidence-envelope-signature` beginning `hmac-sha256=`.
- Access and recovery boundaries: unauthenticated config **401**; missing inbound token **401**; malformed JSON **400**; 270,000-byte inbound body **413**; unsafe `http://evil.example` endpoint **400**; 1,023-byte evidence cap **400**.
- After 22 accepted relays, history contained exactly 20 metadata records. `strings` over the SQLite file found 0 occurrences of the seeded email, evidence message, or error text, supporting the no-raw-payload retention boundary.
- 200 `/health` requests at concurrency 10 all returned **200**.
- With only `PORT` supplied, a fresh release process started, created a 32-byte mode-`600` signing key, logged `signing_key_source="generated"`, and returned the candidate SHA from `/health`.

## Live deployment, privacy, transport, and PWA evidence

- `GET /health` returned `{"build":"96f81cbfd91c5e976cdd35c413841895271c0161","status":"ok"}`. This resolves the prior empty-build deployment failure.
- Local and live SHA-256 hashes matched for `index.html`, JS, CSS, both hero WebPs, and both self-hosted fonts. Key hashes: JS `d51123ff599b17399b4a436a447206eb6f2fee0d436eb5743479a51bf9396c8a`; CSS `9dec7e7d0d4fe02707574c49734bae5fe90074257c141015e5a5cd96dee85b41`.
- Fresh desktop and 390 px live runs have `lang=en`, the expected title, exactly one `h1` and one `main`, descriptive hero alt text, no normal-load console/page errors, no horizontal overflow, and a visible first-Tab skip-link outline. The normal first-load request set contains only the product origin; no analytics, third-party font, or third-party script request occurred.
- The representative browser preview redacts data and recovers correctly. The PWA has an active controller and `envelope-shell-v2`; `registration.update()` succeeded. A fresh 390 px context reloaded offline into the cached shell and displayed “Browser offline” without a page error.
- Live `/`, `/privacy`, and `/terms` return 200; an unknown route returns 404; HTTP redirects to HTTPS. CSP (`default-src 'self'` with the declared billing connect exception), `nosniff`, and `no-referrer` are present. HTML and service worker are `no-cache`; API and health are `no-store`; hashed JS/CSS/fonts are `public, max-age=31536000, immutable`.
- Production payload sizes pass the stated static budgets: initial JS 63,161 B raw / 24,233 B gzip, CSS 16,300 B raw / 4,705 B gzip, self-hosted fonts 115,560 B total, and mobile hero 75,882 B.

## Required remediation and recheck

1. Make the expanded evidence region keyboard reachable and scrollable (for example, a labelled focusable region with an appropriate `tabindex`), then test it with a real long envelope at 390 px.
2. Correct the dark-mode skip-link and `SEALED` foreground/background pairs to at least 4.5:1.
3. Extend the browser/axe suite to build a preview, expand signed JSON, and run axe in light and dark modes, so this regression is gated.
4. Enable HSTS at the HTTPS-serving layer, then re-run live headers and the preview-state accessibility audit.
