<script lang="ts">
  import { Check, Minus } from '@lucide/svelte';
  import PricingCards from '$lib/components/marketing/PricingCards.svelte';
  import { pricingComparison, pricingFaqs } from '$lib/marketing/pricing';
</script>

<svelte:head>
  <title>Pricing · Slimlytics</title>
  <meta
    name="description"
    content="Slimlytics pricing: free self-hosted analytics, plus draft hosted Starter and Pro plans."
  />
</svelte:head>

<main id="main" class="mkt-page">
  <header class="mkt-page-head">
    <p class="eyebrow">Pricing</p>
    <h1>Plans for every stage</h1>
    <p class="mkt-lead">
      Self-host the full product for free, or use hosted Starter and Pro when you want managed
      infrastructure. Creating an account never requires a card today.
    </p>
  </header>

  <PricingCards />

  <section class="mkt-section" aria-labelledby="compare-title">
    <div class="mkt-section-head">
      <p class="eyebrow">Compare</p>
      <h2 id="compare-title">What every plan includes</h2>
    </div>
    <div class="compare-wrap">
      <table class="compare-table">
        <thead>
          <tr>
            <th scope="col">Capability</th>
            <th scope="col">Self-hosted</th>
            <th scope="col">Starter</th>
            <th scope="col">Pro</th>
          </tr>
        </thead>
        <tbody>
          {#each pricingComparison as row}
            <tr>
              <th scope="row">{row.feature}</th>
              <td
                >{#if row.selfHosted}<Check size={16} aria-label="Included" />{:else}<Minus
                    size={16}
                    aria-label="Not included"
                  />{/if}</td
              >
              <td
                >{#if row.starter}<Check size={16} aria-label="Included" />{:else}<Minus
                    size={16}
                    aria-label="Not included"
                  />{/if}</td
              >
              <td
                >{#if row.pro}<Check size={16} aria-label="Included" />{:else}<Minus
                    size={16}
                    aria-label="Not included"
                  />{/if}</td
              >
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <section class="mkt-section mkt-section-alt" aria-labelledby="faq-title">
    <div class="mkt-section-head">
      <p class="eyebrow">FAQ</p>
      <h2 id="faq-title">Common questions</h2>
    </div>
    <div class="faq-list">
      {#each pricingFaqs as item}
        <details class="faq-item">
          <summary>{item.question}</summary>
          <p>{item.answer}</p>
        </details>
      {/each}
    </div>
  </section>

  <section class="mkt-section mkt-cta">
    <div class="mkt-cta-inner">
      <h2>Ready when you are</h2>
      <p>Spin up a free account and install the tracker in a few minutes.</p>
      <div class="mkt-hero-actions">
        <a class="primary" href="/register">Create free account</a>
        <a class="secondary" href="/docs">Self-host docs</a>
      </div>
    </div>
  </section>
</main>
