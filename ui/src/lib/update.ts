import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { writable } from 'svelte/store';

/** Update disponível (assinatura é verificada pelo plugin antes de instalar). */
export const available = writable<Update | null>(null);

export async function checkForUpdate(): Promise<'available' | 'none' | 'error'> {
  try {
    const u = await check();
    available.set(u);
    return u ? 'available' : 'none';
  } catch {
    return 'error';
  }
}

export async function installAndRelaunch(u: Update, onProgress: (p: number) => void) {
  let total = 0;
  let got = 0;
  await u.downloadAndInstall((e) => {
    if (e.event === 'Started') total = e.data.contentLength ?? 0;
    if (e.event === 'Progress') {
      got += e.data.chunkLength;
      if (total) onProgress(got / total);
    }
  });
  await relaunch();
}

/** Verifica ao abrir e a cada 6 h. */
export function startAutoCheck() {
  void checkForUpdate();
  setInterval(() => void checkForUpdate(), 6 * 60 * 60 * 1000);
}
