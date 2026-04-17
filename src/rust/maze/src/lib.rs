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

// Re-export traits and structs
pub use error::Error;
pub use game::{Direction, MazeGame, MoveResult};
#[cfg(feature = "generation")]
pub use generation_algorithm::GenerationAlgorithm;
#[cfg(feature = "generation")]
pub use generator::{Generator, GeneratorOptions};
pub use maze::{MazePrinter, MazeSolver};
pub use maze_path::MazePath;
pub use maze_path_direction::MazePathDirection;
pub use maze_point_offset::MazePointOffset;
pub use maze_solution::MazeSolution;
pub use solver::Solver;
