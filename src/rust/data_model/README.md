# `data_model` Crate

## Introduction

The `data_model` crate is written in `Rust` and defines the following data model objects:

- `Error` - represents a data model error
- `Maze` - represents a maze
- `MazeCellState` - represents an individual maze cell state
- `MazeDefinition` - represents a maze definition
- `MazePoint` - represents a point within a maze
- `User` - represents a user

## Getting Started

### Build
To build the `data_model` crate, run the following from within the `data_model` directory:
```
cargo build
```

### Testing
To test the `data_model` crate, run the following from within the `data_model` directory:
```
cargo test
```

### Benchmarking
No benchmarking tests are currently implemented for the crate

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `data_model` directory:
```
cargo doc --open
```