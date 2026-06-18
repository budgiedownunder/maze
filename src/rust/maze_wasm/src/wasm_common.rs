use data_model::{Maze, MazeDefinition};
use maze::{GameEvent, MazeGame};
#[cfg(any(feature = "wasm-bindgen", feature = "wasm-lite"))]
use maze::GenerationAlgorithm;
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

//************************************************************************************************************
// Currently, we have to have duplicated definitions of MazeWasm for wasm-bindgen and wasm32 builds, due to 
// the fact that we cannot conditionally mark the maze field with #[wasm_bindgen(skip)] 
//- see https://github.com/anza-xyz/agave/pull/1658 for details on this issue
//************************************************************************************************************/

#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub struct MazeWasm {
    #[wasm_bindgen(skip)]
    pub maze: Maze,
}
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
pub struct MazeWasm {
    pub maze: Maze,
}

#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub struct MazeGameWasm {
    #[wasm_bindgen(skip)]
    pub game: MazeGame,
    #[wasm_bindgen(skip)]
    pub tick_events: Vec<GameEvent>,
}

#[cfg(not(feature = "wasm-bindgen"))]
pub struct MazeGameWasm {
    pub game: MazeGame,
    pub tick_events: Vec<GameEvent>,
}

/// Identifies the type of a maze cell.
///
/// Returned by `maze_wasm_get_cell_type`.
#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum MazeCellTypeWasm {
    Empty,
    Start,
    Finish,
    Wall,
    Key,
    Door,
    Enemy,
    Health,
}

/// Identifies the type of a maze cell.
///
/// Returned by `maze_wasm_get_cell_type`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
pub enum MazeCellTypeWasm {
    Empty,
    Start,
    Finish,
    Wall,
    Key,
    Door,
    Enemy,
    Health,
}

/// Identifies the maze generation algorithm to use.
///
/// Passed as an argument to [`MazeWasm::generate`].
#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum GenerationAlgorithmWasm {
    /// Generates a perfect maze using a single-pass iterative depth-first search from the start cell.
    /// See [Randomized depth-first search](https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_depth-first_search).
    RecursiveBacktracking = 0,
}

/// Identifies the maze generation algorithm to use.
///
/// Passed as an argument to `MazeWasm::generate`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum GenerationAlgorithmWasm {
    /// Generates a perfect maze using a single-pass iterative depth-first search from the start cell.
    /// See [Randomized depth-first search](https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_depth-first_search).
    RecursiveBacktracking = 0,
}

/// Direction for player movement in a [`MazeGameWasm`] session.
///
/// Passed as an argument to [`MazeGameWasm::move_player`].
#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum DirectionWasm {
    None  = 0,
    Up    = 1,
    Down  = 2,
    Left  = 3,
    Right = 4,
}

/// Direction for player movement in a [`MazeGameWasm`] session.
///
/// Passed as an argument to `maze_game_wasm_move_player`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum DirectionWasm {
    None  = 0,
    Up    = 1,
    Down  = 2,
    Left  = 3,
    Right = 4,
}

/// Result returned by [`MazeGameWasm::move_player`].
#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum MoveResultWasm {
    None                = 0,
    Moved               = 1,
    Blocked             = 2,
    Complete            = 3,
    BlockedByLockedDoor = 4,
    StartedUnlocking    = 5,
    Stranded            = 6,
    Killed              = 7,
}

/// Result returned by `maze_game_wasm_move_player`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum MoveResultWasm {
    None                = 0,
    Moved               = 1,
    Blocked             = 2,
    Complete            = 3,
    BlockedByLockedDoor = 4,
    StartedUnlocking    = 5,
    Stranded            = 6,
    Killed              = 7,
}

/// Converts a [`GenerationAlgorithmWasm`] value to the corresponding [`maze::GenerationAlgorithm`].
///
/// # Examples
///
/// ```
/// use maze_wasm::wasm_common::{to_generation_algorithm, GenerationAlgorithmWasm};
/// use maze::GenerationAlgorithm;
///
/// let alg = to_generation_algorithm(GenerationAlgorithmWasm::RecursiveBacktracking);
/// assert!(matches!(alg, GenerationAlgorithm::RecursiveBacktracking));
/// ```
#[cfg(any(feature = "wasm-bindgen", feature = "wasm-lite"))]
pub fn to_generation_algorithm(alg: GenerationAlgorithmWasm) -> GenerationAlgorithm {
    match alg {
        GenerationAlgorithmWasm::RecursiveBacktracking => GenerationAlgorithm::RecursiveBacktracking,
    }
}

/// Converts a cell type character to a MazeCellTypeWasm value
///
/// # Returns
///
/// `MazeCellTypeWasm`
///
/// # Examples
///
/// ```
/// use maze_wasm::wasm_common::{to_cell_type_enum, MazeCellTypeWasm};
///
/// assert!(matches!(to_cell_type_enum('W'), MazeCellTypeWasm::Wall));
/// assert!(matches!(to_cell_type_enum(' '), MazeCellTypeWasm::Empty));
/// ```
pub fn to_cell_type_enum(cell_type: char) -> MazeCellTypeWasm {
    match cell_type {
        'S' => MazeCellTypeWasm::Start,
        'F' => MazeCellTypeWasm::Finish,
        'W' => MazeCellTypeWasm::Wall,
        'K' => MazeCellTypeWasm::Key,
        'D' => MazeCellTypeWasm::Door,
        'E' => MazeCellTypeWasm::Enemy,
        'H' => MazeCellTypeWasm::Health,
        _ => MazeCellTypeWasm::Empty,
    }
}
/// Creates an empty maze
///
/// # Returns
///
/// `Maze`
///
/// # Examples
///
/// ```
/// use maze_wasm::wasm_common::new_maze;
///
/// let maze = new_maze();
/// assert_eq!(maze.definition.row_count(), 0);
/// ```
pub fn new_maze() -> Maze {
    let def = MazeDefinition::new(0, 0);
    Maze::new(def)
}

/// Creates a [`MazeGameWasm`] from a maze definition JSON string.
///
/// # Returns
///
/// `Ok(MazeGameWasm)` on success, or `Err(String)` if the JSON is invalid or has no start cell.
///
/// # Examples
///
/// ```
/// use maze_wasm::wasm_common::new_maze_game;
///
/// let wrapper = new_maze_game(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// assert_eq!(wrapper.game.player_row(), 0);
/// assert!(new_maze_game("not json").is_err());
/// ```
pub fn new_maze_game(json: &str) -> Result<MazeGameWasm, String> {
    MazeGame::from_json(json).map(|game| MazeGameWasm {
        game,
        tick_events: Vec::new(),
    })
}

// `maze_wasm` is a `cdylib`, so the `# Examples` doc blocks above are not
// executed as doc tests (and the wasm-bindgen JS examples are validated
// separately by `tests/js/help_examples_tests.mjs`). These unit tests mirror
// the doc examples so their logic is still machine-verified by `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_cell_type_enum_maps_known_chars_and_defaults_to_empty() {
        assert!(matches!(to_cell_type_enum('W'), MazeCellTypeWasm::Wall));
        assert!(matches!(to_cell_type_enum('S'), MazeCellTypeWasm::Start));
        assert!(matches!(to_cell_type_enum(' '), MazeCellTypeWasm::Empty));
    }

    #[test]
    fn new_maze_is_empty() {
        assert_eq!(new_maze().definition.row_count(), 0);
    }

    #[test]
    fn new_maze_game_succeeds_on_valid_json_and_errors_otherwise() {
        let wrapper = new_maze_game(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
        assert_eq!(wrapper.game.player_row(), 0);
        assert!(new_maze_game("not json").is_err());
    }

    #[cfg(any(feature = "wasm-bindgen", feature = "wasm-lite"))]
    #[test]
    fn to_generation_algorithm_maps_the_variant() {
        assert!(matches!(
            to_generation_algorithm(GenerationAlgorithmWasm::RecursiveBacktracking),
            GenerationAlgorithm::RecursiveBacktracking
        ));
    }
}

