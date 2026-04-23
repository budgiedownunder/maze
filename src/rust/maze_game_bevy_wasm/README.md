# `maze_game_bevy_wasm` Crate

## Introduction

The `maze_game_bevy_wasm` crate is a thin `cdylib` wrapper around [`maze_game_bevy`](../maze_game_bevy/README.md) that targets the browser via WebAssembly. It owns all browser-specific concerns: the canvas selector, the `webgl2` Bevy feature, and the `wasm-bindgen` entry point.

## Getting Started

### Build

```
cd src/rust/maze_game_bevy_wasm
wasm-pack build --target web --no-typescript --out-dir ../../react/maze_web_server/public/game
```

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