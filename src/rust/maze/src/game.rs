use data_model::MazeDefinition;
use serde::{Deserialize, Serialize};

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
}

/// A running maze game session.
///
/// Holds the grid, player position, facing direction, completion state, and the
/// set of visited cells in visit order. Create with [`MazeGame::from_json`].
///
/// Cell rules applied during [`MazeGame::move_player`]:
/// - `' '` or `'S'` → [`MoveResult::Moved`]
/// - `'F'` → [`MoveResult::Complete`]
/// - `'W'` or out-of-bounds → [`MoveResult::Blocked`]
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
}

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

        Ok(MazeGame {
            grid: definition.grid,
            player_row: start.row,
            player_col: start.col,
            direction: Direction::None,
            complete: false,
            visited,
            rows,
            cols,
        })
    }

    /// Attempts to move the player one cell in `dir`.
    ///
    /// Returns [`MoveResult::Blocked`] if the target cell is a wall or out of
    /// bounds, [`MoveResult::Complete`] if the player reaches the finish cell,
    /// and [`MoveResult::Moved`] otherwise. The player's facing direction is
    /// always updated to `dir`, even when blocked.
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
            ' ' | 'S' => {
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
    /// Each character is one of `'S'` (start), `'F'` (finish), `'W'` (wall), or `' '` (open).
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
}
