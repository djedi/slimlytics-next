<script lang="ts">
  import { onMount } from 'svelte';
  import { BarChart3, Menu, X } from '@lucide/svelte';

  let open = $state(false);
  let signedIn = $state(false);

  onMount(() => {
    signedIn = Boolean(localStorage.getItem('slimlytics_token'));
  });
</script>

<header class="mkt-header">
  <div class="mkt-header-inner">
    <a class="brand" href="/" aria-label="Slimlytics home">
      <span class="brand-mark"><BarChart3 size={20} /></span>
      <strong>Slimlytics</strong>
    </a>

    <nav class="mkt-nav" class:open aria-label="Marketing">
      <a href="/#features" onclick={() => (open = false)}>Features</a>
      <a href="/pricing" onclick={() => (open = false)}>Pricing</a>
      <a href="/privacy" onclick={() => (open = false)}>Privacy</a>
      <a href="/docs" onclick={() => (open = false)}>Docs</a>
      <div class="mkt-nav-actions">
        {#if signedIn}
          <a class="primary" href="/app" onclick={() => (open = false)}>Open dashboard</a>
        {:else}
          <a class="mkt-link-quiet" href="/login" onclick={() => (open = false)}>Sign in</a>
          <a class="primary" href="/register" onclick={() => (open = false)}>Get started</a>
        {/if}
      </div>
    </nav>

    <button
      class="icon-button mkt-menu"
      type="button"
      aria-label={open ? 'Close menu' : 'Open menu'}
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      {#if open}<X size={18} />{:else}<Menu size={18} />{/if}
    </button>
  </div>
</header>
{#if open}
  <button class="mkt-scrim" type="button" aria-label="Close menu" onclick={() => (open = false)}
  ></button>
{/if}
