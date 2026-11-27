<script lang="ts">
  import Button from '../design/Button.svelte';
  import { t } from '../i18n';
  let { onchoose, oncancel }: { onchoose: (c: 'tray' | 'quit', remember: boolean) => void; oncancel: () => void } =
    $props();
  let remember = $state(false);
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && oncancel()} />

<div class="overlay" role="presentation" onclick={oncancel}>
  <div class="dlg" role="alertdialog" aria-modal="true" aria-label={$t('close.title')} tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
    <h3>{$t('close.title')}</h3>
    <p>{$t('close.body')}</p>
    <label class="rem"><input type="checkbox" bind:checked={remember} /> {$t('close.remember')}</label>
    <div class="act">
      <Button variant="ghost" onclick={() => onchoose('quit', remember)}>{$t('close.quit')}</Button>
      <Button onclick={() => onchoose('tray', remember)}>{$t('close.tray')}</Button>
    </div>
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
    width: 420px;
    background: var(--card);
    border: 1px solid var(--line-2);
    border-radius: 12px;
    padding: 22px;
    animation: pop 0.28s var(--spring);
  }
  h3 {
    margin: 0;
    font-weight: 400;
    font-size: 16px;
  }
  p {
    color: var(--mut);
    margin: 8px 0 16px;
    line-height: 1.55;
  }
  .rem {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--ice-2);
    cursor: pointer;
  }
  .rem input {
    accent-color: var(--ice);
  }
  .act {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
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
