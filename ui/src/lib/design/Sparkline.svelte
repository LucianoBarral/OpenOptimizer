<script lang="ts">
  let { values, min, max }: { values: number[]; min: number; max: number } = $props();
  const id = $props.id();

  const line = $derived.by(() => {
    if (values.length < 2) return '';
    const span = max - min || 1;
    const pts = values.map((v, i) => {
      const x = (i / (values.length - 1)) * 100;
      const y = 24 - ((Math.min(max, Math.max(min, v)) - min) / span) * 20;
      return x.toFixed(1) + ' ' + y.toFixed(1);
    });
    return 'M' + pts.join(' L');
  });
</script>

<svg class="spark" viewBox="0 0 100 26" preserveAspectRatio="none" aria-hidden="true">
  <defs>
    <linearGradient id="fade-{id}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#c9d3dc" stop-opacity=".12" />
      <stop offset="1" stop-color="#c9d3dc" stop-opacity="0" />
    </linearGradient>
  </defs>
  {#if line}
    <path d="{line} L100 26 L0 26Z" fill="url(#fade-{id})" />
    <path d={line} class="ln" />
  {/if}
</svg>

<style>
  .spark {
    display: block;
    height: 26px;
    width: 100%;
    overflow: visible;
  }
  .ln {
    fill: none;
    stroke: var(--ice-2);
    stroke-width: 1.3;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }
</style>
