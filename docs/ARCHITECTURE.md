# Architecture decisions

## Server-Sent Events for dashboard updates

Slimlytics uses Server-Sent Events (SSE) rather than WebSockets for Version 1. Analytics updates are overwhelmingly server-to-browser, so SSE provides the needed directionality with ordinary HTTP semantics, automatic browser reconnection, event IDs, and simpler reverse-proxy operations.

The browser opens one authenticated stream per active site view. The backend emits compact typed updates and heartbeat comments. High-volume activity may be coalesced before browser delivery. After reconnecting, the client refreshes durable overview/report endpoints so a missed transient notification can never become a permanently incorrect total.

WebSockets remain an option only for a future feature that genuinely needs frequent bidirectional messaging.

## PostgreSQL as source of truth

Raw events and sessions are durable in PostgreSQL. Reporting queries and later aggregates derive from that data. Live notifications are hints, not the source of truth. PostgreSQL `LISTEN/NOTIFY` is sufficient for initial multi-process fan-out; Redis or NATS is an intentional scale-out seam rather than a mandatory idle container.

## Cookieless, site-scoped identity

Version 1 does not use cross-site cookies or fingerprinting. Anonymous identifiers are scoped to one site, one rotating period, and privacy-reduced request attributes. This supports approximate uniques and sessions without creating a portable identity graph.

## Separate ingestion and reporting paths

The public collector is authorized only by a site write key and constrained by origin/rate controls. Dashboard/report endpoints require user authentication and site membership. Write keys cannot read analytics. Reporting credentials cannot be embedded in measured websites.

## Monorepo for the application

Backend, dashboard, tracker, migrations, deployment, and technical documentation share one application repository because they evolve under one API contract and release. A possible future public marketing website belongs in a separate repository and deployment lifecycle.
