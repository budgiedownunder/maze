# `maze_game_bevy` Crate

<img src="./screenshots/finish_corridor.png" width="800">

## Introduction

The `maze_game_bevy` crate is written in `Rust` and provides the [Bevy](https://bevyengine.org/) game engine integration for the maze game. It compiles as both a library and a native desktop binary:

- The **library** owns all Bevy systems and app setup
- The **binary** runs the game as a native desktop application

### App flow

1. **Title screen** — layered gold "MAZE GAME" text displayed for 3 seconds, then auto-transitions to the playing state.
2. **3D maze world** — first-person PBR renderer. Wall panels are spawned on the exposed faces of passable cells, ensuring boundary walls are always visible even when the maze data has no explicit outer wall row. N/S-facing panels are a lighter stone grey; E/W-facing panels are darker, providing a directional shading cue at junctions.
3. **Finish orb** — an animated gold sphere hovers and bobs above the finish cell, illuminated by a shadow-casting point light that confines the glow to cells with line-of-sight to the orb.

The player starts at the start cell facing the first open neighbour cycling through S → E → N → W, so the initial view is always into an open corridor rather than a wall.

### Controls

| Input | Action |
|-------|--------|
| `←` / `A` | Turn left |
| `→` / `D` | Turn right |
| `↑` / `W` | Move forward |
| `Q` | Tilt camera up (clamped at +45°) |
| `E` | Tilt camera down (clamped at -90°, looking at the floor) |
| `Space` | Pause / resume (freezes the timer and movement; "PAUSED" overlay shows) |
| `Escape` | Quit |

Pitch is updated continuously at a fixed angular rate while `Q` or `E` is held. Turning and movement are gated by an animation lock, but pitch input is allowed to update during these animations (though not after winning).



### Visual features

- **Procedural brick-pattern** texture on walls; stone-tile texture on floors — generated at runtime, no asset files required.
- **Per-cell wall tint variation** — pick one of six emissive variants for for wall panels, so different corridor sections have different shades.
- **Dead-end landmark objects** — every dead-end cell (passable cell with exactly one open neighbour, excluding start / finish) gets a distinctive landmark — a brazier, urn, broken pillar, or chest — picked by hashing `(row, col, seed)`. Each difficulty can toggle this via `[game.play3d.<difficulty>.landmarks] dead_end_objects`.
- **Sparse wall decorations** — ~1 in 8 wall panels gets a decorative emissive decoration (vent grate, faded poster, rune glyph, or window glow) projected on its inside face. Placement and kind are seeded from `(row, col, face, seed)` so the same maze always decorates the same walls. Each difficulty can toggle this via `[game.play3d.<difficulty>.landmarks] wall_decorations`.
- Floor grid lines at cell boundaries for orientation feedback.
- Start cell highlighted green; finish cell highlighted gold.
- Status bar overlay (top-left corner) — a row container that displays the configured `mode` label.
- **Minimap overlay** (top-right corner) — fixed viewport centred on the player; only explored cells and their immediate neighbours are revealed. The whole panel re-anchors to the window's top-right corner on resize.
- **Win overlay** — on reaching the finish cell, movement stops and a "You Win!" panel appears centred on screen.
- Gold-leaf rain — on win, small gold leaf sprites spawn continuously across the full screen width and fall with gentle rotation and drift, celebrating completion.
- **Lose overlay** — on not completing in time, movement stops and a "You Lose!" panel appears centred on screen.
- Rain-Lightning — on lose, rain sprites spawn continuously across the full screen width and fall accompanied by periodic lightning flashes.

## Getting Started

### Build

To build the native binary:

```
cd src/rust
cargo build -p maze_game_bevy
```

### Run

To run the native desktop application:

```
cd src/rust
cargo run -p maze_game_bevy
```

### Testing

To test the `maze_game_bevy` crate:

```
cd src/rust
cargo test --locked -p maze_game_bevy
```

## Source layout

The crate is organised into focused per-concern modules under `src/`:

```
src/
├── lib.rs                  module decls + public re-exports + build_app
├── palette.rs              cross-module colour constants
├── state.rs                shared state / config types (GameConfig, GameState, etc.)
├── images.rs               generic Bevy Image factory (sampler-tuned)
├── movement.rs             input + animation + win-detection + quit
├── world/                  3D scene construction
│   ├── mod.rs              spawn_world orchestrator + grid helpers
│   ├── textures/           shared procedural world textures
│   │   ├── mod.rs          module declarations
│   │   ├── brick.rs        make_brick_texture (consumed by walls)
│   │   └── tile.rs         make_tile_texture (consumed by floor tile/start/finish)
│   ├── floor/              floor cells, grid lines, start, finish
│   │   ├── mod.rs          FloorCell marker + FloorAssets bundle + spawn_floor_for_cell
│   │   ├── tile.rs         default-tile material + spawn helper
│   │   ├── lines.rs        FloorLine, line meshes + material + spawn_lines_for_cell
│   │   ├── start.rs        StartCell + start material + spawn helper
│   │   └── finish.rs       FinishCell + finish material + spawn helper
│   ├── walls/              wall panels
│   │   ├── mod.rs          per-cell tint hash + WallAssets bundle + spawn_walls_for_cell
│   │   ├── ns_panel.rs     N/S-facing panel mesh, materials, spawn helper
│   │   └── ew_panel.rs     E/W-facing panel mesh, materials, spawn helper
│   ├── decorations/        sparse wall decorations
│   │   ├── mod.rs          shared mesh/dims + placement hash + spawn_decorations_for_cell
│   │   ├── vent.rs         vent-grate texture + material
│   │   ├── poster.rs       faded-poster texture + material
│   │   ├── rune.rs         rune-glyph texture + material
│   │   └── window.rs       window-glow texture + material
│   ├── objects/            3D physical objects placed in the world
│   │   ├── mod.rs          ObjectAssets bundle + spawn_objects_for_cell
│   │   ├── finish/         objects placed at the finish cell
│   │   │   ├── mod.rs      FinishAssets bundle + spawn_finish_for_cell ('F' predicate)
│   │   │   └── orb.rs      FinishOrb + orb mesh/material/light + orb_system
│   │   └── dead_end/       dead-end landmark objects (placement seeded)
│   │       ├── mod.rs      DeadEndObject + DeadEndAssets + dispatcher + hash + is_dead_end
│   │       ├── brazier.rs  brazier: stone column + glow + spawn helper
│   │       ├── urn.rs      urn material + spawn helper
│   │       ├── pillar.rs   broken-pillar material + spawn helper
│   │       └── chest.rs    chest material + spawn helper
│   └── sky/                sky / atmosphere modes
│       ├── mod.rs          spawn_sky dispatcher (today: night)
│       └── night/          dim corridor-lit aesthetic
│           └── mod.rs      ambient + directional lights
├── hud/                    top-screen overlays
│   ├── mod.rs              module declarations
│   ├── minimap.rs          top-right minimap overlay
│   ├── statusbar.rs        top-left mode label
│   └── clock.rs            top-centre countdown clock + lose-state trigger
└── overlays/               full-screen modal layers
    ├── mod.rs              module declarations
    ├── title.rs            title-screen splash
    ├── win.rs              win panel + gold-leaf rain
    ├── lose.rs             lose panel + rain + lightning
    └── pause.rs            paused overlay
```

`spawn_world` is a thin orchestrator: it resolves the maze source into
`GameState` + `GameClock`, spawns the camera and the sky, builds per-domain
asset bundles (`walls`, `floor`, `decorations`, `objects`), runs a per-cell
loop calling each domain's `spawn_*_for_cell`, and finishes with HUD +
paused-overlay spawns. The only items re-exported through `lib.rs` are
`build_app`, `generate_maze_json`, and the public types `GameConfig`,
`Landmarks`, `GameOutcome`, `GameResult`. Everything else is `pub(crate)` or
fully private.
