use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub fn delete_file(file: &str) {
    let _ = fs::remove_file(file);
    let mut count = 0;
    loop {
        // Secondary check, in case there is lag in the operating system
        if !Path::new(file).exists() {
            break;
        }
        count += 1;
        if count == 10 {
            break;
        }
        sleep(Duration::from_millis(10));
    }
}

pub fn delete_files_with_ext(dir: &str, extension: &str) -> std::io::Result<()> {
    let files = fs::read_dir(dir)?;
    for file in files {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy();
                        delete_file(&file_name_str);
                    }
                }
            }
        }
    }
    Ok(())
}
