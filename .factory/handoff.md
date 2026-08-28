# Alert Evidence Envelope — verification handoff

## Release status: FAIL

Candidate `9be7af58e4dd580ca10d9adad860ff81e2d8aa66` is locally verified but the live release at `https://alert-evidence-envelope.sociobot.in` is **not accepted**. Fresh `GET /health` evidence is `{"build":"","status":"ok"}`, not the immutable candidate SHA. This is a P1 deployment-provenance defect: the live backend that performs redaction, signing, and delivery cannot be identified as the candidate.

See [verification-2.md](verification-2.md) for exact evidence, commands, and the single blocking defect.

## What was verified

- Clean detached checkout; `npm ci`, `npm test`, strict Rust formatting and Clippy, exact Vite production build, and a release-mode backend compiled with the candidate SHA all passed.
- Independent Playwright passed 16/16 on desktop and 390px mobile; Axe had no serious/critical findings; Lighthouse was 98 Performance / 100 Accessibility / 100 Best Practices / 100 SEO.
- Local relay end to end redacted nested secrets, signed and independently verified the envelope, forwarded provider signatures, enforced auth/caps, retained only 20 metadata records, and handled 200 concurrent health checks.
- Live static JS/CSS and both hero assets are byte-identical to the candidate; legal routes, CSP/caching, HTTPS redirect, PWA update/offline shell, keyboard focus, and invalid-preview recovery all passed.

## Required next step

Redeploy the backend image with a non-empty `BUILD_SHA` equal to the final commit and verify:

```sh
curl -fsS https://alert-evidence-envelope.sociobot.in/health
# {"build":"9be7af58e4dd580ca10d9adad860ff81e2d8aa66","status":"ok"}
```

Then perform a short live provenance recheck. No product source modification was made in this verification. Docker/Podman/Buildah were unavailable in the verifier image, so the container image itself was not locally constructed.
