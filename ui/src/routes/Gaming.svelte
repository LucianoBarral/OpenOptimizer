<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import Card from '../lib/design/Card.svelte';
  import Button from '../lib/design/Button.svelte';
  import Switch from '../lib/design/Switch.svelte';
  import LogoMark from '../lib/brand/LogoMark.svelte';
  import BusyOverlay from '../lib/brand/BusyOverlay.svelte';
  import { api, type GamingView } from '../lib/ipc/api';
  import { refresh } from '../lib/stores';
  import { t, locale, type MessageKey } from '../lib/i18n';
  import { formatBytes } from '../lib/format';

  let v = $state<GamingView | null>(null);
  let busy = $state(false);
  let error = $state(false);
  let timer: ReturnType<typeof setInterval>;

  const active = $derived(v?.status.state === 'Active');
  const gameName = $derived(
    v?.games.find((g) => g.exe.toLowerCase() === v?.status.game?.toLowerCase())?.name ?? v?.status.game ?? '',
  );
  const minutes = $derived(v?.status.since ? Math.max(0, Math.floor((Date.now() - Date.parse(v.status.since)) / 60000)) : 0);
  const hours = (s: number) => (s >= 3600 ? `${Math.floor(s / 3600)} h ${Math.floor((s % 3600) / 60)} min` : `${Math.floor(s / 60)} min`);

  async function load() {
    if (document.visibilityState !== 'visible') return;
    v = await api.gamingGet();
  }

  async function run(f: () => Promise<GamingView>, overlay = false) {
    error = false;
    if (overlay) busy = true;
    try {
      v = await f();
      await refresh();
    } catch {
      error = true;
    } finally {
      busy = false;
    }
  }

  async function addGame() {
    const path = await open({ multiple: false, filters: [{ name: 'Executável', extensions: ['exe'] }] });
    if (typeof path === 'string') await run(() => api.gamingAddGame(path));
  }

  onMount(() => {
    load();
    timer = setInterval(load, 2000);
  });
  onDestroy(() => clearInterval(timer));
</script>

<div class="page">
  <div class="hd rise">
    <h2>{$t('nav.gaming')}</h2>
    {#if v}<Switch size="lg" checked={v.enabled} label={$t('nav.gaming')} onchange={(on) => run(() => api.gamingSetEnabled(on), true)} />{/if}
  </div>

  {#if error}<p class="err rise">{$t('gaming.error')}</p>{/if}

  {#if v}
    <Card class="hero rise" style="--i:1">
      <LogoMark size={92} mode={active ? 'loop' : 'static'} />
      <div>
        <div class="label">{$t('nav.gaming')}</div>
        <h3>
          {#if !v.enabled}{$t('gaming.off')}
          {:else if active}{$t('gaming.active')} · {gameName}
          {:else}{$t('gaming.waiting')}{/if}
        </h3>
        {#if active}<div class="since">{$t('gaming.since', { m: minutes })}</div>{/if}
        <p>{$t('gaming.body')}</p>
      </div>
    </Card>

    <div class="stats rise" style="--i:2">
      <Card class="s"><div class="label">{$t('gaming.stat.sessions')}</div><div class="n">{v.status.sessions}</div></Card>
      <Card class="s"><div class="label">{$t('gaming.stat.time')}</div><div class="n">{hours(v.status.seconds_in_game)}</div></Card>
      <Card class="s"><div class="label">{$t('gaming.stat.purges')}</div><div class="n">{v.status.purges}</div></Card>
      <Card class="s"><div class="label">{$t('gaming.stat.freed')}</div><div class="n">{formatBytes(v.status.purged_mb * 1024 * 1024, $locale)}<em>GB</em></div></Card>
    </div>

    <section class="rise" style="--i:3">
      <div class="bar">
        <div class="label">{$t('gaming.games')}</div>
        <div class="act">
          <Button variant="ghost" onclick={() => run(() => api.gamingScan(), true)}>{$t('gaming.scan')}</Button>
          <Button variant="ghost" onclick={addGame}>{$t('gaming.add')}</Button>
        </div>
      </div>
      <Card class="list">
        {#each v.games as g (g.exe)}
          <div class="row" class:playing={active && v.status.game?.toLowerCase() === g.exe.toLowerCase()}>
            <div>
              <div class="name">{g.name}</div>
              <div class="exe">{g.exe}</div>
            </div>
            <span class="src">{$t(`gaming.source.${g.source}` as MessageKey)}</span>
            <Switch checked={g.enabled} label={g.name} onchange={(on) => run(() => api.gamingSetGame(g.exe, on))} />
          </div>
        {:else}
          <p class="empty">{$t('gaming.empty')}</p>
        {/each}
      </Card>
    </section>
  {/if}
</div>

{#if busy}<BusyOverlay label={$t('nav.gaming')} />{/if}

<style>
  .page {
    padding: 28px 32px;
    height: 100%;
    overflow: auto;
  }
  .hd {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-right: 110px;
  }
  h2 {
    margin: 0;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  .err {
    color: var(--err);
    margin: 12px 0 0;
  }
  .page :global(.hero) {
    margin-top: 18px;
    padding: 22px 24px;
    display: flex;
    gap: 24px;
    align-items: center;
  }
  h3 {
    margin: 6px 0 0;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.02em;
  }
  .since {
    color: var(--ok);
    font-size: 12px;
    margin-top: 2px;
  }
  .page :global(.hero p) {
    color: var(--mut);
    margin: 10px 0 0;
    line-height: 1.55;
    max-width: 620px;
  }
  .stats {
    margin-top: 16px;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }
  .stats :global(.s) {
    padding: 14px 16px;
  }
  .n {
    font-weight: 300;
    font-size: 24px;
    margin-top: 4px;
    font-variant-numeric: tabular-nums;
  }
  .n em {
    font-style: normal;
    font-size: 12px;
    color: var(--mut);
    margin-left: 3px;
  }
  section {
    margin-top: 18px;
  }
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  .act {
    display: flex;
    gap: 4px;
  }
  section :global(.list) {
    padding: 4px 20px;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr auto 36px;
    gap: 14px;
    align-items: center;
    padding: 11px 0;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child {
    border: 0;
  }
  .row.playing .name {
    color: var(--ok);
  }
  .name {
    font-size: 13.5px;
  }
  .exe {
    color: var(--dim);
    font-size: 11.5px;
    margin-top: 1px;
  }
  .src {
    font-size: 11px;
    color: var(--mut);
  }
  .empty {
    color: var(--mut);
    padding: 12px 0;
    margin: 0;
  }
</style>
