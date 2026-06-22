// Re-export modules
mod error;
mod game;
#[cfg(feature = "generation")]
mod generation_algorithm;
#[cfg(feature = "generation")]
mod generator;
mod limits;
mod maze;
mod maze_path;
mod maze_path_direction;
mod maze_solution;
mod maze_point_offset;
mod solver;
mod topology;

// Re-export traits and structs
pub use data_model::{CellEntity, EnemyType, MazePoint, TreasureStyle};
pub use error::Error;
pub use game::{
    BagItem, Direction, DoorState, Enemy, GameEvent, LoseReason, MazeGame, MazeGameOptions,
    MoveResult, PlayerNotHealedReason,
};
#[cfg(feature = "generation")]
pub use generation_algorithm::GenerationAlgorithm;
#[cfg(feature = "generation")]
pub use generator::{Generator, GeneratorOptions, MAX_AUTO_DOORS};
pub use limits::{MAX_ENEMY_COUNT, MAX_HEALTH_COUNT, MAX_TOTAL_FEATURES, MAX_TREASURE_COUNT};
pub use maze::{MazePrinter, MazeSolver};
pub use maze_path::MazePath;
pub use maze_path_direction::MazePathDirection;
pub use maze_point_offset::MazePointOffset;
pub use maze_solution::MazeSolution;
pub use solver::Solver;
pub use topology::is_dead_end;
