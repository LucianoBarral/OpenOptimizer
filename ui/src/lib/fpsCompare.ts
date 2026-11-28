// Antes → depois de uma mudança de otimização, por jogo, a partir das sessões medidas.
// Honesto por construção: só sessões válidas (≥ 60 s medidos); sem sessões dos dois lados,
// devolve o motivo em vez de um número.
import type { FpsRecordView } from './ipc/bindings/FpsRecordView';

export type Agg = { avg: number; low1: number; n: number };
export type Comparison =
  | { kind: 'ok'; changeAt: number; before: Agg; after: Agg; deltaAvgPct: number; deltaLow1Pct: number }
  | { kind: 'none' } // nenhuma sessão válida
  | { kind: 'no-before' } // todas as sessões são depois das mudanças (otimizou antes de jogar)
  | { kind: 'no-after' }; // houve mudança depois da última sessão: falta jogar de novo

const SIDE = 3;

export function median(xs: number[]): number {
  const v = [...xs].sort((a, b) => a - b);
  const m = v.length >> 1;
  return v.length % 2 ? v[m] : (v[m - 1] + v[m]) / 2;
}

const agg = (l: FpsRecordView[]): Agg => ({
  avg: median(l.map((x) => x.avg_fps)),
  low1: median(l.map((x) => x.low1_fps)),
  n: l.length,
});

/**
 * `changeTimes`: instantes (unix s) em que alguma otimização foi aplicada ou desfeita.
 * Usa a mudança mais recente que tenha sessões válidas dos dois lados; cada lado fica
 * limitado às sessões entre essa mudança e a vizinha, até 3 (as mais próximas dela).
 */
export function compareGame(sessions: FpsRecordView[], changeTimes: number[]): Comparison {
  const valid = sessions.filter((x) => x.valid).sort((a, b) => a.started - b.started);
  if (!valid.length) return { kind: 'none' };
  const changes = [...new Set(changeTimes)].sort((a, b) => a - b);
  for (let i = changes.length - 1; i >= 0; i--) {
    const c = changes[i];
    const prev = changes[i - 1] ?? -Infinity;
    const next = changes[i + 1] ?? Infinity;
    const before = valid.filter((x) => x.started > prev && x.started < c).slice(-SIDE);
    const after = valid.filter((x) => x.started > c && x.started < next).slice(0, SIDE);
    if (before.length && after.length) {
      const b = agg(before);
      const a = agg(after);
      return {
        kind: 'ok',
        changeAt: c,
        before: b,
        after: a,
        deltaAvgPct: ((a.avg - b.avg) / b.avg) * 100,
        deltaLow1Pct: ((a.low1 - b.low1) / b.low1) * 100,
      };
    }
  }
  const last = valid[valid.length - 1].started;
  return changes.some((c) => c > last) ? { kind: 'no-after' } : { kind: 'no-before' };
}
