<script lang="ts">
  import type { Snippet } from 'svelte';
  type Props = {
    variant?: 'primary' | 'ghost';
    progress?: number;
    disabled?: boolean;
    onclick?: () => void;
    children: Snippet;
  };
  let { variant = 'primary', progress = 0, disabled = false, onclick, children }: Props = $props();
</script>

<button type="button" class="btn {variant}" {disabled} {onclick}>
  {#if variant === 'primary'}<i class="fill" style="transform: scaleX({progress})"></i>{/if}
  <span>{@render children()}</span>
</button>

<style>
  .btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 16px;
    border: 0;
    border-radius: 9px;
    font-weight: 500;
    cursor: pointer;
    overflow: hidden;
    transition:
      transform 0.15s var(--ease),
      background 0.2s,
      opacity 0.2s;
  }
  .btn:active:not(:disabled) {
    transform: scale(0.97);
  }
  .btn:disabled {
    cursor: default;
    opacity: 0.55;
  }
  .primary {
    background: var(--ice);
    color: var(--bg);
  }
  .primary:hover:not(:disabled) {
    background: #fff;
  }
  .ghost {
    background: none;
    color: var(--ice-2);
    padding: 0 14px;
  }
  .ghost:hover:not(:disabled) {
    background: #17191d;
  }
  /* Progresso via scaleX (compositor) em vez de width (layout). */
  .fill {
    position: absolute;
    inset: 0;
    transform-origin: left;
    background: rgba(11, 12, 14, 0.08);
    transition: transform 0.3s var(--ease);
  }
  span {
    position: relative;
  }
  .btn:focus-visible {
    outline: 1.5px solid var(--ice);
    outline-offset: 2px;
  }
</style>
