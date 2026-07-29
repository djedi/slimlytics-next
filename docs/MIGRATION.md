# Coexisting with older Slimlytics repositories

This repository is a clean rewrite. It intentionally does not replace, force-push, or rewrite either historical repository:

- https://github.com/djedi/slimlytics
- https://github.com/djedi/go_slimlytics

## Initial deployment

Use a new database, hostname, and container project name. Run the new tracker on a test site or staging origin first. Compare page views, sessions, referrers, campaigns, geography, and realtime behavior before moving production sites.

## Tracker cutover

1. Create the site in the new dashboard.
2. Add its production origin and copy the new write key.
3. Install the new tracker alongside the old tracker for a short comparison window if duplicate analytics storage is acceptable.
4. Confirm privacy controls, outbound/download events, SPA navigation, realtime updates, and report totals.
5. Remove the old tracker after acceptance.
6. Keep the old deployment and database read-only through the chosen historical retention period.

Never reuse a dashboard auth token as a site write key. Never copy historical raw IP addresses into the new database.

## Historical import

Data import is a later feature. Version 1 does not pretend that two older SQLite schemas and the new PostgreSQL schema are magically interchangeable. Any future importer must:

- Identify source schema/version explicitly
- Import into a new transaction or staging schema
- Preserve source IDs in dedicated metadata rather than replacing new UUIDs
- Redact sensitive URL parameters during import
- Recompute aggregates deterministically
- Be idempotent and resumable
- Produce counts and rejection reports
- Leave the source database untouched

## Rollback

Rollback the measured site by restoring its previous tracker snippet. Because the rewrite uses a separate repository, deployment, credentials, and database, rollback does not require altering either historical repository.
