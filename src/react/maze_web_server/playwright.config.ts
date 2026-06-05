import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  // Several game/walk tests are timing-sensitive (real tick-loop / animation timing).
  // Under parallel-worker CPU contention they can miss a window and flake. In CI run a
  // single worker with retries so a transient miss is retried (still reported as flaky)
  // rather than reddening the run; locally keep the default parallelism and no retries.
  // Gated on `process.env.CI`, which GitHub Actions sets automatically.
  workers: process.env.CI ? 1 : undefined,
  retries: process.env.CI ? 2 : 0,
  // Extra headroom for slow/loaded CI runners (defaults: expect 5s, test 30s).
  timeout: process.env.CI ? 60_000 : 30_000,
  expect: { timeout: process.env.CI ? 10_000 : 5_000 },
  use: {
    baseURL: 'http://localhost:5173',
    ignoreHTTPSErrors: true,
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    command: 'npm run dev',
    port: 5173,
    reuseExistingServer: !process.env.CI,
    env: { ...process.env, VITE_MSW: 'true' },
  },
})
