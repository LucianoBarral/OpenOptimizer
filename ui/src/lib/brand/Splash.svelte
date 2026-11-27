<script lang="ts">
  import LogoMark from './LogoMark.svelte';
  let { leaving = false }: { leaving?: boolean } = $props();
</script>

<div class="splash" class:leaving>
  <LogoMark size={168} mode="intro" />
  <div class="name">Open Optimizer</div>
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 22px;
    background: var(--bg);
    transition: opacity 0.35s var(--ease);
  }
  .splash.leaving {
    opacity: 0;
    pointer-events: none;
  }
  .name {
    font-weight: 300;
    font-size: 20px;
    letter-spacing: 0.02em;
    color: var(--ice);
    animation: rise 0.6s var(--ease) 0.55s both;
  }
</style>
