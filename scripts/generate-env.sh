#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -e .env ]]; then
  echo '.env already exists; refusing to overwrite it.' >&2
  exit 1
fi

command -v openssl >/dev/null || { echo 'openssl is required.' >&2; exit 1; }
umask 077

postgres_password="$(openssl rand -hex 24)"
jwt_secret="$(openssl rand -base64 48 | tr -d '\n')"
visitor_secret="$(openssl rand -base64 48 | tr -d '\n')"

POSTGRES_PASSWORD="$postgres_password" \
JWT_SECRET="$jwt_secret" \
VISITOR_HASH_SECRET="$visitor_secret" \
python3 - <<'PY'
import os
from pathlib import Path

template = Path('.env.example').read_text()
template = template.replace('replace-with-a-long-random-password', os.environ['POSTGRES_PASSWORD'])
template = template.replace('replace-with-at-least-32-random-bytes', os.environ['JWT_SECRET'])
template = template.replace('replace-with-a-different-long-random-secret', os.environ['VISITOR_HASH_SECRET'])
Path('.env').write_text(template)
PY

chmod 600 .env
echo 'Created .env with generated secrets and permissions 0600.'
