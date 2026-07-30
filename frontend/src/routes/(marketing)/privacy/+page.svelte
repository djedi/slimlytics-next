<script lang="ts">
  import { Shield } from '@lucide/svelte';
  import { privacyHighlights } from '$lib/marketing/features';
</script>

<svelte:head>
  <title>Privacy · Slimlytics</title>
  <meta
    name="description"
    content="How Slimlytics approaches privacy: cookieless defaults, no form capture, site-scoped IDs, and owner-controlled retention."
  />
</svelte:head>

<main id="main" class="mkt-page">
  <header class="mkt-page-head">
    <p class="eyebrow">Privacy</p>
    <h1>Analytics without the surveillance model</h1>
    <p class="mkt-lead">
      Slimlytics is designed to answer site-analytics questions without building cross-site identity
      profiles or capturing more than you need.
    </p>
  </header>

  <section class="mkt-section">
    <div class="privacy-grid">
      <article class="feature-card">
        <Shield size={22} aria-hidden="true" />
        <h2>Cookieless by default</h2>
        <p>
          The browser tracker does not set tracking cookies. Collection can stay disabled until your
          site grants consent when that is legally or contractually required.
        </p>
      </article>
      <article class="feature-card">
        <Shield size={22} aria-hidden="true" />
        <h2>What we deliberately skip</h2>
        <p>
          No cross-site tracking, no device fingerprinting for identity, no form-field capture, and
          no session replay. Those are non-goals — not delayed features we quietly enable later.
        </p>
      </article>
      <article class="feature-card">
        <Shield size={22} aria-hidden="true" />
        <h2>Sensitive data redaction</h2>
        <p>
          Common secret and identity query parameters (tokens, emails, session keys, and similar)
          are stripped before URLs are stored. Prefer not putting personal data in URLs at all.
        </p>
      </article>
      <article class="feature-card">
        <Shield size={22} aria-hidden="true" />
        <h2>Site-scoped visitor IDs</h2>
        <p>
          Visitor identifiers are derived from privacy-reduced inputs with a rotating server secret
          and stay scoped to each site. Raw IPs are used only transiently for abuse controls and
          coarse location when enabled.
        </p>
      </article>
    </div>
  </section>

  <section class="mkt-section mkt-section-alt">
    <div class="mkt-section-head">
      <p class="eyebrow">Defaults</p>
      <h2>Built-in privacy highlights</h2>
    </div>
    <ul class="privacy-bullets">
      {#each privacyHighlights as item}
        <li><Shield size={16} aria-hidden="true" />{item}</li>
      {/each}
      <li><Shield size={16} aria-hidden="true" />Do Not Track and Global Privacy Control are respected</li>
      <li><Shield size={16} aria-hidden="true" />Site owners control retention, export, and deletion</li>
    </ul>
  </section>

  <section class="mkt-section">
    <div class="mkt-prose">
      <h2>Your control as a site owner</h2>
      <p>
        Configure retention, export site data, delete site data, and delete an anonymous visitor’s
        events when needed. When you self-host, the data never leaves the infrastructure you choose.
      </p>
      <h2>Consent integration</h2>
      <p>
        Initialize the tracker with collection disabled when consent is required. Call the tracker
        consent API only after the visitor grants analytics consent. Revocation stops new events
        immediately; deleting prior data is a separate server-side operation.
      </p>
    </div>
  </section>

  <section class="mkt-section mkt-cta">
    <div class="mkt-cta-inner">
      <h2>Measure without the guilt hangover</h2>
      <p>Create an account and see privacy-first analytics on your own terms.</p>
      <div class="mkt-hero-actions">
        <a class="primary" href="/register">Create free account</a>
        <a class="secondary" href="/docs">Documentation</a>
      </div>
    </div>
  </section>
</main>
