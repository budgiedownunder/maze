use bevy::prelude::*;
use wasm_bindgen::prelude::*;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }));
    app
}

#[wasm_bindgen]
pub fn start() {
    let mut app = make_app();
    maze_game_bevy::build_app(&mut app, None);
    app.run();
}

#[wasm_bindgen]
pub fn start_with_maze(maze_json: &str) {
    let mut app = make_app();
    maze_game_bevy::build_app(&mut app, Some(maze_json));
    app.run();
}
