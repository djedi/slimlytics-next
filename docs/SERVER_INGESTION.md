# Server-side request ingestion

Server ingestion captures crawlers and non-JavaScript requests from trusted access logs. It uses a separate rotatable site secret and never stores a raw client IP.

## Endpoint

Find the endpoint in **Settings → Server collection** or in the authenticated site response:

```text
POST https://slimlytics.com/api/ingest
Content-Type: application/json
X-Slimlytics-Server-Key: {serverWriteKey}
```

The request body contains 1-100 events:

```json
{
  "events": [
    {
      "idempotencyKey": "edge:request-018f67a3",
      "url": "https://example.com/docs?utm_source=ai",
      "userAgent": "GPTBot/1.0",
      "clientIp": "203.0.113.20",
      "occurredAt": "2026-07-31T18:30:00Z",
      "method": "GET",
      "status": 200,
      "eventName": "pageview"
    }
  ]
}
```

URLs must use the site's configured domain. Only GET/HEAD requests and timestamps from the previous seven days are accepted. The source idempotency key prevents log-shipping retries from creating duplicates.

The backend uses the IP transiently for GeoIP enrichment and rotating HMAC identifiers, then discards it. It stores sanitized URLs, bounded request metadata, UA classification, coarse location, and `ingestionSource=server`.

## Caddy logs

Configure Caddy JSON access logs, then stream them through the included forwarder:

```bash
export SLIMLYTICS_SERVER_INGEST_URL='https://slimlytics.com/api/ingest'
export SLIMLYTICS_SERVER_WRITE_KEY='YOUR_SERVER_KEY'
export SLIMLYTICS_SITE_ORIGIN='https://example.com'
tail -F /var/log/caddy/access.log | node scripts/server-log-forwarder.mjs
```

The default `SLIMLYTICS_LOG_MODE=bots` forwards only crawler-like user agents. Set `SLIMLYTICS_LOG_MODE=all` only when browser pageview tracking is disabled or deduplicated upstream; otherwise human pageviews will be counted twice.

Run the forwarder under the site's service manager and restrict its environment file because it contains a collection secret. The secret is sent in a header rather than the URL so normal request logs do not capture it. Rotate the key from Settings after suspected exposure.
