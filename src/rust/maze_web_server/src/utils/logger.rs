use std::path::{Path, PathBuf};

/// Initialises the global logger to write to both stdout and a daily log file.
///
/// Log files are written to `log_dir` and named `{log_file_prefix}{YYYY-MM-DD}.log`.
/// The prefix is used verbatim — include any desired separator as its final character
/// (e.g. `"maze_web_server_"` produces `maze_web_server_2026-04-09.log`).
/// A new file is started each calendar day; old files are not deleted automatically.
///
/// # Errors
/// Returns [`fern::InitError`] if the log directory cannot be created or the
/// global logger has already been set (i.e. `init` was called more than once).
pub fn init(log_dir: &str, log_level: &str, log_file_prefix: &str) -> Result<(), fern::InitError> {
    let dir = build_log_dir(log_dir)?;
    let log_file = log_file_path(&dir, log_file_prefix);

    let level_filter = log_level
        .parse::<log::LevelFilter>()
        .unwrap_or(log::LevelFilter::Info);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level_filter)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;

    Ok(())
}

/// Creates the log directory (including any missing parents) and returns its path.
pub(crate) fn build_log_dir(log_dir: &str) -> std::io::Result<PathBuf> {
    let dir = Path::new(log_dir).to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the full path of today's log file inside `log_dir`.
/// Format: `{prefix}{YYYY-MM-DD}.log` — the prefix is used verbatim.
pub(crate) fn log_file_path(log_dir: &Path, prefix: &str) -> PathBuf {
    let date = chrono::Local::now().format("%Y-%m-%d");
    log_dir.join(format!("{prefix}{date}.log"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_dir_is_created_when_absent() {
        let tmp = std::env::temp_dir().join(format!(
            "maze_log_test_{}",
            uuid::Uuid::new_v4()
        ));
        assert!(!tmp.exists());
        build_log_dir(tmp.to_str().unwrap()).unwrap();
        assert!(tmp.is_dir());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn log_file_path_matches_naming_convention() {
        let dir = Path::new("/tmp/logs");
        let prefix = "maze_web_server_";
        let path = log_file_path(dir, prefix);
        let name = path.file_name().unwrap().to_str().unwrap();

        assert!(name.starts_with(prefix), "name was: {name}");
        assert!(name.ends_with(".log"), "name was: {name}");

        // Date portion must be YYYY-MM-DD (10 chars between prefix and .log suffix)
        let date_part = &name[prefix.len()..name.len() - ".log".len()];
        assert_eq!(date_part.len(), 10, "date part was: {date_part}");
        assert_eq!(date_part.chars().nth(4), Some('-'), "date part was: {date_part}");
        assert_eq!(date_part.chars().nth(7), Some('-'), "date part was: {date_part}");
    }

    #[test]
    fn log_file_path_uses_custom_prefix() {
        let dir = Path::new("/tmp/logs");
        let path = log_file_path(dir, "my-app-");
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("my-app-"), "name was: {name}");
        assert!(name.ends_with(".log"), "name was: {name}");
    }

    #[test]
    fn invalid_log_level_falls_back_to_info() {
        let filter = "nonsense"
            .parse::<log::LevelFilter>()
            .unwrap_or(log::LevelFilter::Info);
        assert_eq!(filter, log::LevelFilter::Info);
    }
}
