// Re-export modules
mod error;
mod game;
#[cfg(feature = "generation")]
mod generation_algorithm;
#[cfg(feature = "generation")]
mod generator;
mod maze;
mod maze_path;
mod maze_path_direction;
mod maze_solution;
mod maze_point_offset;
mod solver;
mod topology;

// Re-export traits and structs
pub use data_model::{CellEntity, EnemyType, TreasureStyle};
pub use error::Error;
pub use game::{
    BagItem, Direction, DoorState, Enemy, GameEvent, LoseReason, MazeGame, MazeGameOptions,
    MoveResult, PlayerNotHealedReason,
};
#[cfg(feature = "generation")]
pub use generation_algorithm::GenerationAlgorithm;
#[cfg(feature = "generation")]
pub use generator::{Generator, GeneratorOptions, MAX_AUTO_DOORS};
pub use maze::{MazePrinter, MazeSolver};
pub use maze_path::MazePath;
pub use maze_path_direction::MazePathDirection;
pub use maze_point_offset::MazePointOffset;
pub use maze_solution::MazeSolution;
pub use solver::Solver;
pub use topology::is_dead_end;

/// Maximum combined count of `'K'` + `'D'` cells a maze may carry. The
/// key-aware solver tracks each as a bit in a `u32` mask, so its search
/// is exponential in their sum; above this cap the solver refuses rather
/// than degrade to a key-blind walk that would misrepresent sealed mazes
/// as playable. Generation, the React Generate dialog, the React editor
/// save flow, and the server save endpoint all enforce the same cap so
/// the solver's error path never fires for a maze produced through the
/// supported tools.
pub const MAX_TOTAL_FEATURES: usize = 16;
