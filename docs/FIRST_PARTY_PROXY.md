# First-party and anti-adblock tracker delivery

Serving the tracker and collection endpoint through the measured site's own domain improves reliability and keeps requests visibly first-party. It must not be used to override a visitor's explicit consent, Do Not Track, or Global Privacy Control choice.

## Built-in neutral paths

The dashboard's **Anti-adblock tracking** option uses the neutral `/s.js` asset and `/api/e/{writeKey}` collection alias. They behave identically to `/tracker.js` and `/api/collect/{writeKey}` but avoid broad filename/path filter rules. The option is enabled by default for newly copied snippets.

The generated snippet works immediately against the Slimlytics domain. For the strongest reliability, proxy the neutral paths through the measured site's own domain and change the snippet host as shown below.

Assume Slimlytics is hosted at `https://analytics.example.com` and the measured site is `https://www.example.com`.

```html
<script async
  src="https://www.example.com/s.js"
  data-write-key="YOUR_SITE_WRITE_KEY"
  data-endpoint="https://www.example.com/_s/e"></script>
```

The private `/_s/` namespace avoids colliding with the measured application's own `/api/` routes.

## Caddy

```caddyfile
www.example.com {
    # Put these handles before the existing application fallback.
    handle /s.js {
        reverse_proxy https://analytics.example.com {
            header_up Host analytics.example.com
        }
        header Cache-Control "public, max-age=86400"
    }

    @slimlytics_collector path /_s/e/*
    handle @slimlytics_collector {
        uri replace /_s/e/ /api/e/
        reverse_proxy https://analytics.example.com {
            header_up Host analytics.example.com
            header_up X-Forwarded-Host {host}
            header_up X-Forwarded-Proto {scheme}
        }
    }

    # Never expose dashboard, authentication, reports, or exports here.
    handle /_s/* {
        respond 404
    }

    # Existing application proxy/routes go here.
}
```

## Nginx

```nginx
location = /s.js {
    proxy_set_header Host analytics.example.com;
    proxy_ssl_server_name on;
    proxy_pass https://analytics.example.com/s.js;
    add_header Cache-Control "public, max-age=86400";
}

location ^~ /_s/e/ {
    proxy_set_header Host analytics.example.com;
    proxy_set_header X-Forwarded-Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_ssl_server_name on;
    rewrite ^/_s/e/(.*)$ /api/e/$1 break;
    proxy_pass https://analytics.example.com;
}

location ^~ /_s/ {
    return 404;
}
```

## Cloudflare

Use narrowly scoped Worker routes for `www.example.com/s.js` and `www.example.com/_s/e/*`. Rewrite only those requests to the corresponding Slimlytics origin paths, preserve request methods and bodies, and set the upstream host correctly. Do not cache collector requests. Cache the tracker asset for no longer than one day unless its URL is versioned.

Reject every other `/_s/` request. Never proxy dashboard, authentication, reports, exports, or arbitrary `/api/` paths.

## Verification

1. Load the first-party `/s.js` URL and confirm JavaScript content with a 200 status.
2. Send a test page view and confirm `/_s/e/{writeKey}` returns 202 Accepted.
3. Verify the dashboard updates within two seconds.
4. Confirm an unexpected `Origin` is rejected.
5. Confirm denied consent, DNT, and GPC suppress browser events.
6. Confirm unrelated `/_s/` paths return 404 and application routes still work.
