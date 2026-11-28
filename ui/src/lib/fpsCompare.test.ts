import { describe, it, expect } from 'vitest';
import { compareGame, median } from './fpsCompare';
import type { FpsRecordView } from './ipc/bindings/FpsRecordView';

const s = (started: number, avg: number, low1 = avg / 2, valid = true): FpsRecordView => ({
  game: 'game.exe',
  started,
  duration_s: 600,
  measured_s: 585,
  frames: avg * 585,
  avg_fps: avg,
  low1_fps: low1,
  low01_fps: low1 / 2,
  ft_p50_ms: 1000 / avg,
  ft_p99_ms: 1000 / low1,
  runtime: 'DXGI',
  valid,
});

describe('fpsCompare', () => {
  it('mediana', () => {
    expect(median([3, 1, 2])).toBe(2);
    expect(median([4, 1, 2, 3])).toBe(2.5);
  });

  it('sem sessões válidas', () => {
    expect(compareGame([s(10, 100, 50, false)], [5]).kind).toBe('none');
  });

  it('otimizou antes de jogar: não há "antes"', () => {
    expect(compareGame([s(100, 120), s(200, 125)], [50]).kind).toBe('no-before');
  });

  it('mudou depois da última sessão: falta jogar de novo', () => {
    expect(compareGame([s(100, 120), s(200, 125)], [300]).kind).toBe('no-after');
  });

  it('sem nenhuma mudança registrada', () => {
    expect(compareGame([s(100, 120)], []).kind).toBe('no-before');
  });

  it('compara medianas de até 3 sessões de cada lado da última mudança', () => {
    const list = [s(10, 60), s(20, 100), s(30, 90), s(40, 80), /* mudança em 50 */ s(60, 120), s(70, 110), s(80, 130), s(90, 200)];
    const c = compareGame(list, [50]);
    expect(c.kind).toBe('ok');
    if (c.kind !== 'ok') return;
    expect(c.changeAt).toBe(50);
    // antes: 3 mais recentes (100, 90, 80) → 90; depois: 3 primeiras (120, 110, 130) → 120
    expect(c.before.avg).toBe(90);
    expect(c.after.avg).toBe(120);
    expect(c.before.n).toBe(3);
    expect(Math.round(c.deltaAvgPct)).toBe(33);
  });

  it('ignora sessões inválidas e usa a última mudança com sessões dos dois lados', () => {
    const list = [s(10, 100), s(20, 1, 1, false), s(30, 110), s(70, 150)];
    const c = compareGame(list, [25, 60, 500]);
    expect(c.kind).toBe('ok');
    if (c.kind !== 'ok') return;
    expect(c.changeAt).toBe(60);
    // antes de 60 (e depois da mudança anterior, 25): só a sessão de 30
    expect(c.before.avg).toBe(110);
    expect(c.after.avg).toBe(150);
  });
});
