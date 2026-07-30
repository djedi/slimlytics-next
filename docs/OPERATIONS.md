# Operations

## Health

- `GET /health` proves the API process is running.
- `GET /ready` proves required dependencies, including PostgreSQL, are available.

Monitor readiness and alert when collection stops receiving events for an expected active site.

## Backups

Run:

```bash
./scripts/backup.sh
```

Copy backups off the application host. A backup sitting beside the database is an unusually optimistic disaster-recovery strategy.

Backups are created with owner-only permissions, written to a temporary file, gzip-checked, and atomically renamed only after success. Set `BACKUP_DIR` to place them outside the checkout. Retain at least seven daily and four weekly off-host copies, adjusted for your recovery requirements.

Test restore regularly against a disposable database:

```bash
./scripts/restore.sh backups/slimlytics-TIMESTAMP.sql.gz
```

The restore command requires an explicit `RESTORE` confirmation, makes a mandatory pre-restore backup, stops the API to prevent concurrent writes, and uses one PostgreSQL transaction. It restarts the API and verifies `/ready`; if restore fails, the transaction rolls back and the API is restarted. Keep the printed pre-restore backup as the rollback point.

## Public TLS deployment

The base Compose file binds Caddy to `127.0.0.1:8080`. Use it locally or behind an existing TLS proxy. For direct public hosting, set `SLIMLYTICS_DOMAIN` and `ACME_EMAIL`, point DNS to the server, open ports 80/443, and use:

```bash
docker compose -f compose.yaml -f compose.production.yaml up -d --build
```

Verify DNS first, then check HTTPS, `/health`, `/ready`, registration/login, and one collection request. Caddy certificate state persists in named volumes. Never publish the base plaintext listener on an Internet-facing interface.

When an existing Dockerized Caddy instance terminates TLS, attach Slimlytics' internal Caddy to that proxy's Docker network instead of exposing a public plaintext port:

```bash
PROXY_NETWORK=caddy_default docker compose \
  -f compose.yaml -f compose.proxy.yaml up -d --build
```

Set `SLIMLYTICS_DOMAIN` to the public hostname, then point the external proxy at the stable `slimlytics-upstream:80` network alias. The overlay configures the backend and SvelteKit public origins and enables trusted-proxy handling. Keep `BIND_ADDRESS=127.0.0.1` so the optional host listener remains local-only.

## Upgrades

For the managed `slimlytics.com` deployment, commit and push a clean `main` branch, then run:

```bash
make deploy
```

The deployment script connects through a dedicated `slimdeploy` SSH account, not root. The canonical production directory must contain a tracked `.slimlytics-deployment` sentinel whose content is `slimlytics-production-v1`; both conditions are checked before any recursive synchronization. The deploy user needs source ownership, Docker access, write access to `backups/` and the sibling `.slimlytics-releases/` snapshot directory, and group-read access to the root-owned `.env` (`0640`). Do not weaken the path or sentinel checks to accommodate another deployment layout—set the documented environment overrides and prepare that target deliberately.

The deployment script runs local gates with a lockfile-pinned Redocly CLI, acquires a token-verified remote deployment lock to prevent overlapping releases, creates a verified database backup and retained source snapshot, preserves production `.env`/backups, applies the complete `compose.yaml` + `compose.proxy.yaml` project with health waiting (preserving the external `caddy_default` route), verifies the internal listener and exact public documentation/API artifacts, and automatically restores/reapplies the source snapshot if deployment fails. Its host, path, public URL, and source-snapshot retention are configurable through the `SLIMLYTICS_DEPLOY_*`, `SLIMLYTICS_PUBLIC_URL`, and `SLIMLYTICS_RETAIN_RELEASES` environment variables documented by `scripts/deploy-production.sh --help`.

For an unmanaged installation:

1. Read release notes and migration notes.
2. Create and verify a backup.
3. Pull the desired tag.
4. Build without replacing running containers: `docker compose build`.
5. Apply with `docker compose up -d`.
6. Verify Compose health, `/health`, `/ready`, login, collection, reports, and SSE.
7. Keep the prior images and backup until verification is complete.

## Rollback

The managed deployment script automatically restores its retained source snapshot and rebuilds backend/frontend if a release fails. Database backups are never automatically restored because a database rollback is a separate destructive decision.

For an unmanaged installation, application rollback is `git checkout <previous-tag>` followed by a rebuild. Database rollback requires restoring the pre-upgrade backup whenever a migration is not backward compatible. Never attempt to reverse a destructive migration by improvising SQL against production.

## Retention

Configure retention per site. The backend deletes expired raw events in bounded batches and should retain daily aggregates when configured. Monitor retention failures and PostgreSQL table growth.

## Scaling

The initial deployment uses one backend instance and PostgreSQL notifications for live events. Scale reporting and ingestion separately only after measuring saturation. Redis or NATS can replace the notification fan-out when multiple API instances or high event rates make PostgreSQL notifications insufficient.

## PostgreSQL maintenance

- Monitor database size, connections, replication/backup freshness, and long-running queries.
- Leave autovacuum enabled.
- Partition high-volume event tables before individual partitions become operationally painful.
- Use a least-privilege application role in production.

## Logging

Use structured logs. Do not log passwords, bearer tokens, write keys, complete database URLs, full IP addresses, or unredacted event payloads. Route logs to a system with retention and access controls appropriate to traffic metadata.

Compose limits each container log to three 10 MiB files and applies conservative CPU, memory, and PID limits. Monitor host disk space, PostgreSQL size, Compose health, and backup age; alert before disk usage reaches 80%.
