import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vitest/config';

const projectRoot = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@app': resolve(projectRoot, '../../neo-fd-desktop/src'),
      '@tauri-apps/api': resolve(projectRoot, 'node_modules/@tauri-apps/api'),
      '@tauri-apps/plugin-dialog': resolve(
        projectRoot,
        'node_modules/@tauri-apps/plugin-dialog',
      ),
      '@tauri-apps/plugin-opener': resolve(
        projectRoot,
        'node_modules/@tauri-apps/plugin-opener',
      ),
      '@vueuse/core': resolve(projectRoot, 'node_modules/@vueuse/core'),
      pinia: resolve(projectRoot, 'node_modules/pinia'),
      vue: resolve(projectRoot, 'node_modules/vue'),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['unit/**/*.spec.ts'],
    setupFiles: ['unit/setup.ts'],
  },
});
