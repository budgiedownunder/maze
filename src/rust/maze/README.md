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
- `MoveResult` - enum representing the outcome of a move attempt (`None`, `Moved`, `Blocked`, `Complete`, `BlockedByLockedDoor`, `StartedUnlocking`, `Stranded`, `Killed`)
- `LoseReason` - enum representing why a game ended in a loss (`Stranded`, `Killed`; extensible)
- `DoorState` - enum representing a door's lifecycle (`Locked`, `Opening`, `Open`)
- `BagItem` - enum representing an item carried in the player's bag (currently `Key`)
- `GameEvent` - enum representing a time-based event emitted by `MazeGame::tick` (`DoorOpened`, `EnemyMoved`, `PlayerDamaged`, `PlayerHealed`, `PlayerNotHealed`, `KeyCollected`, `TreasureCollected`)
- `Enemy` - represents an `'E'` cell at runtime: position, planned next-cell target, per-tick move period, damage per same-cell collision, and an optional per-cell visual rig (`enemy_type`)
- `EnemyType` - per-cell visual rig for an enemy (`goblin` / `ghost`), re-exported from `data_model`; carried on `Enemy` for renderers
- `PlayerNotHealedReason` - enum carried on `GameEvent::PlayerNotHealed` (currently `AlreadyAtMaxHp`)
- `MazeGame` - a running game session tracking player position, direction, visited cells, completion, lose state, the player's bag, per-cell door state, HP / max-HP, and the live enemies and health pickups

For solving a maze you would typically:
1. Create a `maze` instance with `Maze::new()` defined in the `data_model` crate
2. Modify the `maze` definition using functions such as `maze.from_json()`, `maze.definition.insert_rows()` etc or, alternatively, generate one from scratch using `Maze::generate()`
3. Solve for a `solution` using `Maze::solve()`.
4. Access the `solution.path`  to determine the path through the maze

`solve()` is **key-aware**. A maze containing doors (`'D'`) is solved over the state of which keys have been collected and which doors opened, returning the **shortest** route that actually completes it given the key→door gating — collecting the keys it needs and treating a door as passable once a key is held (doors are not minimised; the route may pass through several). It returns an error when the finish can't be reached (e.g. sealed behind a door with no reachable key) and also when the combined `'K'` + `'D'` count exceeds `MAX_TOTAL_FEATURES` (16) — refusing rather than degrading to a key-blind walk that would misrepresent a sealed maze as playable. Because a route may backtrack to fetch a key, `solution.path` can revisit a cell (it is a walk, not a strictly simple path). Mazes with no doors solve as the plain shortest path.

The solver is **enemy-blind**: enemy (`'E'`) and health-pickup (`'H'`) cells both map to `MazeCellState::Empty` and are walked over as plain passages. A returned path therefore proves the maze is *navigable*, not that it is *survivable* — the solver does not model enemy movement or damage, so keeping the player alive past the enemies is the maze author's responsibility.

The crate exports `pub const MAX_TOTAL_FEATURES: usize = 16` as the canonical cap on `'K'` + `'D'` cells in any maze. Every layer that produces or persists a maze — the generator, the WASM bindings, the React Generate dialog, the React editor save flow, and the server save endpoint — enforces the same cap up front so the solver's error path never fires for a maze produced through the supported tools.

`Generator` can **auto-place keys and doors** via `GeneratorOptions.door_count` (default `0` = a lock-free maze). It places that many doors (clamped to what the maze can hold and a small ceiling) on the start→finish solution path: each is anchored to a **junction** (a decision point with a side branch), chosen from the finish back toward the start, and positioned a random few cells *ahead* of it so the junction's branch stays in the segment before the door. Each door's key is then hidden at the **deepest dead-end** of a branch in that preceding segment — typically the anchoring junction's own branch. The result is verified with the key-aware `solve()`, so a generated maze with doors is always completable.

A request whose `2 * door_count + spare_doors + spare_keys` exceeds `MAX_TOTAL_FEATURES` is refused with `Error::Generate` — each real door contributes one key *and* one door to the produced grid, so the budget counts doors twice.

Two further knobs scatter **decoys** and a **spare-key budget** onto off-spine branches *after* the solvability check:

- `spare_doors` — extra `'D'` cells placed on off-spine corridor (or dead-end) cells, visually indistinguishable from the real path doors. Opening one consumes a key the player might have needed for a real door, potentially stranding them (see `MazeGame::lose_reason`). Clamped to `MAX_AUTO_DOORS` and to feasibility; candidates adjacent to an existing key are skipped so the bait isn't telegraphed.
- `spare_keys` — extra `'K'` cells placed on off-spine branches, giving the player a budget to burn on decoys before they risk stranding. Clamped to feasibility; candidates adjacent to any door are skipped so a spare key doesn't accidentally identify a nearby door as real.

Spare placement preserves solvability by construction — decoys never sit on the spine and spare keys only loosen the player's key budget — so no second `solve()` is needed.

`Generator` also **auto-places enemies and health pickups** via `GeneratorOptions.enemy_count` / `GeneratorOptions.health_count` (default `0` = none). Each pass picks uniformly at random from cells that are currently `' '` and at Manhattan distance > 1 from `S` (so the player has at least one safe step from start before the first encounter), so placement never lands on `S` / `F` / `K` / `D` or on a previously-placed `'E'` / `'H'`. The counts are clamped silently to `MAX_ENEMY_COUNT` / `MAX_HEALTH_COUNT` (both `8`) and then to the eligible-cell count. Enemies and health pickups are solver-blind (they map to `MazeCellState::Empty` in `solve()`), so placement cannot make the maze unsolvable — no re-validation is needed.

`GeneratorOptions.treasure_count` (default `0` = none, clamped to `MAX_TREASURE_COUNT` = `12`) auto-places **treasure** (`'T'`) in a final pass, **dead-end-first**: dead-end cells (passable, exactly one open neighbour — `maze::is_dead_end`, the shared topology predicate the 3D renderer also uses to place its dead-end decorations) are claimed before other walkable `' '` cells, so treasure favours the ends of branches. Each placed treasure is assigned a type by weight (≈40% Silver / 30% Gold / 20% Jewels / 10% Diamonds — cheaper types more frequent); non-Silver cells carry a `TreasureOverride { style }` on the definition while Silver stays a bare `'T'`. The type sets the default reward value (Silver 50 / Gold 100 / Jewels 200 / Diamonds 400, per-cell overridable via `value`). Treasure is also solver-blind, so it never affects solvability.

## Game Module

The `game` module (`maze::game`) provides an interactive cell-based game session driven by player input.

### Types

| Type | Description |
|:-----|:------------|
| `MazeGame` | A running game session. Create with `MazeGame::from_json(json)`. |
| `Direction` | `None` \| `Up` \| `Down` \| `Left` \| `Right` |
| `MoveResult` | `None` \| `Moved` \| `Blocked` \| `Complete` \| `BlockedByLockedDoor` \| `StartedUnlocking` \| `Stranded` \| `Killed` |
| `LoseReason` | `Stranded` \| `Killed` |
| `DoorState` | `Locked` \| `Opening { progress }` \| `Open` |
| `BagItem` | `Key { id }` (serialises as `{"type":"key","id":…}`) |
| `GameEvent` | `DoorOpened { cell }` \| `EnemyMoved { id, row, col }` \| `PlayerDamaged { hp_after }` \| `PlayerHealed { hp_after, cell }` \| `PlayerNotHealed { cell, reason, message }` \| `KeyCollected { cell, id }` \| `TreasureCollected { cell, style, value }` |
| `Enemy` | `{ id, row, col, target_row, target_col, move_period_ms, accum_ms, damage, enemy_type }` — one per `'E'` cell at construction |
| `PlayerNotHealedReason` | `AlreadyAtMaxHp` |

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
| `'K'` (key) | `Moved` — the key is auto-collected on walk-over: the cell clears to `' '`, the key enters the bag, and a `GameEvent::KeyCollected { cell, id }` fires |
| `'D'` (door, open) | `Moved` — or `Stranded` if walking through leaves the player without enough still-collectible keys for the closed doors still on any route to the finish |
| `'D'` (door, locked, key held) | `StartedUnlocking` (key consumed; opens over time via `tick`) |
| `'D'` (door, locked, no key / still opening) | `BlockedByLockedDoor` |
| `'E'` (enemy) | `Moved` — or `Killed` if a same-cell enemy collision drops HP to 0. Damage from every enemy on the destination cell sums into a single `GameEvent::PlayerDamaged { hp_after }`. |
| `'H'` (health pickup) | `Moved`. Auto-pickup: when `hp < max_hp` the cell clears to `' '` and a `GameEvent::PlayerHealed { hp_after, cell }` fires, restoring the cell's per-cell `heal_amount` override (else the built-in `1`), clamped to `max_hp`; when `hp == max_hp` the cell stays `'H'` and a `GameEvent::PlayerNotHealed { cell, reason, message }` fires so the host can flash "already at full health" feedback. |
| `'T'` (treasure) | `Moved` — auto-collected on walk-over: the cell clears to `' '`, its value (the per-cell `value` override, else the type's default — Silver 50 / Gold 100 / Jewels 200 / Diamonds 400) is added to the score, and a `GameEvent::TreasureCollected { cell, style, value }` fires |
| `'W'` (wall) | `Blocked` |
| Out of bounds | `Blocked` |

Keys are auto-collected by walking over them — stepping onto a `'K'` cell adds the key to the bag, clears the cell, and queues a `GameEvent::KeyCollected { cell, id }` that flushes on the next `tick`. (`MazeGame::pickup()` remains as the internal collect mechanism; an external call normally finds nothing left since the cell was already cleared on walk-over.) Doors open over real time rather than blocking permanently: holding against a locked door while carrying a key starts it opening, and `MazeGame::tick(dt_ms)` advances and completes the open (emitting `GameEvent::DoorOpened`). Collected items are read via `MazeGame::bag()`, doors via `MazeGame::doors()`, and uncollected keys via `MazeGame::keys()`.

The game also tracks a lose state: `MazeGame::is_lost()` and `MazeGame::lose_reason()` report whether the session has ended in a loss and why. At each door walk-through the runtime compares the minimum closed-door count on any path to the finish (a lock-blind 0-1 BFS from the player's current cell) against the maximum number of keys the player could ultimately hold (`bag.len()` + a state-space BFS over `(cell, collected, opened)` from the current state, falling back to a lock-blind key reachability count above 16 combined `'K'` + `'D'` cells). When the closed-doors count exceeds the available keys, `LoseReason::Stranded` is set and `move_player` returns `MoveResult::Stranded` at the moment of detection. Host-driven losses such as a wall-clock timeout live entirely in the host (the 3D game owns its own countdown).

Enemies and health are driven through the same `tick(dt_ms)` loop that advances doors. Each `'E'` cell yields one `Enemy` at construction, with a move period (default 1500 ms, per-game override via `MazeGameOptions::enemy_move_period_ms`) and a per-collision damage (default 1, per-game override via `MazeGameOptions::enemy_damage`).

A maze may also carry **per-cell entity overrides** (`MazeDefinition::cell_entities`). The engine applies the numeric ones at construction / pickup time, resolving **per-cell → per-game → built-in**: an `'E'` cell's `damage` / `move_period_ms` seed its `Enemy` (and its `enemy_type` rig is carried on the `Enemy` for renderers), and an `'H'` cell's `heal_amount` sets how much that pickup restores, and a `'T'` cell's `value` (or, if unset, the default for its `style`) sets the score it awards. The remaining overrides are **visual and ride on static cells** (`health_style` on `'H'`, `key_holder` on `'K'`, `door_style` on `'D'`), so this crate does not consume them — renderers read them straight from the `MazeDefinition` by cell position. Only `enemy_type` is surfaced through the engine (on the live `Enemy`), because an enemy moves away from its spawn cell. Every move period the enemy commits a one-cell step toward the player along a wall-aware **BFS shortest path**, with a deterministic `N > E > S > W` tie-break; an enemy fully walled off from the player rests in place. Same-cell collisions deal damage from either side — the player walking onto an enemy's cell, or an enemy stepping onto the player's cell — and `LoseReason::Killed` fires when `hp` reaches `0`. The HP cap (`max_hp`, default `3`) and the starting `hp` (default `= max_hp`) are also `MazeGameOptions` knobs. `MazeGame::hp()` / `MazeGame::max_hp()` expose the current HP state; `MazeGame::enemies()` exposes the live enemy collection; `MazeGame::treasures()` the uncollected treasure cells, each with its resolved type and reward value; `MazeGame::score()` exposes the run's current score — the number of keys collected this run **plus** the total value of treasure collected (a monotonic progress measure, so opening a door does not lower it), kept internal to the engine so callers read the getter rather than recompute it. Health pickups remain in-grid as `'H'` cells until consumed (auto-pickup clears them to `' '` on a `PlayerHealed` event), so the live uncollected set is read by scanning the grid for `'H'`.

For host loops that prefer `setTimeout` over frame-rate polling, `MazeGame::time_until_next_event_ms()` reports the number of milliseconds until the next `tick` will fire an event — the soonest enemy commit boundary or door-open completion, or `0` when events queued by a prior `move_player` are waiting to flush, or `None` when no enemies exist, no door is opening, and no events are pending. Sleep until the returned time, call `tick(elapsed)`, repeat.

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