<script lang="ts">
  import LogoMark from './LogoMark.svelte';
  let { label, sub = '' }: { label: string; sub?: string } = $props();
</script>

<div class="busy" role="status" aria-live="polite">
  <LogoMark size={128} mode="loop" />
  <div class="title">{label}</div>
  <div class="sub mono">{sub}</div>
</div>

<style>
  .busy {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 14px;
    background: rgba(11, 12, 14, 0.94);
    animation: fade 0.25s var(--ease) both;
  }
  .title {
    margin-top: 8px;
    font-weight: 300;
    font-size: 22px;
    letter-spacing: -0.01em;
  }
  .sub {
    min-height: 1em;
    font-size: 12px;
    color: var(--mut);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
</style>
