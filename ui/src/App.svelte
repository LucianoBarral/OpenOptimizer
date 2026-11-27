<script lang="ts">
  import TitleBar from './lib/shell/TitleBar.svelte';
  import Sidebar from './lib/shell/Sidebar.svelte';
  import Overview from './routes/Overview.svelte';
  import Performance from './routes/Performance.svelte';
  import History from './routes/History.svelte';
  import Settings from './routes/Settings.svelte';
  import ComingSoon from './routes/ComingSoon.svelte';
  import UpdateBanner from './lib/shell/UpdateBanner.svelte';
  import { onMount } from 'svelte';
  import { route } from './lib/stores';
  import { startAutoCheck } from './lib/update';
  import type { MessageKey } from './lib/i18n';

  onMount(startAutoCheck);

  const soon: Partial<Record<string, MessageKey>> = {
    gaming: 'nav.gaming',
    memory: 'nav.memory',
    drivers: 'nav.drivers',
    apps: 'nav.apps',
  };
</script>

<div class="app">
  <TitleBar />
  <Sidebar />
  <main>
    <UpdateBanner />
    <!-- {#key} remonta a tela ao trocar de rota, reexecutando a entrada escalonada. -->
    {#key $route}
      {#if $route === 'overview'}<Overview />
      {:else if $route === 'performance'}<Performance />
      {:else if $route === 'history'}<History />
      {:else if $route === 'settings'}<Settings />
      {:else}<ComingSoon title={soon[$route] ?? 'soon'} />{/if}
    {/key}
  </main>
</div>

<style>
  .app {
    display: grid;
    grid-template: 36px 1fr / 224px 1fr;
    height: 100vh;
  }
  main {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }
</style>
