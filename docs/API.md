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

## Sites

- `GET /api/sites`
- `POST /api/sites`
- `GET /api/sites/{siteId}`
- `PUT /api/sites/{siteId}`
- `DELETE /api/sites/{siteId}`

A site has a display name, canonical URL, timezone, allowed origins, retention policy, status, and independently rotatable collection write key.

## Collection

- `POST /api/collect/{writeKey}`
- `POST /api/e/{writeKey}` — neutral anti-adblock alias with identical behavior

The browser tracker sends page views and custom events. The collector accepts `sendBeacon` bodies, applies origin checks, normalizes and redacts URLs, classifies obvious bots/internal traffic, derives site-scoped anonymous identifiers, deduplicates event IDs, and persists accepted events.

A write key authorizes ingestion only. It never grants dashboard or reporting access.

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
