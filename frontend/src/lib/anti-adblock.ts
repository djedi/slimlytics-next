export type AntiAdblockServer = 'caddy' | 'nginx' | 'apache';

export interface AntiAdblockConfig {
  serverType: AntiAdblockServer;
  jsPath: string;
  beaconPath: string;
}

interface ProxySite {
  domain: string;
  writeKey: string;
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const JS_PATH = /^\/[A-Za-z0-9][A-Za-z0-9._~-]{5,62}\.js$/;
const BEACON_PATH = /^\/[A-Za-z0-9][A-Za-z0-9._~-]{5,63}$/;

export function validAntiAdblockPath(path: string, kind: 'js' | 'beacon'): boolean {
  return (kind === 'js' ? JS_PATH : BEACON_PATH).test(path);
}

function assertConfig(config: AntiAdblockConfig, writeKey?: string): void {
  if (!validAntiAdblockPath(config.jsPath, 'js')) throw new Error('Invalid JavaScript path');
  if (!validAntiAdblockPath(config.beaconPath, 'beacon')) throw new Error('Invalid beacon path');
  if (config.jsPath === config.beaconPath) throw new Error('Proxy paths must be different');
  if (writeKey && !UUID.test(writeKey)) throw new Error('Invalid write key');
}

function upstream(value: string): { origin: string; host: string; hostname: string } {
  const url = new URL(value);
  if (url.protocol !== 'https:' && url.protocol !== 'http:') throw new Error('Invalid analytics origin');
  if (url.pathname !== '/' || url.search || url.hash) throw new Error('Analytics origin cannot contain a path');
  return { origin: url.origin, host: url.host, hostname: url.hostname };
}

function regexEscape(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function antiAdblockSnippet(jsPath: string): string {
  if (!validAntiAdblockPath(jsPath, 'js')) throw new Error('Invalid JavaScript path');
  return `<script async src="${jsPath}"></script>`;
}

export function proxyConfig(config: AntiAdblockConfig, site: ProxySite, analyticsOrigin: string): string {
  assertConfig(config, site.writeKey);
  const analytics = upstream(analyticsOrigin);
  const bootstrap = `/p/${site.writeKey}/${config.beaconPath.slice(1)}`;
  const collect = `/api/collect/${site.writeKey}`;

  if (config.serverType === 'caddy') {
    return `### SLIMLYTICS ANTI-ADBLOCK PROXY - https://github.com/djedi/slimlytics-next/blob/main/docs/FIRST_PARTY_PROXY.md
### COPY INTO YOUR WEBSITE'S CADDYFILE

# TRACKING CODE
handle ${config.jsPath} {
\trewrite ${config.jsPath} ${bootstrap}
\treverse_proxy ${analytics.origin} {
\t\theader_up Host {upstream_hostport}
\t\theader_up -Cookie
\t\theader_up -Authorization
\t\theader_down -Set-Cookie
\t}
}

# BEACON
handle ${config.beaconPath} {
\trewrite ${config.beaconPath} ${collect}
\treverse_proxy ${analytics.origin} {
\t\theader_up Host {upstream_hostport}
\t\theader_up -Cookie
\t\theader_up -Authorization
\t\theader_down -Set-Cookie
\t}
}

### /SLIMLYTICS`;
  }

  if (config.serverType === 'nginx') {
    return `# SLIMLYTICS ANTI-ADBLOCK PROXY - https://github.com/djedi/slimlytics-next/blob/main/docs/FIRST_PARTY_PROXY.md
# Add these locations inside your website's server block.

# TRACKING CODE
location = ${config.jsPath} {
    proxy_set_header Host ${analytics.host};
    proxy_set_header Cookie "";
    proxy_set_header Authorization "";
    proxy_set_header X-Forwarded-For $remote_addr;
    proxy_hide_header Set-Cookie;
    proxy_ssl_server_name on;
    proxy_ssl_name ${analytics.hostname};
    proxy_pass ${analytics.origin}${bootstrap};
}

# BEACON
location = ${config.beaconPath} {
    proxy_set_header Host ${analytics.host};
    proxy_set_header Cookie "";
    proxy_set_header Authorization "";
    proxy_set_header X-Forwarded-For $remote_addr;
    proxy_hide_header Set-Cookie;
    proxy_ssl_server_name on;
    proxy_ssl_name ${analytics.hostname};
    proxy_pass ${analytics.origin}${collect};
}`;
  }

  return `# SLIMLYTICS ANTI-ADBLOCK PROXY - https://github.com/djedi/slimlytics-next/blob/main/docs/FIRST_PARTY_PROXY.md
# Requires mod_proxy, mod_proxy_http, mod_ssl, and mod_headers.

SSLProxyEngine On

# TRACKING CODE
ProxyPassMatch "^${regexEscape(config.jsPath)}$" "${analytics.origin}${bootstrap}"

# BEACON
ProxyPassMatch "^${regexEscape(config.beaconPath)}$" "${analytics.origin}${collect}"

<LocationMatch "^(?:${regexEscape(config.jsPath)}|${regexEscape(config.beaconPath)})$">
    RequestHeader unset Cookie
    RequestHeader unset Authorization
    RequestHeader unset X-Forwarded-For
    Header always unset Set-Cookie
</LocationMatch>`;
}

export function proxyTestLinks(domain: string, config: AntiAdblockConfig): { script: string; beacon: string } {
  assertConfig(config);
  const origin = /^https?:\/\//i.test(domain) ? new URL(domain).origin : `https://${domain}`;
  return { script: `${origin}${config.jsPath}`, beacon: `${origin}${config.beaconPath}` };
}

export function trackerBootstrapSource(trackerSource: string, writeKey: string, beaconPath: string): string {
  const config: AntiAdblockConfig = { serverType: 'caddy', jsPath: '/proxyx.js', beaconPath };
  assertConfig(config, writeKey);
  const options = JSON.stringify({ writeKey, endpoint: beaconPath, appendWriteKey: false });
  return `${trackerSource}\n;window.Slimlytics.init(${options});\n`;
}
