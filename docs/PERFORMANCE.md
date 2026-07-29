# Performance targets

Version 1 uses explicit targets so “lightweight” means more than wearing a small hat.

## Small-VPS reference environment

- 1 shared vCPU
- 1 GB RAM
- PostgreSQL on the same host
- 30 days of raw traffic retained
- 10 million stored events

## Service-level targets

| Measure | Target |
|---|---:|
| Collector response, p95 | under 100 ms excluding network |
| Sustained ingestion | at least 1,000 events/second on a 4-vCPU benchmark host |
| Overview query, p95 | under 500 ms for 10 million events |
| Common report query, p95 | under 750 ms |
| New event visible through SSE | under 2 seconds |
| API idle RSS | under 100 MB |
| Full application stack on reference VPS | under 768 MB steady-state |
| Tracker gzip size | under 10 KB |
| Tracker main-thread blocking | no long task over 50 ms during initialization |

## Benchmark method

- Use a deterministic synthetic event generator with realistic site, URL, referrer, UTM, country, browser, and session distributions.
- Measure collector latency at the reverse proxy and API layers.
- Warm reporting caches before steady-state report measurements; record cold-query results separately.
- Run PostgreSQL `EXPLAIN (ANALYZE, BUFFERS)` for regressions.
- Record CPU, RSS, database size, event count, and exact commit.
- Fail release qualification when realtime latency exceeds two seconds under the documented normal-load profile.

## Scaling order

1. Add/repair indexes based on measured query plans.
2. Batch collector inserts.
3. Maintain hourly/daily aggregates.
4. Partition raw events by time.
5. Separate PostgreSQL from the application host.
6. Add Redis or NATS only when measured fan-out pressure justifies another service.
