<script lang="ts">
  // Notas de versão: aparece sozinho depois de uma atualização e pode ser aberto em Configurações.
  import Button from '../design/Button.svelte';
  import ReleaseNotes from './ReleaseNotes.svelte';
  import type { Release } from '../changelog';
  import { t, locale } from '../i18n';

  let { list, title, onclose }: { list: Release[]; title: string; onclose: () => void } = $props();

  const fmt = (d: string) => {
    const dt = new Date(d + 'T12:00:00');
    return isNaN(+dt) ? d : dt.toLocaleDateString($locale, { day: '2-digit', month: 'short', year: 'numeric' });
  };
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<div class="overlay" role="presentation" onclick={onclose}>
  <div class="dlg" role="dialog" aria-modal="true" aria-label={title} tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
    <h3>{title}</h3>
    <div class="body">
      {#each list as r (r.version)}
        <section>
          <div class="vh"><span class="v mono">v{r.version}</span>{#if r.date}<span class="d">{fmt(r.date)}</span>{/if}</div>
          <ReleaseNotes sections={r.sections} />
        </section>
      {/each}
    </div>
    <div class="act"><Button onclick={onclose}>{$t('notes.close')}</Button></div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 80;
    animation: fade 0.2s var(--ease);
  }
  .dlg {
    width: min(560px, calc(100vw - 48px));
    max-height: calc(100vh - 80px);
    display: flex;
    flex-direction: column;
    background: var(--card);
    border: 1px solid var(--line-2);
    border-radius: 12px;
    padding: 22px 22px 18px;
    animation: pop 0.28s var(--spring);
  }
  h3 {
    margin: 0 0 14px;
    font-weight: 400;
    font-size: 16px;
  }
  .body {
    overflow: auto;
    min-height: 0;
    margin-right: -10px;
    padding-right: 10px;
  }
  section + section {
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--line);
  }
  .vh {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 10px;
  }
  .v {
    font-size: 14px;
    font-weight: 500;
    color: var(--ice);
  }
  .d {
    color: var(--dim);
    font-size: 12px;
  }
  .act {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
  }
</style>
