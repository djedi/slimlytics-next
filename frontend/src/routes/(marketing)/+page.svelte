<script lang="ts">
  import { env } from '$env/dynamic/public';
  import {
    Activity,
    CircleDot,
    Eye,
    FileText,
    Gauge,
    Globe2,
    Shield,
    Users,
    Zap
  } from '@lucide/svelte';
  import PricingCards from '$lib/components/marketing/PricingCards.svelte';
  import { howItWorks, privacyHighlights, productFeatures } from '$lib/marketing/features';
  import { sparklinePoints } from '$lib/ui';

  const demo = env.PUBLIC_DEMO_MODE === 'true';
  const trend = [42, 48, 45, 61, 58, 72, 68, 80, 76, 88, 84, 95];
</script>

<svelte:head>
  <title>Slimlytics · Private web analytics</title>
  <meta
    name="description"
    content="Privacy-first, cookieless web analytics with real-time Spy, multi-site reports, and self-hosting. Know what works — skip the noise."
  />
</svelte:head>

<main id="main">
  <section class="mkt-hero">
    <div class="mkt-hero-copy">
      <p class="eyebrow">Privacy-first web analytics</p>
      <h1>Know what works.<br /><em>Skip the noise.</em></h1>
      <p class="mkt-lead">
        Focused traffic intelligence, live visitor activity, and goals — without invasive profiles
        or an overgrown interface. Self-host free, or start a hosted plan when you are ready.
      </p>
      <div class="mkt-hero-actions">
        <a class="primary" href="/register">Create free account</a>
        <a class="secondary" href="/pricing">View pricing</a>
        {#if demo}
          <a class="secondary" href="/login">Explore demo</a>
        {/if}
      </div>
      <ul class="mkt-hero-pills" aria-label="Key benefits">
        <li><CircleDot size={14} /> Cookieless by default</li>
        <li><Zap size={14} /> Real-time Spy</li>
        <li><Shield size={14} /> Self-hostable</li>
      </ul>
    </div>

    <aside class="mkt-hero-preview" aria-label="Product preview">
      <div class="mkt-preview-card">
        <div class="mkt-preview-top">
          <span class="live"><i></i>14 online now</span>
          <span class="muted">Last 28 days</span>
        </div>
        <div class="mkt-preview-metrics">
          <div>
            <small>Visitors</small><strong>3,421</strong><span class="positive">+12.8%</span>
          </div>
          <div>
            <small>Page views</small><strong>8,754</strong><span class="positive">+9.4%</span>
          </div>
          <div>
            <small>Bounce</small><strong>38%</strong><span class="muted">healthy</span>
          </div>
        </div>
        <svg class="sparkline mkt-preview-chart" viewBox="0 0 300 70" role="img" aria-label="Sample traffic trend">
          <defs>
            <linearGradient id="mkt-fade" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="var(--accent)" stop-opacity=".28" />
              <stop offset="1" stop-color="var(--accent)" stop-opacity="0" />
            </linearGradient>
          </defs>
          <polygon points={`0,70 ${sparklinePoints(trend, 300, 58)} 300,70`} fill="url(#mkt-fade)" />
          <polyline points={sparklinePoints(trend, 300, 58)} />
        </svg>
        <ul class="mkt-preview-feed">
          <li><Eye size={14} /><span><strong>/pricing</strong><small>Berlin · pageview</small></span></li>
          <li><FileText size={14} /><span><strong>/docs</strong><small>Austin · pageview</small></span></li>
          <li><Users size={14} /><span><strong>signup</strong><small>Tokyo · goal</small></span></li>
        </ul>
      </div>
    </aside>
  </section>

  <section id="features" class="mkt-section">
    <div class="mkt-section-head">
      <p class="eyebrow">Product</p>
      <h2>Everything you need. Nothing you do not.</h2>
      <p class="muted">
        Built for teams who want clear traffic intelligence without the surveillance stack.
      </p>
    </div>
    <div class="feature-grid">
      {#each productFeatures as feature}
        <article class="feature-card">
          <h3>{feature.title}</h3>
          <p>{feature.body}</p>
        </article>
      {/each}
    </div>
  </section>

  <section class="mkt-section mkt-section-alt">
    <div class="mkt-section-head">
      <p class="eyebrow">How it works</p>
      <h2>Live in three steps</h2>
    </div>
    <ol class="how-grid">
      {#each howItWorks as step}
        <li>
          <span class="how-step">{step.step}</span>
          <h3>{step.title}</h3>
          <p>{step.body}</p>
        </li>
      {/each}
    </ol>
  </section>

  <section class="mkt-section">
    <div class="privacy-strip">
      <div>
        <p class="eyebrow">Privacy</p>
        <h2>Analytics that respect your visitors</h2>
        <p class="muted">
          Slimlytics answers site-analytics questions without building cross-site identity profiles.
        </p>
        <a class="secondary" href="/privacy">Read the privacy model</a>
      </div>
      <ul>
        {#each privacyHighlights as item}
          <li><Shield size={15} aria-hidden="true" />{item}</li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="mkt-section mkt-section-alt" aria-labelledby="pricing-teaser-title">
    <div class="mkt-section-head">
      <p class="eyebrow">Pricing</p>
      <h2 id="pricing-teaser-title">Simple plans. Full product when you self-host.</h2>
      <p class="muted">Start free on your own infrastructure, or pick a hosted tier when you want us to run it.</p>
    </div>
    <PricingCards compact />
    <p class="mkt-center">
      <a class="secondary" href="/pricing">Compare plans in detail</a>
    </p>
  </section>

  <section class="mkt-section mkt-cta">
    <div class="mkt-cta-inner">
      <Gauge size={28} aria-hidden="true" />
      <h2>Start measuring what actually matters</h2>
      <p>Create an account, add your site, and see private, useful analytics in minutes.</p>
      <div class="mkt-hero-actions">
        <a class="primary" href="/register">Create free account</a>
        <a class="secondary" href="/docs">Read the docs</a>
      </div>
      <ul class="mkt-cta-stats" aria-label="Product highlights">
        <li><Activity size={15} /> Live Spy stream</li>
        <li><Globe2 size={15} /> Multi-site workspace</li>
        <li><CircleDot size={15} /> Cookieless defaults</li>
      </ul>
    </div>
  </section>
</main>
