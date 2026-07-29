# First-party tracker proxying

Serving the tracker and collection endpoint through the measured site’s own domain improves reliability and keeps requests visibly first-party. It must not be used to bypass a visitor’s explicit consent, DNT, or Global Privacy Control choice.

Assume Slimlytics is hosted at `https://analytics.example.com` and the measured site is `https://www.example.com`.

## Caddy

```caddyfile
www.example.com {
    # Existing application proxy/routes go here.

    handle /slytics-v1.js {
        reverse_proxy https://analytics.example.com {
            header_up Host analytics.example.com
        }
        uri replace /slytics-v1.js /tracker.js
        header Cache-Control "public, max-age=86400, immutable"
    }

    @collector path /_analytics/api/collect/*
    handle @collector {
        uri strip_prefix /_analytics
        reverse_proxy https://analytics.example.com {
            header_up Host analytics.example.com
            header_up X-Forwarded-Host {host}
            header_up X-Forwarded-Proto {scheme}
        }
    }

    # Never expose dashboard, authentication, reports, or exports here.
    handle /_analytics/* {
        respond 404
    }
}
```

Initialize with endpoint `https://www.example.com/_analytics/api`.

## Nginx

```nginx
location = /slytics-v1.js {
    proxy_set_header Host analytics.example.com;
    proxy_ssl_server_name on;
    proxy_pass https://analytics.example.com/tracker.js;
    add_header Cache-Control "public, max-age=86400, immutable";
}

location ^~ /_analytics/api/collect/ {
    proxy_set_header Host analytics.example.com;
    proxy_set_header X-Forwarded-Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_ssl_server_name on;
    rewrite ^/_analytics/(.*)$ /$1 break;
    proxy_pass https://analytics.example.com;
}

location ^~ /_analytics/ {
    return 404;
}
```

## Cloudflare

Use a narrowly scoped Worker route such as `www.example.com/_analytics/api/collect/*`. Reject every other `/_analytics/` path. Rewrite requests to the Slimlytics origin, preserve request methods and bodies, and set the upstream `Host` correctly. Do not cache collector POST requests. Cache only the versioned tracker asset.

## Verification

1. Load the first-party script URL and confirm JavaScript content with a 200 status.
2. Send a test page view and confirm the collector returns an accepted status.
3. Verify the dashboard updates within two seconds.
4. Confirm an unexpected `Origin` is rejected.
5. Confirm DNT/GPC or denied consent suppresses browser events.
