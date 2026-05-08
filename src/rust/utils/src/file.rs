use std::fs;
use std::path::{Path, PathBuf};

/// Checks whether a file exists
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use utils::file::file_exists;
///
/// let temp = tempfile::tempdir().unwrap();
/// let path = temp.path().join("hello.txt");
/// std::fs::File::create(&path).unwrap().write_all(b"hi").unwrap();
///
/// assert!(file_exists(path.to_str().unwrap()));
/// assert!(!file_exists(temp.path().join("missing.txt").to_str().unwrap()));
/// ```
pub fn file_exists(file_path: &str) -> bool {
    let path = PathBuf::from(file_path);
    Path::new(&path).is_file()
}

/// Deletes a file
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use utils::file::{delete_file, file_exists};
///
/// let temp = tempfile::tempdir().unwrap();
/// let path = temp.path().join("scratch.txt");
/// std::fs::File::create(&path).unwrap().write_all(b"x").unwrap();
/// assert!(file_exists(path.to_str().unwrap()));
///
/// delete_file(path.to_str().unwrap());
/// assert!(!file_exists(path.to_str().unwrap()));
/// ```
pub fn delete_file(file: &str) {
    let _ = fs::remove_file(file);
}

/// Deletes all files in a given directory with a given extension
///
/// # Examples
///
/// Drop every `.tmp` file in a directory while leaving other extensions
/// untouched
/// ```
/// use std::io::Write;
/// use utils::file::{delete_files_with_ext, file_exists};
///
/// let temp = tempfile::tempdir().unwrap();
/// for name in ["a.tmp", "b.tmp", "keep.txt"] {
///     std::fs::File::create(temp.path().join(name)).unwrap().write_all(b"x").unwrap();
/// }
///
/// delete_files_with_ext(temp.path().to_str().unwrap(), "tmp")
///     .expect("delete tmp files");
///
/// assert!(!file_exists(temp.path().join("a.tmp").to_str().unwrap()));
/// assert!(!file_exists(temp.path().join("b.tmp").to_str().unwrap()));
/// assert!(file_exists(temp.path().join("keep.txt").to_str().unwrap()));
/// ```
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
///
/// # Examples
///
/// ```
/// use utils::file::dir_exists;
///
/// let temp = tempfile::tempdir().unwrap();
/// assert!(dir_exists(temp.path().to_str().unwrap()));
/// assert!(!dir_exists(temp.path().join("nope").to_str().unwrap()));
/// ```
pub fn dir_exists(dir_path: &str) -> bool {
    let path = PathBuf::from(dir_path);
    Path::new(&path).is_dir()
}

/// Deletes a directory
///
/// # Examples
///
/// ```
/// use utils::file::{delete_dir, dir_exists};
///
/// let temp = tempfile::tempdir().unwrap();
/// let nested = temp.path().join("subdir");
/// std::fs::create_dir_all(&nested).unwrap();
/// assert!(dir_exists(nested.to_str().unwrap()));
///
/// delete_dir(nested.to_str().unwrap());
/// assert!(!dir_exists(nested.to_str().unwrap()));
/// ```
pub fn delete_dir(dir: &str) {
    let _ = fs::remove_dir_all(dir);
}
