use maze_openapi_generator::run_generator;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let working_dir = env::current_dir()?;    
    run_generator(&working_dir.to_string_lossy())
}
