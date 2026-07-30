# Slimlytics Rust CLI

The `slimlytics` CLI authenticates to a Slimlytics account, manages personal API tokens and sites, and generates the complete first-party tracking setup for Caddy, Nginx, or Apache. Its idempotent JSON workflow is designed for local AI agents and infrastructure automation.

## Install

Requirements: `cargo`, `curl`, and `tar`. The installer downloads this repository and performs a locked Cargo build:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/djedi/slimlytics-next/cli-v0.1.1/scripts/install-cli.sh | sh
slimlytics --version
```

Cargo installs the binary under `${CARGO_HOME:-$HOME/.cargo}/bin`. Set `SLIMLYTICS_CLI_REF` to install another branch from its source archive.

Developers can install directly from a checkout:

```bash
cargo install --locked --path cli
```

## Authenticate

Interactive login prompts without echoing the password. It uses the password only to obtain a short-lived session JWT, exchanges that JWT for a personal API token, and stores the API token—not the password.

```bash
slimlytics auth login --email you@example.com
slimlytics auth status
```

For noninteractive input, keep the password out of process arguments and shell history:

```bash
printf '%s' "$SLIMLYTICS_PASSWORD" | \
  slimlytics auth login --email you@example.com --password-stdin
```

The default API is `https://slimlytics.com`. Self-hosted installations can pass `--api-url https://analytics.example.com` or set `SLIMLYTICS_API_URL`.

The credential file lives under the platform's private configuration directory (for example, `~/Library/Application Support/slimlytics/auth.json` on macOS or `~/.config/slimlytics/auth.json` on Linux). The directory is mode `0700` and the file is mode `0600` on Unix. Slimlytics refuses to write credentials through a symbolic link.

Headless agents can bypass the file:

```bash
export SLIMLYTICS_TOKEN='slyt_...'
slimlytics --json account show
```

Import an existing token without exposing it in process arguments:

```bash
printf '%s' "$SLIMLYTICS_TOKEN" | slimlytics auth use-token --token-stdin
```

Log out locally, or revoke the current token before removing it:

```bash
slimlytics auth logout
slimlytics auth logout --revoke
```

## Account and token commands

```bash
slimlytics account show
slimlytics token list
slimlytics token revoke TOKEN_UUID
```

API-token secrets are returned only at creation. Listings contain IDs, names, prefixes, expiry, creation, and last-use timestamps, never the secret or stored digest. Token revocation takes effect on the next request.

## Sites

```bash
slimlytics site list
slimlytics site show example.com
slimlytics site add example.com --name Example --server caddy
slimlytics site delete example.com --yes
```

A site can be selected by UUID or exact domain. Ambiguous domains are rejected; use a UUID to disambiguate.

`site ensure` is the preferred agent operation. It creates the exact domain only if absent, otherwise reuses it, configures the requested server type, and always returns installation instructions:

```bash
slimlytics --json site ensure example.com --server caddy
slimlytics --json site ensure example.com --server nginx
slimlytics --json site ensure example.com --server apache
```

The JSON response has a stable top-level success envelope:

```json
{
  "schemaVersion": 1,
  "ok": true,
  "data": {
    "created": true,
    "site": {
      "id": "...",
      "domain": "example.com",
      "antiAdblockServer": "caddy"
    },
    "tracking": {
      "serverConfig": "...",
      "snippet": "<script async src=\"/...js\"></script>",
      "scriptTestUrl": "https://example.com/...js",
      "beaconTestUrl": "https://example.com/..."
    }
  }
}
```

`created` is `false` when the domain already existed. This makes repeated agent runs safe rather than producing a growing herd of duplicate sites, which is apparently how databases express loneliness.

## Tracking setup

Display the current setup or change its server and optional same-origin paths:

```bash
slimlytics tracking show example.com
slimlytics tracking configure example.com --server nginx
slimlytics tracking configure example.com --server caddy \
  --js-path /assets-helper.js --beacon-path /event-helper
```

The result contains:

- hardened reverse-proxy configuration for the selected server;
- the minimal same-origin `<script>` snippet;
- script and beacon test URLs;
- ordered installation and verification steps.

An AI agent should:

1. Run `slimlytics --json site ensure DOMAIN --server SERVER`.
2. Install `data.tracking.serverConfig` in the domain's web-server configuration.
3. Validate and reload that server.
4. Insert `data.tracking.snippet` before `</body>` in the shared page template.
5. Request both test URLs and require HTTP 200.
6. Load a real page and confirm activity in Slimlytics.

The CLI deliberately does not SSH into unrelated servers or modify website files by itself. The calling agent controls those privileges and can review, back up, validate, and roll back the website changes.

## Machine-readable operation

Pass `--json` for JSON on stdout. Errors go to stderr with a nonzero exit status. No password or API-token secret is printed by successful login/status commands. For unattended environments, prefer `SLIMLYTICS_TOKEN` and a narrowly controlled process environment.
