# `maze` Crate

## Introduction

The `maze` crate is written in `Rust` and defines an API for calculating maze solutions. It exposes the following `struct` and '`trait` types that a developer can use to define and solve mazes:

- `Error` - represents a maze error
- `MazePath` - represents a path composed of a sequence of maze points
- `MazePathDirection` - represents a direction within a maze path
- `MazePointOffset` - represents an offset between maze points
- `MazePrinter` - trait implementation for printing mazes and their solutions
- `MazeSolver` - trait implementation for solving mazes
- `MazeSolution` - represents a maze solution
- `Solver` - represents a maze solver

With these, you would typically:
1. Create a `maze` instance with `Maze::new()` defined in the `data_model` crate
2. Modify the `maze` definition using functions such as `maze.from_json()`, `maze.definition.insert_rows()` etc. 
3. Solve for a `solution` using `Maze::solve()`.
4. Access the `solution.path`  to determine the path through the maze 

## Getting Started

### Build
To build the `maze` crate, run the following from within the `maze` directory:
```
cargo build
```

### Testing
To test the `maze` crate, run the following from within the `maze` directory:
```
cargo test
```

### Benchmarking
To run benchmark tests:
```
cargo bench
```

### Generating Documentation
To generate and view `Rust` documentation for the crate in your default browser, run the following from within the `maze` directory:
```
cargo doc --open
```