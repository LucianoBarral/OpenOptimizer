import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // '..' = raiz do repositório: os CHANGELOGs de lá são embutidos com ?raw (notas de versão no app).
  server: { port: 1420, strictPort: true, fs: { allow: ['..'] } },
  build: { target: 'chrome120' },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
