# `maze_game_bevy` Crate

## Introduction

The `maze_game_bevy` crate is written in `Rust` and provides the [Bevy](https://bevyengine.org/) game engine integration for the maze game. It compiles as both a library and a native desktop binary:

- The **library** owns all Bevy systems and app setup
- The **binary** runs the game as a native desktop application

On launch the app displays a layered gold title screen ("MAZE GAME") for 1.5 seconds, then transitions to a first-person 3D maze world rendered via Bevy PBR. Wall panels are spawned on the exposed faces of passable cells, ensuring boundary walls are always visible even when the maze data has no explicit outer wall row.

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
cd src/rust/maze_game_bevy
cargo run
```

### Testing

To test the `maze_game_bevy` crate:

```
cd src/rust
cargo test --locked -p maze_game_bevy
```

