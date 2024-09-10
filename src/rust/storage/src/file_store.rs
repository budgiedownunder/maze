use std::path::{Path as stdPath, PathBuf};

use maze::Maze;

use crate::MazeDetails;
use crate::Store;
use crate::StoreError;

pub struct FileStore {}

impl FileStore {
    pub fn new() -> Self {
        FileStore {}
    }
}

impl Store for FileStore {
    fn generate_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    fn create_maze(&self, _maze: &mut Maze) -> Result<(), StoreError> {
        Err(StoreError::Other(
            "create_maze() not implemented for FileStore".to_string(),
        ))
    }

    fn delete_maze(&self, id: &str) -> Result<(), StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        Err(StoreError::NotFound(maze.id.clone()))
    }

    fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeDetails, StoreError> {
        let file_id = self.generate_maze_id(name);
        let path = PathBuf::from(file_id.clone());

        if !name.is_empty() && stdPath::new(&path).exists() {
            return Ok(MazeDetails {
                id: file_id,
                name: name.to_string(),
            });
        }
        Err(StoreError::NotFound(name.to_string()))
    }

    fn get_maze_items(&self) -> Result<Vec<MazeDetails>, StoreError> {
        Ok(vec![])
    }
}
