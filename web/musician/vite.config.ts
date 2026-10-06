import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const apiProxyTarget = process.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8080';

// https://vitejs.dev/config/
export default defineConfig({
  base: '/musician/',
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
  },
  server: {
    proxy: {
      '/api': apiProxyTarget,
      '/ws': {
        target: apiProxyTarget.replace(/^http/, 'ws'),
        ws: true,
      },
    },
  },
} as object);
