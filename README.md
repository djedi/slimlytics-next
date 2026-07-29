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
- Optional anti-adblock installation code with neutral script and collection paths
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

The dashboard generates the correct site-specific snippet. A basic installation looks like:

```html
<script
  async
  src="https://analytics.example.com/tracker.js"
  data-write-key="YOUR_SITE_WRITE_KEY"
  data-endpoint="https://analytics.example.com/api/collect">
</script>
```

For best reliability, proxy the tracker and collection endpoint through the site’s own domain. See `docs/FIRST_PARTY_PROXY.md`.

The tracker never captures form values. Cookieless tracking, Do Not Track, and Global Privacy Control behavior are documented in `docs/PRIVACY.md`.

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
- `docs/PRIVACY.md`
- `docs/PERFORMANCE.md`
- `docs/FIRST_PARTY_PROXY.md`
- `docs/MIGRATION.md`
- `docs/OPERATIONS.md`

## Project status

The initial release implements the Version 1 scope. Funnels, retention cohorts, heatmaps, uptime monitoring, white-labeling, and a plugin marketplace remain intentional later features—not suspiciously ambitious Tuesday-afternoon side quests.
