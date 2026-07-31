# Agent integration

Slimlytics exposes the same deterministic analytics semantics through REST and MCP. Agents should use personal API tokens rather than session JWTs.

## Token scopes

Create a token with only the scopes the workflow needs:

- `sites:read`: list and inspect site configuration
- `sites:write`: create or change sites
- `analytics:read`: overview, reports, visitors, journeys, funnels, and MCP analytics tools
- `analytics:write`: goals, annotations, and funnels
- `integrations:read`: connected-service reports through MCP
- `integrations:write`: connect, sync, or disconnect external services

Tokens are shown once, expire, can be revoked, and are never accepted as collection write keys.

## MCP

The Streamable HTTP endpoint is:

```text
POST https://slimlytics.com/api/mcp
Authorization: Bearer slyt_...
Content-Type: application/json
```

Slimlytics implements JSON-RPC 2.0 lifecycle and tools for the stable MCP `2025-11-25` revision. It is stateless and does not open a server-to-client SSE stream. The available tools are:

- `list_sites`
- `analytics_summary`
- `dimension_report`
- `search_console_report`
- `marketing_brief`

Every analytics call requires an explicit site ID and inclusive `from`/`to` dates. Results include a generation time, source evidence, definitions where metrics can be ambiguous, and a freshness field such as `dataThrough`.

Tool calls are recorded in `agent_audit_log` with the account, token, site, request ID, inputs, action, outcome, and timestamp. Secrets and raw visitor addresses are not included.

`marketing_brief` returns the same completed-day payload used by scheduled webhooks, including comparisons, top pages, anomaly evidence, crawler traffic, revenue, generation time, and `dataThrough`.

## Scheduled delivery

Use `/api/sites/{siteId}/report-subscriptions` to create daily or weekly HTTPS webhook delivery. The creation response includes a signing secret once. Verify the URL-safe HMAC-SHA256 value in `X-Slimlytics-Signature` against the exact request body before processing it.

Webhook destinations are DNS-resolved immediately before delivery, pinned for the request, restricted to public addresses on HTTPS port 443, and cannot redirect. Delivery attempts and errors are retained for inspection. An anomaly-only subscription records a skipped delivery when no material anomaly is present.

## Server collection

Authenticated site responses include a separate `serverWriteKey`. Use it only with the batched server request endpoint documented in [SERVER_INGESTION.md](SERVER_INGESTION.md). Supply a stable per-request `idempotencyKey`; raw client IPs are transformed and discarded rather than persisted.

## Reliable writes

Send an `Idempotency-Key` header when an agent creates annotations or funnels:

```text
Idempotency-Key: campaign-agent:550e8400-e29b-41d4-a716-446655440000
```

Keys must be 8-128 URL-safe characters. Slimlytics serializes concurrent requests with the same account, site, operation, and key, then returns the original status and body for retries for 24 hours.

## Interpretation rules

- Treat site-local inclusive dates as the reporting boundary.
- Surface `dataThrough` when reporting conclusions.
- Do not compare Search Console and browser sessions as if they were the same metric.
- Search Console may return top rows rather than every row.
- Human reports exclude known bots and internal traffic.
- AI referral reports measure people arriving from AI products. AI crawler reports measure identified automation and require server-side collection for crawlers that do not execute JavaScript.
