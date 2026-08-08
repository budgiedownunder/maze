# `maze_game_bevy_wasm` Crate

## Introduction

The `maze_game_bevy_wasm` crate is a thin `cdylib` wrapper around [`maze_game_bevy`](../maze_game_bevy/README.md) that targets the browser via WebAssembly. It owns all browser-specific concerns: the canvas selector, the `webgl2` Bevy feature, and the `wasm-bindgen` entry point.

The `start_with_config` entry point accepts the JSON the host page (`public/game/index.html`) builds from the launch subject (a stored maze's saved 3D settings, or a game definition's stored `config`). Its nested `levels` object — `count`, `finishType`, `difficultyChange`, `resetBag`, `alignment`, `taper`, the per-level scene overrides, and the two visibility ranges described below — drives a multi-level run: when `count > 1` the wrapper generates the whole stack up front (via `maze_game_bevy::generate_level_maze_jsons`) and hands it to the game as a `PendingLevels` resource. A `count` of 1 (the default, and the saved-maze `mazeJson` path) is an ordinary single-level game.

Both entry points install a panic hook first, so a Rust panic — during generation, world spawn, or gameplay — reaches the host page as a `maze-game-panic` `CustomEvent` carrying `{ message, location }` instead of a bare `RuntimeError: unreachable`. Only the first panic is reported.

`stop()` asks a running game to shut down and release its world, taking effect on the next frame. Until it existed the browser only reclaimed a game when the document itself was destroyed, which is asynchronous and invisible to the app. `live_bytes()` returns the bytes currently allocated and not yet freed — read it either side of a `stop()` to see whether the world was actually released. It is a callable rather than an in-game readout on purpose: `stop()` drops the app, taking the readout with it. The host page exposes both as `window.__mazeStop` / `window.__mazeLiveBytes` when launched with `?mem=1`.

`mobileMode` (`/game/?mobile_mode=1`) runs with the settings a phone needs, as one switch rather than a handful of parameters: only the player's own floor drawn and animated, interim finishes as portals, and no point light on the keys, treasures or finish orb (their meshes are emissive, so they still glow). The switches that measured null on a device — render scale, MSAA, freezing the pool animation — are deliberately left alone. The MAUI app appends the parameter on iOS and Android, where it knows its platform for a fact; in a browser the host page judges for itself when the parameter is absent, and an explicit `?mobile_mode=0` or `=1` always wins over that judgement.

The individual switches below stay available as diagnostics. They can only *add* restrictions on top of the mode, never lift one: a `bool` cannot distinguish "false by default" from "false on purpose", so a game that wants everything except one of them should not use the mode.

`levels.visibleBelow` / `levels.visibleAbove` (`/game/?floors=<below>,<above>`) bound how much of a multi-level stack is drawn **and** animated, counted from the player's own floor; `0` and `0` is that floor alone. Absent on either side (the default) leaves every level live. Every level is spawned up front and stays spawned, so without this a ten-level run pays for ten floors on every frame. The saving measured on a device came from not *drawing* the other floors; skipping their animation was measured separately and made no difference.

`levels.lightsBelow` / `levels.lightsAbove` bound the **point lights** alone, and **default to the player's own floor** — this is how the game renders, not a switch to turn on. A shadowless point light measured about 7 ms a frame on an iPhone, and a maze spawns one per key and per treasure with no budget, so a tall stack lit throughout carries dozens: a ten-level lava stack ran at 10-25 fps on a desktop with every floor lit, and 45-50 with only the player's floor lit. The scene is untouched — distant floors keep their shape and their content, and their objects keep glowing, because the meshes are emissive. What goes is the light those objects cast on a floor the player is not standing on. Single-level games are unaffected, since the range is around the floor the player is on and there is only one. The range never reaches wider than the scene's, because a floor nobody draws is never lit either. `/game/?lights=all` lights the whole stack again — the field defaults to narrow, so the host page sends explicit `null`s rather than omitting it — and `/game/?lights=<below>,<above>` picks any range between.

`allowLadders` (`/game/?ladders=0` to turn it off) decides whether an interim level may finish with a ladder; it defaults to on. Turned off, the finish type resolves to a portal — and because the rig drawn, the transition animation, the hatch cut into the floor above and the hole in a roofed finish tile all read that one value, they follow without anything else being set. A ladder only makes sense when the floor above it is drawn, so this pairs with a level window that hides it.

`disableObjectGlow` (`/game/?glow=0`) drops the point light every key holder and every treasure carries — the same cost the lights range narrows, removed outright rather than by distance.

`disableOrbLight` (`/game/?light=0`) keeps the finish orb but gives it no light at all: it is emissive so it still glows, but it stops lighting the walls and floor around it, and the win screen loses the glow left behind. `disableOrbShadows` (`/game/?shadows=0`) is the milder version — the light stays but stops casting, which is six scene passes fewer at the price of the light passing through walls. `hideFinishOrb` (`/game/?orb=0`) goes further and leaves the orb unspawned entirely; winning is grid-based so the finish still works, but the player has no marker to aim at, which makes it a measurement configuration rather than a playable one. The orb is the game's **only** shadow-casting light, and a point light's shadow is a cube map — six extra scene passes — on the final level alone, which is what makes reaching the top of a stack dearer than any other floor.

`freezeWallAnimation` (`/game/?wall_animation=0`) stops the water / lava pool wave rewriting a transform on every surface and rock each frame. The pools go still and their ripple texture keeps scrolling — but the lava rocks disappear with it, since a rock only rises out of the surface on its bob, so this is a frame-cost probe rather than a way to judge the look.

`debugMemory` (`/game/?mem=1`) turns on the in-game developer diagnostics readout (memory, entity counts, frame rate). It defaults to off; the MAUI app appends the parameter in Debug builds.

`renderScale` and `msaaSamples` override the render target, for measuring how much of a frame goes on per-pixel work. `renderScale` is the window's scale factor — physical pixels drawn per logical pixel — and `msaaSamples` a multisample count, where the browser accepts only `1` (off) or `4`; any other value leaves Bevy's default in place. Both default to absent, leaving the platform's pixel ratio and Bevy's own default alone. The host page sets them from `/game/?res=<fraction>&msaa=<samples>`, where `res` is a fraction of the device's pixel ratio (`0.5` renders at half linear resolution, a quarter of the pixels) — the page resolves it into the absolute value this field takes, since the browser is what knows the ratio.

A change in `renderScale` is visible in the picture, so it confirms itself. A change in `msaaSamples` is not, and the diagnostics readout does not report the sample count actually in force — so confirm by some other means before drawing a conclusion from an MSAA comparison.

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