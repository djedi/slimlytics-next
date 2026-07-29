# Security policy

## Supported versions

Security fixes are applied to the latest released version. Operators should track tagged releases and rebuild promptly.

## Reporting a vulnerability

Do not open a public issue containing an exploitable vulnerability or real credentials. Use GitHub's private vulnerability reporting feature for this repository with reproduction steps, affected versions, impact, and any suggested mitigation. Repository maintainers should acknowledge a complete report within five business days.

## Operator checklist

- Use the production Compose TLS overlay or terminate TLS at another trusted reverse proxy. The base Compose listener is loopback-only by default.
- Use independent high-entropy values for JWT and visitor-hash secrets.
- Do not publish PostgreSQL to the Internet.
- Restrict site origins and rotate write keys after exposure.
- Run containers as non-root with `no-new-privileges`.
- Back up PostgreSQL and test restores.
- Keep Rust, Node, container base images, and PostgreSQL patched.
- Review reverse-proxy trusted-header settings before enabling `TRUST_PROXY`.
- Keep dashboard authentication separate from public collection write keys.

## Secret handling

`.env` is ignored by Git. CI uses disposable test-only values. Logs and error responses must never expose passwords, JWTs, database URLs containing passwords, write keys, or raw collector identifiers.
