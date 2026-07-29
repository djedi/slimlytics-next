export interface TrendPoint { date: string; visitors: number; pageViews: number }
export interface Overview { visitors: number; sessions: number; pageViews: number; bounceRate: number; avgDuration: number; change: number; currentOnline: number; trend: TrendPoint[] }
export interface Site { id: string; name: string; domain: string; writeKey: string; timezone?: string; allowedOrigins?: string[]; retentionDays?: number; overview?: Overview }
export interface User { id: string; email: string; name?: string }
export interface AuthResponse { token?: string; accessToken?: string; user: User }
export interface ReportRow { label: string; value: number; secondary?: string; change?: number }
export interface Visitor { id: string; country: string; city?: string; device?: string; browser?: string; page?: string; lastSeen?: string; sessions?: number }
export interface LiveEvent { id: string; type: string; page: string; visitorId?: string; country?: string; city?: string; timestamp: string; referrer?: string }
export interface Goal { id: string; name: string; type: string; target: string; conversions?: number; conversionRate?: number }

export class ApiError extends Error {
  constructor(public status: number, message: string) { super(message); this.name = 'ApiError'; }
}

type Fetcher = typeof fetch;
interface WireSite extends Omit<Site, 'writeKey'> { write_key?: string; writeKey?: string }
interface WireMetric { current: number; previous: number; change_percent?: number | null }
interface WireOverview { views: WireMetric; visitors: WireMetric; sessions: WireMetric; events: WireMetric }
interface WireReportRow { value: string; views: number; visitors: number }
interface WireGoal { id: string; name: string; eventName?: string; event_name?: string; pathPattern?: string; path_pattern?: string }
function dates(days: number) {
  const to = new Date();
  const from = new Date(to.getTime() - Math.max(0, days - 1) * 864e5);
  return { from: from.toISOString().slice(0, 10), to: to.toISOString().slice(0, 10) };
}
function dateQuery(days: number) { const range = dates(days); return `from=${range.from}&to=${range.to}`; }
function normalizeSite(site: WireSite): Site { return { ...site, writeKey: site.writeKey ?? site.write_key ?? '' }; }
const trend = Array.from({ length: 28 }, (_, index) => ({ date: new Date(Date.now() - (27 - index) * 864e5).toISOString().slice(0, 10), visitors: 84 + ((index * 17) % 71), pageViews: 151 + ((index * 29) % 129) }));
const baseOverview: Overview = { visitors: 3421, sessions: 3892, pageViews: 8754, bounceRate: 38.4, avgDuration: 164, change: 12.8, currentOnline: 14, trend };
export const demoSites: Site[] = [
  { id: 'northstar', name: 'Northstar Docs', domain: 'docs.northstar.dev', writeKey: 'wk_demo_docs', overview: baseOverview },
  { id: 'journal', name: 'Field Journal', domain: 'journal.example.com', writeKey: 'wk_demo_journal', overview: { ...baseOverview, visitors: 1886, pageViews: 5102, currentOnline: 6, change: 4.2 } },
  { id: 'store', name: 'Little Supply Co.', domain: 'shop.example.com', writeKey: 'wk_demo_shop', overview: { ...baseOverview, visitors: 956, pageViews: 2901, currentOnline: 2, change: -3.7 } }
];

const reportLabels: Record<string, string[]> = {
  pages: ['/docs/getting-started', '/', '/pricing', '/blog/privacy-first-analytics', '/integrations'],
  referrers: ['Google', 'Direct / none', 'github.com', 'Bing', 'newsletter'],
  countries: ['United States', 'Germany', 'United Kingdom', 'Canada', 'Japan'],
  devices: ['Desktop · Chrome', 'Mobile · Safari', 'Desktop · Firefox', 'Mobile · Chrome', 'Tablet · Safari'],
  campaigns: ['spring_launch / email', 'docs_sidebar / referral', 'brand / search', 'july_digest / email', '(not set)']
};
export const demoReport = (type: string): ReportRow[] => (reportLabels[type] ?? []).map((label, i) => ({ label, value: 1420 - i * 219, secondary: `${Math.max(5, 38 - i * 6)}%`, change: i % 2 ? -2.1 : 8.4 }));

export class ApiClient {
  private token = '';
  private base: string;
  constructor(base = '/api', private fetcher: Fetcher = fetch, private demo = false) { this.base = base.replace(/\/$/, ''); }
  setToken(token: string) { this.token = token; }
  private async request<T>(path: string, init: RequestInit = {}, fallback?: () => T): Promise<T> {
    const headers: Record<string, string> = { accept: 'application/json', ...(init.body ? { 'content-type': 'application/json' } : {}), ...(this.token ? { authorization: `Bearer ${this.token}` } : {}), ...(init.headers as Record<string, string> ?? {}) };
    try {
      const response = await this.fetcher(`${this.base}${path}`, { ...init, headers });
      if (!response.ok) {
        let message = `Request failed (${response.status})`;
        try {
          const body = await response.json() as { message?: string; error?: string | { message?: string } };
          message = body.message ?? (typeof body.error === 'string' ? body.error : body.error?.message) ?? message;
        } catch { /* non-JSON */ }
        throw new ApiError(response.status, message);
      }
      if (response.status === 204) return undefined as T;
      return await response.json() as T;
    } catch (error) {
      if (error instanceof ApiError || !this.demo || !fallback) throw error;
      return structuredClone(fallback());
    }
  }
  register(email: string, password: string, name = '') { return this.request<AuthResponse>('/auth/register', { method: 'POST', body: JSON.stringify({ email, password, name }) }); }
  login(email: string, password: string) { return this.request<AuthResponse>('/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) }, () => ({ accessToken: 'demo', user: { id: 'demo', email } })); }
  me() { return this.request<User>('/auth/me'); }
  async sites() { return (await this.request<WireSite[]>('/sites', {}, () => demoSites)).map(normalizeSite); }
  async site(id: string) { return normalizeSite(await this.request<WireSite>(`/sites/${id}`, {}, () => demoSites.find((site) => site.id === id) ?? demoSites[0])); }
  async createSite(site: Pick<Site, 'name' | 'domain'> & { allowedOrigins?: string[] }) {
    return normalizeSite(await this.request<WireSite>('/sites', { method: 'POST', body: JSON.stringify({ ...site, timezone: 'UTC', retentionDays: 365 }) }, () => ({ ...site, id: crypto.randomUUID(), writeKey: `wk_demo_${Date.now()}` })));
  }
  async updateSite(id: string, site: Partial<Site>) { return normalizeSite(await this.request<WireSite>(`/sites/${id}`, { method: 'PUT', body: JSON.stringify(site) }, () => ({ ...demoSites[0], ...site, id }))); }
  deleteSite(id: string) { return this.request<void>(`/sites/${id}`, { method: 'DELETE' }, () => undefined); }
  async overview(id: string, days = 28): Promise<Overview> {
    const wire = await this.request<WireOverview>(`/sites/${id}/overview?${dateQuery(days)}`, {}, () => ({ views: { current: baseOverview.pageViews, previous: 0 }, visitors: { current: baseOverview.visitors, previous: 0 }, sessions: { current: baseOverview.sessions, previous: 0 }, events: { current: 0, previous: 0 } }));
    const change = wire.visitors.change_percent ?? wire.views.change_percent ?? 0;
    return { visitors: wire.visitors.current, sessions: wire.sessions.current, pageViews: wire.views.current, bounceRate: 0, avgDuration: 0, change, currentOnline: 0, trend: [] };
  }
  async report(id: string, type: string, days = 28) { const rows = await this.request<WireReportRow[]>(`/sites/${id}/reports/${type}?${dateQuery(days)}`, {}, () => []); return rows.map((row) => ({ label: row.value, value: row.views, secondary: `${row.visitors} visitors` })); }
  visitors(id: string, days = 28) { return this.request<Visitor[]>(`/sites/${id}/visitors?${dateQuery(days)}`, {}, () => [{ id: 'v1', country: 'United States', city: 'Portland', device: 'Desktop', browser: 'Firefox', page: '/docs', sessions: 4, lastSeen: new Date().toISOString() }]); }
  events(id: string, days = 28) { return this.request<LiveEvent[]>(`/sites/${id}/events?${dateQuery(days)}`, {}, () => [{ id: 'e1', type: 'pageview', page: '/docs/getting-started', visitorId: 'v1', country: 'US', city: 'Portland', timestamp: new Date().toISOString(), referrer: 'Google' }]); }
  async goals(id: string) { const rows = await this.request<WireGoal[]>(`/sites/${id}/goals`, {}, () => []); return rows.map((goal) => ({ id: goal.id, name: goal.name, type: 'event', target: goal.eventName ?? goal.event_name ?? '', conversions: 0, conversionRate: 0 })); }
  async createGoal(id: string, goal: Omit<Goal, 'id'>) { const row = await this.request<WireGoal>(`/sites/${id}/goals`, { method: 'POST', body: JSON.stringify({ name: goal.name, eventName: goal.target }) }, () => ({ ...goal, id: crypto.randomUUID(), eventName: goal.target })); return { id: row.id, name: row.name, type: 'event', target: row.eventName ?? row.event_name ?? goal.target, conversions: 0, conversionRate: 0 }; }
  exportUrl(id: string, days: number) { return `${this.base}/sites/${id}/export.csv?${dateQuery(days)}`; }
  streamUrl(id: string, token?: string) { const query = token ? `?token=${encodeURIComponent(token)}` : ''; return `${this.base}/sites/${id}/stream${query}`; }
  async downloadExport(id: string, days: number) {
    const response = await this.fetcher(`${this.base}/sites/${id}/export.csv?${dateQuery(days)}`, { headers: this.token ? { authorization: `Bearer ${this.token}` } : {} });
    if (!response.ok) throw new ApiError(response.status, `Export failed (${response.status})`);
    return response.blob();
  }
}
