use maze_web_server::api::get_openapi_v1;

/// Generates the `openapi.json` file in the given output directory
///
/// # Arguments
/// * `out_dir` - Output directory
///
/// # Returns
/// Nothing if successful, else an error 
pub fn run_generator(out_dir: &str) -> Result<(), Box<dyn std::error::Error>>  {
    let output_file = "openapi.json";
    println!("Running Maze API OpenAPI generator...");
    let openapi_json = get_openapi_v1().to_json()?;
    let dest_path = std::path::Path::new(out_dir).join(output_file);
    std::fs::write(&dest_path, openapi_json)?;
    println!("Maze API OpenAPI specification sucessfully generated as file:");
    println!("{}", dest_path.clone().to_string_lossy());
    Ok(())
}
  

#[cfg(test)]
mod tests {
    use crate::run_generator;
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn should_be_able_to_create_openapi_json_file() -> Result<(), Box<dyn Error>> {
        run_generator("")?;
        let path = Path::new("openapi.json");
        assert!(path.exists(), "The expected output file '{:?}' was not found", path);
        Ok(())
    }
} 