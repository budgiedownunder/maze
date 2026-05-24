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
- `MoveResult` - enum representing the outcome of a move attempt (`None`, `Moved`, `Blocked`, `Complete`, `BlockedByLockedDoor`, `StartedUnlocking`, `Stranded`)
- `LoseReason` - enum representing why a game ended in a loss (currently `Stranded`; extensible)
- `DoorState` - enum representing a door's lifecycle (`Locked`, `Opening`, `Open`)
- `BagItem` - enum representing an item carried in the player's bag (currently `Key`)
- `GameEvent` - enum representing a time-based event emitted by `MazeGame::tick` (currently `DoorOpened`)
- `MazeGame` - a running game session tracking player position, direction, visited cells, completion, lose state, the player's bag, and per-cell door state

For solving a maze you would typically:
1. Create a `maze` instance with `Maze::new()` defined in the `data_model` crate
2. Modify the `maze` definition using functions such as `maze.from_json()`, `maze.definition.insert_rows()` etc or, alternatively, generate one from scratch using `Maze::generate()`
3. Solve for a `solution` using `Maze::solve()`.
4. Access the `solution.path`  to determine the path through the maze

`solve()` is **key-aware**. A maze containing doors (`'D'`) is solved over the state of which keys have been collected and which doors opened, returning the **shortest** route that actually completes it given the key→door gating — collecting the keys it needs and treating a door as passable once a key is held (doors are not minimised; the route may pass through several). It returns an error when the finish can't be reached, e.g. sealed behind a door with no reachable key. Because a route may backtrack to fetch a key, `solution.path` can revisit a cell (it is a walk, not a strictly simple path). Mazes with no doors solve as the plain shortest path.

`Generator` can **auto-place keys and doors** via `GeneratorOptions.door_count` (default `0` = a lock-free maze). It places that many doors (clamped to what the maze can hold and a small ceiling) on the start→finish solution path: each is anchored to a **junction** (a decision point with a side branch), chosen from the finish back toward the start, and positioned a random few cells *ahead* of it so the junction's branch stays in the segment before the door. Each door's key is then hidden at the **deepest dead-end** of a branch in that preceding segment — typically the anchoring junction's own branch. The result is verified with the key-aware `solve()`, so a generated maze with doors is always completable.

Two further knobs scatter **decoys** and a **spare-key budget** onto off-spine branches *after* the solvability check:

- `spare_doors` — extra `'D'` cells placed on off-spine corridor (or dead-end) cells, visually indistinguishable from the real path doors. Opening one consumes a key the player might have needed for a real door, potentially stranding them (see `MazeGame::lose_reason`). Clamped to `MAX_AUTO_DOORS` and to feasibility; candidates adjacent to an existing key are skipped so the bait isn't telegraphed.
- `spare_keys` — extra `'K'` cells placed on off-spine branches, giving the player a budget to burn on decoys before they risk stranding. Clamped to feasibility; candidates adjacent to any door are skipped so a spare key doesn't accidentally identify a nearby door as real.

Spare placement preserves solvability by construction — decoys never sit on the spine and spare keys only loosen the player's key budget — so no second `solve()` is needed.

## Game Module

The `game` module (`maze::game`) provides an interactive cell-based game session driven by player input.

### Types

| Type | Description |
|:-----|:------------|
| `MazeGame` | A running game session. Create with `MazeGame::from_json(json)`. |
| `Direction` | `None` \| `Up` \| `Down` \| `Left` \| `Right` |
| `MoveResult` | `None` \| `Moved` \| `Blocked` \| `Complete` \| `BlockedByLockedDoor` \| `StartedUnlocking` \| `Stranded` |
| `LoseReason` | `Stranded` |
| `DoorState` | `Locked` \| `Opening { progress }` \| `Open` |
| `BagItem` | `Key { id }` (serialises as `{"type":"key","id":…}`) |
| `GameEvent` | `DoorOpened { cell }` |

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
| `'K'` (key) | `Moved` (key is not collected by moving — pick it up explicitly) |
| `'D'` (door, open) | `Moved` — or `Stranded` if walking through leaves the player without enough still-collectible keys for the closed doors still on any route to the finish |
| `'D'` (door, locked, key held) | `StartedUnlocking` (key consumed; opens over time via `tick`) |
| `'D'` (door, locked, no key / still opening) | `BlockedByLockedDoor` |
| `'W'` (wall) | `Blocked` |
| Out of bounds | `Blocked` |

Keys are not collected by walking over them — call `MazeGame::pickup()` while standing on a key cell to add it to the bag. Doors open over real time rather than blocking permanently: holding against a locked door while carrying a key starts it opening, and `MazeGame::tick(dt_ms)` advances and completes the open (emitting `GameEvent::DoorOpened`). Collected items are read via `MazeGame::bag()`, doors via `MazeGame::doors()`, and uncollected keys via `MazeGame::keys()`.

The game also tracks a lose state: `MazeGame::is_lost()` and `MazeGame::lose_reason()` report whether the session has ended in a loss and why. At each door walk-through the runtime compares the minimum closed-door count on any path to the finish (a lock-blind 0-1 BFS from the player's current cell) against the maximum number of keys the player could ultimately hold (`bag.len()` + a state-space BFS over `(cell, collected, opened)` from the current state, falling back to a lock-blind key reachability count above 16 combined `'K'` + `'D'` cells). When the closed-doors count exceeds the available keys, `LoseReason::Stranded` is set and `move_player` returns `MoveResult::Stranded` at the moment of detection. Host-driven losses such as a wall-clock timeout live entirely in the host (the 3D game owns its own countdown).

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