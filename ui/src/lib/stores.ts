import { writable } from 'svelte/store';
import { api, type TweakView } from './ipc/api';

export type Route = 'overview' | 'performance' | 'gaming' | 'memory' | 'drivers' | 'apps' | 'history' | 'settings';

export const route = writable<Route>('overview');
export const tweaks = writable<TweakView[]>([]);
export const score = writable(0);
/** true durante um lote de tweaks: bloqueia a instalação de updates no meio. */
export const applying = writable(false);

export async function refresh() {
  const [t, s] = await Promise.all([api.tweaks(), api.score()]);
  tweaks.set(t);
  score.set(s);
}

export const pendingCount = (list: TweakView[]) =>
  list.filter((x) => x.state === 'NotApplied' || x.state === 'Partial').length;
