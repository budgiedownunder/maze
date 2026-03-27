# `maze_c` Crate

## Introduction

`maze_c` is a native Rust `staticlib` that exposes maze logic via a C interface. It is currently used for iOS physical devices, where the Wasmer WebAssembly runtime cannot be used.

## C API

All exported functions are prefixed with `maze_c_` and types/structs with `MazeC`. The API mirrors the `maze_wasm` Web Assembly library in terms of overall API features and structure.

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

For iOS, the resulting `libmaze_c.a` should then be copied to `src/csharp/Maze.Wasm.Interop/runtimes/ios-arm64/native/` (device) or `ios-sim-arm64/native/` (simulator) 

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
