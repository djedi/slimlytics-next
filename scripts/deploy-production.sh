#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

remote_host="${SLIMLYTICS_DEPLOY_HOST:-slimdeploy@cheap.redseam.com}"
remote_app="${SLIMLYTICS_DEPLOY_PATH:-/home/dustin/apps/slimlytics}"
public_url="${SLIMLYTICS_PUBLIC_URL:-https://slimlytics.com}"
public_url="${public_url%/}"
retain_releases="${SLIMLYTICS_RETAIN_RELEASES:-5}"
deploy_user="slimdeploy"
sentinel_name=".slimlytics-deployment"
sentinel_value="slimlytics-production-v1"
skip_checks=false

usage() {
  cat <<'EOF'
Usage: scripts/deploy-production.sh [--skip-checks]

Builds, backs up, deploys, and verifies Slimlytics production.

Environment overrides:
  SLIMLYTICS_DEPLOY_HOST       Dedicated slimdeploy SSH destination
                               (default: slimdeploy@cheap.redseam.com)
  SLIMLYTICS_DEPLOY_PATH       Canonical remote app directory
  SLIMLYTICS_PUBLIC_URL        HTTPS public base URL
  SLIMLYTICS_RETAIN_RELEASES   Source rollback snapshots to keep (default: 5)
EOF
}

while (($#)); do
  case "$1" in
    --skip-checks) skip_checks=true ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'Unknown option: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

[[ "$retain_releases" =~ ^[1-9][0-9]*$ ]] || {
  printf 'SLIMLYTICS_RETAIN_RELEASES must be a positive integer.\n' >&2
  exit 2
}
[[ "$remote_host" =~ ^slimdeploy@[A-Za-z0-9.-]+$ ]] || {
  printf 'SLIMLYTICS_DEPLOY_HOST must use the dedicated slimdeploy account and a hostname without shell metacharacters.\n' >&2
  exit 2
}
[[ "$remote_app" =~ ^/[A-Za-z0-9._/-]+$ && "$remote_app" != */../* && "$remote_app" != */.. ]] || {
  printf 'SLIMLYTICS_DEPLOY_PATH must be a canonical absolute path without whitespace or traversal.\n' >&2
  exit 2
}
[[ "$public_url" =~ ^https://[A-Za-z0-9.-]+(:[0-9]+)?$ ]] || {
  printf 'SLIMLYTICS_PUBLIC_URL must be an HTTPS origin without a path.\n' >&2
  exit 2
}
[[ -f "$sentinel_name" ]] && [[ "$(<"$sentinel_name")" == "$sentinel_value" ]] || {
  printf 'Local Slimlytics deployment sentinel is missing or invalid.\n' >&2
  exit 1
}

for command in git rsync ssh python3 make npm node; do
  command -v "$command" >/dev/null || { printf 'Required command not found: %s\n' "$command" >&2; exit 1; }
done

if [[ -n "$(git status --porcelain)" ]]; then
  printf 'Refusing to deploy a dirty working tree. Commit the release first.\n' >&2
  git status --short >&2
  exit 1
fi

branch="$(git branch --show-current)"
[[ "$branch" == "main" ]] || { printf 'Refusing to deploy branch %s; expected main.\n' "$branch" >&2; exit 1; }

git fetch --quiet origin main
revision="$(git rev-parse HEAD)"
remote_revision="$(git rev-parse origin/main)"
[[ "$revision" == "$remote_revision" ]] || {
  printf 'Refusing to deploy: local HEAD is not origin/main. Push first.\n' >&2
  exit 1
}
short_revision="$(git rev-parse --short=12 HEAD)"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
rollback_root="$(dirname "$remote_app")/.slimlytics-releases"
rollback_dir="$rollback_root/${stamp}-${short_revision}"
lock_name=".deploy-lock"
lock_token="${stamp}-${short_revision}-$$"
lock_acquired=false
rollback_started=false

if ! $skip_checks; then
  make test
  make check
  make build
  npm --prefix frontend exec -- redocly lint docs/openapi.json --config redocly.yaml --format stylish
  npm --prefix frontend audit --audit-level=high
fi

expected_scalar_assets_json="$(node frontend/scripts/verify-api-docs-build.mjs --json)"

essh=(ssh -o BatchMode=yes "$remote_host")

remote_script() {
  local command='bash -se --' argument quoted
  for argument in "$@"; do
    printf -v quoted '%q' "$argument"
    command+=" $quoted"
  done
  "${essh[@]}" "$command"
}

release_lock() {
  if ! $lock_acquired; then
    return 0
  fi
  remote_script "$remote_app" "$deploy_user" "$sentinel_name" "$sentinel_value" "$lock_name" "$lock_token" <<'REMOTE'
set -Eeuo pipefail
app=$1
deploy_user=$2
sentinel=$3
sentinel_value=$4
lock_name=$5
lock_token=$6
[[ "$(id -un)" == "$deploy_user" && "$(id -u)" -ne 0 ]]
[[ -d "$app" && "$(realpath -e -- "$app")" == "$app" ]]
[[ -f "$app/$sentinel" && "$(<"$app/$sentinel")" == "$sentinel_value" ]]
lock="$app/$lock_name"
tombstone="$app/$lock_name.releasing.$lock_token"
if [[ -L "$lock" && "$(readlink -- "$lock")" == "$lock_token" ]]; then
  mv -- "$lock" "$tombstone"
  rm -- "$tombstone"
fi
REMOTE
  lock_acquired=false
}

printf 'Acquiring exclusive production deployment lock...\n'
lock_acquired=true
trap release_lock EXIT
remote_script "$remote_app" "$deploy_user" "$sentinel_name" "$sentinel_value" "$lock_name" "$lock_token" <<'REMOTE'
set -Eeuo pipefail
app=$1
deploy_user=$2
sentinel=$3
sentinel_value=$4
lock_name=$5
lock_token=$6
[[ "$(id -un)" == "$deploy_user" && "$(id -u)" -ne 0 ]] || { echo 'Refusing deployment from an unexpected or root remote account.' >&2; exit 1; }
[[ -d "$app" && "$(realpath -e -- "$app")" == "$app" ]] || { echo 'Deployment target is not the canonical app path.' >&2; exit 1; }
[[ -f "$app/$sentinel" && "$(<"$app/$sentinel")" == "$sentinel_value" ]] || { echo 'Deployment target sentinel is invalid.' >&2; exit 1; }
lock="$app/$lock_name"
if ! ln -s -- "$lock_token" "$lock"; then
  echo "Another deployment holds $lock. Refusing concurrent deployment." >&2
  exit 1
fi
REMOTE

printf 'Preparing production rollback snapshot and database backup...\n'
remote_script "$remote_app" "$rollback_dir" "$rollback_root" "$retain_releases" "$deploy_user" "$sentinel_name" "$sentinel_value" "$lock_name" "$lock_token" <<'REMOTE'
set -Eeuo pipefail
app=$1
rollback=$2
rollback_root=$3
retain=$4
deploy_user=$5
sentinel=$6
sentinel_value=$7
lock_name=$8
lock_token=$9

[[ "$(id -un)" == "$deploy_user" && "$(id -u)" -ne 0 ]] || { echo 'Refusing deployment from an unexpected or root remote account.' >&2; exit 1; }
[[ -d "$app" ]] || { echo "Missing app directory: $app" >&2; exit 1; }
[[ "$(realpath -e -- "$app")" == "$app" ]] || { echo "Deployment path is not canonical: $app" >&2; exit 1; }
[[ -f "$app/$sentinel" ]] && [[ "$(<"$app/$sentinel")" == "$sentinel_value" ]] || { echo 'Invalid production sentinel.' >&2; exit 1; }
[[ -L "$app/$lock_name" && "$(readlink -- "$app/$lock_name")" == "$lock_token" ]] || { echo 'Deployment lock was lost or replaced.' >&2; exit 1; }
[[ -f "$app/.env" ]] || { echo 'Missing production .env.' >&2; exit 1; }
[[ -f "$app/compose.yaml" && -f "$app/compose.proxy.yaml" ]] || { echo 'Missing production Compose files.' >&2; exit 1; }
[[ "$(realpath -e -- "$rollback_root")" == "$rollback_root" && "$(dirname "$rollback")" == "$rollback_root" ]] || { echo 'Invalid rollback root.' >&2; exit 1; }
command -v rsync >/dev/null
command -v docker >/dev/null
mkdir "$rollback"
rsync -a --delete \
  --exclude .env --exclude backups --exclude .git --exclude '.deploy-lock*' \
  --exclude target --exclude node_modules --exclude .svelte-kit --exclude build --exclude dist \
  "$app/" "$rollback/"
cd "$app"
./scripts/backup.sh
mapfile -t old < <(find "$rollback_root" -mindepth 1 -maxdepth 1 -type d -print | sort -r | tail -n +$((retain + 1)))
((${#old[@]} == 0)) || rm -rf -- "${old[@]}"
REMOTE

rollback() {
  local trapped_status=$?
  local exit_code="${1:-$trapped_status}"
  if $rollback_started; then
    exit "$exit_code"
  fi
  rollback_started=true
  trap - ERR
  trap '' HUP INT TERM
  printf 'Deployment failed or was interrupted; restoring source snapshot %s...\n' "$rollback_dir" >&2
  remote_script "$remote_app" "$rollback_dir" "$rollback_root" "$deploy_user" "$sentinel_name" "$sentinel_value" "$lock_name" "$lock_token" <<'REMOTE'
set -Eeuo pipefail
app=$1
rollback=$2
rollback_root=$3
deploy_user=$4
sentinel=$5
sentinel_value=$6
lock_name=$7
lock_token=$8

[[ "$(id -un)" == "$deploy_user" && "$(id -u)" -ne 0 ]] || { echo 'Refusing rollback from an unexpected or root remote account.' >&2; exit 1; }
[[ -d "$app" && "$(realpath -e -- "$app")" == "$app" ]] || { echo 'Rollback target is not the canonical app path.' >&2; exit 1; }
[[ -f "$app/$sentinel" ]] && [[ "$(<"$app/$sentinel")" == "$sentinel_value" ]] || { echo 'Rollback target sentinel is invalid.' >&2; exit 1; }
[[ -L "$app/$lock_name" && "$(readlink -- "$app/$lock_name")" == "$lock_token" ]] || { echo 'Deployment lock was lost or replaced before rollback.' >&2; exit 1; }
[[ -d "$rollback" && "$(realpath -e -- "$rollback")" == "$rollback" && "$(realpath -e -- "$(dirname "$rollback")")" == "$rollback_root" ]] || { echo 'Rollback snapshot path is invalid.' >&2; exit 1; }
[[ -f "$rollback/$sentinel" ]] && [[ "$(<"$rollback/$sentinel")" == "$sentinel_value" ]] || { echo 'Rollback snapshot sentinel is invalid.' >&2; exit 1; }
rsync -a --delete \
  --exclude .env --exclude backups --exclude .git --exclude '.deploy-lock*' \
  --exclude target --exclude node_modules --exclude .svelte-kit --exclude build --exclude dist \
  "$rollback/" "$app/"
cd "$app"
compose=(docker compose -f compose.yaml -f compose.proxy.yaml)
"${compose[@]}" config -q
"${compose[@]}" up -d --build --remove-orphans --wait --wait-timeout 240
curl --retry 10 --retry-delay 2 --retry-all-errors --fail --silent http://127.0.0.1:8540/health >/dev/null
curl --retry 10 --retry-delay 2 --retry-all-errors --fail --silent http://127.0.0.1:8540/ready >/dev/null
REMOTE
  printf 'Source rollback is healthy. Database backup remains in %s/backups.\n' "$remote_app" >&2
  exit "$exit_code"
}
trap rollback ERR
trap 'rollback 129' HUP
trap 'rollback 130' INT
trap 'rollback 143' TERM

printf 'Synchronizing release %s to production...\n' "$short_revision"
rsync -az --delete-delay \
  --exclude .git --exclude .env --exclude backups --exclude '.deploy-lock*' \
  --exclude target --exclude node_modules --exclude .svelte-kit --exclude build --exclude dist \
  --exclude .DS_Store \
  ./ "$remote_host:$remote_app/"

remote_script "$remote_app" "$revision" "$deploy_user" "$sentinel_name" "$sentinel_value" "$lock_name" "$lock_token" <<'REMOTE'
set -Eeuo pipefail
app=$1
revision=$2
deploy_user=$3
sentinel=$4
sentinel_value=$5
lock_name=$6
lock_token=$7

[[ "$(id -un)" == "$deploy_user" && "$(id -u)" -ne 0 ]] || { echo 'Refusing deployment from an unexpected or root remote account.' >&2; exit 1; }
[[ -d "$app" && "$(realpath -e -- "$app")" == "$app" ]] || { echo 'Deployment target is not the canonical app path.' >&2; exit 1; }
[[ -f "$app/$sentinel" ]] && [[ "$(<"$app/$sentinel")" == "$sentinel_value" ]] || { echo 'Deployment target sentinel is invalid.' >&2; exit 1; }
[[ -L "$app/$lock_name" && "$(readlink -- "$app/$lock_name")" == "$lock_token" ]] || { echo 'Deployment lock was lost or replaced before Compose application.' >&2; exit 1; }
cd "$app"
printf '%s\n' "$revision" > .deploy-revision
compose=(docker compose -f compose.yaml -f compose.proxy.yaml)
"${compose[@]}" config -q
"${compose[@]}" up -d --build --remove-orphans --wait --wait-timeout 240
curl --retry 10 --retry-delay 2 --retry-all-errors --fail --silent http://127.0.0.1:8540/health >/dev/null
curl --retry 10 --retry-delay 2 --retry-all-errors --fail --silent http://127.0.0.1:8540/ready >/dev/null
"${compose[@]}" ps
REMOTE

printf 'Verifying public health, redirect, exact API contract, and Scalar CSS...\n'
PUBLIC_URL="$public_url" EXPECTED_SCALAR_ASSETS_JSON="$expected_scalar_assets_json" python3 - <<'PY'
import hashlib
import json
import os
import re
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

base = os.environ['PUBLIC_URL']
expected_contract = json.loads(Path('docs/openapi.json').read_text())
expected_scalar_assets = json.loads(os.environ['EXPECTED_SCALAR_ASSETS_JSON'])
expected_scalar_digests = {asset['sha256'] for asset in expected_scalar_assets}

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

for endpoint in ('/health', '/ready'):
    with urllib.request.urlopen(base + endpoint, timeout=30) as response:
        require(response.status == 200, f'{endpoint} returned {response.status}')

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None

try:
    urllib.request.build_opener(NoRedirect).open(base + '/api/docs', timeout=30)
except urllib.error.HTTPError as error:
    require(error.code == 307, f'/api/docs returned {error.code}, expected 307')
    location = error.headers.get('Location')
    require(location is not None, '/api/docs redirect omitted Location')
    require(urllib.parse.urljoin(base, location) == base + '/docs/api', f'Unexpected redirect: {location}')
else:
    raise RuntimeError('/api/docs did not return a redirect')

with urllib.request.urlopen(base + '/api/openapi.json', timeout=30) as response:
    deployed_contract = json.load(response)
require(deployed_contract == expected_contract, 'Deployed OpenAPI contract differs from release artifact')

with urllib.request.urlopen(base + '/docs/api', timeout=30) as response:
    html = response.read().decode()
hrefs = re.findall(r'<link[^>]+href="([^"]+\.css)"', html)
require(bool(hrefs), 'No CSS assets linked from API docs')
deployed_css_digests = set()
for href in hrefs:
    url = urllib.parse.urljoin(base + '/docs/api', href)
    with urllib.request.urlopen(url, timeout=30) as response:
        css_bytes = response.read()
    deployed_css_digests.add(hashlib.sha256(css_bytes).hexdigest())
require(
    bool(expected_scalar_digests & deployed_css_digests),
    'The exact route-associated Scalar CSS asset from this release is not linked by the deployed page'
)
print(f'Verified exact OpenAPI contract and Scalar CSS digest in {len(hrefs)} deployed stylesheet(s).')
PY

trap - ERR HUP INT TERM
release_lock
trap - EXIT
printf 'Deployed %s successfully to %s\n' "$short_revision" "$public_url"
