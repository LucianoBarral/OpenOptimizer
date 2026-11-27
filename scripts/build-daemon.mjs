// Compila o daemon do Modo Gaming e o coloca onde o Tauri espera um externalBin:
// app/binaries/oo-gamingd-<target-triple>.exe
// Uso direto: node scripts/build-daemon.mjs   (o CI roda isto antes do clippy/testes)
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

export function buildDaemon() {
  execSync('cargo build -p oo-gamingd --release', { cwd: root, stdio: 'inherit' });
  const host = execSync('rustc -vV', { encoding: 'utf8' }).match(/^host: (\S+)/m)?.[1];
  if (!host) throw new Error('não foi possível descobrir o target do rustc');
  const dir = path.join(root, 'app', 'binaries');
  fs.mkdirSync(dir, { recursive: true });
  fs.copyFileSync(path.join(root, 'target', 'release', 'oo-gamingd.exe'), path.join(dir, `oo-gamingd-${host}.exe`));
  console.log(`daemon → app/binaries/oo-gamingd-${host}.exe`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) buildDaemon();
