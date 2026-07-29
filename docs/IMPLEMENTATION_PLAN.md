# Slimlytics implementation plan

## Scope

Deliver the complete Version 1/MVP from the Obsidian specification as a new repository, while laying clean extension points for the explicitly listed later features.

## Workstreams

1. Backend: schema, authentication, sites, event ingestion, privacy filtering, bot/internal classification, overview/reports, goals, CSV, SSE, health/readiness, tests.
2. Frontend: Svelte application shell, auth, multi-site overview, responsive charts/cards, site reports, visitor details, live Spy map/activity stream, settings, themes, accessibility, tests.
3. Tracker: asynchronous cookieless browser SDK with SPA navigation, page views, custom events, outbound/download tracking, sendBeacon/fetch fallback, batching, deduplication, DNT/consent, tests.
4. Operations: Docker images/Compose, PostgreSQL, migrations, backup/restore scripts, Caddy/Nginx examples, CI, security/dependency checks, documentation.
5. Integration: database-backed tests, production builds, browser QA, independent spec/security review, GitHub push, CI verification.

## Definition of done

- Local Docker deployment starts successfully and exposes health endpoints and the web UI.
- A user can register/login, create multiple sites, copy a tracking snippet, ingest traffic, see overview/reports, watch SSE activity, create goals, and export CSV.
- Automated backend, frontend, and tracker tests pass.
- Formatting, linting, type checks, and production builds pass.
- Repository is pushed to a new `djedi/slimlytics-next` GitHub repository and first CI run is green.
- Completion notification is sent to Dustin via Telegram.
