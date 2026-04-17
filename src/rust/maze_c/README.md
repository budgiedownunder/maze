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
| -1 | Unknown direction value |

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
