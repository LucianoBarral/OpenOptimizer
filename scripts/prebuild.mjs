// beforeBuildCommand do Tauri: interface + daemon do Modo Gaming.
import { execSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildDaemon } from './build-daemon.mjs';
import { fetchPresentMon } from './fetch-presentmon.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
execSync('npm run build', { cwd: path.join(root, 'ui'), stdio: 'inherit' });
buildDaemon();
await fetchPresentMon();
