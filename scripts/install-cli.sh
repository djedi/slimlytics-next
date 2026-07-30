#!/bin/sh
set -eu

REPOSITORY="https://github.com/djedi/slimlytics-next"
REF="${SLIMLYTICS_CLI_REF:-cli-v0.1.1}"

for command in cargo curl tar mktemp; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "slimlytics installer: required command not found: $command" >&2
    exit 1
  fi
done

workdir=$(mktemp -d "${TMPDIR:-/tmp}/slimlytics-cli.XXXXXX")
trap 'rm -rf "$workdir"' EXIT HUP INT TERM

archive_url="$REPOSITORY/archive/refs/tags/$REF.tar.gz"
curl --fail --silent --show-error --location \
  "$archive_url" \
  --output "$workdir/source.tar.gz"
tar -xzf "$workdir/source.tar.gz" -C "$workdir"
source_dir=""
for candidate in "$workdir"/slimlytics-next-*; do
  if [ -d "$candidate" ]; then
    source_dir=$candidate
    break
  fi
done
if [ -z "$source_dir" ]; then
  echo "slimlytics installer: downloaded archive had an unexpected layout" >&2
  exit 1
fi
cargo install --locked --force --path "$source_dir/cli"
printf '%s\n' 'Installed slimlytics. Run: slimlytics auth login --email you@example.com'
