# `maze_console` Crate

## Introduction

The `maze_console` crate is written in `Rust` and represents a simple console application for managing and solving mazes. 

Internally, the application logic is defined by the `ConsoleApp` type (`console_app.rs`) which implements the public `App` trait defined in `app.rs`.

Mocked tests are implemented using `./tests/mock_app.rs`. 

## Getting Started

### Build
To build the `maze_console` crate, run the following from within the `maze_console` directory:
```
cargo build
```

### Testing
To test the `maze_console` crate, run the following from within the `maze_console` directory:
```
cargo test -- --test-threads=1
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze_console` directory:
```
cargo doc --open
```