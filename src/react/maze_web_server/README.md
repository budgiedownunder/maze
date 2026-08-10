# maze_web_server — React Single Page Application (SPA)

`React 19` + `TypeScript` + `Vite` web frontend for the Rust [`maze_web_server`](../../rust/maze_web_server/README.md).

## Overview

Browser-based UI for the `maze_web_server` REST API. Features:

- **User accounts** — sign up/in/out, edit profile, set a profile avatar, change password, manage email addresses, delete account, plus OAuth sign-in (Google, GitHub, Facebook) rendered when the [`maze_web_server`](../../rust/maze_web_server/README.md) has those providers configured. When signed in, the user's avatar shows in the page header (username as hover text) and links to the account page; avatars also appear on leaderboard rows
- **Maze list** — create, open, rename, duplicate, delete, and play mazes
- **Maze editor** — cell-by-cell editing (walls, start, finish, keys, doors, enemies, health
  pickups, treasure), multi-cell range selection, structural editing (insert/delete rows and columns),
  keyboard shortcuts, and a per-cell override panel for tuning an individual cell's
  characteristics (an enemy's type/damage/move interval, a health pickup's style/heal amount,
  a treasure's style/value, a key holder or door style, or a wall's type — "Default" (inherit the maze's wall
  default), a forced solid texture, or a water / lava / iron-fence skin). Variant types such
  as ghost enemies, potion pickups, and water/lava/iron-fence walls show their own sprite in
  the grid, and overridden cells are marked with a corner badge. A settings toolbar button
  opens a per-maze game-settings editor (sky, wall / enemy / health styles, timer, …) saved
  with the maze; a cell with no per-cell override inherits the maze's wall / enemy / health
  default as its 2D base sprite
- **Maze game** — play a maze at `/play/:id` with keyboard (arrow keys / WASD) or
  on-screen D-pad; visited cells are marked; completion shows a result popup. Wall / enemy /
  health cells render the maze's default sprites (e.g. lava walls, ghost enemies, potion
  pickups) unless a per-cell override says otherwise. Treasure cells render per style and are
  auto-collected on walk-over, tallied into the bag as grouped per-type `[icon] × N` chips
- **3D maze game** — a "Play 3D" button on the maze list and maze editor pages opens a
  "Run / Custom Run" chooser — "Run" launches with the maze's saved settings, "Custom
  Run" with one-off tweaks — then navigates the browser to `/game/?id={mazeId}` on the Rust server, which serves the
  [`Bevy`](https://bevyengine.org/) WebAssembly module ([`maze_game_bevy_wasm`](../../rust/maze_game_bevy_wasm/README.md))
  that runs the first-person 3D game entirely in-browser. On touch devices
  the game accepts both a five-button D-pad (turn / move / tilt) and
  single-finger canvas gestures: swipe left / right to turn, swipe up / down
  to tilt, press-and-hold to move forward. Spacebar (desktop) or the on-screen
  pause button (bottom-right corner) toggles a "PAUSED" overlay that freezes
  the timer and movement.
- **Leaderboards** — a "Leaderboards" page (with a Home tile and nav entry) showing
  per-maze and per-3D-game boards over completed 3D runs, with fastest-time /
  highest-score tabs, your own runs highlighted, your personal score history, and a green
  Play / Play Again button that launches the selected maze or game in 3D. A Daily game's
  board adds a UTC date picker (with quick-pick chips for days that already have runs) to
  browse each day's frozen board. Mazes with no scores yet still appear and are launchable
- **Today's Challenge** — a Home tile that jumps straight into the current day's daily 3D
  game
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

### 3D diagnostics readout

To have every 3D launch turn on the in-game diagnostics readout (memory, visible
and total mesh counts, frame rate — see the
[`maze_game_bevy` README](../../rust/maze_game_bevy/README.md#diagnostic-overlay)):

```bash
npm run dev:debug          # Vite dev server
npm run build:debug        # a dist/ for the Rust server to serve — use this to test on a phone
```

Both set `VITE_DEBUG_MEM=true`, which makes the launch helpers append `mem=1` to
the `/game/` URL. **Development only — never build this way for deployment.** An
ordinary `npm run build` compiles the flag out entirely; the host page still
honours a `mem=1` typed into the address bar, so a normal build can be switched
on for one launch without rebuilding.

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

The suite runs differently in CI: [`playwright.config.ts`](playwright.config.ts) enables retries and a single worker when `CI` is set (GitHub Actions sets it automatically). The game/walk tests are timing-sensitive and can flake under parallel-worker CPU contention, so CI trades parallelism for stability. To reproduce CI's settings locally:

```bash
# bash / macOS / Linux
CI=true npx playwright test

# PowerShell
$env:CI='true'; npx playwright test
```
