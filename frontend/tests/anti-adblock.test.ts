import { describe, expect, it } from 'vitest';
import {
  antiAdblockSnippet,
  proxyConfig,
  proxyTestLinks,
  trackerBootstrapSource,
  validAntiAdblockPath,
  type AntiAdblockConfig
} from '../src/lib/anti-adblock';

const config: AntiAdblockConfig = {
  serverType: 'caddy',
  jsPath: '/456bbb63bb86.js',
  beaconPath: '/0d31360a3101'
};

const site = {
  domain: 'example.com',
  writeKey: 'd8f6f152-7a9e-4eb9-a8a1-468db4c0ea33'
};

it('uses the same minimum path length as the backend validator', () => {
  expect(validAntiAdblockPath('/abcde.js', 'js')).toBe(false);
  expect(validAntiAdblockPath('/abcdef.js', 'js')).toBe(true);
});

it('generates the same minimal same-origin script shape as Clicky', () => {
  expect(antiAdblockSnippet(config.jsPath)).toBe(
    '<script async src="/456bbb63bb86.js"></script>'
  );
});

describe('proxyConfig', () => {
  it('generates a Caddy configuration for both exact paths', () => {
    const output = proxyConfig(config, site, 'https://slimlytics.com');
    expect(output).toContain('handle /456bbb63bb86.js {');
    expect(output).toContain('rewrite /456bbb63bb86.js /p/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33/0d31360a3101');
    expect(output).toContain('handle /0d31360a3101 {');
    expect(output).toContain('rewrite /0d31360a3101 /api/collect/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33');
    expect(output.match(/reverse_proxy https:\/\/slimlytics\.com/g)).toHaveLength(2);
  });

  it('generates an Nginx configuration with TLS SNI and both paths', () => {
    const output = proxyConfig({ ...config, serverType: 'nginx' }, site, 'https://slimlytics.com');
    expect(output).toContain('location = /456bbb63bb86.js {');
    expect(output).toContain('proxy_ssl_server_name on;');
    expect(output).toContain('proxy_pass https://slimlytics.com/p/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33/0d31360a3101;');
    expect(output).toContain('location = /0d31360a3101 {');
  });

  it('generates an Apache configuration with anchored proxy targets', () => {
    const output = proxyConfig({ ...config, serverType: 'apache' }, site, 'https://slimlytics.com');
    expect(output).toContain('SSLProxyEngine On');
    expect(output).toContain('ProxyPassMatch "^/456bbb63bb86\\.js$" "https://slimlytics.com/p/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33/0d31360a3101"');
    expect(output).toContain('ProxyPassMatch "^/0d31360a3101$" "https://slimlytics.com/api/collect/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33"');
  });

  it.each(['caddy', 'nginx', 'apache'] as const)('strips credentials and upstream cookies in %s config', (serverType) => {
    const output = proxyConfig({ ...config, serverType }, site, 'https://slimlytics.com');
    expect(output).toMatch(/Cookie/);
    expect(output).toMatch(/Authorization/);
    expect(output).toMatch(/Set-Cookie/);
  });

  it('sets HTTPS SNI and a non-spoofable client address in Nginx config', () => {
    const output = proxyConfig({ ...config, serverType: 'nginx' }, site, 'https://slimlytics.com');
    expect(output).toContain('proxy_ssl_name slimlytics.com;');
    expect(output).toContain('proxy_set_header X-Forwarded-For $remote_addr;');
  });

  it('removes a browser-supplied client address before Apache adds its proxy address', () => {
    const output = proxyConfig({ ...config, serverType: 'apache' }, site, 'https://slimlytics.com');
    expect(output).toContain('RequestHeader unset X-Forwarded-For');
  });
});

it('creates same-origin links that test the configured JavaScript and beacon paths', () => {
  expect(proxyTestLinks(site.domain, config)).toEqual({
    script: 'https://example.com/456bbb63bb86.js',
    beacon: 'https://example.com/0d31360a3101'
  });
});

it('creates a bootstrap payload that initializes the bundled tracker with the exact beacon path', () => {
  const source = trackerBootstrapSource('/* tracker */', site.writeKey, config.beaconPath);
  expect(source).toContain('/* tracker */');
  expect(source).toContain('window.Slimlytics.init(');
  expect(source).toContain('"writeKey":"d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33"');
  expect(source).toContain('"endpoint":"/0d31360a3101"');
  expect(source).toContain('"appendWriteKey":false');
});

it('rejects unsafe bootstrap parameters instead of emitting executable input', () => {
  expect(() => trackerBootstrapSource('', 'not-a-uuid', '/0d31360a3101')).toThrow();
  expect(() => trackerBootstrapSource('', site.writeKey, '/bad\";alert(1)//')).toThrow();
});
