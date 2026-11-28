<script lang="ts">
  import { parseInline, type Section } from '../changelog';
  let { sections }: { sections: Section[] } = $props();
</script>

{#each sections as s}
  {#if s.title}<div class="st">{s.title}</div>{/if}
  <ul>
    {#each s.items as it}
      <li>
        {#each parseInline(it) as seg}{#if seg.kind === 'bold'}<b>{seg.text}</b>{:else if seg.kind === 'code'}<code>{seg.text}</code>{:else}{seg.text}{/if}{/each}
      </li>
    {/each}
  </ul>
{/each}

<style>
  .st {
    margin: 14px 0 6px;
    font: 500 10px var(--mono);
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--dim);
  }
  .st:first-child {
    margin-top: 0;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    position: relative;
    padding-left: 14px;
    color: var(--ice-2);
    line-height: 1.55;
    margin: 4px 0;
    user-select: text;
  }
  li::before {
    content: '';
    position: absolute;
    left: 2px;
    top: 0.68em;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--dim);
  }
  b {
    color: var(--ice);
    font-weight: 500;
  }
  code {
    font: 11.5px ui-monospace, 'Cascadia Mono', Consolas, monospace;
    background: #1a1c21;
    border: 1px solid var(--line-2);
    border-radius: 4px;
    padding: 0 4px;
  }
</style>
