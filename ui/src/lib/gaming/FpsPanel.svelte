<script lang="ts">
  // "Desempenho medido": sessões do Modo Gaming medidas pelo PresentMon, agrupadas por jogo.
  import { onMount, onDestroy } from 'svelte';
  import Card from '../design/Card.svelte';
  import Sparkline from '../design/Sparkline.svelte';
  import { api } from '../ipc/api';
  import type { FpsRecordView } from '../ipc/bindings/FpsRecordView';
  import type { GameView } from '../ipc/bindings/GameView';
  import { compareGame } from '../fpsCompare';
  import { t, locale } from '../i18n';

  let { games }: { games: GameView[] } = $props();

  let records = $state<FpsRecordView[]>([]);
  let changes = $state<number[]>([]);
  let timer: ReturnType<typeof setInterval>;

  async function load() {
    if (document.visibilityState !== 'visible') return;
    try {
      const [r, h] = await Promise.all([api.fpsHistory(), api.history()]);
      records = r;
      changes = h.map((c) => Date.parse(c.timestamp) / 1000).filter((x) => Number.isFinite(x));
    } catch {
      /* sem dados: a seção mostra o estado vazio */
    }
  }

  // Um grupo por jogo, o jogado mais recentemente primeiro.
  const groups = $derived.by(() => {
    const by = new Map<string, FpsRecordView[]>();
    for (const r of records) by.set(r.game.toLowerCase(), [...(by.get(r.game.toLowerCase()) ?? []), r]);
    return [...by.entries()]
      .map(([exe, list]) => {
        const sorted = [...list].sort((a, b) => a.started - b.started);
        const valid = sorted.filter((x) => x.valid);
        return {
          exe,
          name: games.find((g) => g.exe.toLowerCase() === exe)?.name ?? exe,
          last: sorted[sorted.length - 1],
          valid,
          cmp: compareGame(sorted, changes),
        };
      })
      .sort((a, b) => b.last.started - a.last.started);
  });

  const n0 = (x: number) => Math.round(x).toLocaleString($locale);
  const pct = (x: number) => `${x >= 0 ? '+' : ''}${x.toFixed(1).replace('.', $locale === 'en' ? '.' : ',')}%`;
  const day = (unix: number) => new Date(unix * 1000).toLocaleDateString($locale, { day: '2-digit', month: 'short' });

  onMount(() => {
    load();
    timer = setInterval(load, 15000);
  });
  onDestroy(() => clearInterval(timer));
</script>

<section>
  <div class="label">{$t('fps.title')}</div>
  {#if !groups.length}
    <Card class="pad"><p class="muted">{$t('fps.empty')}</p></Card>
  {:else}
    <div class="grid">
      {#each groups as g (g.exe)}
        <Card class="game">
          <div class="top">
            <div>
              <div class="name">{g.name}</div>
              <div class="sub">{$t('fps.last')} · {day(g.last.started)} · {$t('fps.sessions', { n: g.valid.length })}</div>
            </div>
            <div class="nums">
              <div><span class="big">{n0(g.last.avg_fps)}</span><span class="u">{$t('fps.avg')}</span></div>
              <div><span class="big dim">{n0(g.last.low1_fps)}</span><span class="u">{$t('fps.low1')}</span></div>
            </div>
          </div>
          {#if g.valid.length >= 2}
            {@const vals = g.valid.map((x) => x.avg_fps)}
            <div class="spark"><Sparkline values={vals} min={Math.min(...vals) * 0.9} max={Math.max(...vals) * 1.05} /></div>
          {/if}
          <div class="cmp">
            {#if g.cmp.kind === 'ok'}
              <div class="ba">
                <div><span class="u">{$t('fps.before')}</span><span class="v">{n0(g.cmp.before.avg)} / {n0(g.cmp.before.low1)}</span></div>
                <span class="arrow">→</span>
                <div><span class="u">{$t('fps.after')}</span><span class="v">{n0(g.cmp.after.avg)} / {n0(g.cmp.after.low1)}</span></div>
                <div class="delta" class:up={g.cmp.deltaAvgPct >= 0} class:down={g.cmp.deltaAvgPct < 0}>
                  {pct(g.cmp.deltaAvgPct)} <span class="u">{$t('fps.avg')}</span> · {pct(g.cmp.deltaLow1Pct)} <span class="u">{$t('fps.low1')}</span>
                </div>
              </div>
              <div class="hint">{$t('fps.cmp.ok', { d: day(g.cmp.changeAt) })}</div>
            {:else if g.cmp.kind === 'none'}<div class="hint">{$t('fps.cmp.none')}</div>
            {:else if g.cmp.kind === 'no-before'}<div class="hint">{$t('fps.cmp.noBefore')}</div>
            {:else}<div class="hint">{$t('fps.cmp.noAfter')}</div>{/if}
          </div>
        </Card>
      {/each}
    </div>
  {/if}
  <p class="note">{$t('fps.note')}</p>
</section>

<style>
  section {
    margin-top: 18px;
  }
  .label {
    margin: 0 0 8px 2px;
  }
  section :global(.pad) {
    padding: 14px 18px;
  }
  .muted {
    color: var(--mut);
    margin: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 12px;
  }
  section :global(.game) {
    padding: 16px 18px 14px;
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }
  .name {
    font-size: 14px;
  }
  .sub {
    color: var(--dim);
    font-size: 11.5px;
    margin-top: 2px;
  }
  .nums {
    display: flex;
    gap: 16px;
    text-align: right;
  }
  .nums > div {
    display: flex;
    flex-direction: column;
  }
  .big {
    font-weight: 300;
    font-size: 24px;
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
  }
  .big.dim {
    color: var(--ice-2);
  }
  .u {
    font-size: 10.5px;
    color: var(--dim);
  }
  .spark {
    height: 34px;
    margin: 10px -2px 0;
  }
  .spark :global(svg) {
    width: 100%;
    height: 100%;
  }
  .cmp {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--line);
  }
  .ba {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }
  .ba > div:not(.delta) {
    display: flex;
    flex-direction: column;
  }
  .v {
    font-variant-numeric: tabular-nums;
    color: var(--ice-2);
  }
  .arrow {
    color: var(--dim);
  }
  .delta {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
  .delta.up {
    color: var(--ok);
  }
  .delta.down {
    color: var(--err);
  }
  .hint {
    color: var(--mut);
    font-size: 12px;
    margin-top: 6px;
    line-height: 1.5;
  }
  .note {
    color: var(--dim);
    font-size: 11.5px;
    margin: 10px 2px 0;
    line-height: 1.5;
  }
</style>
