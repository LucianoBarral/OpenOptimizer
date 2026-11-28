<script lang="ts">
  import Button from '../design/Button.svelte';
  import { available, installAndRelaunch } from '../update';
  import { applying } from '../stores';
  import { t, locale } from '../i18n';
  import ReleaseNotes from './ReleaseNotes.svelte';
  import { parseSections, pickLang } from '../changelog';

  let notes = $state(false);
  let progress = $state(0);
  let installing = $state(false);

  async function install() {
    if (!$available) return;
    installing = true;
    try {
      await installAndRelaunch($available, (p) => (progress = p));
    } catch {
      installing = false;
    }
  }
</script>

{#if $available}
  <div class="banner rise">
    <div class="row">
      <span class="msg">{$t('update.available', { v: $available.version })}</span>
      <div class="act">
        {#if $available.body}<Button variant="ghost" onclick={() => (notes = !notes)}>{$t('update.notes')}</Button>{/if}
        <!-- Nunca atualiza no meio de um lote de tweaks. -->
        <Button {progress} disabled={$applying || installing} onclick={install}>{$t('update.install')}</Button>
      </div>
    </div>
    {#if notes && $available.body}<div class="notes"><ReleaseNotes sections={parseSections(pickLang($available.body, $locale))} /></div>{/if}
  </div>
{/if}

<style>
  /* Afastado da faixa dos controles da janela e com respiro antes do título da tela. */
  .banner {
    margin: 22px 32px 14px;
    padding: 10px 10px 10px 16px;
    border: 1px solid var(--line-2);
    border-radius: 12px;
    background: var(--card);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .msg {
    color: var(--ice-2);
  }
  .act {
    display: flex;
    gap: 6px;
  }
  .notes {
    margin: 12px 6px 6px 0;
    padding-top: 12px;
    border-top: 1px solid var(--line);
    max-height: 220px;
    overflow: auto;
  }
</style>
