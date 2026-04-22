use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Maze Game".into(),
            ..default()
        }),
        ..default()
    }));
    maze_game_bevy::build_app(&mut app);
    app.run();
}
