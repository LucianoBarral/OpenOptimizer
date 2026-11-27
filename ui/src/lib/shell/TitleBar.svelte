<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  const win = getCurrentWindow();
</script>

<header data-tauri-drag-region>
  <span class="mono" data-tauri-drag-region>open optimizer</span>
  <div class="ctl">
    <button type="button" aria-label="Minimizar" onclick={() => win.minimize()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0 5h10" stroke="currentColor" /></svg>
    </button>
    <button type="button" aria-label="Maximizar" onclick={() => win.toggleMaximize()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><rect x=".5" y=".5" width="9" height="9" fill="none" stroke="currentColor" /></svg>
    </button>
    <button type="button" class="close" aria-label="Fechar" onclick={() => win.close()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor" /></svg>
    </button>
  </div>
</header>

<style>
  header {
    grid-column: 1 / -1;
    height: 36px;
    display: flex;
    align-items: center;
    padding-left: 14px;
    border-bottom: 1px solid var(--line);
    color: var(--dim);
    font-size: 11.5px;
  }
  .ctl {
    margin-left: auto;
    display: flex;
    height: 100%;
  }
  button {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    background: none;
    border: 0;
    color: var(--mut);
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
    background: #2a1416;
  }
</style>
