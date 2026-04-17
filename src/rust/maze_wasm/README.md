# `maze_wasm` Crate

## Introduction

The `maze_wasm` crate is written in `Rust` and defines the Web Assembly library for defining, generating, solving and playing mazes in consumer applications that support Web Assembly (WASM).

The crate uses `wasm-pack` to generate a JavaScript API wrapper `maze_wasm.js` to the WASM, and uses `cargo` to build the general Web Assembly `maze_wasm.wasm` for use outside of JavaScript.

## Getting Started

### Setup
To setup the build and test environment, run the following from the `maze_wasm` directory:

```
cargo install wasm-pack
cd tests/js
npm install
```

### Build
To build the `maze_wasm` crate and related resources, you need to run commands from within the `maze_wasm` directory.

To build the JavaScript API wrapper in the `./pkg` sub-directory, run:

```
wasm-pack build --target web -- --features "wasm-bindgen"
```

To build the general Web Assembly `maze_wasm.wasm` (for use outside of JavaScript), run:

```
cargo build --target wasm32-unknown-unknown --release --no-default-features --features "wasm-lite"
```

This will generate the release package for `maze_wasm.wasm` in the following directory:

`./src/rust/target/wasm32-unknown-unknown/release`

### Testing
To test the `maze_wasm` crate and the JavaScript API wrapper, run the following from within the `maze_wasm` directory:
```
cargo test
cd tests/js
npm run test_api
npm run test_help_examples
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_wasm` directory depending on what type of build you require the documentation for.

To generate documentation for the JavaScript API (corresponding to the `wasm-bindgen` feature):
```
cargo doc --features "wasm-bindgen" --open
```

To generate documentation for the generalised Web Assembly API:
```
cargo doc --open
```

## MazeGameWasm API

The `MazeGameWasm` type exposes an interactive game session: place a player at the start cell and move them through the maze one step at a time.

### wasm-bindgen (JavaScript)

```js
import init, { DirectionWasm, MazeGameWasm, MoveResultWasm } from 'maze_wasm.js';
await init();

// JSON format: { "grid": [["S", " ", "F"], ...] }
// Cells: "S" = start, "F" = finish, "W" = wall, " " = empty
const game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');

game.player_row();       // → number (0-based row index)
game.player_col();       // → number (0-based column index)
game.player_direction(); // → DirectionWasm (None=0, Up=1, Down=2, Left=3, Right=4)
game.is_complete();      // → boolean

// Returns MoveResultWasm (None=0, Moved=1, Blocked=2, Complete=3)
const result = game.move_player(DirectionWasm.Right);

// Array of { row: number, col: number } objects in visit order
// Includes the start cell; only appended on successful moves
const cells = game.visited_cells();
```

### wasm-lite (C FFI)

For non-JS WASM hosts (Wasmtime, .NET, native via P/Invoke).

```c
// Direction encoding: 0=None, 1=Up, 2=Down, 3=Left, 4=Right
// MoveResult encoding: 0=None, 1=Moved, 2=Blocked, 3=Complete, -1=null pointer

MazeGameWasm* new_maze_game_wasm(const u8* json_string_ptr);  // returns null on error
void          free_maze_game_wasm(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_move_player(MazeGameWasm* maze_game_wasm, i32 dir);
i32           maze_game_wasm_player_row(MazeGameWasm* maze_game_wasm);       // -1 on null
i32           maze_game_wasm_player_col(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_player_direction(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_is_complete(MazeGameWasm* maze_game_wasm);      // 1=true, 0=false, -1=null
i32           maze_game_wasm_visited_cell_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_visited_cell(MazeGameWasm* maze_game_wasm, i32 index,
                                              i32* row_out, i32* col_out);   // 0=ok, -1=error
```

The `json_string_ptr` argument must point to a length-prefixed string (4-byte little-endian length followed by UTF-8 bytes), allocated via `allocate_sized_memory`.

## WebAssembly Target Compatibility

This crate supports both **JavaScript/WebAssembly** builds and **general-purpose WebAssembly** builds for use in non-JS environments such as Wasmtime, .NET, or other native hosts, with the `wasm-lite` feature flag used to disable randomness and `Utc::now()` in those environments that do not support them.
