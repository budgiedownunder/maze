use data_model::MazeDefinition;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Per-uncollected-`'K'`-cell, the set of door-sets achieving the minimum
/// number of non-spine doors on any path from that key back to the spine.
/// Almost always a singleton `Vec` entry for perfect mazes; multi-element
/// entries arise only on loopy authored mazes with tied min-cost paths.
type KeyMinPaths = HashMap<(usize, usize), Vec<HashSet<(usize, usize)>>>;

/// Direction of player movement.
///
/// [`Direction::None`] is the initial facing direction when a game is created —
/// it indicates the player has not yet moved. Passing [`Direction::None`] to
/// [`MazeGame::move_player`] always returns [`MoveResult::None`].
///
/// # Examples
///
/// ```
/// use maze::Direction;
/// let dir = Direction::Right;
/// assert_eq!(dir, Direction::Right);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// No direction — initial state before the player's first move.
    None,
    /// Move toward lower row indices.
    Up,
    /// Move toward higher row indices.
    Down,
    /// Move toward lower column indices.
    Left,
    /// Move toward higher column indices.
    Right,
}

/// Outcome of a move attempt.
///
/// # Examples
///
/// ```
/// use maze::{MazeGame, Direction, MoveResult};
/// let json = r#"{"grid":[["S","W"],["F"," "]]}"#;
/// let mut game = MazeGame::from_json(json).unwrap();
/// assert_eq!(game.move_player(Direction::None), MoveResult::None);
/// assert_eq!(game.move_player(Direction::Right), MoveResult::Blocked);
/// assert_eq!(game.move_player(Direction::Down), MoveResult::Complete);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveResult {
    /// No action was taken — returned when [`Direction::None`] is passed to
    /// [`MazeGame::move_player`].
    None,
    /// The player moved successfully to an empty or start cell.
    Moved,
    /// The move was blocked by a wall or grid boundary.
    Blocked,
    /// The player reached the finish cell — the game is complete.
    Complete,
    /// The move was blocked by a locked door (`'D'`) — either no key was held
    /// or the door is still opening. The player did not move.
    BlockedByLockedDoor,
    /// The player held against a locked door (`'D'`) while carrying a key: a key
    /// was consumed and the door began opening. The player did not move; the
    /// door becomes passable once [`MazeGame::tick`] reports it
    /// [`DoorState::Open`].
    StartedUnlocking,
    /// The player moved successfully through an open door (`'D'`) and now holds
    /// too few keys to open every remaining real door on the solution path —
    /// the game is unwinnable. [`MazeGame::lose_reason`] returns
    /// `Some(LoseReason::Stranded)` and [`MazeGame::is_lost`] returns `true`.
    Stranded,
}

/// Why a game ended in a loss.
///
/// Set when the game transitions to a lost state (see [`MazeGame::is_lost`]).
/// Currently the only loss the maze runtime tracks itself is
/// [`LoseReason::Stranded`]; host-driven losses such as a wall-clock timeout
/// are handled at the host layer (e.g. the 3D game's `tick_clock_system`
/// owns its own countdown) and don't propagate through this enum. The enum
/// stays extensible for future per-step lose causes (death events,
/// environmental hazards, etc.).
///
/// # Examples
///
/// ```
/// use maze::LoseReason;
/// let reason = LoseReason::Stranded;
/// assert_eq!(reason, LoseReason::Stranded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoseReason {
    /// The player no longer holds enough keys (collected + still in the world)
    /// to open every closed door remaining on a route from their current cell
    /// to the finish. Set when the player walks through an open door and the
    /// inequality `closed_doors_to_finish > available_keys` is true.
    Stranded,
}

/// The lifecycle state of a door (`'D'`) cell.
///
/// A door starts [`DoorState::Locked`]. Holding against it with a key (via
/// [`MazeGame::move_player`]) moves it to [`DoorState::Opening`]; once
/// [`MazeGame::tick`] advances its progress to completion it becomes
/// [`DoorState::Open`] — a permanent, passable state.
///
/// # Examples
///
/// ```
/// use maze::DoorState;
/// let phase = DoorState::Opening { progress: 0.0 };
/// assert_eq!(phase, DoorState::Opening { progress: 0.0 });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DoorState {
    /// Closed and locked; requires a key to open.
    Locked,
    /// Currently opening; `progress` runs `0.0..=1.0`.
    Opening {
        /// Fraction of the way open, `0.0..=1.0`.
        progress: f32,
    },
    /// Fully open and permanently passable.
    Open,
}

/// An item carried in the player's bag.
///
/// Modelled as a tagged enum so it serialises to self-describing JSON
/// (e.g. `{"type":"key","id":3}`) across the WASM/JS boundary and is extensible
/// with new item kinds. Keys are currently untyped — any key opens any door.
///
/// # Examples
///
/// ```
/// use maze::BagItem;
/// let item = BagItem::Key { id: 0 };
/// assert_eq!(item, BagItem::Key { id: 0 });
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BagItem {
    /// A key that can open one door.
    Key {
        /// Stable identifier derived from the key's origin cell.
        id: u32,
    },
}

/// A time-based event emitted by [`MazeGame::tick`].
///
/// # Examples
///
/// ```
/// use maze::GameEvent;
/// let event = GameEvent::DoorOpened { cell: (0, 2) };
/// assert_eq!(event, GameEvent::DoorOpened { cell: (0, 2) });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    /// A door finished opening at the given `(row, col)` cell.
    DoorOpened {
        /// The door cell that opened.
        cell: (usize, usize),
    },
}

/// A running maze game session.
///
/// Holds the grid, player position, facing direction, completion state, the set
/// of visited cells in visit order, the player's bag, per-cell door state, and
/// the lose state (set when the player runs out of time or strands themselves).
/// Create with [`MazeGame::from_json`].
///
/// Cell rules applied during [`MazeGame::move_player`]:
/// - `' '`, `'S'`, or `'K'` → [`MoveResult::Moved`] (a key is not collected by
///   moving onto it — use [`MazeGame::pickup`])
/// - `'F'` → [`MoveResult::Complete`]
/// - `'D'` (door) → [`MoveResult::Moved`] when already open (or
///   [`MoveResult::Stranded`] when walking through has left too few keys for
///   the remaining real path doors), else [`MoveResult::StartedUnlocking`]
///   (a key is held) or [`MoveResult::BlockedByLockedDoor`]
/// - `'W'` or out-of-bounds → [`MoveResult::Blocked`]
///
/// Doors open over time — see [`MazeGame::tick`]. The lose state is queried via
/// [`MazeGame::is_lost`] / [`MazeGame::lose_reason`].
///
/// # Examples
///
/// ```
/// use maze::{MazeGame, Direction, MoveResult};
/// let json = r#"{"grid":[["S"," ","F"]]}"#;
/// let mut game = MazeGame::from_json(json).unwrap();
/// assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
/// assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
/// assert!(game.is_complete());
/// ```
#[derive(Debug)]
pub struct MazeGame {
    grid: Vec<Vec<char>>,
    player_row: usize,
    player_col: usize,
    direction: Direction,
    complete: bool,
    visited: Vec<(usize, usize)>,
    rows: usize,
    cols: usize,
    /// Runtime door state per `'D'` cell, seeded `Locked` at construction.
    doors: HashMap<(usize, usize), DoorState>,
    /// Stable key id per `'K'` cell, assigned in row-major order at construction.
    key_ids: HashMap<(usize, usize), u32>,
    /// Items the player has collected, in pickup order.
    bag: Vec<BagItem>,
    /// Finish cell `(row, col)`. Cached at construction so the strand check
    /// doesn't grid-scan on every door walk-through. `None` only for the
    /// defensive case of a maze with no `'F'` cell (the maze pipeline
    /// rejects such mazes; we keep the runtime resilient).
    finish: Option<(usize, usize)>,
    /// Per-uncollected-`'K'`-cell, the set of door-sets achieving the minimum
    /// number of non-spine doors on any path from that key back to the spine.
    /// Tracks all tied min-cost paths so that opening a door on one path
    /// correctly drops the key's cost only when no shorter alternative exists.
    /// Almost always a singleton-path entry for perfect mazes (every generated
    /// maze, virtually every authored one); multi-element entries arise only
    /// when a loopy authored maze has two paths of equal door cost to the same
    /// key. Keys that the player can't reach lock-blind (walled-off pockets in
    /// a hand-authored grid) are simply absent from this map and never count
    /// toward the strand-check budget. Mutated as doors open (the opened door
    /// is removed from every path it appears in) and as keys are picked up
    /// (the key's entry is removed).
    key_min_paths: KeyMinPaths,
    /// Whether the game has ended in a loss (see [`Self::lose_reason`]).
    lost: bool,
    /// Why the game was lost. `None` until the game transitions to a lost
    /// state. Mutually exclusive with [`Self::complete`] in practice — the game
    /// is either won, lost, or in progress.
    lose_reason: Option<LoseReason>,
}

/// Real-time duration a door takes to open once unlocking begins, in milliseconds.
const DOOR_OPEN_MS: f32 = 1000.0;

impl MazeGame {
    /// Creates a game session from a `MazeDefinition` JSON string, placing the
    /// player at the start cell `S`. The initial facing direction is
    /// [`Direction::None`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the JSON is invalid or the maze has no start cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.player_row(), 0);
    /// assert_eq!(game.player_col(), 0);
    /// ```
    pub fn from_json(json: &str) -> Result<Self, String> {
        let definition: MazeDefinition =
            serde_json::from_str(json).map_err(|e| format!("invalid maze JSON: {e}"))?;

        let start = definition
            .get_start()
            .ok_or_else(|| "maze has no start cell".to_string())?;

        let rows = definition.grid.len();
        let cols = if rows > 0 { definition.grid[0].len() } else { 0 };

        let visited = vec![(start.row, start.col)];

        let mut doors = HashMap::new();
        let mut key_ids = HashMap::new();
        let mut next_key_id: u32 = 0;
        for (r, row) in definition.grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                match ch {
                    'D' => {
                        doors.insert((r, c), DoorState::Locked);
                    }
                    'K' => {
                        key_ids.insert((r, c), next_key_id);
                        next_key_id += 1;
                    }
                    _ => {}
                }
            }
        }

        let reachability =
            compute_strand_reachability(&definition.grid, (start.row, start.col));
        // Cache the finish cell once — the strand check needs it on every
        // door walk-through and we don't want to grid-scan each time.
        let finish = definition
            .grid
            .iter()
            .enumerate()
            .find_map(|(r, row)| row.iter().position(|&c| c == 'F').map(|c| (r, c)));

        Ok(MazeGame {
            grid: definition.grid,
            player_row: start.row,
            player_col: start.col,
            direction: Direction::None,
            complete: false,
            visited,
            rows,
            cols,
            doors,
            key_ids,
            bag: Vec::new(),
            finish,
            key_min_paths: reachability.key_min_paths,
            lost: false,
            lose_reason: None,
        })
    }

    /// Attempts to move the player one cell in `dir`.
    ///
    /// Returns [`MoveResult::Blocked`] if the target cell is a wall or out of
    /// bounds, [`MoveResult::Complete`] if the player reaches the finish cell,
    /// and [`MoveResult::Moved`] for an empty, start, key, or already-open door
    /// cell. Moving onto a key (`'K'`) does not collect it — use
    /// [`MazeGame::pickup`]. A locked door (`'D'`) yields
    /// [`MoveResult::StartedUnlocking`] when the player holds a key — consuming
    /// it and beginning the open (see [`MazeGame::tick`]) — or
    /// [`MoveResult::BlockedByLockedDoor`] otherwise. Stepping onto an open
    /// door cell while the player no longer holds enough keys to open every
    /// remaining real door on the solution path yields [`MoveResult::Stranded`]
    /// — the move still succeeds, but the game transitions to lost with
    /// [`LoseReason::Stranded`]. The player's facing direction is always
    /// updated to `dir`, even when blocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, MoveResult};
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
    /// assert_eq!(game.player_col(), 1);
    /// assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
    /// assert_eq!(game.player_col(), 2);
    /// ```
    pub fn move_player(&mut self, dir: Direction) -> MoveResult {
        self.direction = dir;

        let (new_row, new_col) = match dir {
            Direction::None => return MoveResult::None,
            Direction::Up => {
                if self.player_row == 0 {
                    return MoveResult::Blocked;
                }
                (self.player_row - 1, self.player_col)
            }
            Direction::Down => (self.player_row + 1, self.player_col),
            Direction::Left => {
                if self.player_col == 0 {
                    return MoveResult::Blocked;
                }
                (self.player_row, self.player_col - 1)
            }
            Direction::Right => (self.player_row, self.player_col + 1),
        };

        if new_row >= self.rows || new_col >= self.cols {
            return MoveResult::Blocked;
        }

        match self.grid[new_row][new_col] {
            'W' => MoveResult::Blocked,
            'F' => {
                self.player_row = new_row;
                self.player_col = new_col;
                self.visited.push((new_row, new_col));
                self.complete = true;
                MoveResult::Complete
            }
            'D' => match self.doors.get(&(new_row, new_col)).copied() {
                Some(DoorState::Open) => {
                    self.player_row = new_row;
                    self.player_col = new_col;
                    self.visited.push((new_row, new_col));
                    // Walked through an open door — the user-spec trigger
                    // point for stranded detection. Compare the minimum
                    // closed doors the player must open to reach F from
                    // their *current cell* (recomputed each time so it
                    // accounts for shortcuts opened off the lock-blind
                    // spine) against the keys they can still get hold of.
                    if !self.lost
                        && self.closed_doors_to_finish() > self.simulate_reachable_keys()
                    {
                        self.lost = true;
                        self.lose_reason = Some(LoseReason::Stranded);
                        MoveResult::Stranded
                    } else {
                        MoveResult::Moved
                    }
                }
                Some(DoorState::Locked) => {
                    if let Some(pos) = self
                        .bag
                        .iter()
                        .position(|item| matches!(item, BagItem::Key { .. }))
                    {
                        self.bag.remove(pos);
                        self.doors
                            .insert((new_row, new_col), DoorState::Opening { progress: 0.0 });
                        // Commit the key. For every uncollected key whose
                        // min-cost path(s) to the spine traverse this door,
                        // remove the door from those paths so the key's
                        // effective cost drops by one (or stays the same if
                        // a tied path didn't go through it). The
                        // "doors-to-finish" half of the strand inequality is
                        // recomputed on demand at walk-through time, so no
                        // counter needs decrementing here.
                        for paths in self.key_min_paths.values_mut() {
                            for path in paths.iter_mut() {
                                path.remove(&(new_row, new_col));
                            }
                        }
                        MoveResult::StartedUnlocking
                    } else {
                        MoveResult::BlockedByLockedDoor
                    }
                }
                Some(DoorState::Opening { .. }) => MoveResult::BlockedByLockedDoor,
                None => MoveResult::Blocked,
            },
            ' ' | 'S' | 'K' => {
                self.player_row = new_row;
                self.player_col = new_col;
                self.visited.push((new_row, new_col));
                MoveResult::Moved
            }
            _ => MoveResult::Blocked,
        }
    }

    /// Current player row (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.player_row(), 0);
    /// ```
    pub fn player_row(&self) -> usize {
        self.player_row
    }

    /// Current player column (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.player_col(), 0);
    /// ```
    pub fn player_col(&self) -> usize {
        self.player_col
    }

    /// Current player facing direction.
    ///
    /// The initial direction when a game is created is [`Direction::None`],
    /// indicating the player has not yet moved.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction};
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.player_direction(), Direction::None);
    /// ```
    pub fn player_direction(&self) -> Direction {
        self.direction
    }

    /// Whether the player has reached the finish cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction};
    /// let json = r#"{"grid":[["S","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// assert!(!game.is_complete());
    /// game.move_player(Direction::Right);
    /// assert!(game.is_complete());
    /// ```
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// All cells visited by the player (including the start cell), in visit order.
    ///
    /// Each entry is a `(row, col)` pair using 0-based indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction};
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.move_player(Direction::Right);
    /// assert_eq!(game.visited_cells(), &[(0, 0), (0, 1)]);
    /// ```
    pub fn visited_cells(&self) -> &[(usize, usize)] {
        &self.visited
    }

    /// Returns the maze grid as a 2-D slice of characters.
    ///
    /// Each character is one of `'S'` (start), `'F'` (finish), `'W'` (wall),
    /// `'K'` (key), `'D'` (door), or `' '` (open). A collected key's cell becomes
    /// `' '`; door cells keep their `'D'` character — their open/closed state is
    /// tracked separately (see [`MazeGame::doors`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.grid()[0][0], 'S');
    /// assert_eq!(game.grid()[0][2], 'F');
    /// ```
    pub fn grid(&self) -> &[Vec<char>] {
        &self.grid
    }

    /// Advances time-based game state by `dt_ms` milliseconds, returning the
    /// events that occurred (sorted by cell).
    ///
    /// Currently this drives door opening: each door in [`DoorState::Opening`]
    /// has its progress advanced, and a door that completes transitions to
    /// [`DoorState::Open`] (permanently passable) and emits
    /// [`GameEvent::DoorOpened`].
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, MoveResult, GameEvent};
    /// let json = r#"{"grid":[["S","K","D","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.move_player(Direction::Right); // step onto the key
    /// game.pickup();                      // collect it
    /// assert_eq!(game.move_player(Direction::Right), MoveResult::StartedUnlocking);
    /// assert_eq!(game.tick(1000.0), vec![GameEvent::DoorOpened { cell: (0, 2) }]);
    /// ```
    pub fn tick(&mut self, dt_ms: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for (cell, phase) in self.doors.iter_mut() {
            if let DoorState::Opening { progress } = phase {
                *progress += dt_ms / DOOR_OPEN_MS;
                if *progress >= 1.0 {
                    *phase = DoorState::Open;
                    events.push(GameEvent::DoorOpened { cell: *cell });
                }
            }
        }
        events.sort_by_key(|event| match event {
            GameEvent::DoorOpened { cell } => *cell,
        });
        events
    }

    /// Returns the door cells and their current [`DoorState`], sorted by
    /// `(row, col)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, DoorState};
    /// let json = r#"{"grid":[["S","D","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.doors(), vec![((0, 1), DoorState::Locked)]);
    /// ```
    pub fn doors(&self) -> Vec<((usize, usize), DoorState)> {
        let mut doors: Vec<((usize, usize), DoorState)> = self
            .doors
            .iter()
            .map(|(&cell, &phase)| (cell, phase))
            .collect();
        doors.sort_by_key(|&(cell, _)| cell);
        doors
    }

    /// Returns the cells still holding an uncollected key (`'K'`) and the key's
    /// stable id, sorted by `(row, col)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S","K","F"]]}"#;
    /// let game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.keys(), vec![((0, 1), 0)]);
    /// ```
    pub fn keys(&self) -> Vec<((usize, usize), u32)> {
        let mut keys: Vec<((usize, usize), u32)> = self
            .key_ids
            .iter()
            .filter_map(|(&(r, c), &id)| {
                if self.grid[r][c] == 'K' {
                    Some(((r, c), id))
                } else {
                    None
                }
            })
            .collect();
        keys.sort_by_key(|&(cell, _)| cell);
        keys
    }

    /// Returns the items currently in the player's bag, in pickup order.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, BagItem};
    /// let json = r#"{"grid":[["S","K","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.move_player(Direction::Right); // step onto the key
    /// game.pickup();                      // collect it
    /// assert_eq!(game.bag(), &[BagItem::Key { id: 0 }]);
    /// ```
    pub fn bag(&self) -> &[BagItem] {
        &self.bag
    }

    /// Picks up the collectible item (currently a key) at the player's current
    /// cell, adding it to the bag and clearing the cell. Returns the collected
    /// [`BagItem`], or `None` if the current cell holds no collectible.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, BagItem};
    /// let json = r#"{"grid":[["S","K","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.move_player(Direction::Right); // step onto the key cell
    /// assert_eq!(game.pickup(), Some(BagItem::Key { id: 0 }));
    /// assert_eq!(game.bag(), &[BagItem::Key { id: 0 }]);
    /// assert_eq!(game.pickup(), None);    // nothing left to pick up
    /// ```
    pub fn pickup(&mut self) -> Option<BagItem> {
        let cell = (self.player_row, self.player_col);
        if self.grid[cell.0][cell.1] == 'K' {
            let id = self.key_ids.get(&cell).copied().unwrap_or(0);
            self.grid[cell.0][cell.1] = ' ';
            let item = BagItem::Key { id };
            self.bag.push(item.clone());
            // Drop the picked-up key from the strand-check map — it's now in
            // the bag, counted directly as `bag.len()`, no longer "uncollected
            // but reachable".
            self.key_min_paths.remove(&cell);
            Some(item)
        } else {
            None
        }
    }

    /// Greedy simulation of how many keys the player could still collect from
    /// the current state — `bag.len()` plus the number of uncollected keys
    /// reachable given the budget of keys they could spend on intervening
    /// doors. Used by [`Self::move_player`]'s walk-through-door strand check.
    ///
    /// Iteratively picks the uncollected key whose minimum still-closed
    /// path-door count is cheapest under the current set of virtually-opened
    /// doors. If the player's running budget covers that cost, the simulation
    /// "spends" those keys to virtually open the path, then "collects" the key
    /// (budget regains one). Repeats until either no uncollected key is left
    /// or no remaining key is affordable. The greedy is correct for perfect
    /// mazes (each key has a single min-cost path; opening doors only ever
    /// makes other keys cheaper, never costlier).
    /// Minimum number of currently-`Locked` `'D'` cells on any path from the
    /// player's current cell to the finish. Lock-blind 0-1 BFS: entering a
    /// `Locked` door costs 1, every other passable step costs 0 (walls
    /// block; `Open` and `Opening` doors are passable for free since they're
    /// already committed). Returns `u32::MAX` if the finish is unreachable
    /// (defensive — the maze pipeline rejects unsolvable mazes).
    ///
    /// Computed on demand at each walk-through-D strand check so that
    /// opening a non-spine door which creates a shortcut to a downstream
    /// spine cell correctly drops the count: the new route may cross fewer
    /// closed doors than the original lock-blind S→F spine did, which a
    /// static seed-once counter can't see.
    fn closed_doors_to_finish(&self) -> u32 {
        let Some(finish) = self.finish else {
            return u32::MAX;
        };
        let start = (self.player_row, self.player_col);
        let mut dist: HashMap<(usize, usize), u32> = HashMap::new();
        let mut deque: VecDeque<(usize, usize)> = VecDeque::new();
        dist.insert(start, 0);
        deque.push_back(start);
        while let Some((r, c)) = deque.pop_front() {
            let d = dist[&(r, c)];
            if (r, c) == finish {
                return d;
            }
            // Skip stale entries left in the deque by a later, shorter
            // discovery — same idiom as the construction-time BFS.
            // (Without this guard we'd re-expand cells redundantly.)
            // process_order isn't needed: we just check the canonical dist
            // and bail if we're stale.
            // 4-neighbour expansion.
            let mut neighbours: Vec<(usize, usize)> = Vec::with_capacity(4);
            if r > 0 {
                neighbours.push((r - 1, c));
            }
            if c > 0 {
                neighbours.push((r, c - 1));
            }
            if r + 1 < self.rows {
                neighbours.push((r + 1, c));
            }
            if c + 1 < self.cols {
                neighbours.push((r, c + 1));
            }
            for (nr, nc) in neighbours {
                let ch = self.grid[nr][nc];
                if ch == 'W' {
                    continue;
                }
                // Edge cost: 1 only when stepping into a still-Locked 'D'.
                // Open / Opening doors are committed and cost nothing.
                let edge_cost = if ch == 'D'
                    && matches!(self.doors.get(&(nr, nc)), Some(DoorState::Locked))
                {
                    1
                } else {
                    0
                };
                let nd = d + edge_cost;
                if dist.get(&(nr, nc)).is_none_or(|&existing| nd < existing) {
                    dist.insert((nr, nc), nd);
                    if edge_cost == 0 {
                        deque.push_front((nr, nc));
                    } else {
                        deque.push_back((nr, nc));
                    }
                }
            }
        }
        u32::MAX
    }

    fn simulate_reachable_keys(&self) -> u32 {
        // Doors that are already permanently open (or committed to opening).
        let virtually_open: HashSet<(usize, usize)> = self
            .doors
            .iter()
            .filter(|(_, state)| !matches!(state, DoorState::Locked))
            .map(|(&cell, _)| cell)
            .collect();

        // Per-uncollected-key, the still-closed door sets — one Vec entry per
        // tied min-cost path. Cloned so the simulation can mutate freely.
        let mut sim_paths: KeyMinPaths = self
            .key_min_paths
            .iter()
            .map(|(&k, paths)| {
                let pruned: Vec<HashSet<(usize, usize)>> = paths
                    .iter()
                    .map(|p| p.difference(&virtually_open).copied().collect())
                    .collect();
                (k, pruned)
            })
            .collect();

        let mut sim_budget: u32 = self.bag.len() as u32;
        let mut sim_collected: u32 = 0;

        loop {
            // Pick the uncollected key with the cheapest current path cost.
            let mut best: Option<((usize, usize), u32, usize)> = None;
            for (&k, paths) in sim_paths.iter() {
                if let Some((min_cost, min_idx)) = paths
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (p.len() as u32, i))
                    .min_by_key(|&(cost, _)| cost)
                {
                    if best.is_none_or(|(_, bc, _)| min_cost < bc) {
                        best = Some((k, min_cost, min_idx));
                    }
                }
            }
            let Some((k_star, cost, path_idx)) = best else {
                break;
            };
            if sim_budget < cost {
                break;
            }
            // Virtually spend keys on the cheapest path's still-closed doors,
            // then collect K* (+1 to budget).
            sim_budget = sim_budget - cost + 1;
            sim_collected += 1;
            // The doors on this path are now virtually open — strip them from
            // every other key's tracked paths.
            let newly_open = sim_paths[&k_star][path_idx].clone();
            sim_paths.remove(&k_star);
            for paths in sim_paths.values_mut() {
                for path in paths.iter_mut() {
                    for door in newly_open.iter() {
                        path.remove(door);
                    }
                }
            }
        }

        self.bag.len() as u32 + sim_collected
    }

    /// Whether the game has ended in a loss. Mutually exclusive in practice
    /// with [`Self::is_complete`].
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, MoveResult};
    /// // Tiny maze with one key on the spine + a decoy door off the key cell —
    /// // walking through the decoy after picking up the only key strands the
    /// // player and flips `is_lost()` to true.
    /// let json = r#"{"grid":[["S","K","D","F"],["W","D","W","W"],["W"," ","W","W"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.move_player(Direction::Right);
    /// game.pickup();
    /// game.move_player(Direction::Down); // StartedUnlocking the decoy
    /// game.tick(1000.0);
    /// assert!(!game.is_lost());
    /// assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
    /// assert!(game.is_lost());
    /// ```
    pub fn is_lost(&self) -> bool {
        self.lost
    }

    /// Why the game was lost — `None` while in progress or won.
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, Direction, LoseReason};
    /// let json = r#"{"grid":[["S","K","D","F"],["W","D","W","W"],["W"," ","W","W"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.lose_reason(), None);
    /// game.move_player(Direction::Right);
    /// game.pickup();
    /// game.move_player(Direction::Down);
    /// game.tick(1000.0);
    /// game.move_player(Direction::Down); // Stranded
    /// assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    /// ```
    pub fn lose_reason(&self) -> Option<LoseReason> {
        self.lose_reason
    }
}

/// Output of [`compute_strand_reachability`] — the per-key min-path state
/// needed to seed the strand check. (The doors-to-finish half of the
/// strand inequality is recomputed dynamically by
/// [`MazeGame::closed_doors_to_finish`], so no pre-baked spine-door set is
/// stored here.)
struct StrandReachability {
    /// Per uncollected `'K'` cell that's lock-blind reachable from the start,
    /// the set of door-sets achieving the minimum number of non-spine doors
    /// on any path from that key back to the spine. Keys that are walled off
    /// from the start are simply absent from this map. See the field comment
    /// on [`MazeGame::key_min_paths`] for the runtime semantics.
    key_min_paths: KeyMinPaths,
}

/// Returns 4-neighbours of `(r, c)` in `(Up, Left, Down, Right)` order, in
/// bounds and excluding `'W'` cells.
fn passable_neighbours(
    r: usize,
    c: usize,
    grid: &[Vec<char>],
    rows: usize,
    cols: usize,
) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::with_capacity(4);
    if r > 0 && grid[r - 1][c] != 'W' {
        out.push((r - 1, c));
    }
    if c > 0 && grid[r][c - 1] != 'W' {
        out.push((r, c - 1));
    }
    if r + 1 < rows && grid[r + 1][c] != 'W' {
        out.push((r + 1, c));
    }
    if c + 1 < cols && grid[r][c + 1] != 'W' {
        out.push((r, c + 1));
    }
    out
}

/// Builds the per-key min-cost paths needed to seed
/// [`MazeGame::simulate_reachable_keys`]:
///
/// 1. **The spine.** Lock-blind BFS from `'S'` to `'F'` (every non-`'W'` cell
///    treated as passable) gives a shortest path. Spine cells are the
///    sources for the 0-1 BFS in step 2; they're computed locally and
///    consumed there, not returned to the caller.
/// 2. **Per-key min-cost paths to the spine.** Multi-source 0-1 BFS from the
///    union of spine cells, with edge weight 1 only for entering a non-spine
///    `'D'` cell and 0 elsewhere, gives every cell's minimum number of
///    non-spine doors to reach the spine. A predecessor walk back from each
///    reachable `'K'` cell enumerates every tied min-cost path and records
///    its set of off-spine door cells. Keys with the same door-set across
///    multiple tied paths are de-duped so the simulation's "pick the min" is
///    deterministic.
///
/// `key_min_paths` is empty for grids without a reachable finish — defensive
/// against hand-authored edge cases (no `'F'`, walled-off finish). In those
/// degenerate cases `closed_doors_to_finish` returns `u32::MAX` and the
/// strand inequality can never fire.
fn compute_strand_reachability(
    grid: &[Vec<char>],
    start: (usize, usize),
) -> StrandReachability {
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { 0 };

    // ── 1. Lock-blind BFS S → F to identify the spine ──────────────────────
    let mut finish: Option<(usize, usize)> = None;
    'find_finish: for (r, row) in grid.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == 'F' {
                finish = Some((r, c));
                break 'find_finish;
            }
        }
    }

    let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);
    while let Some((r, c)) = queue.pop_front() {
        for (nr, nc) in passable_neighbours(r, c, grid, rows, cols) {
            if visited.insert((nr, nc)) {
                parent.insert((nr, nc), (r, c));
                queue.push_back((nr, nc));
            }
        }
    }

    // Walk back from F to materialise the spine cells (S … F inclusive).
    let spine_cells: HashSet<(usize, usize)> = match finish {
        Some(f) if visited.contains(&f) => {
            let mut cells: HashSet<(usize, usize)> = HashSet::new();
            let mut cur = f;
            loop {
                cells.insert(cur);
                if cur == start {
                    break;
                }
                cur = parent[&cur];
            }
            cells
        }
        _ => HashSet::new(),
    };
    // No spine ⇒ no strand check ⇒ no key paths needed.
    if spine_cells.is_empty() {
        return StrandReachability {
            key_min_paths: HashMap::new(),
        };
    }

    // ── 2. Multi-source 0-1 BFS from spine to every reachable cell ─────────
    //
    // Edge weight when stepping into `(nr, nc)` is 1 if that cell is a
    // non-spine `'D'`, else 0. Spine doors contribute 0 because the cost
    // of opening them lives on the other side of the strand inequality —
    // see `closed_doors_to_finish`, which counts them dynamically against
    // the player's current cell.
    //
    // Each cell is also assigned a `process_order` — the BFS finalization
    // index. That total order across same-dist cells lets the predecessor
    // build below break 0-edge ties acyclically: only neighbours processed
    // before this cell can be its predecessor. Without that tie-break, two
    // adjacent same-dist cells connected by a 0-edge would each list the
    // other as a predecessor, and any path enumeration would loop.
    let edge_cost = |cell: (usize, usize)| -> u32 {
        if grid[cell.0][cell.1] == 'D' && !spine_cells.contains(&cell) {
            1
        } else {
            0
        }
    };

    let mut dist: HashMap<(usize, usize), u32> = HashMap::new();
    let mut process_order: HashMap<(usize, usize), u32> = HashMap::new();
    let mut next_order: u32 = 0;
    let mut deque: VecDeque<(usize, usize)> = VecDeque::new();
    for &cell in spine_cells.iter() {
        dist.insert(cell, 0);
        deque.push_back(cell);
    }
    while let Some((r, c)) = deque.pop_front() {
        // Skip stale dupes left in the deque by an earlier shorter-path update.
        if process_order.contains_key(&(r, c)) {
            continue;
        }
        process_order.insert((r, c), next_order);
        next_order += 1;
        let d = dist[&(r, c)];
        for (nr, nc) in passable_neighbours(r, c, grid, rows, cols) {
            let nd = d + edge_cost((nr, nc));
            if dist.get(&(nr, nc)).is_none_or(|&existing| nd < existing) {
                dist.insert((nr, nc), nd);
                if edge_cost((nr, nc)) == 0 {
                    deque.push_front((nr, nc));
                } else {
                    deque.push_back((nr, nc));
                }
            }
        }
    }

    // Predecessor map: every neighbour `n` of cell `c` such that
    // `dist[n] + edge_cost(c) == dist[c]` lies on a min-cost path into `c`.
    // We additionally require `process_order[n] < process_order[c]` so that
    // 0-edge same-dist siblings can't claim each other as predecessors —
    // the first such sibling popped is the "upstream" one in BFS terms.
    let mut preds: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
    for (&cell, &d) in dist.iter() {
        if spine_cells.contains(&cell) {
            continue; // spine cells are sources — no preds needed
        }
        let cost_into = edge_cost(cell);
        let Some(&cell_order) = process_order.get(&cell) else {
            continue;
        };
        for n in passable_neighbours(cell.0, cell.1, grid, rows, cols) {
            if let (Some(&nd), Some(&n_order)) =
                (dist.get(&n), process_order.get(&n))
            {
                if nd + cost_into == d && n_order < cell_order {
                    preds.entry(cell).or_default().push(n);
                }
            }
        }
    }

    // For each reachable `'K'`, recursively enumerate every min-cost path to
    // the spine. The path's door set is exactly the non-spine `'D'` cells
    // encountered on the way (the key cell itself is `'K'`, not a door, and
    // the path terminates at a spine cell which is treated as cost 0).
    fn enumerate_paths(
        cell: (usize, usize),
        grid: &[Vec<char>],
        preds: &HashMap<(usize, usize), Vec<(usize, usize)>>,
        spine_cells: &HashSet<(usize, usize)>,
        memo: &mut KeyMinPaths,
    ) -> Vec<HashSet<(usize, usize)>> {
        if spine_cells.contains(&cell) {
            return vec![HashSet::new()];
        }
        if let Some(cached) = memo.get(&cell) {
            return cached.clone();
        }
        let mut out: Vec<HashSet<(usize, usize)>> = Vec::new();
        let cell_is_off_spine_door =
            grid[cell.0][cell.1] == 'D' && !spine_cells.contains(&cell);
        if let Some(predecessors) = preds.get(&cell) {
            for &p in predecessors {
                let sub_paths = enumerate_paths(p, grid, preds, spine_cells, memo);
                for sub in sub_paths {
                    let mut path = sub.clone();
                    if cell_is_off_spine_door {
                        path.insert(cell);
                    }
                    out.push(path);
                }
            }
        }
        memo.insert(cell, out.clone());
        out
    }

    let mut memo: KeyMinPaths = HashMap::new();
    let mut key_min_paths: KeyMinPaths =
        HashMap::new();
    for (r, row) in grid.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == 'K' && dist.contains_key(&(r, c)) {
                let mut paths = enumerate_paths((r, c), grid, &preds, &spine_cells, &mut memo);
                // De-duplicate tied paths whose off-spine-door SETS happen to
                // coincide — only the door sets matter to the simulation.
                paths.sort_by(|a, b| {
                    let mut av: Vec<(usize, usize)> = a.iter().copied().collect();
                    av.sort();
                    let mut bv: Vec<(usize, usize)> = b.iter().copied().collect();
                    bv.sort();
                    av.cmp(&bv)
                });
                paths.dedup();
                key_min_paths.insert((r, c), paths);
            }
        }
    }

    StrandReachability { key_min_paths }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── from_json ──────────────────────────────────────────────────────────────

    #[test]
    fn from_json_places_player_at_start() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.player_row(), 0);
        assert_eq!(game.player_col(), 0);
    }

    #[test]
    fn from_json_start_not_at_origin() {
        #[rustfmt::skip]
        let json = r#"{"grid":[[" "," "," "],[" ","S","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.player_row(), 1);
        assert_eq!(game.player_col(), 1);
    }

    #[test]
    fn from_json_initial_direction_is_none() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.player_direction(), Direction::None);
    }

    #[test]
    fn from_json_not_complete_initially() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert!(!game.is_complete());
    }

    #[test]
    fn from_json_visited_cells_contains_start() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.visited_cells(), &[(0, 0)]);
    }

    #[test]
    fn from_json_err_on_invalid_json() {
        let result = MazeGame::from_json("{bad json}");
        assert!(result.is_err());
    }

    #[test]
    fn from_json_err_on_no_start_cell() {
        let json = r#"{"grid":[[" "," ","F"]]}"#;
        let result = MazeGame::from_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no start cell"));
    }

    // ── move_player — basic movement ───────────────────────────────────────────

    #[test]
    fn move_right_into_empty_cell() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
        assert_eq!(game.player_row(), 0);
        assert_eq!(game.player_col(), 1);
        assert_eq!(game.player_direction(), Direction::Right);
    }

    #[test]
    fn move_left_into_start_cell() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // col 1
        assert_eq!(game.move_player(Direction::Left), MoveResult::Moved);
        assert_eq!(game.player_col(), 0);
    }

    #[test]
    fn move_down_into_empty_cell() {
        #[rustfmt::skip]
        let json = r#"{"grid":[["S"," "],["F"," "]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Down), MoveResult::Complete);
        assert_eq!(game.player_row(), 1);
    }

    #[test]
    fn move_up_into_empty_cell() {
        #[rustfmt::skip]
        let json = r#"{"grid":[[" ","F"],["S"," "]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Up), MoveResult::Moved);
        assert_eq!(game.player_row(), 0);
    }

    // ── move_player — reach finish ─────────────────────────────────────────────

    #[test]
    fn reaching_finish_returns_complete() {
        let json = r#"{"grid":[["S","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
        assert!(game.is_complete());
    }

    #[test]
    fn finish_position_updated_on_complete() {
        let json = r#"{"grid":[["S","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        assert_eq!(game.player_row(), 0);
        assert_eq!(game.player_col(), 1);
    }

    // ── move_player — blocked ──────────────────────────────────────────────────

    #[test]
    fn move_into_wall_returns_blocked() {
        let json = r#"{"grid":[["S","W","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Right), MoveResult::Blocked);
        assert_eq!(game.player_col(), 0);
    }

    #[test]
    fn move_left_out_of_bounds_returns_blocked() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Left), MoveResult::Blocked);
        assert_eq!(game.player_col(), 0);
    }

    #[test]
    fn move_up_out_of_bounds_returns_blocked() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Up), MoveResult::Blocked);
        assert_eq!(game.player_row(), 0);
    }

    #[test]
    fn move_right_out_of_bounds_returns_blocked() {
        let json = r#"{"grid":[["F"," ","S"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Right), MoveResult::Blocked);
        assert_eq!(game.player_col(), 2);
    }

    #[test]
    fn move_down_out_of_bounds_returns_blocked() {
        let json = r#"{"grid":[["F"," "],["S"," "]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Down), MoveResult::Blocked);
        assert_eq!(game.player_row(), 1);
    }

    #[test]
    fn direction_updated_even_when_blocked() {
        let json = r#"{"grid":[["S","W","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        assert_eq!(game.player_direction(), Direction::Right);
        game.move_player(Direction::Up);
        assert_eq!(game.player_direction(), Direction::Up);
    }

    // ── visited cells ──────────────────────────────────────────────────────────

    #[test]
    fn visited_cells_grows_with_each_move() {
        let json = r#"{"grid":[["S"," "," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.visited_cells().len(), 1);
        game.move_player(Direction::Right);
        assert_eq!(game.visited_cells().len(), 2);
        game.move_player(Direction::Right);
        assert_eq!(game.visited_cells().len(), 3);
    }

    #[test]
    fn visited_cells_not_updated_on_blocked() {
        let json = r#"{"grid":[["S","W","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        assert_eq!(game.visited_cells().len(), 1);
    }

    #[test]
    fn visited_cells_includes_finish_on_complete() {
        let json = r#"{"grid":[["S","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        assert_eq!(game.visited_cells(), &[(0, 0), (0, 1)]);
    }

    #[test]
    fn visited_cells_order_matches_movement() {
        #[rustfmt::skip]
        let json = r#"{"grid":[["S"," "],["F"," "]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // (0,1)
        game.move_player(Direction::Down);  // (1,1)
        assert_eq!(game.visited_cells(), &[(0, 0), (0, 1), (1, 1)]);
    }

    #[test]
    fn grid_returns_parsed_grid() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.grid(), &[vec!['S', ' ', 'F']]);
    }

    // ── keys & doors — construction ─────────────────────────────────────────────

    #[test]
    fn from_json_seeds_doors_as_locked() {
        let json = r#"{"grid":[["S","D","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.doors(), vec![((0, 1), DoorState::Locked)]);
    }

    #[test]
    fn from_json_lists_keys_with_ids() {
        let json = r#"{"grid":[["S","K","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.keys(), vec![((0, 1), 0)]);
        assert!(game.bag().is_empty());
    }

    #[test]
    fn from_json_assigns_key_ids_and_seeds_multiple_doors() {
        #[rustfmt::skip]
        let json = r#"{"grid":[["S","K"],["K","D"],["D","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.keys(), vec![((0, 1), 0), ((1, 0), 1)]);
        assert_eq!(
            game.doors(),
            vec![((1, 1), DoorState::Locked), ((2, 0), DoorState::Locked)]
        );
    }

    // ── keys — explicit pickup ───────────────────────────────────────────────────

    #[test]
    fn moving_onto_key_does_not_collect_it() {
        let json = r#"{"grid":[["S","K","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
        assert_eq!(game.player_col(), 1);
        assert!(game.bag().is_empty());
        assert_eq!(game.grid()[0][1], 'K'); // key still present
        assert_eq!(game.keys(), vec![((0, 1), 0)]);
    }

    #[test]
    fn pickup_collects_key_at_current_cell() {
        let json = r#"{"grid":[["S","K","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // stand on the key
        assert_eq!(game.pickup(), Some(BagItem::Key { id: 0 }));
        assert_eq!(game.bag(), &[BagItem::Key { id: 0 }]);
        assert_eq!(game.grid()[0][1], ' '); // cell cleared
        assert!(game.keys().is_empty());
    }

    #[test]
    fn pickup_returns_none_when_no_key_present() {
        let json = r#"{"grid":[["S","K","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(game.pickup(), None); // on the start cell
        game.move_player(Direction::Right);
        game.pickup();
        assert_eq!(game.pickup(), None); // already collected
    }

    // ── doors — blocking & unlocking ─────────────────────────────────────────────

    #[test]
    fn locked_door_without_key_blocks_and_does_not_move() {
        let json = r#"{"grid":[["S","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert_eq!(
            game.move_player(Direction::Right),
            MoveResult::BlockedByLockedDoor
        );
        assert_eq!(game.player_col(), 0);
        assert_eq!(game.doors(), vec![((0, 1), DoorState::Locked)]);
    }

    #[test]
    fn door_blocks_if_key_not_picked_up() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // onto the key, but do not pick it up
        assert_eq!(
            game.move_player(Direction::Right),
            MoveResult::BlockedByLockedDoor
        );
    }

    #[test]
    fn locked_door_with_key_starts_unlocking_and_consumes_key() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // onto the key
        game.pickup(); // collect it
        assert_eq!(
            game.move_player(Direction::Right),
            MoveResult::StartedUnlocking
        );
        assert_eq!(game.player_col(), 1); // did not step onto the door
        assert!(game.bag().is_empty()); // key consumed
        assert_eq!(
            game.doors(),
            vec![((0, 2), DoorState::Opening { progress: 0.0 })]
        );
    }

    #[test]
    fn moving_into_opening_door_blocks_without_consuming_another_key() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        assert_eq!(
            game.move_player(Direction::Right),
            MoveResult::BlockedByLockedDoor
        );
        assert!(game.bag().is_empty());
    }

    // ── doors — tick / opening ───────────────────────────────────────────────────

    #[test]
    fn tick_without_opening_doors_emits_nothing() {
        let json = r#"{"grid":[["S","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        assert!(game.tick(1000.0).is_empty());
        assert_eq!(game.doors(), vec![((0, 1), DoorState::Locked)]);
    }

    #[test]
    fn tick_opens_door_after_countdown_and_emits_event() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        assert_eq!(
            game.tick(1000.0),
            vec![GameEvent::DoorOpened { cell: (0, 2) }]
        );
        assert_eq!(game.doors(), vec![((0, 2), DoorState::Open)]);
    }

    #[test]
    fn tick_partial_progress_does_not_open() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        assert!(game.tick(500.0).is_empty());
        assert!(game.tick(400.0).is_empty());
        assert_eq!(
            game.tick(200.0),
            vec![GameEvent::DoorOpened { cell: (0, 2) }]
        );
    }

    #[test]
    fn opened_door_is_passable() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        game.tick(1000.0); // door opens
        assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
        assert_eq!(game.player_col(), 2);
        assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
        assert!(game.is_complete());
    }

    // ── lose state — initial ─────────────────────────────────────────────────────

    #[test]
    fn new_game_is_neither_won_nor_lost() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert!(!game.is_complete());
        assert!(!game.is_lost());
        assert_eq!(game.lose_reason(), None);
    }

    // ── stranded detection — path doors & decoys ─────────────────────────────────

    #[test]
    fn solo_path_door_walked_through_does_not_strand() {
        // S · K · D · F — one real door on the path, exactly one key. Opening
        // and walking through it preserves the inequality (1>0 false after
        // path-door commit: both counters drop to 0).
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        game.tick(1000.0); // door opens
        assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
        assert!(!game.is_lost());
        assert_eq!(game.lose_reason(), None);
    }

    #[test]
    fn decoy_door_with_spare_key_does_not_strand() {
        // S · K · K · D(decoy) — two keys, one decoy door (no path door
        // anywhere since F is reachable without crossing any door). After
        // opening the decoy: path_remaining=0, available=1 (one key still in
        // bag — picked up second key first). 0 > 1 false, not stranded.
        //
        // Grid layout (the decoy hangs off a side branch; F is in the main
        // corridor with no door gating it):
        //  S · K · K
        //          |
        //          D  (decoy — opening it leads to a dead end ' ' below)
        //          |
        //          ' '
        // ... reached via going down from the second K
        //
        // F sits on the top row at the right, accessible without any door.
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","K","F"],
            ["W","W","D","W"],
            ["W","W"," ","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Pick up both keys.
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right);
        game.pickup();
        // Walk down through the decoy.
        game.move_player(Direction::Down); // StartedUnlocking the decoy at (1,2)
        game.tick(1000.0);
        let result = game.move_player(Direction::Down); // walk through decoy
        assert_eq!(result, MoveResult::Moved);
        assert!(!game.is_lost());
    }

    #[test]
    fn decoy_door_with_only_one_key_strands_on_walk_through() {
        // Lock-blind path is the top row: S → K → D(real) → F. Path doors = 1,
        // total keys = 1. The decoy at (1,1) hangs off a side branch from the
        // key cell. If the player detours into the decoy first, they spend
        // their only key on it → available_keys=0, path_remaining=1 → 1>0,
        // stranded once they walk through.
        //
        //  S  K  D(real)  F
        //     |
        //     D(decoy)
        //     |
        //     ' '
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Grab the key at (0,1).
        game.move_player(Direction::Right);
        game.pickup();
        // Walk down into the decoy at (1,1).
        game.move_player(Direction::Down); // StartedUnlocking the decoy
        game.tick(1000.0);
        // Walk through the decoy — this is the trigger point.
        let result = game.move_player(Direction::Down);
        assert_eq!(result, MoveResult::Stranded);
        assert!(game.is_lost());
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    }

    #[test]
    fn stranded_state_is_terminal_subsequent_moves_do_not_change_reason() {
        // Once stranded, walking off the door and back onto it must not
        // re-surface Stranded — the `!self.lost` guard short-circuits the
        // walk-through check on every subsequent crossing.
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Down); // StartedUnlocking the decoy
        game.tick(1000.0);
        // First walk-through — strands.
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        // Step off the door (back up to the now-empty key cell), then step
        // back onto the open door — the second walk-through must be a plain
        // Moved with no fresh Stranded surfaced.
        assert_eq!(game.move_player(Direction::Up), MoveResult::Moved);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Moved);
        // The lose reason stays Stranded throughout.
        assert!(game.is_lost());
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    }

    #[test]
    fn unreachable_keys_do_not_inflate_available_keys() {
        // Hand-authored hazard: a `'K'` walled off from the start cell can
        // never become available, so it must not count toward `available_keys`
        // — otherwise the strand check would think the player has a spare key
        // they can never actually collect, and miss a real strand.
        //
        // Layout: real door on the top-row path, decoy off the key cell, and
        // an unreachable K in a pocket on row 4.
        //
        //  S  K  D(real)  F
        //  W  D(decoy) W  W
        //  W  ' '      W  W
        //  W  W        W  W
        //  W  K(unreachable) W  W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"],
            ["W","W","W","W"],
            ["W","K","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Take the reachable key, spend it on the decoy, walk through.
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Down); // StartedUnlocking the decoy
        game.tick(1000.0);
        // With the bug (counting all `'K'` cells), available_keys would be 2
        // here, the inequality 1>1 would be false, and the walk-through would
        // return `Moved`. With the fix (counting only lock-blind-reachable
        // keys), available_keys is 1, drops to 0 at StartedUnlocking, and the
        // walk-through correctly surfaces `Stranded`.
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    }

    #[test]
    fn no_doors_means_no_strand_ever() {
        // A maze with no doors never triggers the walk-through-D check
        // (there are no `'D'` cells to walk through), so the strand
        // inequality never gets a chance to fire.
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.move_player(Direction::Right);
        assert!(!game.is_lost());
        assert_eq!(game.lose_reason(), None);
    }

    // ── stranded detection — chained keys & key-pile scenarios ───────────────────

    #[test]
    fn chained_keys_behind_one_door_strands_when_only_key_spent_elsewhere() {
        // The originally-reported gap in the simple `available_keys` counter:
        // 5 keys K1..K5 hidden behind a single locked door D_pile, with one
        // freely-accessible spare K0 and one off-spine decoy D_decoy. If the
        // player burns K0 on D_decoy, they're stranded — they can't get K1..K5
        // without first opening D_pile, and they no longer hold any key.
        //
        // Old per-key counter would have counted all six K cells toward
        // `available_keys`, masked the strand. The new design sees that
        // K1..K5 each have a path-door cost of 1 (D_pile) and that no key is
        // affordable at budget 0 → accessible = 0 < 1 closed path door.
        //
        //  Col:   0   1   2          3   4   5         6        7
        //  Row 0: S   K0  ' '        ' ' ' ' ' '       D_path   F
        //  Row 1: W   W   D_decoy    W   W   D_pile    W        W
        //  Row 2: W   W   ' '        W   W   K1        W        W
        //  Row 3: W   W   W          W   W   K2        W        W
        //  Row 4: W   W   W          W   W   K3        W        W
        //  Row 5: W   W   W          W   W   K4        W        W
        //  Row 6: W   W   W          W   W   K5        W        W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K"," "," "," "," ","D","F"],
            ["W","W","D","W","W","D","W","W"],
            ["W","W"," ","W","W","K","W","W"],
            ["W","W","W","W","W","K","W","W"],
            ["W","W","W","W","W","K","W","W"],
            ["W","W","W","W","W","K","W","W"],
            ["W","W","W","W","W","K","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Pick up the lone reachable key.
        game.move_player(Direction::Right);
        game.pickup();
        // Drift onto the open cell next to the decoy.
        game.move_player(Direction::Right);
        // Burn the key on the decoy.
        game.move_player(Direction::Down); // StartedUnlocking the decoy
        game.tick(1000.0);
        // Walk through it. The 5 keys behind D_pile are now unreachable.
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    }

    #[test]
    fn chained_keys_behind_one_door_not_stranded_with_two_spare_keys() {
        // Same chained-keys layout but with TWO directly-reachable spare keys
        // (K0 + K_spare). The player can burn the decoy and still have a key
        // left over for D_path on the spine.
        //
        //  Col:   0    1   2          3   4   5         6        7
        //  Row 0: S    K0  K_spare    ' ' ' ' ' '       D_path   F
        //  Row 1: W    W   D_decoy    W   W   D_pile    W        W
        //  Row 2: W    W   ' '        W   W   K1        W        W
        //  Row 3: W    W   W          W   W   K2        W        W
        //  Row 4: W    W   W          W   W   W         W        W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","K"," "," "," ","D","F"],
            ["W","W","D","W","W","D","W","W"],
            ["W","W"," ","W","W","K","W","W"],
            ["W","W","W","W","W","K","W","W"],
            ["W","W","W","W","W","W","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Walk over both spine keys and pick them up.
        game.move_player(Direction::Right); // → (0,1) K0
        game.pickup();
        game.move_player(Direction::Right); // → (0,2) K_spare
        game.pickup();
        // Step right onto the open spine cell next to the decoy.
        game.move_player(Direction::Right); // → (0,3)
        // Hmm — (1,3) is W, so we can't go down from here to reach the decoy.
        // Step back to (0,2) and go down through D_decoy at (1,2).
        game.move_player(Direction::Left); // → (0,2)
        game.move_player(Direction::Down); // StartedUnlocking D_decoy
        game.tick(1000.0);
        // Walk through. Two keys had been picked up; one was spent on D_decoy,
        // one remains. closed_path = 1 (D_path). accessible = 1 + 0 = 1.
        // 1 > 1 is false → not stranded.
        let result = game.move_player(Direction::Down);
        assert_eq!(result, MoveResult::Moved);
        assert!(!game.is_lost());
    }

    // ── stranded detection — generic decoy scenarios ─────────────────────────────

    #[test]
    fn single_decoy_with_no_spare_strands() {
        // Bare-bones: one key on the spine, one off-spine decoy, one path
        // door on the spine. Burning the lone key on the decoy strands.
        //
        //  S  K  D_path  F          row 0
        //  W  D_decoy  W  W         row 1 (D_decoy at (1,1))
        //  W  ' '      W  W         row 2
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Down); // StartedUnlocking decoy
        game.tick(1000.0);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
    }

    #[test]
    fn two_decoys_with_one_spare_key_strands_after_second_burn() {
        //  Pick up and then waste two keys on decoy1 and decoy2 doors  
        //  Col:   0         1   2         3         4
        //  Row 0: S         K0  K1        D_path    F
        //  Row 1: D_decoy1  W   D_decoy2  W         W
        //  Row 2: ' '       W   ' '       W         W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","K","D","F"],
            ["D","W","D","W","W"],
            [" ","W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Pick up both spine keys.
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right);
        game.pickup();
        // Burn one key on D_d2 at (1,2) — go back to (0,2), Down.
        game.move_player(Direction::Down); // (1,2) D_d2: StartedUnlocking
        game.tick(1000.0);
        // First walk-through: should still be OK (one key left, closed_path=1).
        let r1 = game.move_player(Direction::Down);
        assert_eq!(r1, MoveResult::Moved);
        assert!(!game.is_lost());
        // Walk back up, over to (0,0), down through D_d1 at (1,0).
        game.move_player(Direction::Up);
        game.move_player(Direction::Left);
        game.move_player(Direction::Left);
        game.move_player(Direction::Down); // StartedUnlocking D_d1
        game.tick(1000.0);
        // Second walk-through: no keys left, path door D_path still closed.
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
    }

    // ── stranded detection — keys behind path doors / cascade unlocks ────────────

    #[test]
    fn key_behind_path_door_is_solvable_via_in_order_collection() {
        // Spine: S K0 D_path1 K1 D_path2 F. Two path doors, two keys, one
        // each per segment. Order of operations:
        //   - Pick K0, open D_path1 (consume K0).
        //   - Walk through, pick K1, open D_path2 (consume K1).
        //   - Walk to F.
        let json = r#"{"grid":[["S","K","D","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Pick K0 at (0,1).
        game.move_player(Direction::Right);
        game.pickup();
        // Open D1 at (0,2) (consumes K0).
        game.move_player(Direction::Right); // StartedUnlocking
        game.tick(1000.0);
        // Walk through D1 onto (0,2).
        let r1 = game.move_player(Direction::Right);
        assert_eq!(r1, MoveResult::Moved);
        assert!(!game.is_lost());
        // Step onto K1 at (0,3) and pick it up.
        game.move_player(Direction::Right);
        game.pickup();
        // Open D2 at (0,4) (consumes K1).
        game.move_player(Direction::Right); // StartedUnlocking
        game.tick(1000.0);
        // Walk through D2 onto (0,4).
        let r2 = game.move_player(Direction::Right);
        assert_eq!(r2, MoveResult::Moved);
        // Final step onto F at (0,5).
        assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
        assert!(!game.is_lost());
    }

    #[test]
    fn key_on_spine_is_treated_as_free_at_start() {
        let json = r#"{"grid":[["S","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // StartedUnlocking
        game.tick(1000.0);
        let r = game.move_player(Direction::Right);
        assert_eq!(r, MoveResult::Moved);
        assert!(!game.is_lost());
    }

    #[test]
    fn shared_door_chain_propagates_cost_drop_via_simulation() {
        // Two keys K1 and K2 both reachable only behind the same door D_pile.
        // The player has one spare key K0 on the spine and an additional
        // path door D_path. Order:
        //   - Pick K0.
        //   - Open D_pile with K0 (consume).
        //   - Walk through; pick K1, then K2.
        //   - Open D_path with one of them.
        //   - Walk to F.
        //
        //  Col:   0   1   2     3       4
        //  Row 0: S   K0  ' '   D_path  F
        //  Row 1: W   W   D_p   W       W
        //  Row 2: W   W   K1    W       W
        //  Row 3: W   W   K2    W       W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K"," ","D","F"],
            ["W","W","D","W","W"],
            ["W","W","K","W","W"],
            ["W","W","K","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // (0,1) K0
        game.pickup();
        game.move_player(Direction::Right); // (0,2)
        game.move_player(Direction::Down); // (1,2) D_pile: StartedUnlocking
        game.tick(1000.0);
        let walk1 = game.move_player(Direction::Down); // through D_pile
        assert_eq!(walk1, MoveResult::Moved);
        assert!(!game.is_lost());
        // K1 + K2 are now reachable without any further doors. Pick both up.
        game.move_player(Direction::Down); // (2,2) K1
        game.pickup();
        game.move_player(Direction::Down); // (3,2) K2
        game.pickup();
        // Climb back up to the spine.
        game.move_player(Direction::Up); // (2,2)
        game.move_player(Direction::Up); // (1,2) (now open)
        game.move_player(Direction::Up); // (0,2)
        // Open the path door.
        game.move_player(Direction::Right); // (0,3) D_path: StartedUnlocking
        game.tick(1000.0);
        let walk2 = game.move_player(Direction::Right); // through D_path
        assert_eq!(walk2, MoveResult::Moved);
        assert!(!game.is_lost());
        assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
    }

    #[test]
    fn cascade_unlock_two_doors_one_key_each_solvable() {
        // S K1 D1 K2 D2 F — classic cascade. After K1 opens D1 the player
        // picks up K2 to open D2. No strand at any walk-through.
        let json = r#"{"grid":[["S","K","D","K","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        for _ in 0..2 {
            game.move_player(Direction::Right);
            if let Some(BagItem::Key { .. }) = game.pickup() {
                game.move_player(Direction::Right); // StartedUnlocking
                game.tick(1000.0);
                assert_eq!(game.move_player(Direction::Right), MoveResult::Moved);
            }
        }
        // Final step onto F.
        assert_eq!(game.move_player(Direction::Right), MoveResult::Complete);
        assert!(!game.is_lost());
    }

    // ── stranded detection — non-strand sanity checks ────────────────────────────

    #[test]
    fn walking_through_a_path_door_with_remaining_keys_does_not_strand() {
        // 2 keys, 2 path doors. Open one with one key, walk through; second
        // key still in the bag. Not stranded.
        let json = r#"{"grid":[["S","K","K","D","D","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Right); // (0,3) D: StartedUnlocking
        game.tick(1000.0);
        let r = game.move_player(Direction::Right);
        assert_eq!(r, MoveResult::Moved);
        assert!(!game.is_lost());
    }

    #[test]
    fn does_not_strand_when_player_takes_an_alternative_solution_path() {
        // Regression for a real bug found in manual play. Multi-path maze:
        // there are two routes from S to F, each crossing a different door.
        // The lock-blind shortest path picks one (S → D(0,1) → (1,2) → D(1,3)
        // → F); the *other* route (S → … → D(2,1) → (1,2) → D(1,3) → F)
        // happens to be even shorter and is what the key-aware solver shows
        // to the player. The static `path_doors_remaining_closed` counter
        // is invariant to which route the player picks — it stays at 2 (both
        // lock-blind spine doors) even after the player opens D(2,1), which
        // makes the (1,2)-side of the spine reachable WITHOUT crossing
        // D(0,1). The strand check then over-counts required doors and
        // falsely strands the player.
        //
        //         c0     c1    c2    c3    c4
        //  r0:    S      D     ' '    W     W
        //  r1:   ' '     W     K      D     F
        //  r2:   ' '     D     ' '    W     W
        //  r3:   ' '     W     W      W     W
        //  r4:    K      D     ' '    W     W
        //
        // Both (0,1) and (2,1) connect S's left-column branch to the row
        // containing the spine key K(1,2). From K(1,2) the player needs one
        // more key to open D(1,3) — but K(1,2) itself is a key! So the
        // route is: pick up K(4,0); walk to D(?,1), open it; walk to
        // K(1,2), pick up; walk to D(1,3), open it; reach F. Two keys, two
        // doors — solvable via *either* of the two left-side doors. The
        // strand check must accept either choice.
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","D"," ","W","W"],
            [" ","W","K","D","F"],
            [" ","D"," ","W","W"],
            [" ","W","W","W","W"],
            ["K","D"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Walk down the left column to K(4,0) and pick it up.
        game.move_player(Direction::Down); // (1,0)
        game.move_player(Direction::Down); // (2,0)
        game.move_player(Direction::Down); // (3,0)
        game.move_player(Direction::Down); // (4,0) = K
        game.pickup();
        // Walk back up to (2,0).
        game.move_player(Direction::Up);   // (3,0)
        game.move_player(Direction::Up);   // (2,0)
        // Open D(2,1) — the OFF-lock-blind-spine door — and walk through.
        game.move_player(Direction::Right); // StartedUnlocking D(2,1)
        game.tick(1000.0);
        let result = game.move_player(Direction::Right); // through D(2,1)
        assert_eq!(
            result,
            MoveResult::Moved,
            "expected Moved (alt path still leads to F via K(1,2) + D(1,3)), got {result:?}"
        );
        assert!(
            !game.is_lost(),
            "must not strand when the player takes a valid alternative route to the spine"
        );
    }

    #[test]
    fn walking_through_an_already_opened_decoy_with_no_keys_does_not_re_strand() {
        // Once `lost` is set, the walk-through check short-circuits and
        // returns plain `Moved`. Confirmed elsewhere via the "terminal"
        // test; this one specifically validates the path-door interaction:
        // strand fires at first walk-through, then a subsequent move onto
        // any open door returns `Moved`.
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Down); // StartedUnlocking D_decoy
        game.tick(1000.0);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        // Step back up onto the now-empty spine cell, then back down onto
        // the open decoy — second walk-through must NOT surface Stranded
        // again.
        game.move_player(Direction::Up);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Moved);
    }
}
