# Verification 2 — FAIL

Date: 2026-08-28  
Work order: `alert-evidence-envelope-verify-2`  
Candidate: `9be7af58e4dd580ca10d9adad860ff81e2d8aa66`  
Live URL: `https://alert-evidence-envelope.sociobot.in`

## Decision

**FAIL — do not accept this live release.** The candidate source is healthy and the live static frontend is byte-identical to its production build, but the deployed backend does not provide its required immutable build identity.

| Severity | Defect | Fresh evidence |
| --- | --- | --- |
| P1 | Live backend identity is empty, so the deployed backend cannot be confirmed as the candidate. | `GET /health` returned `{"build":"","status":"ok"}`. The contract requires a build SHA. A local release binary from the candidate returned `{"build":"9be7af58e4dd580ca10d9adad860ff81e2d8aa66","status":"ok"}`. |

This is a deployment/build-argument failure, not a source failure: the source uses compile-time `BUILD_SHA` and the local production build proves that path. A static asset match cannot establish that the running relay binary, which handles credentials, redaction, signing, and delivery, is the candidate.

## Clean-checkout quality gates

A detached worktree at exactly the candidate SHA, without inherited `node_modules` or output, was used.

- `npm ci` passed: 0 vulnerabilities reported.
- `npm test` passed: Svelte check (0 errors/warnings), `cargo test --locked` (4 tests), Vite production build, and Playwright.
- Independent `npx playwright test --reporter=line` passed **16/16** across desktop Chromium and 390 × 844 Chromium.
- `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings` passed.
- `npm run build` passed. `dist/` is 467,334 B total: JS 63,161 B raw / 24,550 B gzip; CSS 16,300 B raw / 4,720 B gzip; self-hosted fonts 115,560 B; mobile hero 75,882 B. All budgets pass.
- `BUILD_SHA=9be7af58e4dd580ca10d9adad860ff81e2d8aa66 cargo build --release --locked` passed. Docker/Podman/Buildah are absent in this verifier image, so the Docker image itself could not be built locally.
- Lighthouse 13.4.1 (live site, Playwright Chromium): Performance **98**, Accessibility **100**, Best Practices **100**, SEO **100**.

## Independent end-to-end relay evidence

Against the local release binary with a fresh SQLite database, admin/inbound tokens, a fixed signing key, and a capture destination:

- Representative checkout alert returned **202** with `checkout-api`, `card declined 42`, and `2026-08-28T02:15:00Z`—sufficient incident context without dashboard access.
- Nested `email`, `token`, and `cookie` became `[REDACTED]`; independent HMAC verification passed; the original provider signature was forwarded; the destination received no raw email and did receive the envelope signature.
- Missing admin auth returned **401**; missing inbound auth **401**; malformed JSON **400**; a 270 KB request **413**; a 1,023-byte cap **400**; and unsafe `http://evil.example` **400**.
- After 22 relays, history held exactly 20 metadata records and contained neither the seeded email nor evidence message, validating the exercised no-raw-payload retention boundary.
- 200 `/health` requests at concurrency 10 all returned **200**.

## Browser, privacy, PWA, and transport

- Fresh live desktop and 390px mobile runs had one `h1`, one `main`, `lang=en`, correct title, no normal-load console/page errors, and no external requests before a license is present. First Tab focused the visible 3px-outline skip link; mobile width was exactly 390px without overflow.
- Axe WCAG A/AA found zero serious/critical violations in light and dark modes. Reduced motion computed to 0.01s.
- The live invalid-preview recovery path showed “Preview stopped” for `{`, then restored and successfully built the signed envelope with no console errors.
- Live service worker was controlling the page; `registration.update()` succeeded, `envelope-shell-v2` existed, and offline reload rendered the cached shell and “Browser offline”. Chromium logs the expected native `ERR_INTERNET_DISCONNECTED` only while deliberately offline.
- `/`, `/privacy`, and `/terms` are HTTP 200 and legal content works without JavaScript; an unknown route is 404. HTML/service worker use `no-cache`, API/health `no-store`, and hashed assets `public, max-age=31536000, immutable`. CSP, `nosniff`, `no-referrer`, and HTTP-to-HTTPS redirect are present.

## Live candidate comparison

The new candidate build matches live static resources exactly:

- JS `assets/index-VMjeu9dy.js`, 63,161 B, SHA-256 `d51123ff599b17399b4a436a447206eb6f2fee0d436eb5743479a51bf9396c8a`
- CSS `assets/index-DDKuM7sd.css`, 16,300 B, SHA-256 `9dec7e7d0d4fe02707574c49734bae5fe90074257c141015e5a5cd96dee85b41`
- Both responsive hero WebP assets also matched.

The live HTML was last modified `Fri, 28 Aug 2026 01:49:59 GMT`, consistent with candidate commit time `2026-08-28T01:49:31+00:00`. This proves the frontend only. The empty health identity is contradictory backend-provenance evidence and the release blocker.

## Required remediation and recheck

Deploy an image built with a non-empty immutable candidate SHA, then prove:

```sh
curl -fsS https://alert-evidence-envelope.sociobot.in/health
# required: {"build":"9be7af58e4dd580ca10d9adad860ff81e2d8aa66","status":"ok"}
```

Re-run live health and static hashes after deployment. No product-code change is indicated; fix the deployment invocation/build argument that produced the empty compile-time value.
