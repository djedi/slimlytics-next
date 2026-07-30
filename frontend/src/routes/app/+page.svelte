<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { env } from '$env/dynamic/public';
  import {
    Activity,
    BarChart3,
    Bell,
    CalendarDays,
    ChevronDown,
    CircleDot,
    Download,
    Eye,
    FileText,
    Gauge,
    Goal as GoalIcon,
    Globe2,
    LayoutDashboard,
    LogOut,
    Menu,
    Monitor,
    Moon,
    Pause,
    Play,
    Plus,
    Search,
    Settings,
    Smartphone,
    Sun,
    Users,
    X,
    Zap
  } from '@lucide/svelte';
  import {
    ApiClient,
    demoReport,
    type AntiAdblockSettings,
    type Goal,
    type LiveEvent,
    type Overview,
    type ReportRow,
    type Site,
    type Visitor
  } from '$lib/api';
  import { applyTheme, duration, sparklinePoints, type Theme } from '$lib/ui';
  import AntiAdblockSettingsPanel from '$lib/components/AntiAdblockSettings.svelte';
  import ReportTable from '$lib/components/ReportTable.svelte';
  import WorldMap from '$lib/components/WorldMap.svelte';

  type View =
    | 'rollup'
    | 'overview'
    | 'spy'
    | 'pages'
    | 'referrers'
    | 'countries'
    | 'devices'
    | 'campaigns'
    | 'visitors'
    | 'goals'
    | 'settings';
  const demo = env.PUBLIC_DEMO_MODE === 'true';
  const api = new ApiClient(env.PUBLIC_API_BASE_URL || '/api', fetch, demo);
  const nav: Array<{ id: View; label: string; icon: typeof Activity }> = [
    { id: 'overview', label: 'Overview', icon: Gauge },
    { id: 'spy', label: 'Spy', icon: Eye },
    { id: 'pages', label: 'Pages', icon: FileText },
    { id: 'referrers', label: 'Referrers', icon: Activity },
    { id: 'countries', label: 'Countries', icon: Globe2 },
    { id: 'devices', label: 'Devices', icon: Monitor },
    { id: 'campaigns', label: 'Campaigns', icon: Zap },
    { id: 'visitors', label: 'Visitors', icon: Users },
    { id: 'goals', label: 'Goals', icon: GoalIcon },
    { id: 'settings', label: 'Settings', icon: Settings }
  ];
  let ready = $state(false);
  let sites = $state<Site[]>([]);
  let site = $state<Site | null>(null);
  let view = $state<View>('rollup');
  let days = $state(28);
  let loading = $state(false);
  let error = $state('');
  let menuOpen = $state(false);
  let overview = $state<Overview | null>(null);
  let report = $state<ReportRow[]>([]);
  let topPages = $state<ReportRow[]>([]);
  let topReferrers = $state<ReportRow[]>([]);
  let visitors = $state<Visitor[]>([]);
  let events = $state<LiveEvent[]>([]);
  let goals = $state<Goal[]>([]);
  let paused = $state(false);
  let spyFilter = $state('');
  let selectedVisitor = $state<Visitor | null>(null);
  let source: EventSource | null = null;
  let theme = $state<Theme>('system');
  let newGoal = $state(false);
  let goalName = $state('');
  let goalTarget = $state('');
  let newSite = $state(false);
  let siteName = $state('');
  let siteDomain = $state('');
  let siteError = $state('');
  let token = '';

  onMount(() => {
    token = localStorage.getItem('slimlytics_token') ?? '';
    theme = (localStorage.getItem('slimlytics_theme') ?? 'system') as Theme;
    if (!token && !demo) {
      void goto('/login');
      return;
    }
    api.setToken(token || 'demo');
    ready = true;
    void loadSites();
    return () => source?.close();
  });

  function logout() {
    localStorage.removeItem('slimlytics_token');
    source?.close();
    void goto('/');
  }
  async function loadSites() {
    loading = true;
    error = '';
    try {
      const loaded = await api.sites();
      sites = await Promise.all(
        loaded.map(async (item) => {
          try {
            return { ...item, overview: await api.overview(item.id, days) };
          } catch {
            return item;
          }
        })
      );
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'Could not load sites.';
    } finally {
      loading = false;
    }
  }
  async function selectSite(next: Site) {
    site = next;
    view = 'overview';
    menuOpen = false;
    await loadView();
  }
  async function setView(next: View) {
    view = next;
    menuOpen = false;
    await loadView();
  }
  async function loadView() {
    if (!site) return;
    loading = true;
    error = '';
    source?.close();
    try {
      if (view === 'overview') {
        const [nextOverview, pages, referrers] = await Promise.all([
          api.overview(site.id, days),
          api.report(site.id, 'pages', days),
          api.report(site.id, 'referrers', days)
        ]);
        overview = nextOverview;
        topPages = (demo ? demoReport('pages') : pages).slice(0, 5);
        topReferrers = (demo ? demoReport('referrers') : referrers).slice(0, 5);
      } else if (['pages', 'referrers', 'countries', 'devices', 'campaigns'].includes(view))
        report = await api.report(site.id, view, days);
      else if (view === 'visitors') visitors = await api.visitors(site.id);
      else if (view === 'spy') {
        events = await api.events(site.id);
        visitors = await api.visitors(site.id);
        connectSpy();
      } else if (view === 'goals') goals = await api.goals(site.id);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'Could not load analytics.';
    } finally {
      loading = false;
    }
  }
  function connectSpy() {
    if (!site || paused || typeof EventSource === 'undefined' || demo) return;
    source?.close();
    source = new EventSource(api.streamUrl(site.id, token));
    const receive = ({ data }: MessageEvent<string>) => {
      try {
        const item = JSON.parse(data) as LiveEvent;
        events = [item, ...events].slice(0, 100);
      } catch {
        /* malformed event */
      }
    };
    source.onmessage = receive;
    source.addEventListener('event', receive as EventListener);
    // Browsers reconnect EventSource automatically; only tear down when we mean to.
    source.onerror = () => {
      if (paused || view !== 'spy') source?.close();
    };
  }
  function toggleSpy() {
    paused = !paused;
    if (paused) source?.close();
    else connectSpy();
  }
  function updateTheme(next: Theme) {
    theme = next;
    localStorage.setItem('slimlytics_theme', next);
    applyTheme(next);
  }
  async function saveAntiAdblock(settings: AntiAdblockSettings) {
    if (!site) return;
    const updated = await api.updateAntiAdblock(site.id, settings);
    const next = { ...updated, overview: site.overview };
    site = next;
    sites = sites.map((item) => (item.id === next.id ? { ...next, overview: item.overview } : item));
  }
  async function addGoal() {
    if (!site || !goalName || !goalTarget) return;
    const goal = await api.createGoal(site.id, { name: goalName, target: goalTarget, type: 'event' });
    goals = [...goals, goal];
    newGoal = false;
    goalName = '';
    goalTarget = '';
  }
  async function addSite() {
    if (!siteName.trim() || !siteDomain.trim()) return;
    siteError = '';
    try {
      const raw = siteDomain.trim().replace(/\/$/, '');
      const origin = /^https?:\/\//i.test(raw) ? new URL(raw).origin : `https://${raw}`;
      const domain = new URL(origin).host;
      const created = await api.createSite({
        name: siteName.trim(),
        domain,
        allowedOrigins: [origin]
      });
      sites = [...sites, { ...created, overview: await api.overview(created.id, days) }];
      newSite = false;
      siteName = '';
      siteDomain = '';
      await selectSite(created);
    } catch (reason) {
      siteError = reason instanceof Error ? reason.message : 'Could not create site.';
    }
  }
  async function downloadCsv() {
    if (!site) return;
    try {
      const blob = await api.downloadExport(site.id, days);
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = `${site.domain}-events.csv`;
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'Could not export events.';
    }
  }
  const filteredEvents = $derived(
    events.filter((item) =>
      `${item.page} ${item.country} ${item.city} ${item.type}`
        .toLowerCase()
        .includes(spyFilter.toLowerCase())
    )
  );
  const mapVisitors = $derived(
    (visitors.length
      ? visitors
      : [{ country: 'United States' }, { country: 'Germany' }, { country: 'Japan' }]
    )
      .slice(0, 6)
      .map((item, index) => ({
        country: item.country,
        code: item.country.slice(0, 2).toUpperCase(),
        x: [25, 52, 83, 47, 69, 32][index],
        y: [37, 31, 42, 60, 65, 73][index],
        count: [12, 7, 5, 4, 3, 2][index]
      }))
  );
</script>

<svelte:head>
  <title>Dashboard · Slimlytics</title>
</svelte:head>

{#if ready}
  <div class="app-shell">
    <aside class:open={menuOpen} aria-label="Primary navigation">
      <div class="sidebar-brand">
        <button
          class="brand"
          onclick={() => {
            site = null;
            view = 'rollup';
          }}
        >
          <span class="brand-mark"><BarChart3 size={20} /></span><strong>Slimlytics</strong>
        </button>
        <button class="icon-button close-menu" aria-label="Close menu" onclick={() => (menuOpen = false)}
          ><X /></button
        >
      </div>
      <button
        class="site-picker"
        onclick={() => {
          site = null;
          view = 'rollup';
        }}
      >
        <span class="site-avatar">{site ? site.name.slice(0, 2).toUpperCase() : 'ALL'}</span>
        <span
          ><small>{site ? 'Current site' : 'Workspace'}</small><strong
            >{site?.name ?? 'All sites'}</strong
          ></span
        ><ChevronDown size={15} />
      </button>
      {#if site}
        <nav>
          {#each nav as item}
            <button class:active={view === item.id} onclick={() => void setView(item.id)}
              ><item.icon size={17} /><span>{item.label}</span>{#if item.id === 'spy'}<i
                ></i
              >{/if}</button
            >
          {/each}
        </nav>
      {:else}
        <nav><button class="active"><LayoutDashboard size={17} />All sites</button></nav>
      {/if}
      <div class="sidebar-foot">
        <div class="online">
          <span></span
          >{sites.reduce((sum, current) => sum + (current.overview?.currentOnline ?? 0), 0)} online
          now
        </div>
        <button onclick={logout}><LogOut size={16} />Sign out</button>
      </div>
    </aside>
    {#if menuOpen}
      <button class="scrim" aria-label="Close menu" onclick={() => (menuOpen = false)}></button>
    {/if}
    <main id="main" class="workspace">
      <header class="topbar">
        <button class="icon-button menu-button" aria-label="Open menu" onclick={() => (menuOpen = true)}
          ><Menu /></button
        >
        <div>
          <p class="breadcrumb">{site ? site.domain : 'Workspace'} <span>/</span></p>
          <h1>{site ? (nav.find((item) => item.id === view)?.label ?? 'Overview') : 'All sites'}</h1>
        </div>
        <div class="top-actions">
          <label class="date-picker"
            ><CalendarDays size={16} /><span class="sr-only">Date range</span
            ><select bind:value={days} onchange={() => void loadView()}
              ><option value={7}>Last 7 days</option><option value={28}>Last 28 days</option
              ><option value={90}>Last 90 days</option></select
            ></label
          ><button class="icon-button" aria-label="Notifications"><Bell size={18} /></button
          ><button class="avatar" aria-label="Account menu">DU</button>
        </div>
      </header>
      {#if error}
        <div class="alert page-alert" role="alert">
          <span>{error}</span><button onclick={() => void loadView()}>Retry</button>
        </div>
      {/if}
      {#if loading}
        <div class="loading" role="status"><span></span><p>Loading analytics…</p></div>
      {:else if !site}
        <section class="page-head">
          <div>
            <p class="eyebrow">Portfolio pulse</p>
            <h2>Your sites at a glance</h2>
            <p class="muted">Traffic across all properties during the last {days} days.</p>
          </div>
          <button class="primary" onclick={() => (newSite = true)}><Plus size={16} /> Add site</button>
        </section>
        {#if sites.length}
          <div class="site-grid">
            {#each sites as item}
              <button class="site-card" onclick={() => void selectSite(item)}
                ><div class="card-head">
                  <div>
                    <span class="site-avatar">{item.name.slice(0, 2).toUpperCase()}</span><span
                      ><strong>{item.name}</strong><small>{item.domain}</small></span
                    >
                  </div>
                  <div class="live"><i></i>{item.overview?.currentOnline ?? 0} live</div>
                </div>
                <svg
                  class="sparkline"
                  viewBox="0 0 300 70"
                  aria-label={`${item.name} 28-day traffic trend`}
                  role="img"
                  ><defs
                    ><linearGradient id={`fade-${item.id}`} x1="0" y1="0" x2="0" y2="1"
                      ><stop offset="0" stop-color="var(--accent)" stop-opacity=".28" /><stop
                        offset="1"
                        stop-color="var(--accent)"
                        stop-opacity="0"
                      /></linearGradient
                    ></defs
                  ><polygon
                    points={`0,70 ${sparklinePoints(item.overview?.trend.map((p) => p.visitors) ?? [], 300, 58)} 300,70`}
                    fill={`url(#fade-${item.id})`}
                  /><polyline
                    points={sparklinePoints(
                      item.overview?.trend.map((p) => p.visitors) ?? [],
                      300,
                      58
                    )}
                  /></svg
                ><div class="metrics compact">
                  <div>
                    <small>Visitors</small><strong>{item.overview?.visitors.toLocaleString()}</strong>
                  </div>
                  <div>
                    <small>Page views</small><strong
                      >{item.overview?.pageViews.toLocaleString()}</strong
                    >
                  </div>
                  <div>
                    <small>Change</small><strong
                      class:negative={(item.overview?.change ?? 0) < 0}
                      class="positive"
                      >{(item.overview?.change ?? 0) > 0 ? '+' : ''}{item.overview?.change}%</strong
                    >
                  </div>
                </div></button
              >
            {/each}
          </div>
        {:else}
          <div class="empty large">
            <Globe2 size={34} /><h2>No sites yet</h2>
            <p>Add your first website to begin measuring private, useful analytics.</p>
            <button class="primary" onclick={() => (newSite = true)}
              ><Plus />Add your first site</button
            >
          </div>
        {/if}
        {#if newSite}
          <div class="modal-backdrop" role="presentation"
            ><form
              class="modal"
              onsubmit={(event) => {
                event.preventDefault();
                void addSite();
              }}
              ><div class="panel-head">
                <h2>Add a site</h2>
                <button
                  type="button"
                  class="icon-button"
                  onclick={() => (newSite = false)}
                  aria-label="Close add site dialog"><X /></button
                >
              </div>
              <label>Site name<input bind:value={siteName} required placeholder="My website" /></label
              ><label
                >Domain<input bind:value={siteDomain} required placeholder="example.com" /></label
              >{#if siteError}<div class="alert" role="alert">{siteError}</div>{/if}<p class="muted">
                HTTPS will be used as the initial allowed tracking origin.
              </p>
              <div class="modal-actions">
                <button type="button" class="secondary" onclick={() => (newSite = false)}>Cancel</button
                ><button class="primary">Create site</button>
              </div></form
            ></div
          >
        {/if}
      {:else if view === 'overview' && overview}
        <section class="metric-grid">
          {#each [{ l: 'Visitors', v: overview.visitors.toLocaleString(), i: Users }, { l: 'Sessions', v: overview.sessions.toLocaleString(), i: Activity }, { l: 'Page views', v: overview.pageViews.toLocaleString(), i: FileText }, { l: 'Bounce rate', v: `${overview.bounceRate}%`, i: Gauge }, { l: 'Avg. duration', v: duration(overview.avgDuration), i: Activity }, { l: 'Online now', v: overview.currentOnline, i: CircleDot }] as metric}
            <article class="metric-card">
              <div>
                <small>{metric.l}</small><metric.i size={17} />
              </div>
              <strong>{metric.v}</strong><span class:negative={overview.change < 0}
                >{overview.change > 0 ? '+' : ''}{overview.change}% <small>vs previous</small></span
              >
            </article>
          {/each}
        </section>
        <section class="panel chart-panel">
          <div class="panel-head">
            <div>
              <p class="eyebrow">Traffic volume</p>
              <h2>Visitors & page views</h2>
            </div>
            <div class="legend">
              <span><i class="visitors"></i>Visitors</span><span><i class="views"></i>Page views</span>
            </div>
          </div>
          <svg class="big-chart" viewBox="0 0 1000 260" role="img" aria-label="Traffic over selected period"
            ><g class="grid-lines"
              ><line x1="0" y1="52" x2="1000" y2="52" /><line x1="0" y1="104" x2="1000" y2="104" /><line
                x1="0"
                y1="156"
                x2="1000"
                y2="156"
              /><line x1="0" y1="208" x2="1000" y2="208" /></g
            ><polyline
              class="views-line"
              points={sparklinePoints(
                overview.trend.map((p) => p.pageViews),
                1000,
                230
              )}
            /><polyline
              class="visitor-line"
              points={sparklinePoints(
                overview.trend.map((p) => p.visitors),
                1000,
                230
              )}
            /></svg
          >
        </section>
        <div class="two-col">
          <ReportTable title="Top pages" rows={topPages} />
          <ReportTable title="Top referrers" rows={topReferrers} />
        </div>
      {:else if view === 'spy'}
        <section class="spy-toolbar">
          <div>
            <span class="live-dot"></span><strong>Live activity</strong><span class="muted"
              >{paused ? 'Stream paused' : 'Updating in real time'}</span
            >
          </div>
          <div>
            <label class="search"
              ><Search size={15} /><span class="sr-only">Filter activity</span
              ><input bind:value={spyFilter} placeholder="Filter pages, places…" /></label
            ><button class="secondary" onclick={toggleSpy}
              >{#if paused}<Play size={15} />Resume{:else}<Pause size={15} />Pause{/if}</button
            >
          </div>
        </section>
        <div class="spy-grid">
          <section class="panel map-panel"><WorldMap visitors={mapVisitors} /></section>
          <section class="panel activity-feed">
            <div class="panel-head">
              <h2>Visitor stream</h2>
              <span>{filteredEvents.length} events</span>
            </div>
            {#if filteredEvents.length}
              <ol>
                {#each filteredEvents as item}
                  <li>
                    <span class="event-icon"><Eye size={15} /></span><button
                      onclick={() =>
                        (selectedVisitor =
                          visitors.find((visitor) => visitor.id === item.visitorId) ?? null)}
                      ><strong>{item.page}</strong><small
                        >{item.city ?? item.country ?? 'Unknown'} · {item.type} · {new Date(
                          item.timestamp
                        ).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</small
                      ></button
                    >
                  </li>
                {/each}
              </ol>
            {:else}
              <div class="empty"><Activity /><p>No matching live activity.</p></div>
            {/if}
          </section>
        </div>
        {#if selectedVisitor}
          <aside class="visitor-drawer" aria-label="Visitor details">
            <button
              class="icon-button"
              onclick={() => (selectedVisitor = null)}
              aria-label="Close visitor details"><X /></button
            >
            <div class="visitor-badge"><Users /></div>
            <p class="eyebrow">Visitor details</p>
            <h2>{selectedVisitor.city ?? 'Unknown city'}, {selectedVisitor.country}</h2>
            <dl>
              <div>
                <dt>Device</dt>
                <dd>{selectedVisitor.device ?? 'Unknown'}</dd>
              </div>
              <div>
                <dt>Browser</dt>
                <dd>{selectedVisitor.browser ?? 'Unknown'}</dd>
              </div>
              <div>
                <dt>Sessions</dt>
                <dd>{selectedVisitor.sessions ?? 1}</dd>
              </div>
              <div>
                <dt>Current page</dt>
                <dd>{selectedVisitor.page ?? '—'}</dd>
              </div>
            </dl>
          </aside>
        {/if}
      {:else if ['pages', 'referrers', 'countries', 'devices', 'campaigns'].includes(view)}
        <section class="page-head">
          <div>
            <p class="eyebrow">Acquisition detail</p>
            <h2>{nav.find((item) => item.id === view)?.label}</h2>
            <p class="muted">Ranked by page views during the selected period.</p>
          </div>
          <button class="secondary button" onclick={() => void downloadCsv()}
            ><Download size={15} />Export CSV</button
          >
        </section>
        <ReportTable
          title={`${nav.find((item) => item.id === view)?.label} report`}
          rows={report}
        />
      {:else if view === 'visitors'}
        <section class="panel">
          <div class="panel-head">
            <div>
              <p class="eyebrow">Audience</p>
              <h2>Recent visitors</h2>
            </div>
            <span>{visitors.length} visitors</span>
          </div>
          {#if visitors.length}
            <div class="visitor-list">
              {#each visitors as visitor}
                <button onclick={() => (selectedVisitor = visitor)}
                  ><span class="country-code">{visitor.country.slice(0, 2).toUpperCase()}</span
                  ><span
                    ><strong>{visitor.city ?? 'Unknown'}, {visitor.country}</strong><small
                      >{visitor.device} · {visitor.browser} · {visitor.page}</small
                    ></span
                  ><span>{visitor.sessions ?? 1} sessions</span></button
                >
              {/each}
            </div>
          {:else}
            <div class="empty"><Users /><p>No visitors in this period.</p></div>
          {/if}
        </section>
      {:else if view === 'goals'}
        <section class="page-head">
          <div>
            <p class="eyebrow">Outcomes</p>
            <h2>Goals</h2>
            <p class="muted">Measure the actions that matter, not just clicks.</p>
          </div>
          <button class="primary" onclick={() => (newGoal = true)}
            ><Plus size={16} />New goal</button
          >
        </section>
        <div class="goal-grid">
          {#each goals as goal}
            <article class="panel goal-card">
              <span><GoalIcon /></span>
              <div>
                <small>{goal.type}</small>
                <h3>{goal.name}</h3>
                <code>{goal.target}</code>
              </div>
              <div>
                <strong>{goal.conversions ?? 0}</strong><small>conversions</small>
              </div>
              <div>
                <strong>{goal.conversionRate ?? 0}%</strong><small>conversion rate</small>
              </div>
            </article>
          {/each}
        </div>
        {#if newGoal}
          <div class="modal-backdrop" role="presentation"
            ><form
              class="modal"
              onsubmit={(event) => {
                event.preventDefault();
                void addGoal();
              }}
              ><div class="panel-head">
                <h2>Create goal</h2>
                <button type="button" class="icon-button" onclick={() => (newGoal = false)}
                  ><X /></button
                >
              </div>
              <label
                >Goal name<input
                  bind:value={goalName}
                  required
                  placeholder="Newsletter signup"
                /></label
              ><label
                >Event name<input bind:value={goalTarget} required placeholder="signup" /></label
              >
              <div class="modal-actions">
                <button type="button" class="secondary" onclick={() => (newGoal = false)}>Cancel</button
                ><button class="primary">Create goal</button>
              </div></form
            ></div
          >
        {/if}
      {:else if view === 'settings'}
        <section class="settings-grid">
          <div class="panel settings-card">
            <p class="eyebrow">Appearance</p>
            <h2>Theme</h2>
            <p class="muted">Use your system preference or override it.</p>
            <div class="theme-options">
              {#each [{ id: 'light', label: 'Light', icon: Sun }, { id: 'dark', label: 'Dark', icon: Moon }, { id: 'system', label: 'System', icon: Smartphone }] as option}
                <button
                  class:active={theme === option.id}
                  onclick={() => updateTheme(option.id as Theme)}
                  ><option.icon /><span>{option.label}</span></button
                >
              {/each}
            </div>
          </div>
          <div class="panel settings-card">
            <p class="eyebrow">Site profile</p>
            <h2>{site.name}</h2>
            <dl>
              <div>
                <dt>Domain</dt>
                <dd>{site.domain}</dd>
              </div>
              <div>
                <dt>Write key</dt>
                <dd><code>{site.writeKey}</code></dd>
              </div>
              <div>
                <dt>Timezone</dt>
                <dd>{site.timezone ?? 'UTC'}</dd>
              </div>
            </dl>
          </div>
          <div class="panel settings-card installation-card">
            {#key site.id}
              <AntiAdblockSettingsPanel
                {site}
                analyticsOrigin={typeof location === 'undefined'
                  ? 'https://slimlytics.com'
                  : location.origin}
                save={saveAntiAdblock}
              />
            {/key}
          </div>
        </section>
      {/if}
    </main>
  </div>
{/if}
