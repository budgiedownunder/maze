# `maze` Crate

## Introduction

The `maze` crate is written in `Rust` and defines an API for calculating maze solutions and running interactive game sessions. It exposes the following `struct` and `trait` types that a developer can use to define, solve, and play mazes:

- `Error` - represents a maze error
- `GenerationAlgorithm` - enum representing a maze generation algorithm
- `Generator` - represents a maze generator
- `GeneratorOptions` - defines maze generation options
- `MazePath` - represents a path composed of a sequence of maze points
- `MazePathDirection` - represents a direction within a maze path
- `MazePointOffset` - represents an offset between maze points
- `MazePrinter` - trait implementation for printing mazes and their solutions
- `MazeSolver` - trait implementation for solving mazes
- `MazeSolution` - represents a maze solution
- `Solver` - represents a maze solver
- `Direction` - enum representing a player movement direction (`None`, `Up`, `Down`, `Left`, `Right`)
- `MoveResult` - enum representing the outcome of a move attempt (`None`, `Moved`, `Blocked`, `Complete`)
- `MazeGame` - a running game session tracking player position, direction, visited cells, and completion

For solving a maze you would typically:
1. Create a `maze` instance with `Maze::new()` defined in the `data_model` crate
2. Modify the `maze` definition using functions such as `maze.from_json()`, `maze.definition.insert_rows()` etc or, alternatively, generate one from scratch using `Maze::generate()`
3. Solve for a `solution` using `Maze::solve()`.
4. Access the `solution.path`  to determine the path through the maze

## Game Module

The `game` module (`maze::game`) provides an interactive cell-based game session driven by player input.

### Types

| Type | Description |
|:-----|:------------|
| `MazeGame` | A running game session. Create with `MazeGame::from_json(json)`. |
| `Direction` | `None` \| `Up` \| `Down` \| `Left` \| `Right` |
| `MoveResult` | `None` \| `Moved` \| `Blocked` \| `Complete` |

### Usage

```rust
use maze::{MazeGame, Direction, MoveResult};

let json = r#"{"grid":[["S"," ","F"]]}"#;
let mut game = MazeGame::from_json(json).unwrap();

// Initial state
assert_eq!(game.player_row(), 0);
assert_eq!(game.player_col(), 0);
assert_eq!(game.player_direction(), Direction::None);
assert!(!game.is_complete());

// Move right — empty cell
assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);

// Move right again — reach finish
assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
assert!(game.is_complete());

// Visited cells (in order)
assert_eq!(game.visited_cells(), &[(0, 0), (0, 1), (0, 2)]);
```

### Cell collision rules

| Cell | Result |
|:-----|:-------|
| `' '` (empty) | `Moved` |
| `'S'` (start) | `Moved` |
| `'F'` (finish) | `Complete` |
| `'W'` (wall) | `Blocked` |
| Out of bounds | `Blocked` |

## Getting Started

### Build
To build the `maze` crate, run the following from within the `maze` directory:
```
cargo build
```

### Testing
To test the `maze` crate (including the game module), run the following from the `src/rust` directory:
```
cargo test --locked -p maze
cargo test --locked -p maze --features generation
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