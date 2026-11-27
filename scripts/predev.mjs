// beforeDevCommand do Tauri: compila o daemon e mantém o servidor do Vite rodando.
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildDaemon } from './build-daemon.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
buildDaemon();
const vite = spawn('npm', ['run', 'dev'], { cwd: path.join(root, 'ui'), stdio: 'inherit', shell: true });
vite.on('exit', (code) => process.exit(code ?? 0));
