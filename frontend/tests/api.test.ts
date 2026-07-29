import { describe, expect, it, vi } from 'vitest';
import { ApiClient, ApiError } from '../src/lib/api';

describe('ApiClient', () => {
  it('uses the configured base URL, bearer auth, and camelCase contract', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({ id: 's1', writeKey: 'wk' }), { status: 200, headers: { 'content-type': 'application/json' } }));
    const api = new ApiClient('https://example.test/api/', fetcher, false);
    api.setToken('secret');
    await api.site('s1');
    expect(fetcher).toHaveBeenCalledWith('https://example.test/api/sites/s1', expect.objectContaining({ headers: expect.objectContaining({ authorization: 'Bearer secret' }) }));
  });

  it('throws a typed error and never silently returns demo data', async () => {
    const api = new ApiClient('/api', vi.fn().mockResolvedValue(new Response('{"message":"Nope"}', { status: 403 })), false);
    await expect(api.sites()).rejects.toMatchObject({ status: 403, message: 'Nope' } satisfies Partial<ApiError>);
  });

  it('provides realistic fallback only when demo mode is explicit', async () => {
    const api = new ApiClient('/api', vi.fn().mockRejectedValue(new Error('offline')), true);
    const sites = await api.sites();
    expect(sites).toHaveLength(3);
    expect(sites[0].overview?.trend).toHaveLength(28);
  });

  it('persists per-site anti-adblock server and path settings', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      id: 's1',
      name: 'Example',
      domain: 'example.com',
      writeKey: 'wk',
      antiAdblockServer: 'caddy',
      antiAdblockJsPath: '/456bbb63bb86.js',
      antiAdblockBeaconPath: '/0d31360a3101'
    }), { status: 200, headers: { 'content-type': 'application/json' } }));
    const api = new ApiClient('/api', fetcher, false);
    api.setToken('secret');
    await api.updateAntiAdblock('s1', {
      serverType: 'caddy',
      jsPath: '/456bbb63bb86.js',
      beaconPath: '/0d31360a3101'
    });
    expect(fetcher).toHaveBeenCalledWith('/api/sites/s1/anti-adblock', expect.objectContaining({
      method: 'PUT',
      body: JSON.stringify({ serverType: 'caddy', jsPath: '/456bbb63bb86.js', beaconPath: '/0d31360a3101' })
    }));
  });
});
