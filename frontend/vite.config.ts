import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  root: resolve(here),
  publicDir: resolve(here, 'static'),
  plugins: [svelte()],
  build: {
    outDir: resolve(here, '../dist'),
    emptyOutDir: true,
    target: 'es2022',
    sourcemap: false,
  },
  server: {
    proxy: { '/api': 'http://127.0.0.1:8080', '/health': 'http://127.0.0.1:8080' },
  },
});
