<script lang="ts">
  import MarketingHeader from '$lib/components/marketing/MarketingHeader.svelte';
  import MarketingFooter from '$lib/components/marketing/MarketingFooter.svelte';

  let { children } = $props();
</script>

<div class="mkt-shell">
  <MarketingHeader />
  {@render children()}
  <MarketingFooter />
</div>
