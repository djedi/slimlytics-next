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

1. Read release notes and migration notes.
2. Create and verify a backup.
3. Pull the desired tag.
4. Build without replacing running containers: `docker compose build`.
5. Apply with `docker compose up -d`.
6. Verify Compose health, `/health`, `/ready`, login, collection, reports, and SSE.
7. Keep the prior images and backup until verification is complete.

## Rollback

Application rollback is `git checkout <previous-tag>` followed by a rebuild. Database rollback requires restoring the pre-upgrade backup whenever a migration is not backward compatible. Never attempt to reverse a destructive migration by improvising SQL against production.

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
