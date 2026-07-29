#!/usr/bin/env bash
set -euo pipefail
umask 077

backup_dir="${BACKUP_DIR:-backups}"
mkdir -p "$backup_dir"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
out="$backup_dir/slimlytics-${stamp}.sql.gz"
tmp="$(mktemp "$backup_dir/.slimlytics-${stamp}.XXXXXX.sql.gz")"
cleanup() { rm -f "$tmp"; }
trap cleanup EXIT

db_user="$(docker compose exec -T db sh -ec 'printf %s "$POSTGRES_USER"')"
db_name="$(docker compose exec -T db sh -ec 'printf %s "$POSTGRES_DB"')"

docker compose exec -T db pg_dump \
  --username "$db_user" \
  --dbname "$db_name" \
  --clean --if-exists --no-owner --no-privileges \
  | gzip -9 > "$tmp"

gzip -t "$tmp"
mv "$tmp" "$out"
trap - EXIT
printf 'Backup written and gzip-verified: %s\n' "$out"
