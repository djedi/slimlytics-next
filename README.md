# Slimlytics

Slimlytics is a lightweight, privacy-minded, self-hostable web analytics platform with real-time traffic visibility. It combines a Rust/Axum ingestion and reporting API, PostgreSQL, a Svelte dashboard, and a small first-party browser tracker.

This repository is a ground-up rewrite. It does not replace or rewrite the history of the older `djedi/slimlytics` or `djedi/go_slimlytics` repositories.

## Highlights

- Multiple sites with independent write keys, origins, timezones, and retention settings
- 28-day overview cards and comparison metrics
- Real-time Spy activity over Server-Sent Events
- Pages, referrers, countries, devices, campaigns, visitors, and custom events
- Standard UTM attribution and basic goals
- CSV export
- Cookieless tracking by default
- Sensitive query-parameter redaction and no form capture
- First-party tracker delivery to reduce accidental blocking
- Clicky-style first-party proxy setup with per-site paths, Caddy/Nginx/Apache configuration, and test links
- Responsive Svelte interface with light, dark, and system themes
- Docker Compose deployment with PostgreSQL and Caddy

## Architecture

```text
Browser tracker ──POST──▶ Rust/Axum API ──▶ PostgreSQL
                              │
Dashboard ◀── JSON + SSE ─────┘
    ▲
    └──────── Caddy ──────── Internet
```

- `backend/` — Axum, Tokio, SQLx, PostgreSQL
- `frontend/` — Svelte 5, SvelteKit, TypeScript
- `tracker/` — framework-independent TypeScript tracker
- `cli/` — installable Rust CLI for account, site, and tracking setup automation
- `migrations/` — PostgreSQL schema
- `docker/` — production container definitions and reverse proxy
- `scripts/` — verified backup and guarded restore tooling

## Quick start

Requirements: Docker with Compose v2+.

Generate a private `.env` with independent random database, JWT, and visitor-hash secrets:

```bash
./scripts/generate-env.sh
```

Review the generated URLs and deployment settings before starting. The script refuses to overwrite an existing `.env`.

Then start the stack:

```bash
make up
docker compose ps
curl --fail http://localhost:8080/health
```

Open http://localhost:8080, register the first user, create a site, and copy its generated tracking snippet. The default listener is deliberately loopback-only and is suitable for local use or an existing TLS reverse proxy.

For a public VPS where this stack should terminate TLS itself, point DNS at the host, set `SLIMLYTICS_DOMAIN` and `ACME_EMAIL` in `.env`, then run:

```bash
docker compose -f compose.yaml -f compose.production.yaml up -d --build
```

The production overlay publishes ports 80/443, enables automatic HTTPS, persists Caddy certificates, and configures the application origin. Do not expose the base HTTP stack directly to the Internet.

## Local development

Requirements:

- Rust 1.97.1 (pinned in `rust-toolchain.toml`)
- Node.js 24+
- PostgreSQL 16+

```bash
make setup
make test
make check
make build
```

Start PostgreSQL with Docker if desired, then run the API and Svelte dev server separately:

```bash
docker compose up -d db
export DATABASE_URL='postgres://slimlytics:YOUR_PASSWORD@localhost:5432/slimlytics'
cargo run --manifest-path backend/Cargo.toml
npm --prefix frontend run dev
```

## Tracker

The dashboard generates random, editable first-party paths plus the matching Caddy, Nginx, or Apache configuration. After installing that server configuration, the measured site needs only a same-origin snippet:

```html
<script async src="/456bbb63bb86.js"></script>
```

The generated JavaScript path returns the complete tracker initialized for that site's exact beacon path. The generated test links verify both routes without inserting a fake analytics event. See `docs/FIRST_PARTY_PROXY.md`.

The tracker never captures form values. Cookieless tracking, Do Not Track, and Global Privacy Control behavior are documented in `docs/PRIVACY.md`.

## Rust CLI and AI agents

Install the `slimlytics` binary with Rust's Cargo:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/djedi/slimlytics-next/cli-v0.2.0/scripts/install-cli.sh | sh
slimlytics auth login --email you@example.com
```

Login exchanges the short-lived password session for an expiring, revocable personal API token. The credential file is stored in the operating system's private configuration directory with `0600` permissions; automation can instead provide `SLIMLYTICS_TOKEN` without writing a file.

The idempotent command intended for AI agents creates or reuses a domain and returns the complete server configuration, same-origin snippet, and test URLs:

```bash
slimlytics --json site ensure example.com --server caddy
```

The complete public guide is available at `https://slimlytics.com/docs/cli`; `docs/CLI.md` is the repository copy. The interactive Scalar API reference is at `https://slimlytics.com/api/docs`, backed by the downloadable OpenAPI 3.1 document at `https://slimlytics.com/api/openapi.json`.

## Operations

```bash
./scripts/backup.sh
./scripts/restore.sh backups/slimlytics-YYYYMMDDTHHMMSSZ.sql.gz
```

Backups are gzip-verified. Restore requires typing `RESTORE` and runs PostgreSQL with `ON_ERROR_STOP`.

Before upgrading:

1. Run and verify a backup.
2. Pull the desired tagged release.
3. Run `docker compose build`.
4. Run `docker compose up -d`.
5. Verify `/health`, `/ready`, the dashboard, and test ingestion.

Rollback by checking out the previous tag and restoring the pre-upgrade backup if the migration is not backward compatible.

## Security

- Never commit `.env`.
- Use unique 32+ byte values for `JWT_SECRET` and `VISITOR_HASH_SECRET`.
- Use the tested production Compose overlay or another trusted TLS reverse proxy; never expose login or analytics traffic over plaintext HTTP.
- Keep PostgreSQL private; Compose does not publish its port.
- Restrict each site to expected origins.
- Rotate site write keys and application secrets after suspected exposure.

See `SECURITY.md` for reporting and deployment guidance.

## Documentation

- `docs/IMPLEMENTATION_PLAN.md`
- `docs/ARCHITECTURE.md`
- `docs/REQUIREMENTS.md`
- `docs/API.md`
- `docs/openapi.json`
- `docs/PRIVACY.md`
- `docs/PERFORMANCE.md`
- `docs/FIRST_PARTY_PROXY.md`
- `docs/CLI.md`
- `docs/MIGRATION.md`
- `docs/OPERATIONS.md`

## Project status

The initial release implements the Version 1 scope. Funnels, retention cohorts, heatmaps, uptime monitoring, white-labeling, and a plugin marketplace remain intentional later features—not suspiciously ambitious Tuesday-afternoon side quests.
