use crate::StoreError;
use maze::Maze;

pub struct MazeDetails {
    pub id: String,
    pub name: String,
}

pub trait Store {
    fn generate_maze_id(&self, name: &str) -> String; // TO DO - REMOVE
    fn create_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    fn delete_maze(&self, id: &str) -> Result<(), StoreError>;
    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    fn get_maze(&self, id: &str) -> Result<Maze, StoreError>;
    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeDetails, StoreError>;
    fn get_maze_items(&self) -> Result<Vec<MazeDetails>, StoreError>;
}
