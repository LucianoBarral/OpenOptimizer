<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '../lib/design/Card.svelte';
  import TweakRow from '../lib/tweakRow/TweakRow.svelte';
  import { tweaks, refresh } from '../lib/stores';
  import { t, type MessageKey } from '../lib/i18n';

  const order = ['fps', 'energy', 'input'];
  const groups = $derived(
    order.map((cat) => ({ cat, items: $tweaks.filter((x) => x.category === cat) })).filter((g) => g.items.length),
  );
  onMount(refresh);
</script>

<div class="page">
  <h2 class="rise">{$t('nav.performance')}</h2>
  {#each groups as g, i (g.cat)}
    <section class="rise" style="--i:{i + 1}">
      <div class="label">{$t(`cat.${g.cat}` as MessageKey)}</div>
      <Card class="list">
        {#each g.items as tw (tw.id)}<TweakRow tweak={tw} expandable />{/each}
      </Card>
    </section>
  {/each}
</div>

<style>
  .page {
    padding: 4px 32px 28px;
    height: 100%;
    overflow: auto;
  }
  h2 {
    margin: 0 0 8px;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  section {
    margin-top: 18px;
  }
  .label {
    margin: 0 0 8px 2px;
  }
  section :global(.list) {
    padding: 4px 20px;
  }
</style>
