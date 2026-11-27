<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import Card from '../lib/design/Card.svelte';
  import Button from '../lib/design/Button.svelte';
  import { checkForUpdate } from '../lib/update';
  import { getClosePref, setClosePref, type ClosePref } from '../lib/closePref';
  import { t, locale, setLocale, type Locale, type MessageKey } from '../lib/i18n';

  let onClose = $state<ClosePref>(getClosePref());
  const closeOpts: ClosePref[] = ['ask', 'tray', 'quit'];

  let version = $state('');
  let checking = $state(false);
  let result = $state<'available' | 'none' | 'error' | null>(null);

  async function check() {
    checking = true;
    result = await checkForUpdate();
    checking = false;
  }
  const langs: { id: Locale; name: string }[] = [
    { id: 'pt-BR', name: 'Português (Brasil)' },
    { id: 'en', name: 'English' },
  ];
  onMount(async () => (version = await getVersion()));
</script>

<div class="page">
  <h2 class="rise">{$t('nav.settings')}</h2>

  <section class="rise" style="--i:1">
    <div class="label">{$t('settings.language')}</div>
    <Card class="pad">
      <div class="seg" role="radiogroup" aria-label={$t('settings.language')}>
        {#each langs as l (l.id)}
          <button type="button" role="radio" aria-checked={$locale === l.id} class:on={$locale === l.id} onclick={() => setLocale(l.id)}>
            {l.name}
          </button>
        {/each}
      </div>
    </Card>
  </section>

  <section class="rise" style="--i:2">
    <div class="label">{$t('settings.onClose')}</div>
    <Card class="pad">
      <div class="seg" role="radiogroup" aria-label={$t('settings.onClose')}>
        {#each closeOpts as o (o)}
          <button
            type="button"
            role="radio"
            aria-checked={onClose === o}
            class:on={onClose === o}
            onclick={() => {
              onClose = o;
              setClosePref(o);
            }}
          >
            {$t(`settings.onClose.${o}` as MessageKey)}
          </button>
        {/each}
      </div>
    </Card>
  </section>

  <section class="rise" style="--i:3">
    <div class="label">{$t('settings.about')}</div>
    <Card class="pad">
      <div class="kv"><span>{$t('settings.version')}</span><span class="mono">{version}</span></div>
      <div class="kv upd">
        <span>
          {#if result === 'none'}{$t('update.upToDate')}{:else if result === 'error'}{$t('update.error')}{/if}
        </span>
        <Button variant="ghost" disabled={checking} onclick={check}>{$t('update.check')}</Button>
      </div>
      <div class="kv">
        <span>{$t('settings.repo')}</span>
        <a href="https://github.com/LucianoBarral/OpenOptimizer" target="_blank" rel="noreferrer">github.com/LucianoBarral/OpenOptimizer</a>
      </div>
    </Card>
  </section>
</div>

<style>
  .page {
    padding: 28px 32px;
    height: 100%;
    overflow: auto;
  }
  h2 {
    margin: 0 0 8px;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  section {
    margin-top: 18px;
    max-width: 620px;
  }
  .label {
    margin: 0 0 8px 2px;
  }
  section :global(.pad) {
    padding: 14px 18px;
  }
  .seg {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    border: 1px solid var(--line-2);
    border-radius: 9px;
  }
  .seg button {
    border: 0;
    background: none;
    color: var(--mut);
    padding: 6px 14px;
    border-radius: 7px;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s;
  }
  .seg button.on {
    background: var(--ice);
    color: var(--bg);
  }
  .kv {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    color: var(--mut);
  }
  .kv span:last-child,
  a {
    color: var(--ice-2);
  }
  a {
    text-decoration: none;
  }
  a:hover {
    color: var(--ice);
  }
</style>
