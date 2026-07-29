# First-party anti-adblock tracker delivery

Slimlytics can serve its tracker and collection beacon through the measured site's own domain using neutral, site-specific paths. This improves reliability without overriding consent, Do Not Track, or Global Privacy Control.

## Dashboard workflow

Open a site's **Settings → Anti-adblock tracking** panel:

1. Select Caddy, Nginx, or Apache.
2. Keep the generated JavaScript and beacon paths, or enter two unused neutral paths.
3. Save the configuration.
4. Copy the generated server configuration into the measured site's server block and reload the server.
5. Copy the minimal same-origin tracking code into the site's `<head>`.
6. Open both generated test links.

A site may receive defaults such as:

```text
JavaScript path: /456bbb63bb86.js
Beacon path:     /0d31360a3101
```

The matching installation code is deliberately minimal:

```html
<script async src="/456bbb63bb86.js"></script>
```

The generated server configuration keeps the write key out of the site's HTML snippet. Like any browser analytics ingestion key, it remains visible in the returned JavaScript bootstrap and grants collection access only—not dashboard or reporting access. The browser sends collection requests to the exact beacon path rather than to `/{beacon}/{writeKey}`.

## How it works

The generated configuration exposes exactly two same-origin routes:

- The JavaScript path proxies to a public Slimlytics bootstrap route containing the tracker and site initialization.
- The beacon path proxies to `/api/collect/{writeKey}`.

The bootstrap bundle initializes the tracker with `appendWriteKey: false`, preserving the exact browser-facing beacon path. It retains the standard tracker privacy behavior and sends each event in the collector's normal payload shape.

Server configurations are generated from validated structured fields. Users cannot change the Slimlytics upstream origin, and paths cannot contain nested segments, query strings, fragments, backslashes, or control characters.

## Proxy security

The generated Caddy, Nginx, and Apache configurations:

- Match only the two selected paths.
- Remove browser `Cookie` and `Authorization` headers before proxying.
- Remove upstream `Set-Cookie` responses.
- Preserve the request method, body, content type, origin, and user agent.
- Configure the upstream host and HTTPS SNI.
- Avoid trusting a browser-supplied `X-Forwarded-For` value.

Caddy replaces `X-Forwarded-For` by default unless global `trusted_proxies` behavior changes that policy. Review Caddy's trusted-proxy configuration if another proxy sits in front of the measured site.

## Server requirements

### Caddy

Place both generated `handle` blocks before a broad application fallback, then validate and reload:

```bash
caddy validate --config /etc/caddy/Caddyfile
caddy reload --config /etc/caddy/Caddyfile
```

### Nginx

Place both exact `location =` blocks inside the site's `server` block:

```bash
nginx -t
nginx -s reload
```

### Apache

The generated configuration uses anchored `ProxyPassMatch` directives and requires `proxy`, `proxy_http`, `ssl`, and `headers`. Validate and reload using the commands appropriate for the distribution, commonly:

```bash
apachectl configtest
apachectl graceful
```

## Verification

1. Open the generated JavaScript test link. It should return JavaScript with a `200` status and `X-Content-Type-Options: nosniff`.
2. Open the beacon test link. It should return `{"status":"ok"}` without inserting an analytics event.
3. Load a measured page and confirm the browser requests the custom JavaScript and beacon paths on the measured site's domain.
4. Confirm a new page view appears in Slimlytics.
5. Confirm denied consent, DNT, and GPC suppress browser events.
6. Confirm unrelated paths still reach the measured application normally.

The legacy `/s.js` and `/api/e/{writeKey}` aliases remain available for backward compatibility, but new installations should use the generated per-site proxy flow.
