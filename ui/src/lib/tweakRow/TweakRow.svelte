<script lang="ts">
  import Switch from '../design/Switch.svelte';
  import ConfirmDialog from '../shell/ConfirmDialog.svelte';
  import { api, isErrorView, type TweakView, type ErrorView } from '../ipc/api';
  import { refresh, applying } from '../stores';
  import { t, type MessageKey } from '../i18n';

  let { tweak, expandable = false }: { tweak: TweakView; expandable?: boolean } = $props();
  let error = $state<ErrorView | null>(null);
  let conflict = $state(false);
  let open = $state(false);
  let busy = $state(false);
  const k = (s: string) => s as MessageKey;
  const on = $derived(tweak.state === 'Applied');
  const unsupported = $derived(tweak.state === 'Unsupported');

  async function toggle(v: boolean, force = false) {
    error = null;
    busy = true;
    applying.set(true);
    try {
      if (v) {
        const r = await api.applyTweaks([tweak.id]);
        const res = r.results[0];
        if (res && !res.ok && res.error) error = res.error;
      } else {
        await api.revertTweak(tweak.id, force);
      }
    } catch (e) {
      if (isErrorView(e) && e.kind === 'Conflict') conflict = true;
      else if (isErrorView(e)) error = e;
    } finally {
      applying.set(false);
      busy = false;
      await refresh();
    }
  }
</script>

<div class="row">
  <div class="txt">
    <div class="ttl">{$t(k(`tweak.${tweak.id}.title`))}</div>
    <div class="desc">
      {unsupported ? $t('state.unsupported') : $t(k(`tweak.${tweak.id}.desc`))}
      {#if tweak.requires_reboot}<span class="chip">{$t('state.reboot')}</span>{/if}
    </div>
    {#if error}<div class="err">{$t(k(`err.${error.kind}`), { code: error.code ?? '' })}</div>{/if}
    {#if expandable}
      <button type="button" class="why" aria-expanded={open} onclick={() => (open = !open)}>{$t('action.why')}</button>
      {#if open}<p class="rationale">{$t(k(`tweak.${tweak.id}.why`))}</p>{/if}
    {/if}
  </div>
  <span class="tag mono">{$t(k(`cat.${tweak.category}`))}</span>
  <Switch checked={on} disabled={unsupported || busy} label={$t(k(`tweak.${tweak.id}.title`))} onchange={(v) => toggle(v)} />
</div>

{#if conflict}
  <ConfirmDialog
    title={$t('conflict.title')}
    body={$t('conflict.body')}
    onconfirm={() => {
      conflict = false;
      toggle(false, true);
    }}
    oncancel={() => (conflict = false)}
  />
{/if}

<style>
  .row {
    display: grid;
    grid-template-columns: 1fr auto 36px;
    gap: 14px;
    align-items: center;
    padding: 13px 0;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child {
    border: 0;
  }
  .ttl {
    font-size: 13.5px;
  }
  .desc {
    color: var(--mut);
    font-size: 12px;
    margin-top: 1px;
  }
  .tag {
    font-size: 10.5px;
    color: var(--mut);
    letter-spacing: 0.04em;
  }
  .err {
    color: var(--err);
    font-size: 12px;
    margin-top: 4px;
  }
  .chip {
    margin-left: 6px;
    border: 1px solid var(--line-2);
    border-radius: 5px;
    padding: 0 5px;
    font-size: 11px;
  }
  .why {
    background: none;
    border: 0;
    padding: 0;
    margin-top: 6px;
    color: var(--ice-2);
    font-size: 12px;
    cursor: pointer;
  }
  .why:hover {
    color: var(--ice);
  }
  .rationale {
    color: var(--mut);
    font-size: 12px;
    line-height: 1.55;
    margin: 6px 0 0;
    max-width: 560px;
    animation: rise 0.3s var(--ease);
  }
</style>
