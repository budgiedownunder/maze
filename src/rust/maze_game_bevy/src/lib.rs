use bevy::prelude::*;

pub fn build_app(app: &mut App) {
    app.add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text2d::new("Maze Game"),
        TextFont { font_size: 72.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        let mut app = App::new();
        build_app(&mut app);
        app.update();
        app
    }

    #[test]
    fn setup_spawns_one_camera() {
        let mut app = make_app();
        let count = app.world_mut().query::<&Camera2d>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn setup_spawns_title_text() {
        let mut app = make_app();
        let mut query = app.world_mut().query::<&Text2d>();
        let texts: Vec<&str> = query.iter(app.world()).map(|t| t.0.as_str()).collect();
        assert_eq!(texts, vec!["Maze Game"]);
    }
}
