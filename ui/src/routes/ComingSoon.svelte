<script lang="ts">
  import { t, type MessageKey } from '../lib/i18n';
  let { title }: { title: MessageKey } = $props();
</script>

<div class="page">
  <h2 class="rise">{$t(title)}</h2>
  <p class="rise" style="--i:1"><span class="label">{$t('soon')}</span>{$t('soon.body')}</p>
</div>

<style>
  .page {
    padding: 28px 32px;
  }
  h2 {
    margin: 0;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  p {
    color: var(--mut);
    margin-top: 10px;
    display: flex;
    gap: 10px;
    align-items: baseline;
  }
</style>
