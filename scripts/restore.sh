#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 backups/slimlytics-YYYYMMDDTHHMMSSZ.sql.gz" >&2
  exit 64
fi

backup="$1"
[[ -f "$backup" ]] || { echo "Backup not found: $backup" >&2; exit 1; }
gzip -t "$backup"

read -r -p "Restore $backup into the Compose database? Type RESTORE: " answer
[[ "$answer" == RESTORE ]] || { echo 'Cancelled.'; exit 1; }

# A successful pre-restore backup is mandatory. If this fails, no services are stopped.
pre_restore_output="$(./scripts/backup.sh)"
printf '%s\n' "$pre_restore_output"

db_user="$(docker compose exec -T db sh -ec 'printf %s "$POSTGRES_USER"')"
db_name="$(docker compose exec -T db sh -ec 'printf %s "$POSTGRES_DB"')"

restart_backend() { docker compose start backend >/dev/null 2>&1 || true; }
trap restart_backend EXIT

docker compose stop backend
gzip -dc "$backup" | docker compose exec -T db psql \
  --single-transaction \
  --set ON_ERROR_STOP=on \
  --username "$db_user" \
  --dbname "$db_name"

docker compose start backend
for _ in $(seq 1 30); do
  if docker compose exec -T backend curl --fail --silent http://127.0.0.1:8080/ready >/dev/null; then
    trap - EXIT
    echo 'Restore completed; backend is ready.'
    exit 0
  fi
  sleep 2
done

echo 'Restore completed, but backend readiness failed. Inspect docker compose logs backend.' >&2
exit 1
