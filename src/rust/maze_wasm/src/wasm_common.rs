use maze::{Maze, Definition};
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

//************************************************************************************************************
// Currently, we have to have duplicated definitions of MazeWasm for wasm-bindgen and wasm32 builds, due to 
// the fact that we cannot conditionally mark the maze field with #[wasm_bindgen(skip)] 
//- see https://github.com/anza-xyz/agave/pull/1658 for details on this issue
//************************************************************************************************************/

#[cfg(feature = "wasm-bindgen")]
#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen)]
/// Web assembly representation of a maze
pub struct MazeWasm {
    //#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen(skip))] - does not work
    #[wasm_bindgen(skip)]
    pub maze: Maze,
}

#[cfg(not(feature = "wasm-bindgen"))]
#[cfg_attr(not(feature = "wasm-bindgen"), repr(C))]
/// Web assembly representation of a maze
pub struct MazeWasm {
    //#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen(skip))] - does not work
    pub maze: Maze,
}

impl Clone for MazeWasm {
    fn clone(&self) -> Self {
        MazeWasm {
            maze: self.maze.clone(),
        }
    }
}

#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen)]
#[cfg_attr(not(feature = "wasm-bindgen"), repr(C))]
/// Web assembly enum for a maze cell type
pub enum MazeCellTypeWasm {
    Empty,
    Start,
    Finish,
    Wall,
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
    let def = Definition::new(0, 0);
    Maze::new(def)
}

