# `maze` Crate

## Introduction

The `maze` crate is written in `Rust` and defines an API for defining mazes and calculating their solutions. It exposes the following `struct` types that a developer can use to define and solve mazes:

- `Definition` - represents a maze definition
- `Maze` - represents a maze
- `Offset` - represents an offset between maze points
- `Path` - represents a path composed of a sequence of maze points
- `Solution` - represents a maze solution
- `Solver` - represents a maze solver
- `StdOutLinePrinter` - represents a line printer for targetting `stdout`

With these, you would typically:
1. Create a `maze` instance with ` Maze::new()`
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