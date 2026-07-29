<script lang="ts">
  import { ArrowDown, ArrowUp, Minus } from '@lucide/svelte';
  import type { ReportRow } from '../api';
  let { title, rows }: { title: string; rows: ReportRow[] } = $props();
</script>

<section class="panel report">
  <div class="panel-head"><h2>{title}</h2><span>{rows.length} results</span></div>
  {#if rows.length}
    <div class="table-wrap">
      <table aria-label={title}>
        <thead><tr><th scope="col">Name</th><th scope="col">Share</th><th scope="col">Views</th><th scope="col"><span class="sr-only">Change</span></th></tr></thead>
        <tbody>{#each rows as row}
          <tr><th scope="row">{row.label}</th><td class="muted">{row.secondary ?? '—'}</td><td class="numeric">{row.value.toLocaleString()}</td><td class:positive={(row.change ?? 0) > 0} class:negative={(row.change ?? 0) < 0}>{#if (row.change ?? 0) > 0}<ArrowUp size={13}/>{:else if (row.change ?? 0) < 0}<ArrowDown size={13}/>{:else}<Minus size={13}/>{/if}<span class="sr-only">{row.change ?? 0}%</span></td></tr>
        {/each}</tbody>
      </table>
    </div>
  {:else}<div class="empty"><p>No report data for this period.</p><span>Try a wider date range.</span></div>{/if}
</section>
