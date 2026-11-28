// CHANGELOG.md (PT) e CHANGELOG.en.md embutidos no build: as notas no app são exatamente as da Release.
import rawPt from '../../../CHANGELOG.md?raw';
import rawEn from '../../../CHANGELOG.en.md?raw';
import { isNewer, parseChangelog, type Release } from './changelog';

const byLocale: Record<string, Release[]> = {
  'pt-BR': parseChangelog(rawPt),
  en: parseChangelog(rawEn),
};

export const releasesFor = (locale: string) => byLocale[locale] ?? byLocale['pt-BR'];

const KEY = 'oo.lastSeenVersion';

/**
 * Versões que o usuário ainda não viu desde a última vez que abriu o app.
 * Instalação nova (nada salvo) não mostra nada: só quem atualizou vê as novidades.
 */
export function unseenVersions(current: string): string[] {
  let last: string | null = null;
  try {
    last = localStorage.getItem(KEY);
    localStorage.setItem(KEY, current);
  } catch {
    return [];
  }
  if (!last || !isNewer(current, last)) return [];
  const from = last;
  return byLocale['pt-BR'].map((r) => r.version).filter((v) => isNewer(v, from) && !isNewer(v, current));
}
