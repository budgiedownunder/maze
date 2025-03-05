mod console_app;
use crate::console_app::ConsoleApp;
use maze_console::app::App;
use storage::get_store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_config = storage::FileStoreConfig::default();
    let mut store = get_store(storage::StoreConfig::File(file_config))?;
    let user = store.init_default_admin_user()?;
    let mut app = ConsoleApp::new(store, &user);
    app.run()?;
    Ok(())
}
