use data_model::MazeDefinition;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

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
/// Extensible: future variants could cover death events, environmental hazards,
/// etc.
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
    /// The wall-clock countdown reached zero — signalled by the caller via
    /// [`MazeGame::time_out`].
    Timeout,
    /// The player no longer holds enough keys (collected + still in the world)
    /// to open every real door remaining on the solution path. Set when the
    /// player walks through an open door and the inequality
    /// `path_doors_remaining_closed > available_keys` is true.
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
/// [`MazeGame::is_lost`] / [`MazeGame::lose_reason`]; [`MazeGame::time_out`]
/// sets it from the caller's countdown.
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
    /// Cells holding a real door on the lock-blind shortest path from `'S'` to
    /// `'F'` — the doors the player must open to finish the maze. Computed once
    /// in [`MazeGame::from_json`] and never mutated. Any `'D'` cell **not** in
    /// this set is a decoy: opening it consumes a key without bringing the
    /// player closer to the finish.
    solution_path_doors: HashSet<(usize, usize)>,
    /// Number of doors in [`Self::solution_path_doors`] that are still
    /// `Locked` / `Opening` (i.e. not yet committed open by the player). Seeded
    /// to `solution_path_doors.len()` and decremented once per path-door at the
    /// moment the player commits a key to it (the `StartedUnlocking` branch of
    /// [`Self::move_player`]).
    path_doors_remaining_closed: u32,
    /// Total keys still available to the player: keys currently in the bag plus
    /// uncollected `'K'` cells in the world. Seeded to the total `'K'` count at
    /// construction and decremented once each time a key is consumed at a door
    /// (`StartedUnlocking`). Pickup is a no-op for this counter — a key just
    /// moves from "in the world" to "in the bag".
    available_keys: u32,
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
        let mut total_keys: u32 = 0;
        for (r, row) in definition.grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                match ch {
                    'D' => {
                        doors.insert((r, c), DoorState::Locked);
                    }
                    'K' => {
                        key_ids.insert((r, c), next_key_id);
                        next_key_id += 1;
                        total_keys += 1;
                    }
                    _ => {}
                }
            }
        }

        let solution_path_doors =
            compute_solution_path_doors(&definition.grid, (start.row, start.col));
        let path_doors_remaining_closed = solution_path_doors.len() as u32;

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
            solution_path_doors,
            path_doors_remaining_closed,
            available_keys: total_keys,
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
                    // Walked through an open door — the user-spec trigger point
                    // for stranded detection. The counters were updated at the
                    // moment the key was committed (StartedUnlocking below), so
                    // the inequality reflects the post-commit state.
                    if !self.lost
                        && self.path_doors_remaining_closed > self.available_keys
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
                        // Commit the key: both counters drop in lock-step for a
                        // path door (LHS-1, RHS-1 → inequality preserved); for
                        // a decoy only RHS drops, which may flip the
                        // inequality and surface as `Stranded` when the player
                        // later walks through this (or any) open door.
                        self.available_keys = self.available_keys.saturating_sub(1);
                        if self.solution_path_doors.contains(&(new_row, new_col)) {
                            self.path_doors_remaining_closed =
                                self.path_doors_remaining_closed.saturating_sub(1);
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
            Some(item)
        } else {
            None
        }
    }

    /// Marks the game as lost with [`LoseReason::Timeout`] — called by the
    /// caller when the wall-clock countdown reaches zero. Idempotent: a
    /// subsequent call does not overwrite an existing [`LoseReason`] (e.g. if
    /// the player was already stranded).
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::{MazeGame, LoseReason};
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// game.time_out();
    /// assert!(game.is_lost());
    /// assert_eq!(game.lose_reason(), Some(LoseReason::Timeout));
    /// ```
    pub fn time_out(&mut self) {
        if !self.lost {
            self.lost = true;
            self.lose_reason = Some(LoseReason::Timeout);
        }
    }

    /// Whether the game has ended in a loss. Mutually exclusive in practice
    /// with [`Self::is_complete`].
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::MazeGame;
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// assert!(!game.is_lost());
    /// game.time_out();
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
    /// use maze::{MazeGame, LoseReason};
    /// let json = r#"{"grid":[["S"," ","F"]]}"#;
    /// let mut game = MazeGame::from_json(json).unwrap();
    /// assert_eq!(game.lose_reason(), None);
    /// game.time_out();
    /// assert_eq!(game.lose_reason(), Some(LoseReason::Timeout));
    /// ```
    pub fn lose_reason(&self) -> Option<LoseReason> {
        self.lose_reason
    }
}

/// Computes the set of door cells that gate the lock-blind shortest path from
/// the start cell `'S'` to the finish cell `'F'`. Walls (`'W'`) block; every
/// other cell (including `'K'` and `'D'`) is treated as passable — so the set
/// is exactly the doors the player must open to complete the maze, with no
/// detour for keys folded in.
///
/// Returns an empty set if the grid has no `'F'` cell or the finish is
/// unreachable. Both are defensive — the maze pipeline produces only solvable
/// mazes with a finish — but keep the runtime robust against hand-authored
/// edge cases.
fn compute_solution_path_doors(
    grid: &[Vec<char>],
    start: (usize, usize),
) -> HashSet<(usize, usize)> {
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { 0 };

    // Locate the finish cell — first `'F'` in row-major order (mazes have at
    // most one, enforced by `data_model` validation).
    let mut finish: Option<(usize, usize)> = None;
    'find_finish: for (r, row) in grid.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == 'F' {
                finish = Some((r, c));
                break 'find_finish;
            }
        }
    }
    let Some(finish) = finish else {
        return HashSet::new();
    };

    // Lock-blind BFS from `start`, recording each cell's parent so the path can
    // be reconstructed once `finish` is dequeued.
    let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some((r, c)) = queue.pop_front() {
        if (r, c) == finish {
            // Walk back along `parent`, collecting any door cells on the path.
            let mut path_doors: HashSet<(usize, usize)> = HashSet::new();
            let mut cur = (r, c);
            loop {
                if grid[cur.0][cur.1] == 'D' {
                    path_doors.insert(cur);
                }
                if cur == start {
                    return path_doors;
                }
                cur = parent[&cur];
            }
        }

        // 4-neighbour expansion — same order as the Lee solver (Up, Left, Down,
        // Right) for parity, though the path doors set is order-agnostic.
        let mut neighbours: Vec<(usize, usize)> = Vec::with_capacity(4);
        if r > 0 {
            neighbours.push((r - 1, c));
        }
        if c > 0 {
            neighbours.push((r, c - 1));
        }
        if r + 1 < rows {
            neighbours.push((r + 1, c));
        }
        if c + 1 < cols {
            neighbours.push((r, c + 1));
        }
        for (nr, nc) in neighbours {
            if grid[nr][nc] == 'W' {
                continue;
            }
            if visited.insert((nr, nc)) {
                parent.insert((nr, nc), (r, c));
                queue.push_back((nr, nc));
            }
        }
    }

    HashSet::new()
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

    // ── lose state — initial & timeout ───────────────────────────────────────────

    #[test]
    fn new_game_is_neither_won_nor_lost() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let game = MazeGame::from_json(json).unwrap();
        assert!(!game.is_complete());
        assert!(!game.is_lost());
        assert_eq!(game.lose_reason(), None);
    }

    #[test]
    fn time_out_marks_game_lost_with_timeout_reason() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.time_out();
        assert!(game.is_lost());
        assert_eq!(game.lose_reason(), Some(LoseReason::Timeout));
    }

    #[test]
    fn time_out_is_idempotent() {
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.time_out();
        game.time_out();
        assert_eq!(game.lose_reason(), Some(LoseReason::Timeout));
    }

    #[test]
    fn time_out_after_stranded_preserves_stranded_reason() {
        // Same layout as `decoy_door_with_only_one_key_strands_on_walk_through`
        // — strand the player, then fire time_out; the original Stranded
        // reason must survive (a timer race must not erase what really lost
        // the game).
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","D","F"],
            ["W","D","W","W"],
            ["W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.pickup();
        game.move_player(Direction::Down);
        game.tick(1000.0);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        game.time_out();
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
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
    fn no_doors_means_no_strand_ever() {
        // A maze with no doors has an empty solution_path_doors set →
        // path_doors_remaining_closed starts at 0, can never exceed
        // available_keys, no walk-through-D events.
        let json = r#"{"grid":[["S"," ","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        game.move_player(Direction::Right);
        assert!(!game.is_lost());
        assert_eq!(game.lose_reason(), None);
    }

    #[test]
    fn time_out_after_complete_still_marks_lost() {
        // Documented contract: complete and lost are mutually exclusive in
        // practice, but time_out from the caller is unconditional — once the
        // countdown fires, the caller may not yet know the player just
        // finished. We accept the call and mark lost = true. UIs gate this
        // by checking is_complete first.
        let json = r#"{"grid":[["S","F"]]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right);
        assert!(game.is_complete());
        game.time_out();
        // The lose state is set, but is_complete still reflects the win.
        assert!(game.is_lost());
        assert!(game.is_complete());
    }
}
