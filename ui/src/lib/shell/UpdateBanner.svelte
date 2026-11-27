<script lang="ts">
  import Button from '../design/Button.svelte';
  import { available, installAndRelaunch } from '../update';
  import { applying } from '../stores';
  import { t } from '../i18n';

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
      <span class="label">{$t('update.available', { v: $available.version })}</span>
      <div class="act">
        {#if $available.body}<Button variant="ghost" onclick={() => (notes = !notes)}>{$t('update.notes')}</Button>{/if}
        <!-- Nunca atualiza no meio de um lote de tweaks. -->
        <Button {progress} disabled={$applying || installing} onclick={install}>{$t('update.install')}</Button>
      </div>
    </div>
    {#if notes && $available.body}<p>{$available.body}</p>{/if}
  </div>
{/if}

<style>
  .banner {
    margin: 16px 32px 0;
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
  .label {
    color: var(--ice-2);
  }
  .act {
    display: flex;
    gap: 6px;
  }
  p {
    color: var(--mut);
    margin: 8px 0 4px;
    white-space: pre-wrap;
    line-height: 1.5;
  }
</style>
