# Slimlytics development instructions

Slimlytics is a new Rust/Svelte/PostgreSQL analytics platform. The source requirements are in `/Users/dustin/sd/Obsidian/Personal/Slimlytics.md`.

## Architecture

- `backend/`: Rust, Axum, Tokio, SQLx/PostgreSQL.
- `frontend/`: Svelte 5 + SvelteKit + TypeScript.
- `tracker/`: framework-independent TypeScript browser tracker.
- `migrations/`: SQLx PostgreSQL migrations owned by the backend task.
- Root files, Docker, CI, and docs are owned by the coordinating agent.

## API contract

Backend prefix: `/api`.

- `GET /health`, `GET /ready`
- `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/auth/me`
- `GET|POST /api/sites`; `GET|PATCH|DELETE /api/sites/{id}`
- `POST /api/collect/{write_key}` accepts tracker page views/custom events
- `GET /api/sites/{id}/overview?days=28`
- `GET /api/sites/{id}/reports/{pages|referrers|countries|devices|campaigns}`
- `GET /api/sites/{id}/visitors`
- `GET /api/sites/{id}/events`
- `GET|POST /api/sites/{id}/goals`
- `GET /api/sites/{id}/export.csv`
- `GET /api/sites/{id}/stream` is authenticated SSE

Use camelCase JSON at the API boundary. Frontend reads `PUBLIC_API_BASE_URL`, defaulting to `/api` semantics through its API client. Tracker is initialized with endpoint and write key.

## Quality gates

- TDD for behavior: create a failing test, run it, implement, rerun.
- Never commit secrets.
- Parameterized SQL only.
- Privacy defaults: cookieless, redact sensitive query parameters, hash/truncate IP-derived IDs, never capture form values.
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` must pass.
- `npm run check`, `npm test`, and `npm run build` must pass.
- Do not commit independently; the coordinating agent owns commits and pushes.
