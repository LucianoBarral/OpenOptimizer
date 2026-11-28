// CHANGELOG.md da raiz do repositório embutido no build: as notas no app são exatamente as da Release.
import raw from '../../../CHANGELOG.md?raw';
import { isNewer, parseChangelog } from './changelog';

export const releases = parseChangelog(raw);

const KEY = 'oo.lastSeenVersion';

/**
 * Versões que o usuário ainda não viu desde a última vez que abriu o app.
 * Instalação nova (nada salvo) não mostra nada: só quem atualizou vê as novidades.
 */
export function unseenReleases(current: string) {
  let last: string | null = null;
  try {
    last = localStorage.getItem(KEY);
    localStorage.setItem(KEY, current);
  } catch {
    return [];
  }
  if (!last || !isNewer(current, last)) return [];
  return releases.filter((r) => isNewer(r.version, last) && !isNewer(r.version, current));
}
