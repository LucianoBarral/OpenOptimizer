// Baixa o PresentMon (Intel, MIT) numa versão fixa, confere o SHA-256 e o coloca onde o Tauri
// espera um externalBin: app/binaries/presentmon-<target-triple>.exe
// Uso direto: node scripts/fetch-presentmon.mjs   (prebuild, predev e CI chamam isto)
import { execSync } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const VERSION = '2.6.0';
const ASSET = `PresentMon-${VERSION}-x64.exe`;
const URL = `https://github.com/GameTechDev/PresentMon/releases/download/v${VERSION}/${ASSET}`;
const SHA256 = 'b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sha = (buf) => crypto.createHash('sha256').update(buf).digest('hex');

export async function fetchPresentMon() {
  const host = execSync('rustc -vV', { encoding: 'utf8' }).match(/^host: (\S+)/m)?.[1];
  if (!host) throw new Error('não foi possível descobrir o target do rustc');
  const dest = path.join(root, 'app', 'binaries', `presentmon-${host}.exe`);
  // Cache: só baixa de novo se o arquivo não existir ou não bater com o hash fixo.
  if (fs.existsSync(dest) && sha(fs.readFileSync(dest)) === SHA256) {
    console.log(`presentmon ${VERSION} já presente (SHA-256 ok)`);
    return;
  }
  const res = await fetch(URL);
  if (!res.ok) throw new Error(`download do PresentMon falhou: HTTP ${res.status}`);
  const buf = Buffer.from(await res.arrayBuffer());
  const got = sha(buf);
  if (got !== SHA256) throw new Error(`SHA-256 do PresentMon não confere: ${got} (esperado ${SHA256})`);
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.writeFileSync(dest, buf);
  console.log(`presentmon ${VERSION} → ${path.relative(root, dest)} (SHA-256 ok)`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  fetchPresentMon().catch((e) => {
    console.error(e.message);
    process.exit(1);
  });
}
