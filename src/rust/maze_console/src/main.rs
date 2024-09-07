mod console_app;
use crate::console_app::ConsoleApp;
use maze_console::app::App;

// Helper functions
//fn app_name() -> &'static str {
//  env!("CARGO_PKG_NAME")
//}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = ConsoleApp::new();
    app.run()?;
    Ok(())
}
