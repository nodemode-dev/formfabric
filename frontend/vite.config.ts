import { defineConfig } from 'vite';

const frontendRaw = Number(process.env.FFB_FRONTEND_PORT);
const FRONTEND_PORT = Number.isInteger(frontendRaw) && frontendRaw > 0 && frontendRaw < 65534 ? frontendRaw : 5280;
const BACKEND_PORT = Number(process.env.FFB_PORT) || 8280;
const WINDOW_TOKEN = (process.env.FFB_WINDOW_TOKEN ?? '').trim();
if (WINDOW_TOKEN && WINDOW_TOKEN.length < 32) {
  throw new Error('FFB_WINDOW_TOKEN is shorter than 32 characters: the dev proxy refuses to start');
}
const PAGE_ORIGINS = new Set([FRONTEND_PORT, FRONTEND_PORT + 2].flatMap((port) => [`http://localhost:${port}`, `http://127.0.0.1:${port}`]));

type Hooked = { headers: Record<string, string | string[] | undefined>; setHeader(name: string, value: string): void };

function stamp(proxyReq: Hooked, req: Hooked): void {
  const origin = req.headers.origin;
  const page = typeof origin === 'string' ? PAGE_ORIGINS.has(origin.toLowerCase()) : req.headers['sec-fetch-site'] === 'same-origin';
  if (WINDOW_TOKEN && page) proxyReq.setHeader('x-ffb-window', WINDOW_TOKEN);
}

function withToken(proxy: { on(event: string, fn: (a: Hooked, b: Hooked) => void): void }): void {
  proxy.on('proxyReq', stamp);
  proxy.on('proxyReqWs', stamp);
}

export default defineConfig({
  root: '.',
  publicDir: 'public',
  server: {
    port: FRONTEND_PORT,
    host: false,
    strictPort: true,
    hmr: { port: FRONTEND_PORT + 1 },
    proxy: {
      '/api': { target: `http://127.0.0.1:${BACKEND_PORT}`, changeOrigin: true, ws: true, configure: withToken },
    },
  },
  preview: {
    port: FRONTEND_PORT + 2,
    host: false,
    strictPort: true,
    proxy: {
      '/api': { target: `http://127.0.0.1:${BACKEND_PORT}`, changeOrigin: true, ws: true, configure: withToken },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    target: 'es2022',
    chunkSizeWarningLimit: 1200,
  },
});
