use bevy::prelude::*;

fn main() {
    // Fail fast on a mistyped `MAZE_DEMO` rather than silently launching the
    // normal game — the value selects a native demo world, so a typo would be
    // confusing otherwise.
    if let Err(message) = maze_game_bevy::validate_demo_env() {
        eprintln!("error: {message}");
        std::process::exit(1);
    }

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Maze 3D".into(),
            ..default()
        }),
        ..default()
    }));
    maze_game_bevy::build_app(&mut app, None);
    app.run();
}
