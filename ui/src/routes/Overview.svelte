<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Card from '../lib/design/Card.svelte';
  import Ring from '../lib/design/Ring.svelte';
  import Button from '../lib/design/Button.svelte';
  import Switch from '../lib/design/Switch.svelte';
  import Sparkline from '../lib/design/Sparkline.svelte';
  import TweakRow from '../lib/tweakRow/TweakRow.svelte';
  import BusyOverlay from '../lib/brand/BusyOverlay.svelte';
  import { api, type HardwareView, type ChangeView, type MetricsView } from '../lib/ipc/api';
  import { tweaks, score, refresh, applying, pendingCount, optimizeRequest, route } from '../lib/stores';
  import type { GamingView } from '../lib/ipc/api';

  let gaming = $state<GamingView | null>(null);
  let gamingBusy = $state(false);
  let gamingTick = 0;

  async function toggleGaming(on: boolean) {
    gamingBusy = true;
    try {
      gaming = await api.gamingSetEnabled(on);
    } catch {
      gaming = await api.gamingGet();
    } finally {
      gamingBusy = false;
    }
  }
  import { t, locale } from '../lib/i18n';
  import { formatBytes, formatMs } from '../lib/format';

  const N = 24;
  let hw = $state<HardwareView | null>(null);
  let m = $state<MetricsView | null>(null);
  let cpu = $state<number[]>([]);
  let mem = $state<number[]>([]);
  let timer = $state<number[]>([]);
  let pend = $state<ChangeView[]>([]);
  let progress = $state(0);
  let label = $state('');
  let doneN = $state<number | null>(null);
  let restoreSkipped = $state(false);
  let timerId: ReturnType<typeof setInterval>;

  const push = (a: number[], v: number) => [...a, v].slice(-N);

  async function tick() {
    // Só consulta com a janela visível: minimizado, nenhum custo.
    if (document.visibilityState !== 'visible') return;
    const x = await api.metrics();
    m = x;
    // Estado do Modo Gaming a cada 2 s.
    if (gamingTick++ % 2 === 0) gaming = await api.gamingGet();
    cpu = push(cpu, x.cpu);
    mem = push(mem, x.mem_used / 2 ** 30);
    timer = push(timer, x.timer_ms);
  }

  let loaded = $state(false);
  let busy = $state(false);
  let busySub = $state('');

  onMount(async () => {
    hw = await api.hardware();
    await refresh();
    pend = await api.pending();
    loaded = true;
    await tick();
    timerId = setInterval(tick, 1000);
  });
  onDestroy(() => clearInterval(timerId));

  const toApply = $derived($tweaks.filter((x) => x.state === 'NotApplied' || x.state === 'Partial'));
  const n = $derived(pendingCount($tweaks));

  // "Otimizar agora" pedido pela bandeja (pode chegar antes desta tela montar).
  $effect(() => {
    if ($optimizeRequest > 0 && loaded && !$applying) {
      optimizeRequest.set(0);
      if (n > 0) optimize();
    }
  });

  async function optimize() {
    applying.set(true);
    busy = true;
    busySub = '';
    progress = 0;
    doneN = null;
    const un = await api.onProgress((p) => {
      progress = p.index / p.total;
      label = $t('action.applying', { i: p.index, n: p.total });
      busySub = `${p.index}/${p.total}`;
    });
    try {
      // Aplicar leva milissegundos; a animação fica no mínimo ~1,4 s para ser legível.
      const [r] = await Promise.all([
        api.applyTweaks(toApply.map((x) => x.id)),
        new Promise((res) => setTimeout(res, 1400)),
      ]);
      doneN = r.results.filter((x) => x.ok).length;
      restoreSkipped = !r.restore_point;
    } finally {
      un();
      applying.set(false);
      busy = false;
      label = '';
      await refresh();
    }
  }

  async function restorePending() {
    for (const p of pend) await api.revertChange(p.id, true);
    pend = await api.pending();
    await refresh();
  }
</script>

<div class="page">
  <div class="hd rise" style="--i:0">
    <div>
      <h2>{$t('nav.overview')}</h2>
      {#if hw}<p>{hw.cpu} · {hw.gpus[0] ?? ''} · {formatBytes(hw.ram_bytes, $locale)} GB</p>{/if}
    </div>
  </div>

  {#if pend.length}
    <div class="banner rise" style="--i:1">
      <span>{$t('pending.banner')}</span>
      <Button onclick={restorePending}>{$t('action.restore')}</Button>
    </div>
  {/if}

  <div class="grid">
    <Card class="hero rise" style="--i:2">
      <Ring value={$score} />
      <div>
        <h3>{n > 0 ? $t('overview.recommended', { n }) : $t('overview.allDone')}</h3>
        <p>{doneN !== null ? $t('overview.doneBody', { n: doneN }) : $t('overview.body')}</p>
        <Button {progress} disabled={n === 0 || $applying} onclick={optimize}>
          {label || (n === 0 ? $t('action.optimized') : $t('action.optimize'))}
        </Button>
        {#if restoreSkipped}<p class="note">{$t('snapshot.skipped')}</p>{/if}
      </div>
    </Card>

    <div class="metrics rise" style="--i:3">
      <Card class="m">
        <div class="label">{$t('metric.cpu')}</div>
        <div class="v">{m ? Math.round(m.cpu) : '–'}<em>%</em></div>
        <Sparkline values={cpu} min={0} max={100} />
      </Card>
      <Card class="m">
        <div class="label">{$t('metric.memory')}</div>
        <div class="v">{m ? formatBytes(m.mem_used, $locale) : '–'}<em>GB</em></div>
        <Sparkline values={mem} min={0} max={m ? m.mem_total / 2 ** 30 : 1} />
      </Card>
      <Card class="m">
        <div class="label">{$t('metric.timer')}</div>
        <div class="v">{m ? formatMs(m.timer_ms) : '–'}<em>ms</em></div>
        <Sparkline values={timer} min={0} max={16} />
      </Card>
      <Card class="m">
        <div class="label">{$t('metric.gpu')}</div>
        <div class="gpu">{hw?.gpus[0] ?? '–'}</div>
      </Card>
    </div>

    <Card class="list rise" style="--i:4">
      {#each $tweaks.slice(0, 4) as tw (tw.id)}<TweakRow tweak={tw} />{/each}
    </Card>

    <Card class="gm rise" style="--i:5">
      <div class="top">
        <button type="button" class="gm-link" onclick={() => route.set('gaming')}>
          <div class="label">{$t('nav.gaming')}</div>
          <h4>
            {#if !gaming?.enabled}{$t('gaming.off')}
            {:else if gaming.status.state === 'Active'}{$t('gaming.active')}
            {:else}{$t('gaming.waiting')}{/if}
          </h4>
        </button>
        <Switch
          size="lg"
          checked={gaming?.enabled ?? false}
          disabled={!gaming || gamingBusy}
          label={$t('nav.gaming')}
          onchange={toggleGaming}
        />
      </div>
      <p>
        {gaming?.status.state === 'Active'
          ? (gaming.games.find((x) => x.exe.toLowerCase() === gaming?.status.game?.toLowerCase())?.name ?? gaming.status.game)
          : $t('gaming.body')}
      </p>
    </Card>
  </div>
</div>

{#if busy}<BusyOverlay label={$t('busy.optimizing')} sub={busySub} />{/if}

<style>
  .page {
    padding: 28px 32px;
    height: 100%;
    overflow: auto;
  }
  .hd h2 {
    margin: 0;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  .hd p {
    color: var(--mut);
    margin: 4px 0 0;
  }
  .banner {
    margin-top: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 16px;
    border: 1px solid var(--line-2);
    border-radius: 12px;
    background: var(--card);
  }
  .grid {
    margin-top: 18px;
    display: grid;
    grid-template-columns: 1.5fr 1fr;
    gap: 16px;
  }
  .grid :global(.hero) {
    padding: 24px;
    display: flex;
    gap: 28px;
    align-items: center;
  }
  h3 {
    margin: 0;
    font-weight: 400;
    font-size: 17px;
    letter-spacing: -0.01em;
  }
  .grid :global(.hero p) {
    color: var(--mut);
    margin: 6px 0 18px;
    line-height: 1.55;
    max-width: 320px;
  }
  .grid :global(.hero .note) {
    margin: 10px 0 0;
    font-size: 12px;
  }
  .metrics {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .metrics :global(.m) {
    padding: 15px 16px 12px;
  }
  .v {
    font-weight: 300;
    font-size: 25px;
    letter-spacing: -0.025em;
    margin: 5px 0 8px;
    font-variant-numeric: tabular-nums;
  }
  .v em {
    font-style: normal;
    font-size: 12px;
    color: var(--mut);
    margin-left: 3px;
  }
  .gpu {
    margin-top: 8px;
    font-size: 13.5px;
    line-height: 1.35;
  }
  .grid :global(.list) {
    padding: 4px 20px;
  }
  .grid :global(.gm) {
    padding: 20px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 16px;
  }
  .gm-link {
    background: none;
    border: 0;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }
  .gm-link:hover h4 {
    color: #fff;
  }
  .top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }
  h4 {
    margin: 8px 0 0;
    font-weight: 300;
    font-size: 28px;
    letter-spacing: -0.03em;
  }
  .grid :global(.gm p) {
    color: var(--mut);
    margin: 0;
    line-height: 1.5;
  }
</style>
