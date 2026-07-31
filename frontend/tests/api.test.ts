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
    await expect(api.journeys('northstar')).resolves.toEqual([]);
    await expect(api.searchConsoleStatus('northstar')).resolves.toMatchObject({
      configured: false,
      connected: false
    });
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

  it('reads collection health for operational and agent diagnostics', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      acceptedTotal: 42,
      rejectedTotal: 3,
      lastAcceptedAt: '2026-07-30T22:00:00Z',
      lastRejectedAt: null,
      lastRejectionCode: null,
      lastTrackerVersion: '1.0.0'
    }), { status: 200, headers: { 'content-type': 'application/json' } }));
    const api = new ApiClient('/api', fetcher, false);
    api.setToken('secret');

    await expect(api.collectionHealth('s1')).resolves.toMatchObject({
      acceptedTotal: 42,
      lastTrackerVersion: '1.0.0'
    });
    expect(fetcher).toHaveBeenCalledWith(
      '/api/sites/s1/collection-health',
      expect.objectContaining({ headers: expect.objectContaining({ authorization: 'Bearer secret' }) })
    );
  });

  it('normalizes engagement metrics and daily trend evidence', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      views: { current: 12, previous: 8, change_percent: 50 },
      visitors: { current: 7, previous: 5, change_percent: 40 },
      sessions: { current: 8, previous: 6, change_percent: 33.3 },
      events: { current: 2, previous: 1, change_percent: 100 },
      currentOnline: 1,
      bounceRate: 37.5,
      avgDurationSeconds: 83.2,
      trend: [{ date: '2026-07-30', visitors: 7, pageViews: 12 }]
    }), { status: 200, headers: { 'content-type': 'application/json' } }));
    const api = new ApiClient('/api', fetcher, false);

    await expect(api.overview('s1', 7)).resolves.toMatchObject({
      visitors: 7,
      pageViews: 12,
      bounceRate: 37.5,
      avgDuration: 83.2,
      trend: [{ date: '2026-07-30', visitors: 7, pageViews: 12 }]
    });
  });

  it('normalizes goal conversion evidence returned by the API', async () => {
    const fetcher = vi.fn().mockResolvedValue(new Response(JSON.stringify([{
      id: 'g1',
      name: 'Signup',
      eventName: 'signup',
      pathPattern: null,
      conversions: 9,
      conversionRate: 12.5
    }]), { status: 200, headers: { 'content-type': 'application/json' } }));
    const api = new ApiClient('/api', fetcher, false);

    await expect(api.goals('s1')).resolves.toEqual([{
      id: 'g1',
      name: 'Signup',
      type: 'event',
      target: 'signup',
      conversions: 9,
      conversionRate: 12.5
    }]);
  });

  it('exposes dated marketing intelligence for humans and agents', async () => {
    const responses = [
      [{ steps: ['/', '/pricing'], sessions: 12, visitors: 10 }],
      [{ source: 'newsletter', medium: 'email', campaign: 'launch', visitors: 20, conversions: 4, revenue: 199.8 }],
      [{ date: '2026-07-30', metric: 'pageViews', value: 100, baseline: 50, deviationPercent: 100, direction: 'up' }],
      [{ id: 'f1', name: 'Signup', steps: [{ label: 'Visit', path: '/' }], createdAt: '2026-07-30T00:00:00Z' }],
      { id: 'f1', name: 'Signup', from: '2026-07-03', to: '2026-07-30', steps: [{ index: 1, label: 'Visit', visitors: 10, conversionRate: 100 }] }
    ];
    const fetcher = vi.fn();
    for (const body of responses) {
      fetcher.mockResolvedValueOnce(new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'content-type': 'application/json' }
      }));
    }
    const api = new ApiClient('/api', fetcher, false);

    await expect(api.journeys('s1', 28)).resolves.toHaveLength(1);
    await expect(api.attribution('s1', 28)).resolves.toMatchObject([{ revenue: 199.8 }]);
    await expect(api.anomalies('s1', 28)).resolves.toMatchObject([{ direction: 'up' }]);
    await expect(api.funnels('s1')).resolves.toMatchObject([{ id: 'f1' }]);
    await expect(api.funnelReport('s1', 'f1', 28)).resolves.toMatchObject({ name: 'Signup' });

    expect(fetcher.mock.calls[0][0]).toMatch(/^\/api\/sites\/s1\/insights\/journeys\?from=/);
    expect(fetcher.mock.calls[4][0]).toMatch(/^\/api\/sites\/s1\/funnels\/f1\/report\?from=/);
  });

  it('manages Search Console connection, sync, and reports', async () => {
    const fetcher = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ configured: true, connected: false }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ authorizationUrl: 'https://accounts.google.com/o/oauth2/v2/auth' }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ status: 'ok', rows: 12 }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify([{ value: 'analytics', clicks: 10, impressions: 100, ctr: 0.1, position: 3.2 }]), { status: 200 }));
    const api = new ApiClient('/api', fetcher, false);

    await expect(api.searchConsoleStatus('s1')).resolves.toMatchObject({ configured: true });
    await expect(api.connectSearchConsole('s1')).resolves.toHaveProperty('authorizationUrl');
    await expect(api.syncSearchConsole('s1', 28)).resolves.toMatchObject({ rows: 12 });
    await expect(api.searchConsoleReport('s1', 'query', 28)).resolves.toHaveLength(1);

    expect(fetcher.mock.calls[2][0]).toMatch(/^\/api\/sites\/s1\/integrations\/search-console\/sync\?from=/);
    expect(fetcher.mock.calls[3][0]).toContain('dimension=query');
  });

  it('manages signed scheduled marketing briefs', async () => {
    const subscription = {
      id: 'r1', siteId: 's1', name: 'Weekly brief', webhookUrl: 'https://hooks.example.com/report',
      frequency: 'weekly', anomalyOnly: false, enabled: true,
      nextRunAt: '2026-08-07T12:00:00Z', createdAt: '2026-07-31T12:00:00Z'
    };
    const fetcher = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify([subscription]), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ ...subscription, signingSecret: 'once' }), { status: 201 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ status: 'success' }), { status: 200 }));
    const api = new ApiClient('/api', fetcher, false);

    await expect(api.reportSubscriptions('s1')).resolves.toHaveLength(1);
    await expect(api.createReportSubscription('s1', {
      name: 'Weekly brief', webhookUrl: 'https://hooks.example.com/report',
      frequency: 'weekly', anomalyOnly: false, enabled: true
    })).resolves.toMatchObject({ signingSecret: 'once' });
    await expect(api.deliverReportSubscription('s1', 'r1')).resolves.toEqual({ status: 'success' });
    expect(fetcher.mock.calls[1][1]).toMatchObject({ method: 'POST' });
  });
});
