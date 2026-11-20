import { defineConfig } from 'vite';

const frontendRaw = Number(process.env.FFB_FRONTEND_PORT);
const FRONTEND_PORT = Number.isInteger(frontendRaw) && frontendRaw > 0 && frontendRaw < 65534 ? frontendRaw : 5280;
const BACKEND_PORT = Number(process.env.FFB_PORT) || 8280;

export default defineConfig({
  root: '.',
  publicDir: 'public',
  server: {
    port: FRONTEND_PORT,
    host: false,
    strictPort: true,
    hmr: { port: FRONTEND_PORT + 1 },
    proxy: {
      '/api': { target: `http://127.0.0.1:${BACKEND_PORT}`, changeOrigin: true, ws: true },
    },
  },
  preview: {
    port: FRONTEND_PORT + 2,
    host: false,
    strictPort: true,
    proxy: {
      '/api': { target: `http://127.0.0.1:${BACKEND_PORT}`, changeOrigin: true, ws: true },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    target: 'es2022',
    chunkSizeWarningLimit: 1200,
  },
});
