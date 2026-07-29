# Slimlytics backend

Production-oriented Axum/Tokio/SQLx PostgreSQL API for cookieless, multi-user web analytics.

## Run

```sh
export DATABASE_URL=postgres://postgres:postgres@localhost/slimlytics
export JWT_SECRET='replace-with-at-least-32-random-bytes'
export VISITOR_HASH_SECRET='replace-with-a-different-32-byte-secret'
cargo run
```

Migrations in `../migrations` run on startup. `SLIMLYTICS_BIND` defaults to `0.0.0.0:8080`. Use a trusted reverse proxy that overwrites `X-Forwarded-For`; that value contributes to ephemeral anonymous identifiers and rate limiting. Never log request bodies or the visitor-hash secret.

## API

* `GET /health`, `GET /ready`
* `POST /api/auth/register`, `POST /api/auth/login`
* Bearer protected site CRUD, key rotation, overview, dimension reports (`pages`, `referrers`, `countries`, `devices`, `campaigns`), visitor timelines, custom events, goals, CSV export, and SSE stream.
* `POST /api/collect/{write_key}` requires an exact `Origin` allowlist match. URLs are stored without query strings/fragments; UTM fields are extracted into dedicated columns. IP addresses and user-agent strings are never persisted.

SSE accepts replay position through `Last-Event-ID` or `last_event_id`; it sends IDs, 15-second heartbeats, and a `resync` event if replay exceeds 1,000 rows or a subscriber lags.

## Verification

Unit/HTTP tests do not require PostgreSQL:

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Database-backed tests (use a **disposable** database):

```sh
TEST_DATABASE_URL=postgres://postgres:postgres@localhost/slimlytics_test \
  cargo test --all-targets -- --include-ignored --test-threads=1
```

All SQL values are bound parameters. The only dynamic SQL is the report grouping expression, selected from a closed server-side allowlist.
