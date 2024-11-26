use crate::StoreError;
use maze::Maze;
use serde::Serialize;
use utoipa::ToSchema;

/// Contains the identifying details for a maze item
#[derive(Serialize, ToSchema)]
pub struct MazeItem {
    pub id: String,
    pub name: String,
}

/// Represents a store for holding mazes and related objects
pub trait Store {
    /// Adds a new maze to the store and sets the allocated `id` within the maze object
    fn create_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    /// Deletes a maze from the store
    fn delete_maze(&self, id: &str) -> Result<(), StoreError>;
    /// Updates a maze within the store
    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    /// Loads a maze from the store
    fn get_maze(&self, id: &str) -> Result<Maze, StoreError>;
    /// Locates a maze item by its name within the store
    fn find_maze_by_name(&self, name: &str) -> Result<MazeItem, StoreError>;
    /// Returns the list of maze items within the store, sorted
    /// alphabetically in ascending order
    fn get_maze_items(&self) -> Result<Vec<MazeItem>, StoreError>;
}
