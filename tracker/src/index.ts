export type Consent = boolean | 'granted' | 'denied';
export type EventProperties = Record<string, string | number | boolean | null>;

export interface TrackerEvent {
  id: string;
  type: 'page' | 'event';
  timestamp: string;
  url: string;
  title?: string;
  referrer?: string;
  name?: string;
  properties?: EventProperties;
}

export interface TrackerPayload {
  sentAt: string;
  events: TrackerEvent[];
}

export type Transport = (url: string, payload: TrackerPayload) => boolean | Promise<boolean>;

export interface TrackerOptions {
  writeKey: string;
  endpoint?: string;
  appendWriteKey?: boolean;
  autoTrack?: boolean;
  batchSize?: number;
  batchInterval?: number;
  respectDnt?: boolean;
  consent?: Consent;
  transport?: Transport;
  downloadExtensions?: string[];
}

const SENSITIVE = /^(token|access_token|auth|authorization|password|passwd|secret|api_?key|email|phone|session|code|signature)$/i;
const DEFAULT_DOWNLOADS = ['pdf', 'zip', 'csv', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'dmg', 'exe', 'mp3', 'mp4'];

export function redactUrl(value: string): string {
  try {
    const base = typeof location === 'undefined' ? 'https://slimlytics.invalid' : location.href;
    const url = new URL(value, base);
    for (const key of [...url.searchParams.keys()]) {
      if (SENSITIVE.test(key)) url.searchParams.set(key, '[REDACTED]');
    }
    url.hash = '';
    return url.toString();
  } catch {
    return value.split('#')[0].replace(/([?&](?:token|password|email|secret)\s*=)[^&]*/gi, '$1%5BREDACTED%5D');
  }
}

function id(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) return crypto.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}-${Math.random().toString(36).slice(2)}`;
}

function privacySignal(): boolean {
  if (typeof navigator === 'undefined') return false;
  const nav = navigator as Navigator & { globalPrivacyControl?: boolean; msDoNotTrack?: string };
  return nav.globalPrivacyControl === true || nav.doNotTrack === '1' || nav.msDoNotTrack === '1';
}

async function defaultTransport(url: string, payload: TrackerPayload): Promise<boolean> {
  const results = await Promise.all(payload.events.map(async (event) => {
    const body = JSON.stringify(toCollectInput(event));
    if (typeof navigator !== 'undefined' && typeof navigator.sendBeacon === 'function') {
      if (navigator.sendBeacon(url, new Blob([body], { type: 'application/json' }))) return true;
    }
    if (typeof fetch !== 'function') return false;
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body,
      keepalive: true,
      credentials: 'omit'
    });
    return response.ok;
  }));
  return results.every(Boolean);
}

export function toCollectInput(event: TrackerEvent) {
  return {
    name: event.type === 'page' ? 'pageview' : event.name,
    url: event.url,
    title: event.title,
    referrer: event.referrer,
    occurredAt: event.timestamp,
    properties: event.properties ?? {},
    screenWidth: typeof screen === 'undefined' ? undefined : screen.width
  };
}

function collectorUrl(endpoint: string, writeKey: string, appendWriteKey = true): string {
  if (!appendWriteKey) return endpoint;
  if (endpoint.includes('{writeKey}')) return endpoint.replace('{writeKey}', encodeURIComponent(writeKey));
  return `${endpoint.replace(/\/$/, '')}/${encodeURIComponent(writeKey)}`;
}

export interface Tracker {
  page(properties?: EventProperties): string | undefined;
  event(name: string, properties?: EventProperties): string | undefined;
  consent(value: Consent): void;
  flush(): Promise<boolean>;
  destroy(): void;
}

export function createTracker(options: TrackerOptions): Tracker {
  if (!options.writeKey?.trim()) throw new Error('Slimlytics: writeKey is required');
  const endpoint = collectorUrl(options.endpoint ?? '/api/collect', options.writeKey, options.appendWriteKey);
  const transport = options.transport ?? defaultTransport;
  const queue: TrackerEvent[] = [];
  const queuedIds = new Set<string>();
  const disposers: Array<() => void> = [];
  const batchSize = Math.max(1, options.batchSize ?? 10);
  let allowed = options.consent !== false && options.consent !== 'denied';
  let destroyed = false;
  let sending: Promise<boolean> | undefined;

  const enabled = () => !destroyed && allowed && !(options.respectDnt !== false && privacySignal());
  const enqueue = (event: TrackerEvent): string | undefined => {
    if (!enabled() || queuedIds.has(event.id)) return undefined;
    queuedIds.add(event.id);
    queue.push(event);
    if (queue.length >= batchSize) void flush();
    return event.id;
  };
  const context = () => ({
    timestamp: new Date().toISOString(),
    url: redactUrl(typeof location === 'undefined' ? '' : location.href)
  });

  const page = (properties?: EventProperties) => enqueue({
    id: id(), type: 'page', ...context(),
    title: typeof document === 'undefined' ? undefined : document.title.slice(0, 500),
    referrer: typeof document === 'undefined' || !document.referrer ? undefined : redactUrl(document.referrer),
    properties
  });
  const event = (name: string, properties?: EventProperties) => {
    if (!name?.trim()) throw new Error('Slimlytics: event name is required');
    return enqueue({ id: id(), type: 'event', ...context(), name: name.slice(0, 120), properties });
  };
  const flush = async (): Promise<boolean> => {
    if (sending) return sending;
    if (!enabled() || queue.length === 0) return false;
    const events = queue.splice(0, batchSize);
    sending = Promise.resolve(transport(endpoint, { sentAt: new Date().toISOString(), events }))
      .then((ok) => {
        if (!ok) queue.unshift(...events);
        else events.forEach((item) => queuedIds.delete(item.id));
        return ok;
      })
      .catch(() => { queue.unshift(...events); return false; })
      .finally(() => { sending = undefined; });
    return sending;
  };

  if (options.autoTrack !== false && typeof window !== 'undefined') {
    page();
    const originalPush = history.pushState;
    const originalReplace = history.replaceState;
    const trackNavigation = () => queueMicrotask(() => page());
    history.pushState = function (...args) { originalPush.apply(this, args); trackNavigation(); };
    history.replaceState = function (...args) { originalReplace.apply(this, args); trackNavigation(); };
    addEventListener('popstate', trackNavigation);
    disposers.push(() => {
      history.pushState = originalPush;
      history.replaceState = originalReplace;
      removeEventListener('popstate', trackNavigation);
    });

    const onClick = (click: MouseEvent) => {
      const target = click.target instanceof Element ? click.target.closest('a[href]') as HTMLAnchorElement | null : null;
      if (!target || target.hasAttribute('data-slimlytics-ignore')) return;
      const url = new URL(target.href, location.href);
      const extension = url.pathname.split('.').pop()?.toLowerCase();
      if (extension && (options.downloadExtensions ?? DEFAULT_DOWNLOADS).includes(extension)) {
        event('download', { url: redactUrl(url.href), label: (target.textContent ?? '').trim().slice(0, 120) });
      } else if (/^https?:$/.test(url.protocol) && url.origin !== location.origin) {
        event('outbound', { url: redactUrl(url.href), label: (target.textContent ?? '').trim().slice(0, 120) });
      }
    };
    document.addEventListener('click', onClick, true);
    disposers.push(() => document.removeEventListener('click', onClick, true));
  }

  const timer = typeof window === 'undefined' ? undefined : window.setInterval(() => void flush(), options.batchInterval ?? 5_000);
  const onHidden = () => { if (document.visibilityState === 'hidden') void flush(); };
  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', onHidden);
    disposers.push(() => document.removeEventListener('visibilitychange', onHidden));
  }

  return {
    page,
    event,
    consent(value) {
      allowed = value === true || value === 'granted';
      if (!allowed) { queue.length = 0; queuedIds.clear(); }
    },
    flush,
    destroy() {
      if (destroyed) return;
      void flush();
      destroyed = true;
      if (timer !== undefined) clearInterval(timer);
      disposers.splice(0).forEach((dispose) => dispose());
    }
  };
}

let singleton: Tracker | undefined;
export function init(options: TrackerOptions): Tracker {
  singleton?.destroy();
  singleton = createTracker(options);
  return singleton;
}
export const page = (properties?: EventProperties) => singleton?.page(properties);
export const event = (name: string, properties?: EventProperties) => singleton?.event(name, properties);
export const consent = (value: Consent) => singleton?.consent(value);

export function trackerOptionsFromScript(script: HTMLScriptElement | null): TrackerOptions | undefined {
  const writeKey = script?.dataset.writeKey?.trim();
  if (!script || !writeKey) return undefined;
  return {
    writeKey,
    endpoint: script.dataset.endpoint || '/api/collect',
    autoTrack: script.dataset.autoTrack !== 'false',
    respectDnt: script.dataset.respectDnt !== 'false',
    consent: script.dataset.consent === 'denied' ? 'denied' : 'granted'
  };
}

if (typeof window !== 'undefined') {
  (window as typeof window & { Slimlytics?: unknown }).Slimlytics = { init, page, event, consent, createTracker };
  const options = trackerOptionsFromScript(document.currentScript as HTMLScriptElement | null);
  if (options) init(options);
}
