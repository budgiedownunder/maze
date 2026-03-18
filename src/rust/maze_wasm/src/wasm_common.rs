use data_model::{Maze, MazeDefinition};
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

impl Clone for MazeWasm {
    fn clone(&self) -> Self {
        MazeWasm {
            maze: self.maze.clone(),
        }
    }
}

#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
pub enum MazeCellTypeWasm {
    Empty,
    Start,
    Finish,
    Wall,
}

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
    /// Two-phase recursive backtracking — see [`maze::GenerationAlgorithm::RecursiveBacktracking`].
    RecursiveBacktracking = 0,
}

/// Identifies the maze generation algorithm to use.
///
/// Passed as an argument to `MazeWasm::generate`.
#[cfg(not(feature = "wasm-bindgen"))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum GenerationAlgorithmWasm {
    /// Two-phase recursive backtracking — see `maze::GenerationAlgorithm::RecursiveBacktracking`.
    RecursiveBacktracking = 0,
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

