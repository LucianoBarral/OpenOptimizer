<script lang="ts">
  // Faixa superior do conteúdo: invisível (mesma cor do fundo), arrasta a janela e
  // guarda os controles. Fica FORA da área que rola, então nada passa por baixo dela.
  import { getCurrentWindow } from '@tauri-apps/api/window';
  const win = getCurrentWindow();
</script>

<div class="topbar" data-tauri-drag-region>
  <div class="ctl">
    <button type="button" aria-label="Minimizar" onclick={() => win.minimize()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 5h8" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button type="button" aria-label="Maximizar" onclick={() => win.toggleMaximize()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button type="button" class="close" aria-label="Fechar" onclick={() => win.close()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M2 2l6 6M8 2L2 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /></svg>
    </button>
  </div>
</div>

<style>
  .topbar {
    flex: none;
    height: 44px;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    padding-right: 12px;
  }
  .ctl {
    display: flex;
    gap: 2px;
  }
  button {
    width: 30px;
    height: 28px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 7px;
    background: none;
    color: var(--dim);
    cursor: pointer;
    transition:
      background 0.15s,
      color 0.15s;
  }
  button:hover {
    background: #17191d;
    color: var(--ice);
  }
  .close:hover {
    background: #3a1a1d;
    color: #ffd9dc;
  }
  button:focus-visible {
    outline: 1.5px solid var(--ice);
    outline-offset: 1px;
  }
</style>
