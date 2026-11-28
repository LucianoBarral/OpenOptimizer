<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import LogoMark from '../brand/LogoMark.svelte';
  import { route, tweaks, pendingCount, type Route } from '../stores';
  import { t, type MessageKey } from '../i18n';

  type Item = { id: Route; key: MessageKey; icon: string; soon?: boolean };
  const groups: { label?: MessageKey; items: Item[] }[] = [
    { items: [{ id: 'overview', key: 'nav.overview', icon: 'M3 3h7v9H3zM14 3h7v5h-7zM14 12h7v9h-7zM3 16h7v5H3z' }] },
    {
      label: 'nav.optimize',
      items: [
        { id: 'performance', key: 'nav.performance', icon: 'M13 2 4 14h7l-1 8 9-12h-7z' },
        { id: 'gaming', key: 'nav.gaming', icon: 'M6 6h12a4 4 0 0 1 4 4v4a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4v-4a4 4 0 0 1 4-4zM7 10v4M5 12h4' },
        { id: 'memory', key: 'nav.memory', icon: 'M4 6h16v12H4zM8 6V3M12 6V3M16 6V3M8 21v-3M12 21v-3M16 21v-3', soon: true },
      ],
    },
    {
      label: 'nav.system',
      items: [
        { id: 'drivers', key: 'nav.drivers', icon: 'M12 3v12M7 10l5 5 5-5M4 21h16', soon: true },
        { id: 'apps', key: 'nav.apps', icon: 'M4 6h16M4 12h10M4 18h7M18 13a3 3 0 1 1 0 6 3 3 0 0 1 0-6z', soon: true },
        { id: 'history', key: 'nav.history', icon: 'M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5M12 7v5l3 2' },
      ],
    },
  ];
  const settings: Item = {
    id: 'settings',
    key: 'nav.settings',
    icon: 'M12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6zM19 12l2-1-1-3-2 .5-1.5-1.5.5-2-3-1-1 2h-2l-1-2-3 1 .5 2L6 7.5 4 7 3 10l2 1v2l-2 1 1 3 2-.5L7.5 18 7 20l3 1 1-2h2l1 2 3-1-.5-2 1.5-1.5 2 .5 1-3-2-1z',
  };

  const els: Record<string, HTMLElement> = {};
  let y = $state(0);
  let version = $state('');
  const badge = $derived(pendingCount($tweaks));

  // Configurações acompanha a altura da janela, então o indicador é remedido ao redimensionar.
  let h = $state(0);
  $effect(() => {
    void h;
    const el = els[$route];
    if (el) y = el.offsetTop;
  });
  onMount(async () => {
    version = await getVersion();
  });
</script>

{#snippet navItem(it: Item)}
  <button type="button" class="nav" class:on={$route === it.id} bind:this={els[it.id]} onclick={() => route.set(it.id)}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" stroke-linecap="round"><path d={it.icon} /></svg>
    {$t(it.key)}
    {#if it.soon}<span class="soon">{$t('soon')}</span>
    {:else if it.id === 'performance' && badge > 0}<span class="bd mono">{badge}</span>{/if}
  </button>
{/snippet}

<svelte:window bind:innerHeight={h} />

<aside>
  <div class="brand" data-tauri-drag-region>
    <LogoMark size={30} />
    <span data-tauri-drag-region>Open Optimizer</span>
  </div>
  <nav>
    <div class="ind" style="transform: translateY({y}px)"></div>
    {#each groups as g}
      {#if g.label}<div class="sec label">{$t(g.label)}</div>{/if}
      {#each g.items as it (it.id)}{@render navItem(it)}{/each}
    {/each}
    <!-- Configurações fica no pé da barra (margin-top:auto), separada das funções. -->
    <div class="bottom">{@render navItem(settings)}</div>
  </nav>
  <div class="foot"><span class="mono">v{version}</span></div>
</aside>

<style>
  aside {
    background: var(--panel);
    border-right: 1px solid var(--line);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px 18px;
    font-weight: 500;
    font-size: 13.5px;
    letter-spacing: -0.01em;
  }
  nav {
    position: relative;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .sec {
    padding: 16px 10px 6px;
  }
  .bottom {
    margin-top: auto;
    padding-top: 12px;
    display: flex;
    flex-direction: column;
  }
  .ind {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 34px;
    border-radius: 7px;
    background: #17191d;
    border: 1px solid var(--line-2);
    transition: transform 0.38s var(--ease);
  }
  .nav {
    position: relative;
    display: flex;
    align-items: center;
    gap: 11px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: var(--mut);
    cursor: pointer;
    text-align: left;
    transition: color 0.2s;
  }
  .nav:hover {
    color: var(--ice-2);
  }
  .nav.on {
    color: var(--ice);
  }
  .nav svg {
    flex: none;
    transition: transform 0.3s var(--spring);
  }
  .nav:hover svg {
    transform: translateX(1px) scale(1.06);
  }
  .bd {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--ice-2);
    border: 1px solid var(--line-2);
    border-radius: 5px;
    padding: 0 5px;
    line-height: 17px;
  }
  .soon {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--dim);
  }
  .foot {
    margin-top: 10px;
    padding: 12px 10px 2px;
    border-top: 1px solid var(--line);
    color: var(--dim);
    font-size: 11.5px;
  }
</style>
