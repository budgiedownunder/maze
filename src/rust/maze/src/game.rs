use data_model::MazeDefinition;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Combined `'K'` + `'D'` cell count above which the strand check's
/// state-space BFS falls back to lock-blind key reachability.
///
/// The BFS explores `(cell, collected_keys_bitmask, opened_doors_bitmask)`
/// states; its size is bounded by `cells * 2^(K+D)`, so we cap K+D at the
/// same width the `u32` masks afford and the solver itself uses. Above
/// the cap, the fallback over-counts keys (treats every door as
/// passable), which is safe for the strand inequality — over-counting
/// keys only ever delays a strand, never invents one.
const MAX_GATED_FEATURES: usize = 16;

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
    /// The player moved successfully through an open door (`'D'`) and the
    /// number of keys they can still hold (`bag.len()` + reachable
    /// uncollected `'K'` cells) is now less than the number of closed
    /// doors on every remaining route from the current cell to the
    /// finish — the game is unwinnable. [`MazeGame::lose_reason`]
    /// returns `Some(LoseReason::Stranded)` and [`MazeGame::is_lost`]
    /// returns `true`.
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
///   [`MoveResult::Stranded`] when walking through leaves the player with
///   fewer reachable keys than closed doors remaining on any route to the
///   finish), else [`MoveResult::StartedUnlocking`] (a key is held) or
///   [`MoveResult::BlockedByLockedDoor`]
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
    /// door cell while the player's reachable-key count is below the number
    /// of closed doors remaining on any route to the finish yields
    /// [`MoveResult::Stranded`] — the move still succeeds, but the game
    /// transitions to lost with [`LoseReason::Stranded`]. The player's
    /// facing direction is always updated to `dir`, even when blocked.
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
                    // Walked through an open door — the trigger point for
                    // stranded detection. Compare the minimum closed doors
                    // on any route from the player's current cell to F
                    // against the keys they can still hold (bag + keys
                    // reachable from the current world state). The
                    // `closed > bag_keys` guard skips the state-space BFS
                    // whenever the bag alone already covers the remaining
                    // closed doors — the common play state.
                    let closed = self.closed_doors_to_finish();
                    let bag_keys = self.bag.len() as u32;
                    if !self.lost
                        && closed > bag_keys
                        && closed > self.simulate_reachable_keys()
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
                        // Both halves of the strand inequality are recomputed
                        // on demand at walk-through time — see
                        // [`Self::closed_doors_to_finish`] and
                        // [`Self::simulate_reachable_keys`] — so no per-key
                        // bookkeeping is needed when a door commits.
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

    /// Minimum number of currently-`Locked` `'D'` cells on any path from the
    /// player's current cell to the finish. Lock-blind 0-1 BFS: entering a
    /// `Locked` door costs 1, every other passable step costs 0 (walls
    /// block; `Open` and `Opening` doors are passable for free since they're
    /// already committed). Returns `u32::MAX` if the finish is unreachable
    /// (defensive — the maze pipeline rejects unsolvable mazes).
    ///
    /// Computed on demand at each walk-through-D strand check: opening a
    /// door anywhere can create a shortcut whose route to F crosses
    /// fewer closed doors than the previous best, and the count needs
    /// to reflect the world state at the moment of the check.
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

    /// How many keys the player could still hold from the current
    /// state — `bag.len()` plus the largest number of uncollected `'K'`
    /// cells reachable on any play sequence. Used by
    /// [`Self::move_player`]'s walk-through-door strand check.
    ///
    /// State-space BFS from the player's current cell over
    /// `(cell, collected_keys_mask, opened_doors_mask)`. A neighbour is
    /// reachable if it's passable; a still-`Locked` `'D'` is passable
    /// only when the player has an unspent key in hand
    /// (`bag.len() + collected.count_ones() > opened.count_ones()`), at
    /// which point the door's bit is set in `opened` and the key is
    /// virtually spent. Walking onto a `'K'` sets the key's bit in
    /// `collected`. Already-`Open` and `Opening` doors aren't indexed —
    /// they're passable for free (the key has already been committed).
    /// Returns `bag.len() + max(collected.count_ones())` over all
    /// reachable states.
    ///
    /// Falls back to [`Self::lock_blind_reachable_keys`] when
    /// `#K + #D > MAX_GATED_FEATURES` — the state space is exponential
    /// in their sum. The fallback over-counts reachable keys (treats
    /// every door as passable), which is safe for the strand inequality:
    /// over-counting keys only ever delays a strand, never invents one.
    fn simulate_reachable_keys(&self) -> u32 {
        let mut key_bit: HashMap<(usize, usize), u32> = HashMap::new();
        let mut door_bit: HashMap<(usize, usize), u32> = HashMap::new();
        for (r, row) in self.grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                match ch {
                    'K' => {
                        let idx = key_bit.len() as u32;
                        key_bit.insert((r, c), idx);
                    }
                    'D' if matches!(self.doors.get(&(r, c)), Some(DoorState::Locked)) => {
                        let idx = door_bit.len() as u32;
                        door_bit.insert((r, c), idx);
                    }
                    _ => {}
                }
            }
        }

        if key_bit.len() + door_bit.len() > MAX_GATED_FEATURES {
            return self.bag.len() as u32 + self.lock_blind_reachable_keys();
        }

        let start = ((self.player_row, self.player_col), 0u32, 0u32);
        let mut visited: HashSet<((usize, usize), u32, u32)> = HashSet::new();
        let mut queue: VecDeque<((usize, usize), u32, u32)> = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        let mut max_collected: u32 = 0;
        let bag_keys = self.bag.len() as u32;
        while let Some(((r, c), collected, opened)) = queue.pop_front() {
            max_collected = max_collected.max(collected.count_ones());
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
                let (next_collected, next_opened) =
                    if let Some(&idx) = door_bit.get(&(nr, nc)) {
                        let bit = 1u32 << idx;
                        if opened & bit == 0 {
                            // Spending a key — need one in hand.
                            if bag_keys + collected.count_ones() <= opened.count_ones() {
                                continue;
                            }
                            (collected, opened | bit)
                        } else {
                            (collected, opened)
                        }
                    } else if let Some(&idx) = key_bit.get(&(nr, nc)) {
                        (collected | (1u32 << idx), opened)
                    } else {
                        (collected, opened)
                    };
                let next = ((nr, nc), next_collected, next_opened);
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }

        bag_keys + max_collected
    }

    /// Lock-blind fallback for [`Self::simulate_reachable_keys`] when
    /// `#K + #D > MAX_GATED_FEATURES`: count the `'K'` cells reachable
    /// from the player's current cell treating every door as passable.
    /// An upper bound on the true reachable-keys count, which is safe
    /// for the strand inequality.
    fn lock_blind_reachable_keys(&self) -> u32 {
        let start = (self.player_row, self.player_col);
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        let mut count: u32 = 0;
        while let Some((r, c)) = queue.pop_front() {
            if self.grid[r][c] == 'K' {
                count += 1;
            }
            for (nr, nc) in passable_neighbours(r, c, &self.grid, self.rows, self.cols) {
                if visited.insert((nr, nc)) {
                    queue.push_back((nr, nc));
                }
            }
        }
        count
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

    // ── stranded detection — keys-side multi-path reachability ───────────────────

    #[test]
    fn does_not_strand_when_key_becomes_reachable_via_an_off_tracked_path() {
        // Keys-side analogue of the doors-side multi-path bug fixed in
        // 7E.5. The off-spine key K1(3,0) has TWO lock-blind paths back to
        // the spine:
        //   - via Dx(2,0) — cost 1 (single off-spine door; the tracked path)
        //   - via Dz(2,2) + Dy(1,2) — cost 2 (NOT tracked in `key_min_paths`)
        //
        // After the player burns K0 + K_s on the Dy and Dz decoys, K1 is
        // freely reachable via (2,2) → (3,2) → (3,1) → (3,0) — no closed
        // doors on that route. But the static `key_min_paths` still lists
        // [{Dx}] for K1 (Dx is still Locked, and neither Dy nor Dz was on
        // the tracked path so opening them didn't drop the cost), so the
        // greedy `simulate_reachable_keys` thinks K1 costs 1 key. With
        // budget 0 it can't afford it, returns 0, and the inequality
        // `closed_doors_to_finish(=1, just D_p) > available(=0)` falsely
        // strands the player.
        //
        //         c0   c1   c2   c3   c4
        //  r0:    S    K0   K_s  D_p  F
        //  r1:   ' '   W    Dy   W    W
        //  r2:    Dx   W    Dz   W    W
        //  r3:    K1  ' '  ' '   W    W
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","K","D","F"],
            [" ","W","D","W","W"],
            ["D","W","D","W","W"],
            ["K"," "," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        // Pick up K0 + K_s (bag = 2).
        game.move_player(Direction::Right); // (0,1) K0
        game.pickup();
        game.move_player(Direction::Right); // (0,2) K_s
        game.pickup();
        assert_eq!(game.bag().len(), 2);
        // Burn one key opening Dy(1,2), walk through.
        game.move_player(Direction::Down); // StartedUnlocking Dy
        game.tick(1000.0);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Moved);
        assert!(!game.is_lost());
        // Burn the other key opening Dz(2,2). This is the bug-trigger
        // walk-through: K1 is now freely reachable via the off-tracked
        // route, so the player has 1 future key for the 1 remaining
        // closed door (D_p) — strand must NOT fire.
        game.move_player(Direction::Down); // StartedUnlocking Dz
        game.tick(1000.0);
        let result = game.move_player(Direction::Down);
        assert_eq!(
            result,
            MoveResult::Moved,
            "K1 is freely reachable via (2,2)→(3,2)→(3,1)→(3,0); \
             the strand check must not fire on this walk-through (got {result:?})"
        );
        assert!(
            !game.is_lost(),
            "the player has 1 future key (K1) for the 1 remaining closed door (D_p)"
        );
    }

    #[test]
    fn strand_still_fires_when_only_tracked_path_to_key_is_walled_off() {
        // Negative control for the multi-path keys-side fix: same shape
        // as the multi-path-bug test, but with (3,1) replaced by W. K1
        // now has only ONE lock-blind path back to the spine (via Dx).
        // After the player burns K0 + K_s on the Dy/Dz decoys, K1 is
        // genuinely unreachable (Dx still Locked, bag empty, the alt
        // route (3,2)→(3,1) is now walled off), so the strand check
        // must still fire. Verifies the keys-side BFS doesn't
        // over-correct away from real strands.
        //
        //         c0   c1   c2   c3   c4
        //  r0:    S    K0   K_s  D_p  F
        //  r1:   ' '   W    Dy   W    W
        //  r2:    Dx   W    Dz   W    W
        //  r3:    K1   W   ' '   W    W      <- (3,1) is W; alt route closed
        #[rustfmt::skip]
        let json = r#"{"grid":[
            ["S","K","K","D","F"],
            [" ","W","D","W","W"],
            ["D","W","D","W","W"],
            ["K","W"," ","W","W"]
        ]}"#;
        let mut game = MazeGame::from_json(json).unwrap();
        game.move_player(Direction::Right); // (0,1) K0
        game.pickup();
        game.move_player(Direction::Right); // (0,2) K_s
        game.pickup();
        game.move_player(Direction::Down); // StartedUnlocking Dy
        game.tick(1000.0);
        assert_eq!(game.move_player(Direction::Down), MoveResult::Moved);
        assert!(!game.is_lost());
        game.move_player(Direction::Down); // StartedUnlocking Dz
        game.tick(1000.0);
        // K1 is genuinely unreachable now: (3,1)=W cuts the alt route,
        // and Dx is still Locked with no keys to open it.
        assert_eq!(game.move_player(Direction::Down), MoveResult::Stranded);
        assert!(game.is_lost());
        assert_eq!(game.lose_reason(), Some(LoseReason::Stranded));
    }
}
