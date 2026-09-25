// @ts-check
import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import tailwindcss from '@tailwindcss/vite';
import { existsSync, readFileSync } from 'node:fs';
import { parseEnv } from 'node:util';

// The dev proxy targets the backend port from the repo root `.env`.
const rootEnv = existsSync('../.env') ? parseEnv(readFileSync('../.env', 'utf8')) : {};
const backendPort = rootEnv.PORT || '8080';

// Production builds use a placeholder base path that the backend replaces at startup with
// BASE_PATH, so the same build works under any prefix. Must match BASE_PLACEHOLDER in
// backend/src/static_files.rs.
const isBuild = process.argv.includes('build');

export default defineConfig({
  base: isBuild ? '/__TT_BASE__' : '/',
  trailingSlash: 'ignore',
  build: { format: 'directory' },
  integrations: [svelte()],
  vite: {
    plugins: [tailwindcss()],
    optimizeDeps: {
      // Dynamically imported (QR popup): pre-bundle it at startup, otherwise Vite discovers it
      // on first use, re-optimizes and the in-flight import fails with "Outdated Optimize Dep".
      include: ['qrcode'],
    },
    server: {
      // Dev only: forward API calls to the backend so the frontend can use relative URLs.
      proxy: {
        '/api': `http://127.0.0.1:${backendPort}`,
      },
    },
  },
});
