<script lang="ts">
  import WindowControls from './lib/shell/WindowControls.svelte';
  import Sidebar from './lib/shell/Sidebar.svelte';
  import Overview from './routes/Overview.svelte';
  import Performance from './routes/Performance.svelte';
  import History from './routes/History.svelte';
  import Settings from './routes/Settings.svelte';
  import Gaming from './routes/Gaming.svelte';
  import ComingSoon from './routes/ComingSoon.svelte';
  import UpdateBanner from './lib/shell/UpdateBanner.svelte';
  import CloseDialog from './lib/shell/CloseDialog.svelte';
  import Splash from './lib/brand/Splash.svelte';
  import WhatsNew from './lib/shell/WhatsNew.svelte';
  import { unseenVersions, releasesFor } from './lib/changelogData';
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { reduceMotion } from './lib/motion';
  import { route, optimizeRequest } from './lib/stores';
  import { startAutoCheck } from './lib/update';
  import { api } from './lib/ipc/api';
  import { getClosePref, setClosePref } from './lib/closePref';
  import { t, locale, type MessageKey } from './lib/i18n';

  // Apresentação: a logo "acelera" (~1.4 s) e some; o app carrega por baixo nesse tempo.
  const reduced = get(reduceMotion);
  let splash = $state(true);
  let leaving = $state(false);
  let askClose = $state(false);
  // Novidades das versões instaladas desde a última abertura (só depois de atualizar).
  let fresh = $state<string[]>([]);
  let freshVersion = $state('');

  function close(choice: 'tray' | 'quit') {
    if (choice === 'tray') api.hideToTray();
    else api.quitApp();
  }

  onMount(() => {
    startAutoCheck();
    const t1 = setTimeout(() => (leaving = true), reduced ? 300 : 1500);
    const t2 = setTimeout(() => (splash = false), reduced ? 400 : 1900);
    getVersion()
      .then((v) => {
        freshVersion = v;
        fresh = unseenVersions(v);
      })
      .catch(() => {});
    const unClose = api.onCloseRequested(() => {
      const pref = getClosePref();
      if (pref === 'ask') askClose = true;
      else close(pref);
    });
    const unTray = api.onTrayOptimize(() => {
      route.set('overview');
      optimizeRequest.update((n) => n + 1);
    });
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
      unClose.then((f) => f());
      unTray.then((f) => f());
    };
  });

  // Menu da bandeja no idioma atual.
  $effect(() => {
    api.setTrayLabels($t('tray.open'), $t('tray.optimize'), $t('tray.quit')).catch(() => {});
  });

  const soon: Partial<Record<string, MessageKey>> = {
    gaming: 'nav.gaming',
    memory: 'nav.memory',
    drivers: 'nav.drivers',
    apps: 'nav.apps',
  };
</script>

<div class="app">
  <Sidebar />
  <main>
    <WindowControls />
    <UpdateBanner />
    <!-- {#key} remonta a tela ao trocar de rota, reexecutando a entrada escalonada. -->
    <div class="view">
      {#key $route}
        {#if $route === 'overview'}<Overview />
        {:else if $route === 'performance'}<Performance />
        {:else if $route === 'history'}<History />
        {:else if $route === 'settings'}<Settings />
        {:else if $route === 'gaming'}<Gaming />
        {:else}<ComingSoon title={soon[$route] ?? 'soon'} />{/if}
      {/key}
    </div>
  </main>
</div>

{#if askClose}
  <CloseDialog
    onchoose={(c, remember) => {
      askClose = false;
      if (remember) setClosePref(c);
      close(c);
    }}
    oncancel={() => (askClose = false)}
  />
{/if}

{#if !splash && fresh.length && !askClose}
  <WhatsNew list={releasesFor($locale).filter((r) => fresh.includes(r.version))} title={$t('notes.updated', { v: freshVersion })} onclose={() => (fresh = [])} />
{/if}

{#if splash}<Splash {leaving} />{/if}

<style>
  .app {
    display: grid;
    grid-template-columns: 224px 1fr;
    height: 100vh;
  }
  /* Coluna: faixa do topo (fixa) → banner de atualização → tela (só ela rola). */
  main {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .view {
    position: relative;
    flex: 1;
    min-height: 0;
    /* O conteúdo esmaece ao chegar na faixa do topo, em vez de ser cortado seco. */
    -webkit-mask-image: linear-gradient(to bottom, transparent 0, #000 14px);
    mask-image: linear-gradient(to bottom, transparent 0, #000 14px);
  }
</style>
