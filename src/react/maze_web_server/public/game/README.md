# `maze_game_bevy_wasm` Crate

## Introduction

The `maze_game_bevy_wasm` crate is a thin `cdylib` wrapper around [`maze_game_bevy`](../maze_game_bevy/README.md) that targets the browser via WebAssembly. It owns all browser-specific concerns: the canvas selector, the `webgl2` Bevy feature, and the `wasm-bindgen` entry point.

The `start_with_config` entry point accepts the JSON the host page (`public/game/index.html`) builds from the server's `play3d-config` response. Its nested `levels` object — `count`, `finishType`, `difficultyChange`, `resetBag`, `alignment` — drives a multi-level run: when `count > 1` the wrapper generates the whole stack up front (via `maze_game_bevy::generate_level_maze_jsons`) and hands it to the game as a `PendingLevels` resource. A `count` of 1 (the default, and the saved-maze `mazeJson` path) is an ordinary single-level game.

## Getting Started

### Setup
To setup the build and test environment, run the following from the `maze_game_bevy_wasm` directory:

```
cargo install wasm-pack
cargo install wasm-opt
```

### Build

```
cd src/rust/maze_game_bevy_wasm
wasm-pack build --target web --no-typescript --out-dir ../../react/maze_web_server/public/game
```

### Size-optimised build

For a smaller WASM (≈19 MiB instead of ≈50 MiB), build under a size-tuned cargo profile and then post-process with `wasm-opt`. The `--config` flags applied to the cargo build live entirely on the command line — they do **not** affect any other `cargo build --release` in the workspace, so the server and other crates remain fast to compile.

**Prerequisite:** `wasm-opt` on your `PATH` — installed via the [workspace setup steps](../README.md#setup). (We don't let `wasm-pack` run its bundled `wasm-opt` automatically because that binary crashes on the default-release WASM with a binaryen Precompute internal error; running `wasm-opt` ourselves after the size-tuned build sidesteps the bug)

Run the following from the `maze_game_bevy_wasm` directory:

**Bash:**

```bash
wasm-pack build --target web --no-typescript --out-dir ../../react/maze_web_server/public/game -- \
  --config 'profile.release.lto="fat"' \
  --config 'profile.release.codegen-units=1' \
  --config 'profile.release.package.maze_game_bevy.opt-level="z"' \
  --config 'profile.release.package.maze_game_bevy.strip="symbols"' \
  --config 'profile.release.package.maze_game_bevy_wasm.opt-level="z"' \
  --config 'profile.release.package.maze_game_bevy_wasm.strip="symbols"'

wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
  ../../react/maze_web_server/public/game/maze_game_bevy_wasm_bg.wasm \
  -o ../../react/maze_web_server/public/game/maze_game_bevy_wasm_bg.wasm
```

**PowerShell:**

Requires **PowerShell 7.3 or later** — relies on `$PSNativeCommandArgumentPassing = 'Windows'`, which became the default on Windows in PowerShell 7.3. On Windows PowerShell 5.1 or PowerShell 7.2 (with the legacy default), the embedded double quotes inside each `--config` value are not forwarded to `cargo` intact and the TOML parse will fail. Check with `$PSVersionTable.PSVersion`.

```powershell
wasm-pack build --target web --no-typescript --out-dir ../../react/maze_web_server/public/game -- `
  --config 'profile.release.lto="fat"' `
  --config 'profile.release.codegen-units=1' `
  --config 'profile.release.package.maze_game_bevy.opt-level="z"' `
  --config 'profile.release.package.maze_game_bevy.strip="symbols"' `
  --config 'profile.release.package.maze_game_bevy_wasm.opt-level="z"' `
  --config 'profile.release.package.maze_game_bevy_wasm.strip="symbols"'

wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int `
  ../../react/maze_web_server/public/game/maze_game_bevy_wasm_bg.wasm `
  -o ../../react/maze_web_server/public/game/maze_game_bevy_wasm_bg.wasm
```

Trade-off: this size-optimised pipeline takes ≈5× longer (≈7 minutes) than the default build (≈1–2 minutes), but the smaller binary greatly reduces download time and improves game start-up experience.

### Testing

```
cd src/rust
cargo test --locked -p maze_game_bevy_wasm
```

### Serving

1. Build the React app so `public/game/` lands in `dist/game/`:
   ```
   cd src/react/maze_web_server
   npm run build
   ```
2. Start `maze_web_server`.
3. Navigate to `https://localhost:8443/game/` in a browser.