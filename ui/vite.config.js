import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('.', import.meta.url));
export default defineConfig({
  root,
  clearScreen: false,
  envDir: false,
  envPrefix: 'PUBLIC_UI_',
  plugins: [tailwindcss(), {
    name: 'alexia-local-only',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const host = (req.headers.host || '').replace(/:\d+$/, '');
        if (!['localhost', '127.0.0.1', '[::1]'].includes(host)
          || ['forwarded', 'x-forwarded-for', 'x-forwarded-host', 'cf-connecting-ip'].some(key => req.headers[key])) {
          res.statusCode = 403;
          res.end('Esta interfaz solo está disponible en tu equipo.');
          return;
        }
        res.setHeader('Cache-Control', 'no-store');
        res.setHeader('X-Content-Type-Options', 'nosniff');
        res.setHeader('X-Frame-Options', 'DENY');
        res.setHeader('Referrer-Policy', 'no-referrer');
        next();
      });
    },
  }],
  server: {
    host: '127.0.0.1',
    strictPort: true,
    allowedHosts: ['localhost'],
    cors: false,
    fs: { strict: true, allow: [root] },
    proxy: process.env.ALEXIA_API_URL ? {
      '/api': { target: process.env.ALEXIA_API_URL, changeOrigin: true, xfwd: false },
    } : {},
  },
});
