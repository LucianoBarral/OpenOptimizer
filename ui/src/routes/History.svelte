<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '../lib/design/Card.svelte';
  import Button from '../lib/design/Button.svelte';
  import ConfirmDialog from '../lib/shell/ConfirmDialog.svelte';
  import { api, isErrorView, type ChangeView } from '../lib/ipc/api';
  import { refresh } from '../lib/stores';
  import { t, locale, type MessageKey } from '../lib/i18n';

  let list = $state<ChangeView[]>([]);
  let confirm = $state<null | (() => Promise<void>)>(null);
  const k = (s: string) => s as MessageKey;

  const batches = $derived.by(() => {
    const out: { id: string; when: string; items: ChangeView[] }[] = [];
    for (const c of list) {
      let b = out.find((x) => x.id === c.batch_id);
      if (!b) out.push((b = { id: c.batch_id, when: c.timestamp, items: [] }));
      b.items.push(c);
    }
    return out;
  });

  async function load() {
    list = await api.history();
    await refresh();
  }

  async function undo(id: string) {
    try {
      await api.revertChange(id);
    } catch (e) {
      if (isErrorView(e) && e.kind === 'Conflict') {
        confirm = async () => {
          await api.revertChange(id, true);
        };
        return;
      }
    }
    await load();
  }

  async function undoAll() {
    const errs = await api.revertAll();
    if (errs.some((e) => e.kind === 'Conflict')) {
      confirm = async () => {
        await api.revertAll(true);
      };
      return;
    }
    await load();
  }

  onMount(load);
</script>

<div class="page">
  <div class="hd rise">
    <h2>{$t('nav.history')}</h2>
    {#if list.some((c) => c.status === 'Applied' || c.status === 'Pending')}
      <Button variant="ghost" onclick={undoAll}>{$t('action.undoAll')}</Button>
    {/if}
  </div>

  {#if !list.length}
    <p class="empty rise" style="--i:1">{$t('history.empty')}</p>
  {/if}

  {#each batches as b, i (b.id)}
    <section class="rise" style="--i:{i + 1}">
      <div class="label">{new Date(b.when).toLocaleString($locale)}</div>
      <Card class="list">
        {#each b.items as c (c.id)}
          <div class="row">
            <span>{$t(k(`tweak.${c.tweak_id}.title`))}</span>
            <span class="st mono" class:applied={c.status === 'Applied'}>{$t(k(`history.${c.status}`))}</span>
            {#if c.status === 'Applied' || c.status === 'Pending'}
              <Button variant="ghost" onclick={() => undo(c.id)}>{$t('action.undo')}</Button>
            {:else}<span></span>{/if}
          </div>
        {/each}
      </Card>
    </section>
  {/each}
</div>

{#if confirm}
  <ConfirmDialog
    title={$t('conflict.title')}
    body={$t('conflict.body')}
    onconfirm={async () => {
      const f = confirm;
      confirm = null;
      await f?.();
      await load();
    }}
    oncancel={() => (confirm = null)}
  />
{/if}

<style>
  .page {
    padding: 28px 32px;
    height: 100%;
    overflow: auto;
  }
  .hd {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  h2 {
    margin: 0;
    font-weight: 300;
    font-size: 26px;
    letter-spacing: -0.025em;
  }
  .empty {
    color: var(--mut);
    margin-top: 24px;
  }
  section {
    margin-top: 18px;
  }
  .label {
    margin: 0 0 8px 2px;
  }
  section :global(.list) {
    padding: 4px 8px 4px 20px;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr auto 96px;
    gap: 14px;
    align-items: center;
    min-height: 46px;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child {
    border: 0;
  }
  .st {
    font-size: 11px;
    color: var(--mut);
  }
  .st.applied {
    color: var(--ok);
  }
</style>
