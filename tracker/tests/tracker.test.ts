import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createTracker, redactUrl, toCollectInput, trackerOptionsFromScript } from '../src/index';

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('privacy', () => {
  it('redacts sensitive query values and preserves safe parameters', () => {
    expect(redactUrl('https://app.test/a?utm_source=email&token=secret&email=a%40b.com#x'))
      .toBe('https://app.test/a?utm_source=email&token=%5BREDACTED%5D&email=%5BREDACTED%5D');
  });

  it('does not track when DNT or GPC is enabled', async () => {
    Object.defineProperty(navigator, 'doNotTrack', { value: '1', configurable: true });
    const send = vi.fn();
    const tracker = createTracker({ writeKey: 'key', endpoint: '/api/collect', transport: send });
    tracker.page();
    await tracker.flush();
    expect(send).not.toHaveBeenCalled();
  });
});

describe('tracker', () => {
  beforeEach(() => {
    Object.defineProperty(navigator, 'doNotTrack', { value: '0', configurable: true });
    history.replaceState({}, '', '/start?token=nope&utm_source=test');
  });
  afterEach(() => vi.useRealTimers());

  it('is cookieless, batches unique page and custom events, and supports consent', async () => {
    const send = vi.fn().mockResolvedValue(true);
    const tracker = createTracker({ writeKey: 'wk_1', endpoint: '/collect', transport: send, autoTrack: false });
    tracker.page();
    tracker.page();
    tracker.event('signup', { plan: 'pro' });
    await tracker.flush();
    const [url, payload] = send.mock.calls[0];
    expect(url).toBe('/collect/wk_1');
    expect(payload.events).toHaveLength(3);
    expect(new Set(payload.events.map((event: { id: string }) => event.id)).size).toBe(3);
    expect(payload.events[0].url).not.toContain('nope');
    expect(document.cookie).toBe('');
    tracker.consent(false);
    tracker.event('blocked');
    await tracker.flush();
    expect(send).toHaveBeenCalledTimes(1);
  });

  it('tracks SPA navigation, downloads and outbound links without form values', async () => {
    const send = vi.fn().mockResolvedValue(true);
    const tracker = createTracker({ writeKey: 'wk', transport: send, autoTrack: true, batchInterval: 10_000 });
    // Auto-track flushes the first pageview immediately.
    await tick();
    expect(send).toHaveBeenCalled();
    history.pushState({}, '', '/next');
    await tick();
    const download = document.createElement('a');
    download.href = '/report.pdf';
    download.textContent = 'report';
    document.body.append(download);
    download.addEventListener('click', (click) => click.preventDefault());
    download.click();
    const outbound = document.createElement('a');
    outbound.href = 'https://outside.test/path';
    document.body.append(outbound);
    outbound.addEventListener('click', (click) => click.preventDefault());
    outbound.click();
    const form = document.createElement('form');
    form.innerHTML = '<input value="private">';
    document.body.append(form);
    form.dispatchEvent(new Event('submit', { bubbles: true }));
    await tracker.flush();
    const events = send.mock.calls.flatMap((call) => call[1].events as Array<{ type: string; name?: string }>);
    expect(events.some((event) => event.type === 'page')).toBe(true);
    expect(events.some((event) => event.name === 'download')).toBe(true);
    expect(events.some((event) => event.name === 'outbound')).toBe(true);
    expect(JSON.stringify(events)).not.toContain('private');
    tracker.destroy();
  });

  it('bootstraps options from installation script attributes', () => {
    const script = document.createElement('script');
    script.dataset.writeKey = 'wk_anti';
    script.dataset.endpoint = 'https://analytics.example/api/e';
    script.dataset.autoTrack = 'false';
    script.dataset.respectDnt = 'true';
    script.dataset.consent = 'denied';

    expect(trackerOptionsFromScript(script)).toEqual({
      writeKey: 'wk_anti',
      endpoint: 'https://analytics.example/api/e',
      autoTrack: false,
      respectDnt: true,
      consent: 'denied'
    });
    expect(trackerOptionsFromScript(document.createElement('script'))).toBeUndefined();
  });

  it('can send to an exact first-party beacon path without appending the write key', async () => {
    const urls: string[] = [];
    const tracker = createTracker({
      writeKey: 'site-key',
      endpoint: '/0d31360a3101',
      appendWriteKey: false,
      autoTrack: false,
      transport: async (url) => { urls.push(url); return true; }
    });
    tracker.page();
    await tracker.flush();
    expect(urls).toEqual(['/0d31360a3101']);
    tracker.destroy();
  });
});

describe('default transport', () => {
  it('maps queued events to the collector API contract', () => {
    const mapped = toCollectInput({
      id: 'event-1',
      type: 'event',
      name: 'signup',
      timestamp: '2026-07-29T00:00:00Z',
      url: 'https://example.com/thanks',
      properties: { plan: 'pro' }
    });
    expect(mapped).toMatchObject({
      name: 'signup',
      occurredAt: '2026-07-29T00:00:00Z',
      url: 'https://example.com/thanks',
      properties: { plan: 'pro' }
    });
  });

  it('prefers fetch with JSON content-type over sendBeacon', async () => {
    const beacon = vi.fn().mockReturnValue(true);
    Object.defineProperty(navigator, 'sendBeacon', { value: beacon, configurable: true });
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(null, { status: 202 }));
    const tracker = createTracker({ writeKey: 'wk', autoTrack: false });
    tracker.event('ping');
    await tracker.flush();
    expect(fetchSpy).toHaveBeenCalledOnce();
    expect(fetchSpy.mock.calls[0][1]).toMatchObject({
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      keepalive: true,
      credentials: 'omit'
    });
    expect(beacon).not.toHaveBeenCalled();
  });

  it('omits blank referrers from the collector payload', () => {
    const mapped = toCollectInput({
      id: 'event-2',
      type: 'page',
      timestamp: '2026-07-29T00:00:00Z',
      url: 'https://example.com/',
      referrer: '   '
    });
    expect(mapped.referrer).toBeUndefined();
  });
});
