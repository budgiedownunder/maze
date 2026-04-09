# maze_web_server — React Single Page Application (SPA)

`React 18` + `TypeScript` + `Vite` web frontend for the Rust [`maze_web_server`](../../rust/maze_web_server/README.md).

## Prerequisites

- Node.js 24+

## Setup

```bash
npm install
```

Then download the Playwright browser binary:

```bash
npx playwright install chromium
```

## Development

Start the Vite dev server:

```bash
npm run dev
```

Mock Service Worker (MSW) intercepts all API calls automatically in dev mode, so the Rust server does not need to be running. If you want to test against the real Rust server instead, start it first (`cargo run` in `src/rust/maze_web_server/`) — the Vite dev server proxies `/api` to `https://localhost:8443`.

## Production build

```bash
npm run build
```

Output goes to `dist/`. Point the Rust server's `static_dir` at this folder:

```toml
# src/rust/maze_web_server/config.toml
static_dir = "../../react/maze_web_server/dist"
```

## Unit tests

Uses Vitest + React Testing Library + Mock Service Worker (MSW).

```bash
npm test
```

## E2E tests

Uses Playwright (Chromium). The Vite dev server starts automatically before the tests run — the Rust server does not need to be running.

```bash
npx playwright test
```

Other useful options:

```bash
npx playwright test --ui          # interactive UI mode
npx playwright test --headed      # watch the browser as tests run
npx playwright test auth.spec.ts  # run a single file
```
