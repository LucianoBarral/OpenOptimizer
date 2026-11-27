<script lang="ts">
  import type { Snippet } from 'svelte';
  let { class: klass = '', style = '', children }: { class?: string; style?: string; children: Snippet } = $props();
</script>

<div class="card {klass}" {style}>{@render children()}</div>

<style>
  .card {
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: 12px;
    transition:
      border-color 0.25s,
      background 0.25s;
  }
  .card:hover {
    border-color: var(--line-2);
    background: var(--card-h);
  }
</style>
