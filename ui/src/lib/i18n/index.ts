import { writable, derived } from 'svelte/store';
import ptBR, { type MessageKey } from './pt-BR';
import en from './en';

export type Locale = 'pt-BR' | 'en';
export type { MessageKey };

const dicts: Record<Locale, Record<MessageKey, string>> = { 'pt-BR': ptBR, en };

function initial(): Locale {
  try {
    const s = localStorage.getItem('oo.locale');
    if (s === 'pt-BR' || s === 'en') return s;
  } catch {
    /* sem storage */
  }
  return navigator.language.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en';
}

export const locale = writable<Locale>(initial());

export function setLocale(l: Locale) {
  locale.set(l);
  try {
    localStorage.setItem('oo.locale', l);
  } catch {
    /* ignora */
  }
}

/** Uso: {$t('nav.overview')} — reage à troca de idioma. */
export const t = derived(
  locale,
  (l) =>
    (key: MessageKey, vars: Record<string, string | number> = {}) =>
      dicts[l][key].replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? '')),
);
