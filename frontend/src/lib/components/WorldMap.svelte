<script lang="ts">
  export interface MapVisitor { country: string; code: string; x: number; y: number; count: number }
  let { visitors }: { visitors: MapVisitor[] } = $props();
</script>

<figure class="world-map">
  <svg viewBox="0 0 800 390" role="img" aria-label="Live visitor world map">
    <rect width="800" height="390" rx="18" class="ocean"/>
    <g class="land" aria-hidden="true">
      <path d="M72 92l61-48 113 15 59 53-27 45-68 7-30 56-65-19-17-51-45-21z"/>
      <path d="M230 226l53 15 31 62-29 79-34-50-20-57z"/>
      <path d="M385 77l80-31 58 21 26 45-53 25-41-14-31 31-51-24z"/>
      <path d="M431 162l79-5 49 42-21 108-49 59-31-67-31-37z"/>
      <path d="M528 84l128-34 94 51-29 73-83 11-28-38-81 9-39-31z"/>
      <path d="M665 270l62-17 44 46-39 35-69-21z"/>
    </g>
    {#each visitors as visitor}
      <g transform={`translate(${visitor.x * 8},${visitor.y * 3.9})`}>
        <circle r={Math.min(18, 5 + visitor.count / 2)} class="pulse"/>
        <circle r="4" class="dot"><title>{visitor.country}: {visitor.count}</title></circle>
      </g>
    {/each}
  </svg>
  <figcaption><strong>Global activity</strong><ul>{#each visitors as visitor}<li><span>{visitor.code}</span> {visitor.country}: {visitor.count}</li>{/each}</ul></figcaption>
</figure>
