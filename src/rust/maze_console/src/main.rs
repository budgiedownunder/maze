mod console_app;
use crate::console_app::ConsoleApp;
use maze_console::app::App;
use storage::get_store;

// Helper functions
//fn app_name() -> &'static str {
//  env!("CARGO_PKG_NAME")
//}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = get_store(storage::StoreType::File)?;
    let mut app = ConsoleApp::new(store);
    app.run()?;
    Ok(())
}
