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

## Generated first-party proxy delivery

Each site stores a closed server type plus validated JavaScript and beacon paths. The dashboard renders server configuration from those structured values rather than storing arbitrary configuration text. The generated proxy exposes exactly two routes on the measured site's domain and strips cookies and authorization before forwarding.

The JavaScript route targets a SvelteKit bootstrap endpoint. Its complete tracker IIFE is embedded into the adapter-node server build as a generated raw artifact, eliminating runtime filesystem and working-directory assumptions. The appended initializer contains the public write key and selects the exact same-origin beacon path. The beacon proxy rewrites that path to the canonical Rust collection endpoint; authenticated reporting routes are never proxied.

## Monorepo for the application

Backend, dashboard, tracker, migrations, deployment, and technical documentation share one application repository because they evolve under one API contract and release. A first public marketing surface (landing, pricing, privacy, docs hub) ships in the SvelteKit frontend for signup conversion; a dedicated marketing site can still be extracted later if the lifecycle diverges.
