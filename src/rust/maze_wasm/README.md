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
game.is_complete();      // → boolean (player reached the finish)
game.is_lost();          // → boolean (player stranded, or HP drained to zero)
game.lose_reason();      // → 'stranded' | 'killed' | null
game.hp();               // → number (current player HP)
game.max_hp();           // → number (maximum player HP; constant for the session)

// Returns MoveResultWasm (None=0, Moved=1, Blocked=2, Complete=3,
//                         BlockedByLockedDoor=4, StartedUnlocking=5,
//                         Stranded=6, Killed=7)
const result = game.move_player(DirectionWasm.Right);

// Array of { row: number, col: number } objects in visit order
// Includes the start cell; only appended on successful moves
const cells = game.visited_cells();

// Keys are not collected by moving — call pickup() while standing on a key cell.
const item    = game.pickup();          // → { type: 'key', id } or null
const keys    = game.keys();            // → [{ row, col, id }]  (uncollected keys)
const doors   = game.doors();           // → [{ row, col, state: 'locked' | 'opening' | 'open' }]
const bag     = game.bag();             // → [{ type: 'key', id }]  (collected items)
const enemies = game.enemies();         // → [{ row, col, id }]  (live enemies, stable enemy-id order)
const pickups = game.health_pickups(); // → [{ row, col, id }]  (uncollected 'H' cells, row-major order)

// Advance time-based state (opening doors, enemy AI, queued damage / heal events).
// Returns the events that occurred during the tick (or queued by prior move_player calls):
//   { type: 'doorOpened',     row, col }
//   { type: 'enemyMoved',     id, row, col }
//   { type: 'playerDamaged',  hpAfter }
//   { type: 'playerHealed',   hpAfter, row, col }
//   { type: 'playerNotHealed', row, col, reason, message }
const events = game.tick(16);

// Time in ms until the next tick will produce an event — for setTimeout-driven
// host loops that sleep instead of polling at frame rate. Returns 0 when a
// move_player call has queued events waiting to flush, the soonest pending
// enemy commit or door-open completion otherwise, or null when the game is
// idle (no enemy planning a step, no door opening, no pending events).
const waitMs = game.time_until_next_event_ms();  // → number | null
```

### wasm-lite (C FFI)

For non-JS WASM hosts (Wasmtime, .NET, native via P/Invoke).

```c
// Direction encoding: 0=None, 1=Up, 2=Down, 3=Left, 4=Right
// MoveResult encoding: 0=None, 1=Moved, 2=Blocked, 3=Complete,
//                      4=BlockedByLockedDoor, 5=StartedUnlocking, 6=Stranded,
//                      7=Killed, -1=null pointer

MazeGameWasm* new_maze_game_wasm(const u8* json_string_ptr);  // returns null on error
void          free_maze_game_wasm(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_move_player(MazeGameWasm* maze_game_wasm, i32 dir);
i32           maze_game_wasm_player_row(MazeGameWasm* maze_game_wasm);       // -1 on null
i32           maze_game_wasm_player_col(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_player_direction(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_is_complete(MazeGameWasm* maze_game_wasm);      // 1=true, 0=false, -1=null
i32           maze_game_wasm_is_lost(MazeGameWasm* maze_game_wasm);           // 1=true, 0=false, -1=null
i32           maze_game_wasm_lose_reason(MazeGameWasm* maze_game_wasm);       // 0=None, 1=Stranded, 2=Killed, -1=null
i32           maze_game_wasm_pickup(MazeGameWasm* maze_game_wasm,
                                    u32* kind_out, u32* id_out);              // 0=ok, -1=no item / null
i32           maze_game_wasm_bag_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_bag_item(MazeGameWasm* maze_game_wasm, i32 index,
                                          u32* kind_out, u32* id_out);        // 0=ok, -1=error
i32           maze_game_wasm_door_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_door(MazeGameWasm* maze_game_wasm, i32 index,
                                      u32* row_out, u32* col_out, u32* state_out);
                                                                              // state: 0=Locked, 1=Opening, 2=Open
i32           maze_game_wasm_tick(MazeGameWasm* maze_game_wasm, f32 dt_ms);   // returns event count; buffers on session
i32           maze_game_wasm_tick_event_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_tick_event(MazeGameWasm* maze_game_wasm, i32 index,
                                            u32* kind_out, u32* row_out, u32* col_out);
                                                                              // kind: 0=DoorOpened, 1=EnemyMoved,
                                                                              // 2=PlayerDamaged, 3=PlayerHealed, 4=PlayerNotHealed
i32           maze_game_wasm_get_tick_event_payload(MazeGameWasm* maze_game_wasm, i32 index,
                                                    u32* payload_out);        // enemy id / hp_after / reason; 0=ok, -1=error
i32           maze_game_wasm_get_tick_event_string_payload(MazeGameWasm* maze_game_wasm, i32 index,
                                                           u8* buf_out, u32* len_out);
                                                                              // PlayerNotHealed message; two-call protocol
                                                                              // (buf_out=null reads len_out, then re-call); 0=ok, -1=error
i32           maze_game_wasm_key_count(MazeGameWasm* maze_game_wasm);         // uncollected only
i32           maze_game_wasm_get_key(MazeGameWasm* maze_game_wasm, i32 index,
                                     u32* row_out, u32* col_out, u32* id_out);
                                                                              // 0=ok, -1=error
i32           maze_game_wasm_visited_cell_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_visited_cell(MazeGameWasm* maze_game_wasm, i32 index,
                                              i32* row_out, i32* col_out);   // 0=ok, -1=error
i32           maze_game_wasm_hp(MazeGameWasm* maze_game_wasm);                // -1 on null
i32           maze_game_wasm_max_hp(MazeGameWasm* maze_game_wasm);            // -1 on null
i32           maze_game_wasm_enemy_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_enemy(MazeGameWasm* maze_game_wasm, i32 index,
                                       u32* row_out, u32* col_out, u32* id_out);
                                                                              // 0=ok, -1=error
i32           maze_game_wasm_health_pickup_count(MazeGameWasm* maze_game_wasm);
i32           maze_game_wasm_get_health_pickup(MazeGameWasm* maze_game_wasm, i32 index,
                                               u32* row_out, u32* col_out, u32* id_out);
                                                                              // 0=ok, -1=error; id is always 0
```

The `json_string_ptr` argument must point to a length-prefixed string (4-byte little-endian length followed by UTF-8 bytes), allocated via `allocate_sized_memory`.

## WebAssembly Target Compatibility

This crate supports both **JavaScript/WebAssembly** builds and **general-purpose WebAssembly** builds for use in non-JS environments such as Wasmtime, .NET, or other native hosts, with the `wasm-lite` feature flag used to disable randomness and `Utc::now()` in those environments that do not support them.
