import type { MessageKey } from './i18n';

type T = (key: MessageKey, vars?: Record<string, string | number>) => string;

/** Título de um tweak, incluindo os dinâmicos do Modo Gaming (`ifeo:<exe>`). */
export function tweakTitle(t: T, id: string): string {
  if (id.startsWith('ifeo:')) return t('tweak.ifeo.title', { exe: id.slice(5) });
  return t(`tweak.${id}.title` as MessageKey);
}
