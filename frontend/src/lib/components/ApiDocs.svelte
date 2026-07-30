<script lang="ts">
  import '@scalar/api-reference/style.css';
  import { onMount } from 'svelte';

  let loadError = $state(false);

  onMount(() => {
    let mounted = true;
    void import('@scalar/api-reference')
      .then(({ createApiReference }) => {
        if (!mounted) return;
        createApiReference('#scalar-api-reference', {
          url: '/api/openapi.json',
          theme: 'purple',
          layout: 'modern',
          hideModels: false,
          hideDownloadButton: false,
          showSidebar: true,
          telemetry: false,
          withDefaultFonts: false
        });
      })
      .catch(() => {
        if (mounted) loadError = true;
      });
    return () => {
      mounted = false;
    };
  });
</script>

<svelte:head>
  <title>API Reference — Slimlytics</title>
  <meta name="description" content="Interactive OpenAPI reference for the Slimlytics API" />
</svelte:head>

<noscript>
  JavaScript is required for the interactive Scalar reference. Download the
  <a href="/api/openapi.json">OpenAPI 3.1 document</a> instead.
</noscript>

{#if loadError}
  <main class="fallback">
    <h1>API reference failed to load</h1>
    <p>The machine-readable API contract is still available.</p>
    <a href="/api/openapi.json">Download OpenAPI 3.1 JSON</a>
  </main>
{/if}

<div id="scalar-api-reference" aria-label="Slimlytics API reference"></div>

<style>
  :global(body) { margin: 0; }
  .fallback { max-width: 48rem; margin: 4rem auto; padding: 2rem; font-family: system-ui, sans-serif; }
  .fallback a { color: #12b886; }
</style>
