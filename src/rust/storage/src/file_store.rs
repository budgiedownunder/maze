use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path as StdPath, PathBuf};

use maze::Maze;
use utils::file::{delete_file};

use crate::MazeItem;
use crate::Store;
use crate::StoreError;

pub struct FileStore {}

impl FileStore {
    pub fn new() -> Self {
        FileStore {}
    }

    fn make_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    fn maze_exists(&self, path: &str) -> bool {
        let path = PathBuf::from(path);
        StdPath::new(&path).exists()
    }

    fn save_maze_to_file(
        &self,
        maze: &mut Maze,
        path: &str,
        overwrite: bool,
    ) -> Result<(), StoreError> {
        if !overwrite {
            let os_path = PathBuf::from(path);
            if StdPath::new(&os_path).exists() {
                return Err(StoreError::IdAlreadyExists(path.to_string()));
            }
        }
        let s = maze.to_json()?;
        let mut file = File::create(path)?;
        file.write_all(s.as_bytes())?;
        maze.id = path.to_string();
        Ok(())
    }
}

impl Store for FileStore {
    fn create_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        if maze.name.is_empty() {
            return Err(StoreError::NameMissing());
        }
        let file_id = self.make_maze_id(&maze.name);
        if self.maze_exists(&file_id) {
            return Err(StoreError::IdAlreadyExists(file_id.to_string()));
        }
        self.save_maze_to_file(maze, &file_id, false)?;
        Ok(())
    }

    fn delete_maze(&self, id: &str) -> Result<(), StoreError> {
        if id.is_empty() {
            return Err(StoreError::IdMissing());
        }
        delete_file(id);
        Ok(())
    }

    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        if maze.id.is_empty() {
            return Err(StoreError::IdMissing());
        }
        if !self.maze_exists(&maze.id) {
            return Err(StoreError::IdNotFound(maze.id.to_string()));
        }
        self.save_maze_to_file(maze, &maze.id.clone(), true)?;
        Ok(())
    }

    fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
        if !self.maze_exists(id) {
            return Err(StoreError::IdNotFound(id.to_string()));
        }
        let file = File::open(id)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, Maze>(reader) {
            Ok(mut maze) => {
                maze.id = id.to_string();
                Ok(maze)
            }
            Err(error) => Err(StoreError::from(error)),
        }
    }

    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeItem, StoreError> {
        let file_id = self.make_maze_id(name);
        let path = PathBuf::from(file_id.clone());

        if !name.is_empty() && StdPath::new(&path).exists() {
            return Ok(MazeItem {
                id: file_id,
                name: name.to_string(),
            });
        }
        Err(StoreError::NameNotFound(name.to_string()))
    }

    fn get_maze_items(&self) -> Result<Vec<MazeItem>, StoreError> {
        let mut items: Vec<MazeItem> = Vec::new();
        let current_dir = std::env::current_dir()?;

        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(path_str) = path.to_str() {
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Some(name) = path.file_stem() {
                            if let Some(name_str) = name.to_str() {
                                items.push(MazeItem {
                                    id: path_str.to_string(),
                                    name: name_str.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    //use crate::file_store::FileStore;
    use super::*;
    use utils::file::{delete_files_with_ext};

    #[test]
    fn can_save_maze_to_valid_file_path() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => panic!("Failed to save to file: {}", error),
        }
    }

    #[test]
    fn cannot_save_maze_to_invalid_file_path() {
        let store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = "";

        match store.save_maze_to_file(&mut maze, path, true) {
            Ok(_) => panic!("Successfully saved to file: {} but did not expect to", path),
            Err(error) => assert_io_err_not_found(error),
        }
    }

    #[test]
    #[should_panic(expected = "Item with id './maze.json' already exists")]
    fn cannot_save_maze_to_existing_file_path_if_overwrite_disabled() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.save_maze_to_file(&mut maze, &path, false) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully saved to existing file: {} despite overwrite being false",
                    path
                );
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }
    #[test]
    fn can_save_maze_to_existing_file_path_if_overwrite_enabled() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_create_maze_that_does_not_exist() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);

        delete_file(&path);

        match store.create_maze(&mut maze) {
            Ok(_) => delete_file(&path),
            Err(error) => panic!("Failed to create maze: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "No name provided")]
    fn cannot_create_maze_with_empty_name() {
        let store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", false, false);
        match store.create_maze(&mut maze) {
            Ok(_) => panic!("Successfully saved unnamed maze but did not expect to"),
            Err(error) => panic!("{}", error),
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'maze.json' already exists")]
    fn cannot_create_maze_that_exists() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.create_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully created maze when file: {} existed, when should not have",
                    path
                );
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_update_existing_maze() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.update_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'maze.json' not found")]
    fn cannot_update_non_existant_maze() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.update_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully updated maze when file: {} did not exist",
                    path
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "No id provided")]
    fn cannot_update_maze_with_no_id() {
        let store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", false, true);

        match store.update_maze(&mut maze) {
            Ok(_) => {
                panic!("Successfully updated maze when maze had no id");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_delete_maze() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                if store.maze_exists(&maze.id) {
                    match store.delete_maze(&maze.id) {
                        Ok(_) => {
                            if store.maze_exists(&maze.id) {
                                panic!("Maze file {} still exists after maze delete", maze.id);
                            }
                        }
                        Err(error) => panic!("Failed to delete maze: {}", error),
                    }
                } else {
                    panic!("Saved maze file not found prior to maze delete test");
                }
            }
            Err(error) => panic!("Failed to save maze to file: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "No id provided")]
    fn cannot_delete_maze_with_empty_id() {
        let store = FileStore::new();

        match store.delete_maze("") {
            Ok(()) => {
                panic!("delete_maze() suceeded for blank id");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_get_maze_that_exists() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.create_maze(&mut maze) {
            Ok(_) => match store.get_maze(&maze.id) {
                Ok(maze_loaded) => {
                    delete_file(&path);
                    if maze_loaded != maze {
                        panic!("Loaded maze content is different to maze content saved");
                    }
                }
                Err(error) => {
                    delete_file(&path);
                    panic!("Failed to load saved maze: {}", error);
                }
            },
            Err(error) => {
                panic!("Failed to create maze: {}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'missing.json' not found")]
    fn cannot_get_maze_that_does_not_exist() {
        let store = FileStore::new();
        let path = "./missing.json";

        delete_file(path);

        match store.get_maze("missing.json") {
            Ok(_) => {
                panic!("Succesfully loaded maze content when file is missing");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn maze_item_list_should_be_empty() {
        let store = FileStore::new();

        let _ = delete_files_with_ext(".", "json");

        match store.get_maze_items() {
            Ok(items) => {
                if !items.is_empty() {
                    panic!("Maze item list is not empty ({} items found)", items.len());
                }
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn maze_item_list_should_not_be_empty() {
        let store = FileStore::new();

        let _ = delete_files_with_ext(".", "json");

        let (_, mut maze_1) = init_test_maze(&store, "maze_1", false, true);
        match store.create_maze(&mut maze_1) {
            Ok(_) => {}
            Err(error) => panic!("Failed to create maze {}: {}", maze_1.name, error),
        }

        let (_, mut maze_2) = init_test_maze(&store, "maze_2", false, true);
        match store.create_maze(&mut maze_2) {
            Ok(_) => {}
            Err(error) => panic!("Failed to create maze {}: {}", maze_2.name, error),
        }

        match store.get_maze_items() {
            Ok(items) => {
                if items.len() != 2 {
                    panic!("Maze item list does not contain the expected number of items (2 expected, {} found)", items.len());
                }
                check_maze_item(&items, 0, "maze_1");
                check_maze_item(&items, 1, "maze_2");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }

        fn check_maze_item(items: &[MazeItem], idx: usize, expected: &str) {
            if items[idx].name != expected {
                panic!(
                    "Item at index {} contains unexpected value (expected = {}, found: {})",
                    idx, expected, items[idx].name
                );
            }
        }
    }

    fn init_test_maze(
        store: &FileStore,
        name: &str,
        set_id: bool,
        set_name: bool,
    ) -> (String, Maze) {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'], 
            vec!['F', ' ', 'W']
        ];
        let mut maze = Maze::from_vec(grid);
        if set_name {
            maze.name = name.to_string();
        }
        if set_id {
            maze.id = store.make_maze_id(name);
        }
        let path = format!("./{}.json", name);
        (path, maze)
    }

    fn assert_io_err_not_found(error: StoreError) {
        match error {
            StoreError::Io(io_error) => match io_error.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!("io::ErrorKind::NotFound error expected (got: {})", io_error),
            },
            _ => panic!("io::ErrorKind::NotFound error expected (got: {})", error),
        }
    }
}
