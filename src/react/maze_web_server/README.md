# maze_web_server — React Single Page Application (SPA)

`React 19` + `TypeScript` + `Vite` web frontend for the Rust [`maze_web_server`](../../rust/maze_web_server/README.md).

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

To run against the real Rust server, start it first (`cargo run` in `src/rust/maze_web_server/`) — the Vite dev server proxies `/api` to `https://localhost:8443`.

To run with Mock Service Worker (MSW) instead (no Rust server required):

```bash
npm run dev:mock
```

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
