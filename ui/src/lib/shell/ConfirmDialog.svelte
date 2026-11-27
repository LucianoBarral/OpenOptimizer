<script lang="ts">
  import Button from '../design/Button.svelte';
  import { t } from '../i18n';
  let { title, body, onconfirm, oncancel }: { title: string; body: string; onconfirm: () => void; oncancel: () => void } =
    $props();
  let cancelWrap = $state<HTMLDivElement>();
  $effect(() => cancelWrap?.querySelector('button')?.focus());
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && oncancel()} />

<div class="overlay" role="presentation" onclick={oncancel}>
  <div
    class="dlg"
    role="alertdialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
  >
    <h3>{title}</h3>
    <p>{body}</p>
    <div class="act">
      <div bind:this={cancelWrap}><Button variant="ghost" onclick={oncancel}>{$t('action.cancel')}</Button></div>
      <Button onclick={onconfirm}>{$t('action.continue')}</Button>
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
    z-index: 50;
    animation: fade 0.2s var(--ease);
  }
  .dlg {
    width: 400px;
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
    margin: 8px 0 20px;
    line-height: 1.55;
  }
  .act {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
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
