use bevy::prelude::*;
use maze::MazeGame;
use std::f32::consts::PI;

const CELL_SIZE: f32 = 2.0;
const HALF_CELL: f32 = CELL_SIZE / 2.0;
const WALL_HEIGHT: f32 = 3.0;
const WALL_THICKNESS: f32 = 0.05;
const EYE_HEIGHT: f32 = 1.7;
// Inset each panel by this amount on each exposed edge to create visible border lines.
const BORDER_GAP: f32 = 0.10;
const PANEL_W: f32 = CELL_SIZE - 2.0 * BORDER_GAP;
const PANEL_H: f32 = WALL_HEIGHT - BORDER_GAP;
const PANEL_Y: f32 = (WALL_HEIGHT + BORDER_GAP) / 2.0;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
enum AppState {
    #[default]
    TitleScreen,
    Playing,
}

#[derive(Resource)]
struct TitleTimer(Timer);

#[derive(Component)]
struct TitleEntity;

#[derive(Component)]
struct WallCell;

#[derive(Component)]
struct FloorCell;

#[derive(Component)]
struct StartCell;

#[derive(Component)]
struct FinishCell;

#[derive(Resource)]
#[allow(dead_code)]
struct GameState {
    game: MazeGame,
    grid: Vec<Vec<char>>,
}

pub fn build_app(app: &mut App) {
    app.init_state::<AppState>()
        .insert_resource(TitleTimer(Timer::from_seconds(1.5, TimerMode::Once)))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(AppState::TitleScreen), setup_title)
        .add_systems(Update, tick_title.run_if(in_state(AppState::TitleScreen)))
        .add_systems(OnExit(AppState::TitleScreen), teardown_title)
        .add_systems(OnEnter(AppState::Playing), spawn_world);
}

fn setup_title(mut commands: Commands) {
    commands.spawn((Camera2d, TitleEntity));
    // Shadow layer — offset down-right to create depth illusion
    commands.spawn((
        Text2d::new("MAZE GAME"),
        TextFont { font_size: 96.0, ..default() },
        TextColor(Color::srgb(0.25, 0.15, 0.0)),
        Transform::from_translation(Vec3::new(4.0, -4.0, -0.1)),
        TitleEntity,
    ));
    // Main gold layer
    commands.spawn((
        Text2d::new("MAZE GAME"),
        TextFont { font_size: 96.0, ..default() },
        TextColor(Color::srgb(1.0, 0.75, 0.1)),
        TitleEntity,
    ));
    // Subtitle
    commands.spawn((
        Text2d::new("Loading\u{2026}"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -80.0, 0.0)),
        TitleEntity,
    ));
}

fn tick_title(
    time: Res<Time>,
    mut timer: ResMut<TitleTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_state.set(AppState::Playing);
    }
}

fn teardown_title(mut commands: Commands, query: Query<Entity, With<TitleEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let grid = demo_grid();
    let json = grid_to_json(&grid);
    let game = MazeGame::from_json(&json).expect("valid demo maze");

    let start_row = game.player_row();
    let start_col = game.player_col();

    commands.insert_resource(GameState { game, grid: grid.clone() });

    // Camera at start cell, facing South (yaw = PI, looking into the maze interior)
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(cell_centre(start_row, start_col))
            .with_rotation(Quat::from_rotation_y(PI)),
    ));

    commands.spawn(AmbientLight { brightness: 300.0, ..default() });

    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    // Mesh handles — None in headless tests where Assets<Mesh> is absent.
    // PANEL_W / PANEL_H inset panels from their edges by BORDER_GAP, creating dark gap
    // lines between adjacent coplanar panels and between walls and the floor.
    let wall_ns = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(PANEL_W, PANEL_H, WALL_THICKNESS)));
    let wall_ew = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(WALL_THICKNESS, PANEL_H, PANEL_W)));
    // Thin cuboid floor tile — Plane3d does not resolve reliably in the asset pipeline.
    let floor_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, 0.01, CELL_SIZE)));

    // emissive: LinearRgba writes directly to the framebuffer without sRGB conversion or
    // lighting interaction. base_color: BLACK ensures PBR diffuse contributes nothing.
    let wall_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.0, 0.35, 0.35, 1.0),
            ..default()
        })
    });
    let floor_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.12, 0.12, 0.12, 1.0),
            ..default()
        })
    });
    let start_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.0, 0.55, 0.45, 1.0),
            ..default()
        })
    });
    let finish_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.65, 0.58, 0.0, 1.0),
            ..default()
        })
    });

    let rows = grid.len();
    for (r, row) in grid.iter().enumerate() {
        let cols = row.len();
        for (c, &cell) in row.iter().enumerate() {
            if cell == 'W' {
                continue;
            }
            let x = c as f32 * CELL_SIZE + 1.0;
            let z = r as f32 * CELL_SIZE + 1.0;

            // Spawn wall panels in a single call so all components (Mesh3d,
            // MeshMaterial3d, Transform, marker) are applied atomically.
            // North face
            if r == 0 || grid[r - 1][c] == 'W' {
                let pos = Vec3::new(x, PANEL_Y, z - HALF_CELL);
                match (wall_ns.clone(), wall_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // South face
            if r + 1 >= rows || grid[r + 1][c] == 'W' {
                let pos = Vec3::new(x, PANEL_Y, z + HALF_CELL);
                match (wall_ns.clone(), wall_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // East face
            if c + 1 >= cols || grid[r][c + 1] == 'W' {
                let pos = Vec3::new(x + HALF_CELL, PANEL_Y, z);
                match (wall_ew.clone(), wall_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // West face
            if c == 0 || grid[r][c - 1] == 'W' {
                let pos = Vec3::new(x - HALF_CELL, PANEL_Y, z);
                match (wall_ew.clone(), wall_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }

            // --- Floor plane ---
            match cell {
                'S' => match (floor_mesh.clone(), start_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((StartCell, FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((StartCell, FloorCell, Transform::from_xyz(x, 0.0, z))); }
                },
                'F' => match (floor_mesh.clone(), finish_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((FinishCell, FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((FinishCell, FloorCell, Transform::from_xyz(x, 0.0, z))); }
                },
                _ => match (floor_mesh.clone(), floor_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((FloorCell, Transform::from_xyz(x, 0.0, z))); }
                },
            }
        }
    }
}

fn demo_grid() -> Vec<Vec<char>> {
    vec![
        vec!['S', ' ', ' ', ' ', ' ', ' ', ' '],
        vec![' ', 'W', 'W', 'W', 'W', 'W', ' '],
        vec![' ', 'W', ' ', ' ', ' ', 'W', ' '],
        vec![' ', 'W', ' ', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', 'W', ' ', ' ', ' '],
        vec!['W', 'W', 'W', 'W', ' ', 'W', 'W'],
        vec![' ', ' ', ' ', ' ', ' ', ' ', 'F'],
    ]
}

fn grid_to_json(grid: &[Vec<char>]) -> String {
    let rows: Vec<String> = grid
        .iter()
        .map(|row| {
            let cols: Vec<String> = row.iter().map(|c| format!("\"{}\"", c)).collect();
            format!("[{}]", cols.join(","))
        })
        .collect();
    format!("{{\"grid\":[{}]}}", rows.join(","))
}

fn cell_centre(row: usize, col: usize) -> Vec3 {
    Vec3::new(
        col as f32 * CELL_SIZE + 1.0,
        EYE_HEIGHT,
        row as f32 * CELL_SIZE + 1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    fn make_title_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app);
        app.update();
        app
    }

    fn make_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app);
        app.update(); // OnEnter(TitleScreen) runs
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update(); // OnExit(TitleScreen) + OnEnter(Playing) run
        app
    }

    fn expected_wall_panel_count(grid: &[Vec<char>]) -> usize {
        let rows = grid.len();
        grid.iter()
            .enumerate()
            .flat_map(|(r, row)| {
                let cols = row.len();
                row.iter().enumerate().filter_map(move |(c, &cell)| {
                    if cell == 'W' {
                        return None;
                    }
                    Some(
                        (r == 0 || grid[r - 1][c] == 'W') as usize
                            + (r + 1 >= rows || grid[r + 1][c] == 'W') as usize
                            + (c == 0 || grid[r][c - 1] == 'W') as usize
                            + (c + 1 >= cols || grid[r][c + 1] == 'W') as usize,
                    )
                })
            })
            .sum()
    }

    #[test]
    fn title_spawns_camera2d() {
        let mut app = make_title_app();
        let count = app.world_mut().query::<&Camera2d>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn title_spawns_text() {
        let mut app = make_title_app();
        let count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
        assert!(count >= 2, "expected at least 2 text entities, got {count}");
    }

    #[test]
    fn playing_spawns_camera3d() {
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&Camera3d>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn playing_wall_marker_count() {
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&WallCell>().iter(app.world()).count();
        let expected = expected_wall_panel_count(&demo_grid());
        assert_eq!(count, expected, "wall panel count mismatch");
    }

    #[test]
    fn playing_non_wall_marker_count() {
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
        let grid = demo_grid();
        let expected = grid.iter().flat_map(|r| r.iter()).filter(|&&c| c != 'W').count();
        assert_eq!(count, expected, "floor cell count mismatch");
    }

    #[test]
    fn playing_no_title_entities() {
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&TitleEntity>().iter(app.world()).count();
        assert_eq!(count, 0);
    }
}
