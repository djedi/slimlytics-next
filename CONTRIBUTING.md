# Contributing

## Development loop

1. Create a focused branch.
2. Write one failing behavior test.
3. Run it and confirm the expected failure.
4. Implement the smallest passing change.
5. Refactor with the suite green.
6. Run `make test`, `make check`, and `make build`.
7. Open a pull request with behavior, privacy, migration, and deployment notes.

## Security and privacy review

Every collection-path change must answer:

- Can untrusted input reach SQL, logs, HTML, headers, or filesystem paths unsafely?
- Does this collect more identifying information than before?
- Are sensitive URL parameters still redacted?
- Is origin and site authorization enforced?
- Are duplicate and replayed events safe?
- Does denial or revocation of consent stop new browser events?

Never use production analytics payloads or credentials as test fixtures.

## Database changes

Migrations are append-only after release. Use explicit indexes and constraints, test against PostgreSQL, and document whether rollback requires restoring a backup.

## Commit format

Use conventional prefixes such as `feat:`, `fix:`, `test:`, `docs:`, `refactor:`, and `chore:`.
