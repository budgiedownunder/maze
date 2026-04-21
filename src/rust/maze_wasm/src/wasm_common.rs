use data_model::{Maze, MazeDefinition};
use maze::MazeGame;
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
}

#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
pub struct MazeGameWasm {
    pub game: MazeGame,
}

impl Clone for MazeWasm {
    fn clone(&self) -> Self {
        MazeWasm {
            maze: self.maze.clone(),
        }
    }
}

/// Identifies the type of a maze cell.
///
/// Returned by [`MazeWasm::get_cell_type`].
#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum MazeCellTypeWasm {
    Empty,
    Start,
    Finish,
    Wall,
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
    None     = 0,
    Moved    = 1,
    Blocked  = 2,
    Complete = 3,
}

/// Result returned by `maze_game_wasm_move_player`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum MoveResultWasm {
    None     = 0,
    Moved    = 1,
    Blocked  = 2,
    Complete = 3,
}

/// Converts a [`GenerationAlgorithmWasm`] value to the corresponding [`maze::GenerationAlgorithm`].
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
pub fn to_cell_type_enum(cell_type: char) -> MazeCellTypeWasm {
    match cell_type {
        'S' => MazeCellTypeWasm::Start,
        'F' => MazeCellTypeWasm::Finish,
        'W' => MazeCellTypeWasm::Wall,
        _ => MazeCellTypeWasm::Empty,
    }
}
/// Creates an empty maze
///
/// # Returns
///
/// `Maze`
///
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
pub fn new_maze_game(json: &str) -> Result<MazeGameWasm, String> {
    MazeGame::from_json(json).map(|game| MazeGameWasm { game })
}

