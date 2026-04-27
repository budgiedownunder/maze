# maze_web_server — React Single Page Application (SPA)

`React 19` + `TypeScript` + `Vite` web frontend for the Rust [`maze_web_server`](../../rust/maze_web_server/README.md).

## Overview

Browser-based UI for the `maze_web_server` REST API. Features:

- **User accounts** — sign up/in/out, edit profile, change password, delete account, plus OAuth sign-in (Google, GitHub, Facebook) rendered when the [`maze_web_server`](../../rust/maze_web_server/README.md) has those providers configured
- **Maze list** — create, open, rename, duplicate, delete, and play mazes
- **Maze editor** — cell-by-cell editing (walls, start, finish), multi-cell range selection,
  structural editing (insert/delete rows and columns), keyboard shortcuts
- **Maze game** — play a maze at `/play/:id` with keyboard (arrow keys / WASD) or
  on-screen D-pad; visited cells are marked; completion shows a result popup
- **3D maze game** — a "Play 3D" button on the maze list and maze editor pages
  navigates the browser to `/game/?id={mazeId}` on the Rust server, which serves the
  [`Bevy`](https://bevyengine.org/) WebAssembly module ([`maze_game_bevy_wasm`](../../rust/maze_game_bevy_wasm/README.md))
  that runs the first-person 3D game entirely in-browser
- **In-browser WASM** — maze generation, solving, and game logic run locally via the
  `maze_wasm` WebAssembly module with no server round-trip

## OAuth integration

The SPA never sees client secrets. The Rust [`maze_web_server`](../../rust/maze_web_server/README.md) runs the full OAuth flow and, on success, redirects to `/oauth/callback#token=...&expires_at=...`. The `OAuthCallbackPage` component reads the fragment, hands the token to `AuthContext.setAuthFromTokenResponse`, then clears the fragment from the URL via `history.replaceState` so the token is not retained in browser history, referer headers, or shared URLs. The list of provider buttons rendered on `/login` and `/signup` is server-driven via `GET /api/v1/features` — no client rebuild needed when a provider is added or toggled server-side.

## Tech stack

| Layer | Technology |
|-------|-----------|
| Framework | React 19 + TypeScript + Vite |
| Routing | React Router v7 |
| Styling | Plain CSS with CSS custom properties (light/dark mode) |
| Unit tests | Vitest + React Testing Library + Mock Service Worker |
| E2E tests | Playwright (Chromium) |
| WASM | `maze_wasm` (local Rust crate, bundled via wasm-pack) |

## Prerequisites

- Node.js 24+

### WASM dependency

The app depends on the prebuilt `maze_wasm` package. Build it once before running `npm install`:

```bash
# from src/rust/maze_wasm/
wasm-pack build --target web -- --features "wasm-bindgen"
```

See [`maze_wasm/README.md`](../../rust/maze_wasm/README.md) for full build instructions.

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

In mock mode the login handler accepts any email address and password.

## Production build

```bash
npm run build
```

Output goes to `dist/`. Point the Rust server's `static_dir` at this folder:

```toml
# src/rust/maze_web_server/config.toml
static_dir = "../../react/maze_web_server/dist"
```

## Linting

```bash
npm run lint
```

## Unit tests

Uses Vitest + React Testing Library + Mock Service Worker (MSW). Test files are in `tests/unit/`.

```bash
npm test
```

## E2E tests

Uses Playwright (Chromium). The Vite dev server starts automatically before the tests run — the Rust server does not need to be running. Test files are in `tests/e2e/`.

```bash
npx playwright test
```

Other useful options:

```bash
npx playwright test --ui          # interactive UI mode
npx playwright test --headed      # watch the browser as tests run
npx playwright test auth.spec.ts  # run a single file
```
