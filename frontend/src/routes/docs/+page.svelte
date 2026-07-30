<script lang="ts">
  import { BookOpen, Code2, Terminal } from '@lucide/svelte';
  import MarketingHeader from '$lib/components/marketing/MarketingHeader.svelte';
  import MarketingFooter from '$lib/components/marketing/MarketingFooter.svelte';
</script>

<svelte:head>
  <title>Documentation · Slimlytics</title>
  <meta
    name="description"
    content="Slimlytics documentation hub: getting started, CLI automation, and API reference."
  />
</svelte:head>

<div class="mkt-shell">
  <MarketingHeader />
  <main id="main" class="mkt-page">
    <header class="mkt-page-head">
      <p class="eyebrow">Documentation</p>
      <h1>Get productive quickly</h1>
      <p class="mkt-lead">
        Create an account, add a site, install the tracker, and explore reports. Use the CLI for
        automation or the HTTP API for integrations.
      </p>
    </header>

    <section class="mkt-section">
      <div class="docs-grid">
        <article class="feature-card docs-card">
          <BookOpen size={22} aria-hidden="true" />
          <h2>Getting started</h2>
          <ol class="docs-steps">
            <li><a href="/register">Create an account</a> and sign in.</li>
            <li>Add a site from the dashboard workspace.</li>
            <li>Install the first-party snippet or generated proxy config.</li>
            <li>Open Overview and Spy as traffic arrives.</li>
          </ol>
          <a class="primary" href="/register">Create free account</a>
        </article>
        <article class="feature-card docs-card">
          <Terminal size={22} aria-hidden="true" />
          <h2>CLI</h2>
          <p>
            Automate account, site, and tracking setup with the installable <code>slimlytics</code> command.
          </p>
          <a class="secondary" href="/docs/cli">Open CLI documentation</a>
        </article>
        <article class="feature-card docs-card">
          <Code2 size={22} aria-hidden="true" />
          <h2>HTTP API</h2>
          <p>
            Interactive OpenAPI reference for authentication, sites, collection, reports, goals, and
            export.
          </p>
          <a class="secondary" href="/docs/api">Open API reference</a>
        </article>
      </div>
    </section>

    <section class="mkt-section mkt-section-alt">
      <div class="mkt-prose">
        <h2>Self-hosting</h2>
        <p>
          Generate a private <code>.env</code>, run <code>make up</code>, and open the app. Full
          deployment steps — including the production TLS overlay — live in the project README.
        </p>
        <pre class="docs-snippet"><code>./scripts/generate-env.sh
make up
curl --fail http://localhost:8080/health</code></pre>
      </div>
    </section>
  </main>
  <MarketingFooter />
</div>
