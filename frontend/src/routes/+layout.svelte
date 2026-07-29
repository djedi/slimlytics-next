<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { applyTheme, type Theme } from '$lib/ui';
  let { children } = $props();
  onMount(() => {
    const theme = (localStorage.getItem('slimlytics_theme') ?? 'system') as Theme;
    applyTheme(theme);
  });
</script>
<svelte:head><title>Slimlytics · Private web analytics</title></svelte:head>
<a class="skip-link" href="#main">Skip to content</a>
{@render children()}
