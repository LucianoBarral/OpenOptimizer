<script lang="ts">
  type Props = {
    checked?: boolean;
    disabled?: boolean;
    size?: 'sm' | 'lg';
    label: string;
    onchange?: (v: boolean) => void;
  };
  let { checked = false, disabled = false, size = 'sm', label, onchange }: Props = $props();

  const dims = { sm: { w: 34, h: 20 }, lg: { w: 46, h: 26 } } as const;
  const PAD = 3;
  // Percurso do knob derivado das medidas do trilho: nunca sai do trilho.
  const d = $derived(dims[size]);
  const knob = $derived(d.h - PAD * 2);
  const travel = $derived(d.w - PAD * 2 - knob);
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  class="sw"
  class:on={checked}
  style="--w:{d.w}px;--h:{d.h}px;--pad:{PAD}px;--k:{knob}px;--travel:{travel}px"
  onclick={() => onchange?.(!checked)}><span class="knob"></span></button>

<style>
  .sw {
    flex: none; /* obrigatório: o flexbox não pode espremer o trilho */
    position: relative;
    width: var(--w);
    height: var(--h);
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: #202227;
    cursor: pointer;
    transition: background 0.25s var(--ease);
  }
  .sw:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .sw.on {
    background: var(--ice);
  }
  .knob {
    position: absolute;
    top: var(--pad);
    left: var(--pad);
    width: var(--k);
    height: var(--k);
    border-radius: 50%;
    background: var(--mut);
    transition:
      transform 0.42s var(--spring),
      background 0.25s;
  }
  .sw.on .knob {
    transform: translateX(var(--travel));
    background: var(--bg);
  }
  .sw:focus-visible {
    outline: 1.5px solid var(--ice);
    outline-offset: 2px;
  }
</style>
