<script lang="ts">
  import { get } from 'svelte/store';
  import { t } from '../i18n';
  import { reduceMotion } from '../motion';
  let { value }: { value: number } = $props();

  const C = 2 * Math.PI * 70; // circunferência (r = 70)
  let shown = $state(0);

  // Conta do valor exibido atual até o novo, sincronizado com o arco (1.4 s, ease-out cúbico).
  $effect(() => {
    const to = value;
    const from = shown;
    if (get(reduceMotion)) {
      shown = to;
      return;
    }
    const t0 = performance.now();
    let raf = 0;
    const step = (now: number) => {
      const p = Math.min(1, (now - t0) / 1400);
      shown = Math.round(from + (to - from) * (1 - (1 - p) ** 3));
      if (p < 1) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="ring">
  <svg width="156" height="156" viewBox="0 0 156 156" aria-hidden="true">
    <circle cx="78" cy="78" r="70" fill="none" stroke="var(--line-2)" stroke-width="4" />
    <circle
      class="arc"
      cx="78"
      cy="78"
      r="70"
      fill="none"
      stroke="var(--ice)"
      stroke-width="4"
      stroke-linecap="round"
      stroke-dasharray={C}
      stroke-dashoffset={C * (1 - value / 100)}
      transform="rotate(-90 78 78)"
    />
  </svg>
  <div class="n">
    <div><b>{shown}</b><small>{$t('overview.score')}</small></div>
  </div>
</div>

<style>
  .ring {
    width: 156px;
    height: 156px;
    position: relative;
    flex: none;
  }
  .arc {
    transition: stroke-dashoffset 1.4s var(--ease);
  }
  .n {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    text-align: center;
  }
  b {
    font-weight: 200;
    font-size: 50px;
    letter-spacing: -0.045em;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  small {
    display: block;
    font: 500 10px var(--mono);
    letter-spacing: 0.14em;
    color: var(--dim);
    margin-top: 6px;
  }
</style>
