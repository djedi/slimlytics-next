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
    Compass,
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
    Send,
    Settings,
    Smartphone,
    MapPin,
    Sun,
    Trash2,
    Users,
    X,
    Zap
  } from '@lucide/svelte';
  import {
    ApiClient,
    demoReport,
    type AntiAdblockSettings,
    type Anomaly,
    type Attribution,
    type CollectionHealth,
    type Funnel,
    type FunnelReport,
    type Goal,
    type Journey,
    type LiveEvent,
    type Overview,
    type ReportRow,
    type ReportSubscription,
    type SearchConsoleStatus,
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
    | 'insights'
    | 'spy'
    | 'pages'
    | 'referrers'
    | 'countries'
    | 'regions'
    | 'cities'
    | 'devices'
    | 'browsers'
    | 'operating-systems'
    | 'campaigns'
    | 'visitors'
    | 'goals'
    | 'settings';
  const demo = env.PUBLIC_DEMO_MODE === 'true';
  const api = new ApiClient(env.PUBLIC_API_BASE_URL || '/api', fetch, demo);
  const nav: Array<{ id: View; label: string; icon: typeof Activity }> = [
    { id: 'overview', label: 'Overview', icon: Gauge },
    { id: 'insights', label: 'Insights', icon: BarChart3 },
    { id: 'spy', label: 'Spy', icon: Eye },
    { id: 'pages', label: 'Pages', icon: FileText },
    { id: 'referrers', label: 'Referrers', icon: Activity },
    { id: 'countries', label: 'Countries', icon: Globe2 },
    { id: 'regions', label: 'Regions', icon: MapPin },
    { id: 'cities', label: 'Cities', icon: MapPin },
    { id: 'devices', label: 'Devices', icon: Monitor },
    { id: 'browsers', label: 'Browsers', icon: Compass },
    { id: 'operating-systems', label: 'Operating systems', icon: Smartphone },
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
  let journeys = $state<Journey[]>([]);
  let attribution = $state<Attribution[]>([]);
  let anomalies = $state<Anomaly[]>([]);
  let funnels = $state<Funnel[]>([]);
  let funnelReports = $state<FunnelReport[]>([]);
  let landingPages = $state<ReportRow[]>([]);
  let exitPages = $state<ReportRow[]>([]);
  let sources = $state<ReportRow[]>([]);
  let content = $state<ReportRow[]>([]);
  let aiReferrers = $state<ReportRow[]>([]);
  let aiCrawlers = $state<ReportRow[]>([]);
  let collectionHealth = $state<CollectionHealth | null>(null);
  let searchConsole = $state<SearchConsoleStatus | null>(null);
  let reportSubscriptions = $state<ReportSubscription[]>([]);
  let briefName = $state('Weekly marketing brief');
  let briefWebhook = $state('');
  let briefFrequency = $state<'daily' | 'weekly'>('weekly');
  let briefAnomaliesOnly = $state(false);
  let newSigningSecret = $state('');
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
    // Keep stats fresh while the dashboard stays open (e.g. phone browsing + desktop dashboard).
    const refresh = () => {
      if (document.visibilityState !== 'visible' || loading) return;
      if (!site) void refreshSitesQuietly().catch(() => {});
      else if (view !== 'settings') void refreshViewQuietly().catch(() => {});
    };
    const interval = window.setInterval(refresh, 15_000);
    const onFocus = () => refresh();
    window.addEventListener('focus', onFocus);
    document.addEventListener('visibilitychange', onFocus);
    return () => {
      source?.close();
      window.clearInterval(interval);
      window.removeEventListener('focus', onFocus);
      document.removeEventListener('visibilitychange', onFocus);
    };
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
      await refreshSitesQuietly();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'Could not load sites.';
    } finally {
      loading = false;
    }
  }
  async function refreshSitesQuietly() {
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
      await refreshViewQuietly(true);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'Could not load analytics.';
    } finally {
      loading = false;
    }
  }
  async function refreshViewQuietly(resetStream = false) {
    if (!site) return;
    if (view === 'overview') {
      const [nextOverview, pages, referrers] = await Promise.all([
        api.overview(site.id, days),
        api.report(site.id, 'pages', days),
        api.report(site.id, 'referrers', days)
      ]);
      overview = nextOverview;
      topPages = (demo ? demoReport('pages') : pages).slice(0, 5);
      topReferrers = (demo ? demoReport('referrers') : referrers).slice(0, 5);
      // Keep sidebar "online now" in sync.
      sites = sites.map((item) =>
        item.id === site?.id ? { ...item, overview: nextOverview } : item
      );
    } else if (view === 'insights') {
      const [
        nextJourneys,
        nextAttribution,
        nextAnomalies,
        nextFunnels,
        landing,
        exits,
        sourceRows,
        contentRows,
        referrals,
        crawlers
      ] = await Promise.all([
        api.journeys(site.id, days),
        api.attribution(site.id, days),
        api.anomalies(site.id, days),
        api.funnels(site.id),
        api.report(site.id, 'landing-pages', days),
        api.report(site.id, 'exit-pages', days),
        api.report(site.id, 'sources', days),
        api.report(site.id, 'content', days),
        api.report(site.id, 'ai-referrers', days),
        api.report(site.id, 'ai-crawlers', days)
      ]);
      journeys = nextJourneys;
      attribution = nextAttribution;
      anomalies = nextAnomalies;
      funnels = nextFunnels;
      landingPages = landing;
      exitPages = exits;
      sources = sourceRows;
      content = contentRows;
      aiReferrers = referrals;
      aiCrawlers = crawlers;
      funnelReports = await Promise.all(
        nextFunnels.map((funnel) => api.funnelReport(site!.id, funnel.id, days))
      );
    } else if (
      [
        'pages',
        'referrers',
        'countries',
        'regions',
        'cities',
        'devices',
        'browsers',
        'operating-systems',
        'campaigns'
      ].includes(view)
    )
      report = await api.report(site.id, view, days);
    else if (view === 'visitors') visitors = await api.visitors(site.id);
    else if (view === 'spy') {
      events = await api.events(site.id);
      visitors = await api.visitors(site.id);
      if (resetStream || !source || source.readyState === EventSource.CLOSED) connectSpy();
    } else if (view === 'goals') goals = await api.goals(site.id);
    else if (view === 'settings')
      [collectionHealth, searchConsole, reportSubscriptions] = await Promise.all([
        api.collectionHealth(site.id),
        api.searchConsoleStatus(site.id),
        api.reportSubscriptions(site.id)
      ]);
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
  async function connectSearchConsole() {
    if (!site) return;
    const { authorizationUrl } = await api.connectSearchConsole(site.id);
    location.assign(authorizationUrl);
  }
  async function syncSearchConsole() {
    if (!site) return;
    loading = true;
    try {
      await api.syncSearchConsole(site.id, days);
      searchConsole = await api.searchConsoleStatus(site.id);
    } finally {
      loading = false;
    }
  }
  async function disconnectSearchConsole() {
    if (!site || !confirm('Disconnect Search Console and remove its cached metrics?')) return;
    await api.disconnectSearchConsole(site.id);
    searchConsole = await api.searchConsoleStatus(site.id);
  }
  async function createBrief(event: SubmitEvent) {
    event.preventDefault();
    if (!site || !briefName.trim() || !briefWebhook.trim()) return;
    const created = await api.createReportSubscription(site.id, {
      name: briefName.trim(), webhookUrl: briefWebhook.trim(), frequency: briefFrequency,
      anomalyOnly: briefAnomaliesOnly, enabled: true
    });
    newSigningSecret = created.signingSecret ?? '';
    reportSubscriptions = [...reportSubscriptions, created];
    briefWebhook = '';
  }
  async function toggleBrief(subscription: ReportSubscription) {
    if (!site) return;
    const updated = await api.updateReportSubscription(site.id, {
      ...subscription, enabled: !subscription.enabled
    });
    reportSubscriptions = reportSubscriptions.map((item) => item.id === updated.id ? updated : item);
  }
  async function deliverBrief(subscription: ReportSubscription) {
    if (!site) return;
    await api.deliverReportSubscription(site.id, subscription.id);
    reportSubscriptions = await api.reportSubscriptions(site.id);
  }
  async function deleteBrief(subscription: ReportSubscription) {
    if (!site || !confirm(`Delete ${subscription.name}?`)) return;
    await api.deleteReportSubscription(site.id, subscription.id);
    reportSubscriptions = reportSubscriptions.filter((item) => item.id !== subscription.id);
  }
  async function rotateServerKey() {
    if (!site || !confirm('Rotate the server ingestion key? Existing log shippers will stop working.')) return;
    const result = await api.rotateServerKey(site.id);
    site = { ...site, serverWriteKey: result.serverWriteKey };
    sites = sites.map((item) => item.id === site?.id ? { ...item, serverWriteKey: result.serverWriteKey } : item);
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
      {:else if view === 'insights'}
        <section class="page-head">
          <div>
            <p class="eyebrow">Marketing intelligence</p>
            <h2>What is driving outcomes</h2>
            <p class="muted">Attribution, journeys, content, funnels, and AI traffic.</p>
          </div>
        </section>
        <div class="metric-grid compact-insights">
          <article class="metric-card">
            <div><small>Revenue</small><Zap /></div>
            <strong>{attribution
                .reduce((sum, row) => sum + row.revenue, 0)
                .toLocaleString(undefined, { style: 'currency', currency: 'USD' })}</strong>
            <span>{attribution.reduce((sum, row) => sum + row.conversions, 0)} conversions</span>
          </article>
          <article class="metric-card">
            <div><small>Journeys</small><Activity /></div>
            <strong>{journeys.length}</strong><span
              >{journeys.reduce((sum, row) => sum + row.sessions, 0)} sessions</span
            >
          </article>
          <article class="metric-card">
            <div><small>Anomalies</small><Bell /></div>
            <strong>{anomalies.length}</strong><span>30% threshold</span>
          </article>
          <article class="metric-card">
            <div><small>Funnels</small><GoalIcon /></div>
            <strong>{funnels.length}</strong><span>Sequential visitors</span>
          </article>
        </div>
        <div class="two-col insight-section">
          <ReportTable title="Landing pages" rows={landingPages.slice(0, 10)} />
          <ReportTable title="Exit pages" rows={exitPages.slice(0, 10)} />
        </div>
        <div class="two-col insight-section">
          <ReportTable title="Traffic sources" rows={sources.slice(0, 10)} />
          <ReportTable title="Content" rows={content.slice(0, 10)} />
        </div>
        <div class="two-col insight-section">
          <ReportTable title="AI referrals" rows={aiReferrers.slice(0, 10)} />
          <ReportTable title="AI crawlers" rows={aiCrawlers.slice(0, 10)} />
        </div>
        <section class="panel insight-section">
          <div class="panel-head">
            <h2>First-touch attribution</h2><span>{attribution.length} channels</span>
          </div>
          {#if attribution.length}
            <div class="table-wrap">
              <table>
                <thead
                  ><tr
                    ><th>Source / medium</th><th>Campaign</th><th>Visitors</th
                    ><th>Conversions</th><th>Revenue</th></tr
                  ></thead
                >
                <tbody>
                  {#each attribution as row}
                    <tr>
                      <th>{row.source} / {row.medium}</th>
                      <td>{row.campaign}</td>
                      <td class="numeric">{row.visitors.toLocaleString()}</td>
                      <td class="numeric">{row.conversions.toLocaleString()}</td>
                      <td class="numeric"
                        >{row.revenue.toLocaleString(undefined, {
                          style: 'currency',
                          currency: 'USD'
                        })}</td
                      >
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="empty"><Activity /><p>No attributed traffic in this period.</p></div>
          {/if}
        </section>
        <div class="two-col insight-section">
          <section class="panel">
            <div class="panel-head"><h2>Common journeys</h2><span>Ordered paths</span></div>
            {#if journeys.length}
              <div class="insight-list">
                {#each journeys.slice(0, 10) as journey}
                  <div
                    ><strong>{journey.steps.join(' → ')}</strong><span
                      >{journey.sessions} sessions</span
                    ></div
                  >
                {/each}
              </div>
            {:else}
              <div class="empty"><Activity /><p>No journeys yet.</p></div>
            {/if}
          </section>
          <section class="panel">
            <div class="panel-head"><h2>Anomalies</h2><span>Trailing baseline</span></div>
            {#if anomalies.length}
              <div class="insight-list">
                {#each anomalies as anomaly}
                  <div
                    ><strong>{anomaly.date}</strong><span
                      >{anomaly.deviationPercent > 0 ? '+' : ''}{anomaly.deviationPercent.toFixed(
                        1
                      )}%</span
                    ></div
                  >
                {/each}
              </div>
            {:else}
              <div class="empty"><Bell /><p>No material anomalies.</p></div>
            {/if}
          </section>
        </div>
        <section class="panel insight-section">
          <div class="panel-head"><h2>Funnels</h2><span>Sequential unique visitors</span></div>
          {#if funnelReports.length}
            <div class="funnel-list">
              {#each funnelReports as funnel}
                <article>
                  <strong>{funnel.name}</strong>
                  <div>
                    {#each funnel.steps as step}
                      <span
                        ><small>{step.label}</small><b>{step.visitors}</b
                        ><i>{step.conversionRate.toFixed(1)}%</i></span
                      >
                    {/each}
                  </div>
                </article>
              {/each}
            </div>
          {:else}
            <div class="empty"><GoalIcon /><p>No funnels configured.</p></div>
          {/if}
        </section>
      {:else if
        [
          'pages',
          'referrers',
          'countries',
          'regions',
          'cities',
          'devices',
          'browsers',
          'operating-systems',
          'campaigns'
        ].includes(view)}
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
          <div class="panel settings-card">
            <p class="eyebrow">Collection</p>
            <h2>{collectionHealth?.lastAcceptedAt ? 'Receiving events' : 'Waiting for events'}</h2>
            <dl>
              <div>
                <dt>Accepted</dt>
                <dd>{collectionHealth?.acceptedTotal.toLocaleString() ?? 0}</dd>
              </div>
              <div>
                <dt>Rejected</dt>
                <dd>{collectionHealth?.rejectedTotal.toLocaleString() ?? 0}</dd>
              </div>
              <div>
                <dt>Last event</dt>
                <dd>{collectionHealth?.lastAcceptedAt
                    ? new Date(collectionHealth.lastAcceptedAt).toLocaleString()
                    : 'Never'}</dd>
              </div>
              <div>
                <dt>Tracker</dt>
                <dd>{collectionHealth?.lastTrackerVersion ?? 'Unknown'}</dd>
              </div>
              {#if collectionHealth?.lastRejectionCode}
                <div>
                  <dt>Last rejection</dt>
                  <dd><code>{collectionHealth.lastRejectionCode}</code></dd>
                </div>
              {/if}
            </dl>
          </div>
          <div class="panel settings-card">
            <p class="eyebrow">Server collection</p>
            <h2>Request ingestion</h2>
            <dl>
              <div>
                <dt>Endpoint</dt>
                <dd><code>/api/ingest</code></dd>
              </div>
              <div>
                <dt>Server key</dt>
                <dd><code>{site.serverWriteKey}</code></dd>
              </div>
              <div>
                <dt>Batch limit</dt>
                <dd>100 requests</dd>
              </div>
            </dl>
            <div class="test-links">
              <button class="secondary" onclick={() => void rotateServerKey()}>Rotate key</button>
            </div>
          </div>
          <div class="panel settings-card brief-settings">
            <p class="eyebrow">Delivery</p>
            <h2>Marketing briefs</h2>
            <form class="brief-form" onsubmit={createBrief}>
              <label>Name<input bind:value={briefName} maxlength="120" required /></label>
              <label>Webhook URL<input bind:value={briefWebhook} type="url" inputmode="url" placeholder="https://hooks.example.com/report" required /></label>
              <label>Frequency<select bind:value={briefFrequency}><option value="daily">Daily</option><option value="weekly">Weekly</option></select></label>
              <label class="check-field"><input type="checkbox" bind:checked={briefAnomaliesOnly} />Anomalies only</label>
              <button class="primary"><Plus />Create</button>
            </form>
            {#if newSigningSecret}
              <p class="success-message">Signing secret: <code>{newSigningSecret}</code></p>
            {/if}
            {#if reportSubscriptions.length}
              <ul class="brief-list">
                {#each reportSubscriptions as subscription}
                  <li>
                    <span><strong>{subscription.name}</strong><small>{subscription.frequency} · {subscription.lastStatus ?? 'pending'}</small></span>
                    <button class="icon-button" title="Send now" aria-label="Send now" onclick={() => void deliverBrief(subscription)}><Send /></button>
                    <button class="secondary compact" onclick={() => void toggleBrief(subscription)}>{subscription.enabled ? 'Pause' : 'Enable'}</button>
                    <button class="icon-button" title="Delete" aria-label="Delete" onclick={() => void deleteBrief(subscription)}><Trash2 /></button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
          <div class="panel settings-card">
            <p class="eyebrow">Organic search</p>
            <h2>Google Search Console</h2>
            {#if !searchConsole?.configured}
              <p class="muted">OAuth credentials are not configured on this deployment.</p>
            {:else if !searchConsole.connected}
              <p class="muted">Connect verified search properties for query and page performance.</p>
              <div class="test-links">
                <button class="primary" onclick={() => void connectSearchConsole()}>Connect</button>
              </div>
            {:else}
              <dl>
                <div><dt>Property</dt><dd>{searchConsole.propertyUrl ?? 'No match'}</dd></div>
                <div>
                  <dt>Last sync</dt>
                  <dd>{searchConsole.lastSyncedAt
                      ? new Date(searchConsole.lastSyncedAt).toLocaleString()
                      : 'Never'}</dd>
                </div>
                {#if searchConsole.lastError}
                  <div><dt>Status</dt><dd>{searchConsole.lastError}</dd></div>
                {/if}
              </dl>
              <div class="test-links">
                <button class="primary" onclick={() => void syncSearchConsole()}>Sync now</button>
                <button class="secondary" onclick={() => void disconnectSearchConsole()}
                  >Disconnect</button
                >
              </div>
            {/if}
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
