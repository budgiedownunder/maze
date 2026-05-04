use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CommsError;
use crate::template::TemplateLoader;

/// Loader backed by a directory of `.toml` template files.
///
/// Template name is the file stem: `password_reset.toml` is loaded under the
/// name `"password_reset"`. Sub-directories are not walked — keep templates
/// flat in one directory.
#[derive(Debug, Clone)]
pub struct FsTemplateLoader {
    root: PathBuf,
}

impl FsTemplateLoader {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path_for(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.toml"))
    }

    /// Returns true if the directory exists and is readable. Used by
    /// `LayeredTemplateLoader` to decide whether the FS layer is active.
    pub fn dir_exists(&self) -> bool {
        self.root.is_dir()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl TemplateLoader for FsTemplateLoader {
    fn load(&self, name: &str) -> Result<String, CommsError> {
        let path = self.path_for(name);
        match fs::read_to_string(&path) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(CommsError::TemplateNotFound(name.to_owned()))
            }
            Err(e) => Err(CommsError::Config(format!(
                "reading template '{name}' from {}: {e}",
                path.display()
            ))),
        }
    }

    fn names(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                out.push(stem.to_owned());
            }
        }
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Write;

    fn write_file(dir: &Path, name: &str, contents: &str) {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).expect("create");
        f.write_all(contents.as_bytes()).expect("write");
    }

    #[test]
    fn loads_template_from_disk() {
        let dir = tempdir();
        write_file(&dir, "greet.toml", "channel = \"email\"\nsubject = \"\"\ntext = \"\"");
        let loader = FsTemplateLoader::new(&dir);
        assert!(loader.load("greet").unwrap().contains("channel = \"email\""));
    }

    #[test]
    fn names_are_sorted_file_stems() {
        let dir = tempdir();
        write_file(&dir, "b.toml", "");
        write_file(&dir, "a.toml", "");
        write_file(&dir, "ignored.txt", "");
        let loader = FsTemplateLoader::new(&dir);
        assert_eq!(loader.names(), vec!["a".to_owned(), "b".to_owned()]);
    }

    #[test]
    fn missing_directory_yields_empty_names() {
        let loader = FsTemplateLoader::new(std::env::temp_dir().join("comms-no-such-dir-xyz"));
        assert!(loader.names().is_empty());
        assert!(!loader.dir_exists());
    }

    #[test]
    fn missing_template_returns_not_found() {
        let dir = tempdir();
        let loader = FsTemplateLoader::new(&dir);
        let err = loader.load("absent").expect_err("must miss");
        assert!(matches!(err, CommsError::TemplateNotFound(n) if n == "absent"));
    }

    fn tempdir() -> PathBuf {
        let unique = format!(
            "comms-fs-loader-test-{}",
            uuid::Uuid::new_v4().simple()
        );
        let dir = std::env::temp_dir().join(unique);
        fs::create_dir_all(&dir).expect("create tempdir");
        dir
    }
}
