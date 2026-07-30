<script lang="ts">
  import { Check } from '@lucide/svelte';
  import { pricingPlans } from '$lib/marketing/pricing';

  let { compact = false }: { compact?: boolean } = $props();
</script>

<div class="pricing-grid" class:compact>
  {#each pricingPlans as plan}
    <article class="pricing-card" class:highlighted={plan.highlighted}>
      {#if plan.highlighted}<span class="pricing-badge">Most popular</span>{/if}
      <p class="eyebrow">{plan.name}</p>
      <h3>{plan.tagline}</h3>
      <p class="pricing-amount">
        <strong>{plan.price}</strong>
        <span>{plan.priceNote}</span>
      </p>
      <ul>
        {#each plan.features as feature}
          <li><Check size={15} aria-hidden="true" />{feature}</li>
        {/each}
      </ul>
      <a class={plan.highlighted ? 'primary wide' : 'secondary wide'} href={plan.ctaHref}
        >{plan.ctaLabel}</a
      >
    </article>
  {/each}
</div>
{#if !compact}
  <p class="pricing-disclaimer muted">
    Hosted plan limits are draft marketing defaults. Checkout is not required to create an account
    today — self-hosted always includes the full product.
  </p>
{/if}
