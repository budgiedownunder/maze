import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

const isMswMode = process.env.VITE_MSW === 'true'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    // In MSW mode the app is fully self-contained; disable the proxy so requests
    // that leak past the Service Worker fail at the browser rather than hitting
    // an offline Rust server.
    proxy: isMswMode ? undefined : {
      '/api': { target: 'https://localhost:8443', changeOrigin: true, secure: false },
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
    exclude: ['**/node_modules/**', '**/tests/e2e/**'],
  },
  build: { outDir: 'dist' },
})
