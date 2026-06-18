# `maze_c` Crate

## Introduction

`maze_c` is a native Rust `staticlib` that exposes maze logic via a C interface. It is currently used for iOS physical devices, where the Wasmer WebAssembly runtime cannot be used.

## C API

All exported functions are prefixed with `maze_c_` and types/structs with `MazeC`. The API mirrors the `maze_wasm` Web Assembly library in terms of overall API features and structure.

## MazeGame

Game session functions use the `maze_c_maze_game_*` prefix. A session is created from a JSON maze definition and freed when done.

```c
// Lifecycle
*MazeGameC maze_c_new_maze_game(const char* json);   // returns null on error
void        maze_c_free_maze_game(MazeGameC* ptr);   // null-safe

// Movement — returns MoveResult encoding (see below)
int32_t maze_c_maze_game_move_player(MazeGameC* ptr, int32_t dir);

// State getters (valid pointer assumed)
int32_t maze_c_maze_game_player_row(MazeGameC* ptr);
int32_t maze_c_maze_game_player_col(MazeGameC* ptr);
int32_t maze_c_maze_game_player_direction(MazeGameC* ptr);
int32_t maze_c_maze_game_is_complete(MazeGameC* ptr);   // 0 or 1
int32_t maze_c_maze_game_is_lost(MazeGameC* ptr);       // 0 or 1
int32_t maze_c_maze_game_lose_reason(MazeGameC* ptr);   // see LoseReason encoding

// Bag / pickup (valid pointer assumed; out parameters may be null)
uint8_t maze_c_maze_game_pickup(MazeGameC* ptr, uint32_t* kind_out, uint32_t* id_out);
                                                        // 1 = picked up, 0 = nothing there
int32_t maze_c_maze_game_bag_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_bag_item(MazeGameC* ptr, int32_t index,
                                      uint32_t* kind_out, uint32_t* id_out);
                                                        // 1 = success, 0 = out-of-range

// Doors / tick / events (valid pointer assumed; out parameters may be null)
int32_t maze_c_maze_game_door_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_door(MazeGameC* ptr, int32_t index,
                                  uint32_t* row_out, uint32_t* col_out, uint32_t* state_out);
                                                        // 1 = success, 0 = out-of-range
int32_t maze_c_maze_game_tick(MazeGameC* ptr, float dt_ms);
                                                        // returns event count; buffers events on the session
int32_t maze_c_maze_game_tick_event_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_tick_event(MazeGameC* ptr, int32_t index,
                                        uint32_t* kind_out, uint32_t* row_out, uint32_t* col_out);
                                                        // 1 = success, 0 = out-of-range
uint8_t maze_c_maze_game_get_tick_event_payload(MazeGameC* ptr, int32_t index,
                                                uint32_t* payload_out);
                                                        // enemy id / hp_after / reason code; 0 for DoorOpened
uint8_t maze_c_maze_game_get_tick_event_string_payload(MazeGameC* ptr, int32_t index,
                                                       uint8_t* buf_out, uint32_t* len_out);
                                                        // PlayerNotHealed message; two-call protocol
                                                        // (buf_out=null reads len_out, then re-call to copy)

// HP / enemies / health pickups (valid pointer assumed; out parameters may be null)
uint32_t maze_c_maze_game_hp(MazeGameC* ptr);
uint32_t maze_c_maze_game_max_hp(MazeGameC* ptr);
int32_t maze_c_maze_game_enemy_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_enemy(MazeGameC* ptr, int32_t index,
                                   uint32_t* row_out, uint32_t* col_out, uint32_t* id_out,
                                   uint32_t* damage_out, float* move_period_ms_out,
                                   int32_t* enemy_type_out);
                                                        // 1 = success, 0 = out-of-range
                                                        // damage_out / move_period_ms_out: resolved per-enemy values
                                                        // enemy_type_out: -1 = no rig override, 0 = goblin, 1 = ghost
int32_t maze_c_maze_game_health_pickup_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_health_pickup(MazeGameC* ptr, int32_t index,
                                           uint32_t* row_out, uint32_t* col_out, uint32_t* id_out);
                                                        // id_out always 0; 1 = success, 0 = out-of-range
int32_t maze_c_maze_game_treasure_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_treasure(MazeGameC* ptr, int32_t index,
                                      uint32_t* row_out, uint32_t* col_out,
                                      int32_t* style_out, uint32_t* value_out);
                                                        // 1 = success, 0 = out-of-range
                                                        // style_out: 0 = silver, 1 = gold, 2 = diamonds, 3 = jewels
                                                        // value_out: resolved reward (override else the type's default)

// Keys (uncollected; valid pointer assumed; out parameters may be null)
int32_t maze_c_maze_game_key_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_key(MazeGameC* ptr, int32_t index,
                                 uint32_t* row_out, uint32_t* col_out, uint32_t* id_out);
                                                        // 1 = success, 0 = out-of-range

// Visited cells (valid pointer assumed)
int32_t maze_c_maze_game_visited_cell_count(MazeGameC* ptr);
uint8_t maze_c_maze_game_get_visited_cell(MazeGameC* ptr, int32_t index,
                                          int32_t* row_out, int32_t* col_out);
                                          // returns 1=success, 0=out-of-range
```

**Direction encoding** (`dir` parameter and `player_direction` return):

| Value | Direction |
|:-----:|:----------|
| 0 | None |
| 1 | Up |
| 2 | Down |
| 3 | Left |
| 4 | Right |

**MoveResult encoding** (return of `move_player`):

| Value | Result |
|:-----:|:-------|
| 0 | None |
| 1 | Moved |
| 2 | Blocked |
| 3 | Complete |
| 4 | BlockedByLockedDoor |
| 5 | StartedUnlocking |
| 6 | Stranded |
| 7 | Killed (HP reached zero from an enemy collision) |
| -1 | Unknown direction value |

**LoseReason encoding** (return of `lose_reason`):

| Value | Reason |
|:-----:|:-------|
| 0 | None (the game is not lost) |
| 1 | Stranded (the player can no longer hold enough keys to open every closed door remaining on a route to the finish) |
| 2 | Killed (the player's HP reached zero from enemy collisions) |

**BagItemKind encoding** (`kind_out` of `pickup` / `get_bag_item`):

| Value | Kind |
|:-----:|:-----|
| 0 | Key |

**DoorState encoding** (`state_out` of `get_door`):

| Value | State |
|:-----:|:------|
| 0 | Locked |
| 1 | Opening |
| 2 | Open (permanent, passable) |

**GameEvent kind encoding** (`kind_out` of `get_tick_event`):

| Value | Kind | `(row_out, col_out)` | Payload (`get_tick_event_payload`) |
|:-----:|:-----|:---------------------|:-----------------------------------|
| 0 | DoorOpened | the door cell that opened | `0` |
| 1 | EnemyMoved | the enemy's new cell | enemy id |
| 2 | PlayerDamaged | `(0, 0)` (unused) | HP after the hit |
| 3 | PlayerHealed | the consumed pickup cell | HP after the heal |
| 4 | PlayerNotHealed | the spared pickup cell | reason code (`0` = already at max HP); message via `get_tick_event_string_payload` |
| 5 | KeyCollected | the consumed key cell | collected key id |
| 6 | TreasureCollected | the consumed treasure cell | treasure score value |

**Memory ownership:** The caller must call `maze_c_free_maze_game` when done. Passing `null` to `free` is safe and has no effect.

**Error handling:** `maze_c_new_maze_game` returns `null` on failure (invalid JSON or no start cell); call `maze_c_get_last_error()` to retrieve the message. Getter functions assume a valid (non-null) pointer, matching the existing `maze_c` convention.

## Error Handling

Functions that can fail return `u8` (`1` = success, `0` = error). On failure the error message is stored in a thread-local and can be retrieved via `maze_c_get_last_error()`. The pointer is valid until the next `maze_c_*` call on the same thread.

## Building

To build the `maze_c` crate, run the following from within the `maze_c` directory:

```bash
# Local device
cargo build

# Cross-compile for iOS device
cargo build --release --target aarch64-apple-ios

# Cross-compile for iOS simulator
cargo build --release --target aarch64-apple-ios-sim
```

For iOS, the resulting `libmaze_c.a` should then be copied to `src/csharp/Maze.Interop/runtimes/ios-arm64/native/` (device) or `ios-sim-arm64/native/` (simulator) 

## Testing
To test the `maze_c` crate, run the following from within the `maze_c` directory:
```
cargo test
```

## Benchmarking
To run benchmark tests, run the following from within the `maze_c` directory:
```
cargo bench
```

## Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_c` directory:
```
cargo doc --open
```
