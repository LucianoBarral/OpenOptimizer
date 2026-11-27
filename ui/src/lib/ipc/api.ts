import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { HardwareView } from './bindings/HardwareView';
import type { MetricsView } from './bindings/MetricsView';
import type { TweakView } from './bindings/TweakView';
import type { ChangeView } from './bindings/ChangeView';
import type { BatchResult } from './bindings/BatchResult';
import type { ErrorView } from './bindings/ErrorView';
import type { Progress } from './bindings/Progress';

/** Wrapper tipado dos comandos Tauri (nomes iguais aos de app/src/commands.rs). */
export const api = {
  hardware: () => invoke<HardwareView>('hardware'),
  metrics: () => invoke<MetricsView>('metrics'),
  tweaks: () => invoke<TweakView[]>('tweaks'),
  score: () => invoke<number>('score'),
  applyTweaks: (ids: string[]) => invoke<BatchResult>('apply_tweaks', { ids }),
  revertTweak: (id: string, force = false) => invoke<void>('revert_tweak', { id, force }),
  revertChange: (id: string, force = false) => invoke<void>('revert_change', { id, force }),
  revertAll: (force = false) => invoke<ErrorView[]>('revert_all', { force }),
  history: () => invoke<ChangeView[]>('history'),
  pending: () => invoke<ChangeView[]>('pending'),
  onProgress: (cb: (p: Progress) => void) => listen<Progress>('apply-progress', (e) => cb(e.payload)),
  setTrayLabels: (open: string, optimize: string, quit: string) =>
    invoke<void>('set_tray_labels', { open, optimize, quit }),
  hideToTray: () => invoke<void>('hide_to_tray'),
  quitApp: () => invoke<void>('quit_app'),
  onCloseRequested: (cb: () => void) => listen('close-requested', () => cb()),
  onTrayOptimize: (cb: () => void) => listen('tray-optimize', () => cb()),
};

export const isErrorView = (e: unknown): e is ErrorView => typeof e === 'object' && e !== null && 'kind' in e;

export type { HardwareView, MetricsView, TweakView, ChangeView, BatchResult, ErrorView, Progress };
