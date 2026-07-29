<script lang="ts">
  import { untrack } from 'svelte';
  import type { AntiAdblockServer, AntiAdblockSettings, Site } from '$lib/api';
  import {
    antiAdblockSnippet,
    proxyConfig,
    proxyTestLinks,
    validAntiAdblockPath,
    type AntiAdblockConfig
  } from '$lib/anti-adblock';

  let { site, analyticsOrigin, save }: {
    site: Site;
    analyticsOrigin: string;
    save: (settings: AntiAdblockSettings) => Promise<void>;
  } = $props();

  let serverType = $state<AntiAdblockServer>(untrack(() => site.antiAdblockServer));
  let jsPath = $state(untrack(() => site.antiAdblockJsPath));
  let beaconPath = $state(untrack(() => site.antiAdblockBeaconPath));
  let busy = $state(false);
  let status = $state('');
  let error = $state('');
  let copied = $state<'config' | 'snippet' | ''>('');

  function config(): AntiAdblockConfig { return { serverType, jsPath, beaconPath }; }
  function configurationCode(): string {
    try { return proxyConfig(config(), site, analyticsOrigin); }
    catch { return 'Enter valid JavaScript and beacon paths to generate your server configuration.'; }
  }
  function snippet(): string {
    try { return antiAdblockSnippet(jsPath); }
    catch { return 'Enter a valid JavaScript path to generate the tracking snippet.'; }
  }
  function testLinks(): { script: string; beacon: string } | undefined {
    try { return proxyTestLinks(site.domain, config()); }
    catch { return undefined; }
  }
  function validate(): string {
    if (!validAntiAdblockPath(jsPath, 'js')) return 'JavaScript path must be one safe path ending in .js.';
    if (!validAntiAdblockPath(beaconPath, 'beacon')) return 'Beacon path must be one safe path without a file extension requirement.';
    if (jsPath === beaconPath) return 'JavaScript and beacon paths must be different.';
    return '';
  }
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = validate(); status = '';
    if (error) return;
    busy = true;
    try {
      await save(config());
      status = 'Configuration saved.';
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Unable to save configuration.';
    } finally {
      busy = false;
    }
  }
  async function copy(kind: 'config' | 'snippet') {
    await navigator.clipboard.writeText(kind === 'config' ? configurationCode() : snippet());
    copied = kind;
    setTimeout(() => { if (copied === kind) copied = ''; }, 1800);
  }
</script>

<div class="anti-adblock-flow">
  <div class="flow-heading">
    <div>
      <p class="eyebrow">First-party delivery</p>
      <h2>Anti-adblock tracking</h2>
      <p class="muted">Proxy the tracker through your website using neutral, site-specific paths.</p>
    </div>
    <span class="step-badge">1 · Configure</span>
  </div>

  <form class="proxy-fields" onsubmit={submit}>
    <label>Server type
      <select bind:value={serverType}>
        <option value="caddy">Caddy</option>
        <option value="nginx">Nginx</option>
        <option value="apache">Apache</option>
      </select>
    </label>
    <label>JavaScript path
      <input bind:value={jsPath} aria-describedby="path-help" autocomplete="off" spellcheck="false" />
    </label>
    <label>Beacon path
      <input bind:value={beaconPath} aria-describedby="path-help" autocomplete="off" spellcheck="false" />
    </label>
    <p id="path-help" class="field-help">Defaults are generated per site. You may replace them with other neutral, unused paths.</p>
    {#if error}<p class="alert" role="alert">{error}</p>{/if}
    {#if status}<p class="success-message" role="status">{status}</p>{/if}
    <button class="primary" type="submit" disabled={busy}>{busy ? 'Saving…' : 'Save configuration'}</button>
  </form>

  <section class="generated-block">
    <div class="generated-heading"><div><span class="step-badge">2 · Server</span><h3>Configure your website</h3></div><button class="secondary" type="button" onclick={() => copy('config')}>{copied === 'config' ? 'Copied!' : 'Copy server config'}</button></div>
    <p class="muted">Add this inside the server block for <strong>{site.domain}</strong>, then reload your web server.</p>
    <pre><code>{configurationCode()}</code></pre>
  </section>

  <section class="generated-block">
    <div class="generated-heading"><div><span class="step-badge">3 · Install</span><h3>Add the tracking code</h3></div><button class="secondary" type="button" onclick={() => copy('snippet')}>{copied === 'snippet' ? 'Copied!' : 'Copy tracking code'}</button></div>
    <p class="muted">Paste this before the closing <code>&lt;/head&gt;</code> tag on your website.</p>
    <pre><code>{snippet()}</code></pre>
  </section>

  <section class="generated-block test-block">
    <div><span class="step-badge">4 · Test</span><h3>Verify both proxy paths</h3></div>
    <p class="muted">Save and reload your server configuration before testing. Each link should open successfully.</p>
    {#if testLinks()}
      {@const links = testLinks()!}
      <div class="test-links">
        <a class="secondary button" href={links.script} target="_blank" rel="noreferrer">Test JavaScript path</a>
        <a class="secondary button" href={links.beacon} target="_blank" rel="noreferrer">Test beacon path</a>
      </div>
    {/if}
  </section>

  <p class="privacy-note">This improves request delivery without bypassing consent, Do Not Track, or Global Privacy Control.</p>
</div>
