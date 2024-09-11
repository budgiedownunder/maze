use crate::StoreError;
use maze::Maze;

pub struct MazeItem {
    pub id: String,
    pub name: String,
}

pub trait Store {
    fn create_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    fn delete_maze(&self, id: &str) -> Result<(), StoreError>;
    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError>;
    fn get_maze(&self, id: &str) -> Result<Maze, StoreError>;
    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeItem, StoreError>;
    fn get_maze_items(&self) -> Result<Vec<MazeItem>, StoreError>;
}
