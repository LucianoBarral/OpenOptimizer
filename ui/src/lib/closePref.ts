/** O que fazer ao clicar no X da janela. */
export type ClosePref = 'ask' | 'tray' | 'quit';

const KEY = 'oo.onClose';

export function getClosePref(): ClosePref {
  try {
    const v = localStorage.getItem(KEY);
    if (v === 'tray' || v === 'quit') return v;
  } catch {
    /* sem storage: pergunta */
  }
  return 'ask';
}

export function setClosePref(v: ClosePref) {
  try {
    localStorage.setItem(KEY, v);
  } catch {
    /* ignora */
  }
}
