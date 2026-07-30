# API overview

All JSON endpoints use camelCase fields. Authenticated requests send an `Authorization: Bearer <token>` header. Error responses use a stable machine-readable code and a human-readable message.

## System

- `GET /health` — process liveness
- `GET /ready` — PostgreSQL readiness

## Authentication

- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/auth/me`

Passwords are hashed with Argon2. Access tokens are short-lived JWTs signed with `JWT_SECRET`.

## Account API tokens

- `POST /api/account/tokens` — create a personal API token using a session JWT
- `GET /api/account/tokens` — list active token metadata
- `DELETE /api/account/tokens/{tokenId}` — immediately revoke a token
- `DELETE /api/account/tokens/current` — revoke the personal token authenticating this request

Creation accepts `{ "name": "slimlytics-cli", "expiresInDays": 365 }`. The response contains the `slyt_...` secret exactly once. Slimlytics stores only a SHA-256 digest of the 256-bit random secret, and list responses expose only a short prefix and timestamps. Tokens expire after 365 days by default; accepted bounds are 1–3650 days. A personal token can use account and site APIs but cannot mint another token—creating one requires a password-authenticated session JWT.

API tokens and JWTs use the same Bearer header. Revoked or expired tokens return `401`. See `CLI.md` for the supported client and agent workflow.

## Sites

- `GET /api/sites`
- `POST /api/sites`
- `POST /api/sites/ensure` — atomically create or reuse a canonical domain
- `GET /api/sites/{siteId}`
- `PUT /api/sites/{siteId}`
- `PUT /api/sites/{siteId}/anti-adblock`
- `DELETE /api/sites/{siteId}`

A site has a display name, canonical URL, timezone, allowed origins, retention policy, status, independently rotatable collection write key, and a persisted anti-adblock server type, JavaScript path, and beacon path. New sites receive random neutral path defaults. Domains are canonicalized case-insensitively and globally unique; an account cannot claim a domain already managed by another account. `ensure` returns `{ "created": boolean, "site": {...} }` and is safe for retrying agents. The anti-adblock update body is `{ "serverType": "caddy|nginx|apache", "jsPath": "/...js", "beaconPath": "/..." }`.

## Collection

- `POST /api/collect/{writeKey}`
- `GET /api/collect/{writeKey}` — non-ingesting proxy diagnostic
- `POST /api/e/{writeKey}` — neutral anti-adblock alias with identical behavior
- `GET /api/e/{writeKey}` — non-ingesting legacy-alias diagnostic

The browser tracker sends page views and custom events. The collector accepts `sendBeacon` bodies, applies origin checks, normalizes and redacts URLs, classifies obvious bots/internal traffic, derives site-scoped anonymous identifiers, deduplicates event IDs, and persists accepted events.

A write key authorizes ingestion only. It never grants dashboard or reporting access.

## First-party tracker bootstrap

- `GET /p/{writeKey}/{beaconName}`

Returns the complete JavaScript tracker plus a site initializer targeting the exact same-origin `/{beaconName}` path. The bundle is embedded in the adapter-node build rather than loaded from the runtime working directory. Invalid keys or path names return `400`. See `FIRST_PARTY_PROXY.md` for the dashboard-generated server configurations.

## Reporting

- `GET /api/sites/{siteId}/overview?from=2026-07-01&to=2026-07-28`
- `GET /api/sites/{siteId}/reports/pages`
- `GET /api/sites/{siteId}/reports/referrers`
- `GET /api/sites/{siteId}/reports/countries`
- `GET /api/sites/{siteId}/reports/devices`
- `GET /api/sites/{siteId}/reports/campaigns`
- `GET /api/sites/{siteId}/visitors`
- `GET /api/sites/{siteId}/events`

Overview, reports, visitors, events, and export accept inclusive `from` and `to` dates. Reports also accept a bounded `limit`. The overview includes the prior equivalent period for percentage comparisons.

## Goals

- `GET /api/sites/{siteId}/goals`
- `POST /api/sites/{siteId}/goals`

Version 1 goals match an event name and an optional SQL-LIKE path pattern. This covers explicit custom-event goals and page-path goals without collecting additional identity data.

## Export

- `GET /api/sites/{siteId}/export.csv`

The export uses the caller’s site authorization and report filters. Content-Disposition provides a safe filename.

## Real time

- `GET /api/sites/{siteId}/stream?token={accessToken}`

The response is `text/event-stream`. Native browser `EventSource` supplies the short-lived access token as a query parameter because it cannot set an Authorization header. Events include monotonically useful event IDs, typed JSON payloads, and heartbeat comments. Browsers may reconnect with `Last-Event-ID`; clients reconcile against durable overview/report endpoints after reconnection.

## Tracker API

The global tracker provides:

- `init(options)`
- `page(properties?)`
- `event(name, properties?)`
- `consent(granted)`
- `flush()`

Initialization options include the collector endpoint, site write key, batching interval, automatic SPA/page tracking, outbound/download tracking, DNT/GPC behavior, and initial consent state.
