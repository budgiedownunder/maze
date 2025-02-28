use std::fs;
use std::path::{Path, PathBuf};

/// Deletes a file
pub fn delete_file(file: &str) {
    let _ = fs::remove_file(file);
}

/// Deletes all files in a given directory with a given extension
pub fn delete_files_with_ext(dir: &str, extension: &str) -> std::io::Result<()> {
    let files = fs::read_dir(dir)?;
    for file in files {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    delete_file(&path.to_string_lossy());
                }
            }
        }
    }
    Ok(())
}

/// Checks whether a directory exists
pub fn dir_exists(dir_path: &str) -> bool {
    let path = PathBuf::from(dir_path);
    Path::new(&path).is_dir()
}

/// Checks whether a file exists
pub fn file_exists(file_path: &str) -> bool {
    let path = PathBuf::from(file_path);
    Path::new(&path).is_file()
}
