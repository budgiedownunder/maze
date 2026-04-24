use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use maze::{Direction, MazeGame, MoveResult};
use std::collections::HashSet;
use std::f32::consts::PI;

const CELL_SIZE: f32 = 2.0;
const HALF_CELL: f32 = CELL_SIZE / 2.0;
const WALL_HEIGHT: f32 = 3.0;
const WALL_THICKNESS: f32 = 0.05;
const EYE_HEIGHT: f32 = 1.7;
// Inset each panel by this amount on each exposed edge to create visible border lines.
const BORDER_GAP: f32 = 0.01;
const PANEL_W: f32 = CELL_SIZE - 2.0 * BORDER_GAP;
const PANEL_H: f32 = WALL_HEIGHT - BORDER_GAP;
const PANEL_Y: f32 = (WALL_HEIGHT + BORDER_GAP) / 2.0;
const TURN_DUR: f32 = 0.12;
const MOVE_DUR: f32 = 0.18;
const LINE_W: f32 = 0.06;
const LINE_H: f32 = 0.01;
const LINE_Y: f32 = 0.015;
const MAP_CELL_PX: f32 = 10.0;
const MAP_MARGIN: f32 = 12.0;
const MAP_RADIUS: i32 = 5;                      // cells visible in each direction from player
const MAP_VIEW: i32 = MAP_RADIUS * 2 + 1;       // total cells per side (7)

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
enum AppState {
    #[default]
    TitleScreen,
    Playing,
}

#[derive(Resource)]
struct PendingMazeJson(Option<String>);

#[derive(Resource)]
struct TitleTimer(Timer);

#[derive(Component)]
struct TitleEntity;

#[derive(Component, PartialEq)]
enum TitleTextKind { Shadow, Gold, Sub }

#[derive(Component)]
struct WinMainText;

#[derive(Component)]
struct WinSubText;

#[derive(Component)]
struct WinBackground;

#[derive(Component)]
struct WallCell;

#[derive(Component)]
struct FloorCell;

#[derive(Component)]
struct StartCell;

#[derive(Component)]
struct FinishCell;

#[derive(Component)]
struct FloorLine;

#[derive(Component)]
struct FinishOrb;

#[derive(Component)]
struct WinOverlay;

#[derive(Component)]
struct MinimapCamera;

#[derive(Component)]
struct MinimapPlayer;

#[derive(Component)]
struct MinimapCell {
    dr: i32,
    dc: i32,
}

#[derive(Resource)]
struct MinimapConfig {
    center_x: f32,
    center_y: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GridFacing {
    North,
    East,
    South,
    West,
}

impl GridFacing {
    fn turn_left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    fn to_direction(self) -> Direction {
        match self {
            Self::North => Direction::Up,
            Self::East => Direction::Right,
            Self::South => Direction::Down,
            Self::West => Direction::Left,
        }
    }

    fn to_yaw(self) -> f32 {
        match self {
            Self::North => 0.0,
            Self::East  => PI + PI / 2.0,
            Self::South => PI,
            Self::West  => PI / 2.0,
        }
    }
}

struct Animation {
    start_pos: Vec3,
    target_pos: Vec3,
    start_yaw: f32,
    target_yaw: f32,
    elapsed: f32,
    duration: f32,
}

impl Animation {
    fn progress(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    fn current_pos(&self) -> Vec3 {
        self.start_pos.lerp(self.target_pos, self.progress())
    }

    fn current_yaw(&self) -> f32 {
        self.start_yaw + (self.target_yaw - self.start_yaw) * self.progress()
    }
}

#[derive(Resource)]
struct GameState {
    game: MazeGame,
    grid: Vec<Vec<char>>,
    facing: GridFacing,
    visual_pos: Vec3,
    visual_yaw: f32,
    anim: Option<Animation>,
    explored: HashSet<(usize, usize)>,
    won: bool,
}

pub fn build_app(app: &mut App, maze_json: Option<&str>) {
    app.insert_resource(PendingMazeJson(maze_json.map(String::from)))
        .init_state::<AppState>()
        .insert_resource(TitleTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(AppState::TitleScreen), setup_title)
        .add_systems(Update, tick_title.run_if(in_state(AppState::TitleScreen)))
        .add_systems(Update, title_resize_system.run_if(in_state(AppState::TitleScreen)))
        .add_systems(OnExit(AppState::TitleScreen), teardown_title)
        .add_systems(OnEnter(AppState::Playing), spawn_world)
        .add_systems(Update, movement_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, win_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, minimap_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, orb_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, quit_system);
}

fn setup_title(mut commands: Commands) {
    commands.spawn((Camera2d, TitleEntity));
    // Shadow layer — offset down-right; font size updated reactively by title_resize_system
    commands.spawn((
        Text2d::new("MAZE GAME"),
        TextFont { font_size: 96.0, ..default() },
        TextColor(Color::srgb(0.25, 0.15, 0.0)),
        Transform::from_translation(Vec3::new(4.0, -4.0, -0.1)),
        TitleEntity,
        TitleTextKind::Shadow,
    ));
    // Main gold layer
    commands.spawn((
        Text2d::new("MAZE GAME"),
        TextFont { font_size: 96.0, ..default() },
        TextColor(Color::srgb(1.0, 0.75, 0.1)),
        TitleEntity,
        TitleTextKind::Gold,
    ));
    // Subtitle
    commands.spawn((
        Text2d::new("Starting..."),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -80.0, 0.0)),
        TitleEntity,
        TitleTextKind::Sub,
    ));
}

fn title_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    mut texts: Query<(&mut TextFont, &mut Transform, &TitleTextKind)>,
) {
    let width = window.single().map(|w| w.width()).unwrap_or(1280.0);
    if (width - *last_width).abs() < 0.5 { return; }
    *last_width = width;

    let font_size = (width / 5.5).min(96.0);
    let shadow_off = font_size / 24.0;
    let subtitle_size = (font_size / 4.0).max(14.0);
    let subtitle_y = -(font_size * 0.85);

    for (mut font, mut t, kind) in &mut texts {
        match kind {
            TitleTextKind::Sub => {
                font.font_size = subtitle_size;
                t.translation = Vec3::new(0.0, subtitle_y, 0.0);
            }
            TitleTextKind::Shadow => {
                font.font_size = font_size;
                t.translation = Vec3::new(shadow_off, -shadow_off, -0.1);
            }
            TitleTextKind::Gold => {
                font.font_size = font_size;
            }
        }
    }
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
    pending: Res<PendingMazeJson>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut color_materials: Option<ResMut<Assets<ColorMaterial>>>,
    mut images: Option<ResMut<Assets<Image>>>,
    window: Query<&Window>,
) {
    let (game, grid) = match pending.0.as_deref() {
        Some(json) => {
            let game = MazeGame::from_json(json).expect("maze JSON was validated by the REST API");
            let grid = game.grid().to_vec();
            (game, grid)
        }
        None => {
            let grid = demo_grid();
            let json = grid_to_json(&grid);
            (MazeGame::from_json(&json).expect("demo grid is hardcoded and always valid"), grid)
        }
    };

    let start_row = game.player_row();
    let start_col = game.player_col();
    let start_pos = cell_centre(start_row, start_col);
    let facing = initial_facing(&grid, start_row, start_col);
    let start_yaw = facing.to_yaw();

    let mut explored = HashSet::new();
    explore_cell(&mut explored, &grid, start_row, start_col);

    commands.insert_resource(GameState {
        game,
        grid: grid.clone(),
        facing,
        visual_pos: start_pos,
        visual_yaw: start_yaw,
        anim: None,
        explored,
        won: false,
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(start_pos).with_rotation(Quat::from_rotation_y(start_yaw)),
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

    // Procedural textures — multiplied into emissive color via emissive_texture.
    let brick_tex = images.as_mut().map(|imgs| make_brick_texture(imgs));
    let tile_tex  = images.as_mut().map(|imgs| make_tile_texture(imgs));

    // emissive: LinearRgba writes directly to the framebuffer without sRGB conversion or
    // lighting interaction. base_color: BLACK ensures PBR diffuse contributes nothing.
    // N/S-facing panels (ahead/behind) — cool stone grey
    let wall_ns_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.38, 0.38, 0.40, 1.0),
            emissive_texture: brick_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(3.0, 5.0)),
            ..default()
        })
    });
    // E/W-facing panels (sides) — slightly darker stone grey for orientation distinction
    let wall_ew_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.14, 0.14, 0.16, 1.0),
            emissive_texture: brick_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(3.0, 5.0)),
            ..default()
        })
    });
    let floor_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.12, 0.12, 0.12, 1.0),
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(2.0, 2.0)),
            ..default()
        })
    });
    let start_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.0, 0.6, 0.0, 1.0),
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(2.0, 2.0)),
            ..default()
        })
    });
    let finish_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.8, 0.8, 0.8, 1.0),
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(2.0, 2.0)),
            ..default()
        })
    });
    let orb_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(0.35)));
    let orb_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(1.2, 0.9, 0.1, 1.0),
            ..default()
        })
    });

    // Grid lines: E-W strip (runs along X), N-S strip (runs along Z)
    let line_ew = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, LINE_H, LINE_W)));
    let line_ns = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(LINE_W, LINE_H, CELL_SIZE)));
    let line_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.28, 0.28, 0.28, 1.0),
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
            // North face (N/S-facing panel — teal)
            if r == 0 || grid[r - 1][c] == 'W' {
                let pos = Vec3::new(x, PANEL_Y, z - HALF_CELL);
                match (wall_ns.clone(), wall_ns_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // South face (N/S-facing panel — teal)
            if r + 1 >= rows || grid[r + 1][c] == 'W' {
                let pos = Vec3::new(x, PANEL_Y, z + HALF_CELL);
                match (wall_ns.clone(), wall_ns_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // East face (E/W-facing panel — blue-teal)
            if c + 1 >= cols || grid[r][c + 1] == 'W' {
                let pos = Vec3::new(x + HALF_CELL, PANEL_Y, z);
                match (wall_ew.clone(), wall_ew_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }
            // West face (E/W-facing panel — blue-teal)
            if c == 0 || grid[r][c - 1] == 'W' {
                let pos = Vec3::new(x - HALF_CELL, PANEL_Y, z);
                match (wall_ew.clone(), wall_ew_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((WallCell, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((WallCell, Transform::from_translation(pos))); }
                }
            }

            // --- Floor grid lines ---
            // Each shared edge is spawned once: always South + East; North/West only
            // when the neighbour in that direction is a wall or grid boundary.
            let spawn_line = |commands: &mut Commands, mesh: Option<Handle<Mesh>>, mat: Option<Handle<StandardMaterial>>, pos: Vec3| {
                match (mesh, mat) {
                    (Some(m), Some(mt)) => { commands.spawn((FloorLine, Transform::from_translation(pos), Mesh3d(m), MeshMaterial3d(mt))); }
                    _ => { commands.spawn((FloorLine, Transform::from_translation(pos))); }
                }
            };
            // South edge (always)
            spawn_line(&mut commands, line_ew.clone(), line_mat.clone(), Vec3::new(x, LINE_Y, z + HALF_CELL));
            // East edge (always)
            spawn_line(&mut commands, line_ns.clone(), line_mat.clone(), Vec3::new(x + HALF_CELL, LINE_Y, z));
            // North edge (only if wall/boundary to north, to avoid duplicating shared interior edges)
            if r == 0 || grid[r - 1][c] == 'W' {
                spawn_line(&mut commands, line_ew.clone(), line_mat.clone(), Vec3::new(x, LINE_Y, z - HALF_CELL));
            }
            // West edge (only if wall/boundary to west)
            if c == 0 || grid[r][c - 1] == 'W' {
                spawn_line(&mut commands, line_ns.clone(), line_mat.clone(), Vec3::new(x - HALF_CELL, LINE_Y, z));
            }

            // --- Floor plane ---
            match cell {
                'S' => match (floor_mesh.clone(), start_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((StartCell, FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((StartCell, FloorCell, Transform::from_xyz(x, 0.0, z))); }
                },
                'F' => {
                    match (floor_mesh.clone(), finish_mat.clone()) {
                        (Some(mesh), Some(mat)) => {
                            commands.spawn((FinishCell, FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                        }
                        _ => { commands.spawn((FinishCell, FloorCell, Transform::from_xyz(x, 0.0, z))); }
                    }
                    const ORB_BASE_Y: f32 = 1.0;
                    match (orb_mesh.clone(), orb_mat.clone()) {
                        (Some(mesh), Some(mat)) => {
                            commands.spawn((
                                FinishOrb,
                                Mesh3d(mesh),
                                MeshMaterial3d(mat),
                                Transform::from_xyz(x, ORB_BASE_Y, z),
                            ));
                        }
                        _ => { commands.spawn((FinishOrb, Transform::from_xyz(x, ORB_BASE_Y, z))); }
                    }
                    commands.spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.85, 0.2),
                            intensity: 80_000.0,
                            radius: 0.35,
                            shadows_enabled: true,
                            ..default()
                        },
                        Transform::from_xyz(x, ORB_BASE_Y, z),
                    ));
                }
                _ => match (floor_mesh.clone(), floor_mat.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((FloorCell, Transform::from_xyz(x, 0.0, z), Mesh3d(mesh), MeshMaterial3d(mat)));
                    }
                    _ => { commands.spawn((FloorCell, Transform::from_xyz(x, 0.0, z))); }
                },
            }
        }
    }

    // --- Minimap overlay ---
    // Fixed MAP_VIEW×MAP_VIEW viewport centred on the player. Cell colours update
    // each frame in minimap_system based on fog-of-war exploration state.
    let map_size = MAP_VIEW as f32 * MAP_CELL_PX;
    let (center_x, center_y) = if let Ok(win) = window.single() {
        (
            win.width()  / 2.0 - MAP_MARGIN - map_size / 2.0,
            win.height() / 2.0 - MAP_MARGIN - map_size / 2.0,
        )
    } else {
        (200.0, 200.0)
    };

    commands.insert_resource(MinimapConfig { center_x, center_y });

    // Overlay camera — does not clear the colour buffer so the 3D scene shows through.
    commands.spawn((
        Camera2d,
        Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
        MinimapCamera,
    ));

    // Dark background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.05, 0.05, 0.05),
            custom_size: Some(Vec2::splat(map_size + 4.0)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y, -0.5),
    ));

    // Fixed grid of viewport sprites — one per slot, initially all dark (unexplored).
    for dr in -MAP_RADIUS..=MAP_RADIUS {
        for dc in -MAP_RADIUS..=MAP_RADIUS {
            let sx = center_x + dc as f32 * MAP_CELL_PX;
            let sy = center_y - dr as f32 * MAP_CELL_PX;
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.05, 0.05, 0.05),
                    custom_size: Some(Vec2::splat(MAP_CELL_PX - 1.0)),
                    ..default()
                },
                Transform::from_xyz(sx, sy, 0.0),
                MinimapCell { dr, dc },
            ));
        }
    }

    // Player marker: filled triangle pointing up (North) by default, rotated to match facing.
    let arrow_mesh = meshes.as_mut().map(|m| {
        m.add(Triangle2d::new(
            Vec2::new(0.0, 4.5),
            Vec2::new(-3.0, -3.0),
            Vec2::new(3.0, -3.0),
        ))
    });
    let arrow_mat = color_materials.as_mut().map(|m| {
        m.add(ColorMaterial { color: Color::srgb(1.0, 0.85, 0.0), ..default() })
    });
    match (arrow_mesh, arrow_mat) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_xyz(center_x, center_y, 1.0)
                    .with_rotation(Quat::from_rotation_z(PI)),
                MinimapPlayer,
            ));
        }
        _ => {
            commands.spawn((Transform::from_xyz(center_x, center_y, 1.0), MinimapPlayer));
        }
    }
}

fn win_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    mut win_texts: Query<(&mut TextFont, &mut Transform, Option<&WinMainText>), With<WinOverlay>>,
    mut win_sprites: Query<&mut Sprite, With<WinBackground>>,
) {
    let width = window.single().map(|w| w.width()).unwrap_or(1280.0);
    if (width - *last_width).abs() < 0.5 { return; }
    *last_width = width;

    let scale = (width / 5.5).min(96.0) / 96.0;
    for (mut font, mut t, is_main) in &mut win_texts {
        if is_main.is_some() {
            font.font_size = 72.0 * scale;
            t.translation = Vec3::new(0.0, 16.0 * scale, 11.0);
        } else {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, -36.0 * scale, 11.0);
        }
    }
    for mut sprite in &mut win_sprites {
        sprite.custom_size = Some(Vec2::new(340.0 * scale, 130.0 * scale));
    }
}

fn movement_system(
    mut commands: Commands,
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<GameState>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let dt = time.delta_secs();

    // Advance active animation; snap to target when complete
    let anim_done = if let Some(ref mut anim) = state.anim {
        anim.elapsed += dt;
        anim.elapsed >= anim.duration
    } else {
        false
    };
    if anim_done {
        let anim = state.anim.take().unwrap();
        state.visual_pos = anim.target_pos;
        state.visual_yaw = anim.target_yaw;

        // Check whether the player just arrived at the finish cell
        let (r, c) = (state.game.player_row(), state.game.player_col());
        if !state.won && state.grid[r][c] == 'F' {
            state.won = true;
            commands.spawn((WinOverlay, WinBackground, Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                custom_size: Some(Vec2::new(340.0, 130.0)),
                ..default()
            }, Transform::from_xyz(0.0, 0.0, 10.0)));
            commands.spawn((WinOverlay, WinMainText,
                Text2d::new("You Win!"),
                TextFont { font_size: 72.0, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.1)),
                Transform::from_xyz(0.0, 16.0, 11.0),
            ));
            commands.spawn((WinOverlay, WinSubText,
                Text2d::new("Maze complete"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, -36.0, 11.0),
            ));
        }
    } else if state.anim.is_some() {
        let pos = state.anim.as_ref().unwrap().current_pos();
        let yaw = state.anim.as_ref().unwrap().current_yaw();
        state.visual_pos = pos;
        state.visual_yaw = yaw;
    }

    // Process input only when idle, game not won, and input is available (absent in headless tests)
    if state.anim.is_none() && !state.won {
        let Some(keys) = keys else { return; };
        let left = keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA);
        let right = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD);
        let forward = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);

        if left {
            state.facing = state.facing.turn_left();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            state.anim = Some(Animation {
                start_pos,
                target_pos: start_pos,
                start_yaw,
                target_yaw: start_yaw + PI / 2.0,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if right {
            state.facing = state.facing.turn_right();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            state.anim = Some(Animation {
                start_pos,
                target_pos: start_pos,
                start_yaw,
                target_yaw: start_yaw - PI / 2.0,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if forward {
            let dir = state.facing.to_direction();
            let result = state.game.move_player(dir);
            if matches!(result, MoveResult::Moved | MoveResult::Complete) {
                let (row, col) = (state.game.player_row(), state.game.player_col());
                let nrows = state.grid.len();
                let ncols = state.grid[0].len();
                explore_cell_raw(&mut state.explored, nrows, ncols, row, col);
                let target_pos = cell_centre(row, col);
                let (start_pos, start_yaw) = (state.visual_pos, state.visual_yaw);
                state.anim = Some(Animation {
                    start_pos,
                    target_pos,
                    start_yaw,
                    target_yaw: start_yaw,
                    elapsed: 0.0,
                    duration: MOVE_DUR,
                });
            }
        }
    }

    // Update camera transform every frame
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = state.visual_pos;
        transform.rotation = Quat::from_rotation_y(state.visual_yaw);
    }
}

fn minimap_system(
    state: Res<GameState>,
    config: Res<MinimapConfig>,
    mut cells: Query<(&MinimapCell, &mut Sprite)>,
    mut player_q: Query<&mut Transform, With<MinimapPlayer>>,
) {
    let pr = state.game.player_row() as i32;
    let pc = state.game.player_col() as i32;
    let nrows = state.grid.len() as i32;
    let ncols = if state.grid.is_empty() { 0 } else { state.grid[0].len() as i32 };

    for (cell, mut sprite) in &mut cells {
        let mr = pr + cell.dr;
        let mc = pc + cell.dc;
        sprite.color = if mr < 0 || mc < 0 || mr >= nrows || mc >= ncols {
            // Outside grid boundary
            Color::srgb(0.0, 0.0, 0.0)
        } else {
            let (r, c) = (mr as usize, mc as usize);
            if !state.explored.contains(&(r, c)) {
                Color::srgb(0.05, 0.05, 0.05)
            } else {
                match state.grid[r][c] {
                    'W' => Color::srgb(0.18, 0.18, 0.18),
                    'S' => Color::srgb(0.0, 0.7, 0.0),
                    'F' => Color::srgb(0.9, 0.9, 0.9),
                    _   => Color::srgb(0.45, 0.45, 0.45),
                }
            }
        };
    }

    // Player marker stays fixed at minimap centre; only rotation changes.
    if let Ok(mut t) = player_q.single_mut() {
        t.translation = Vec3::new(config.center_x, config.center_y, 1.0);
        t.rotation = Quat::from_rotation_z(state.visual_yaw);
    }
}

fn quit_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut exit: bevy::ecs::message::MessageWriter<AppExit>,
) {
    if let Some(keys) = keys {
        if keys.just_pressed(KeyCode::Escape) {
            exit.write(AppExit::Success);
        }
    }
}

fn orb_system(time: Res<Time>, mut orb: Query<&mut Transform, With<FinishOrb>>) {
    const ORB_BASE_Y: f32 = 1.0;
    if let Ok(mut t) = orb.single_mut() {
        t.translation.y = ORB_BASE_Y + 0.15 * (time.elapsed_secs() * 2.0).sin();
        t.rotate_y(time.delta_secs() * 1.2);
    }
}

fn explore_cell(explored: &mut HashSet<(usize, usize)>, grid: &[Vec<char>], row: usize, col: usize) {
    explore_cell_raw(explored, grid.len(), if grid.is_empty() { 0 } else { grid[0].len() }, row, col);
}

fn explore_cell_raw(explored: &mut HashSet<(usize, usize)>, nrows: usize, ncols: usize, row: usize, col: usize) {
    explored.insert((row, col));
    if row > 0           { explored.insert((row - 1, col)); }
    if row + 1 < nrows   { explored.insert((row + 1, col)); }
    if col > 0           { explored.insert((row, col - 1)); }
    if col + 1 < ncols   { explored.insert((row, col + 1)); }
}

fn make_image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    image
}

fn make_brick_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    const BRICK_W: u32 = 30;
    const BRICK_H: u32 = 14;
    const MORTAR: u32 = 2;
    const ROW_H: u32 = BRICK_H + MORTAR;   // 16
    const COL_W: u32 = BRICK_W + MORTAR;   // 32

    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let row = y / ROW_H;
            let y_in_row = y % ROW_H;
            let is_mortar = if y_in_row >= BRICK_H {
                true
            } else {
                let offset = if row.is_multiple_of(2) { 0 } else { COL_W / 2 };
                (x + offset) % COL_W >= BRICK_W
            };
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_mortar {
                35
            } else {
                // deterministic per-brick variation for surface texture
                let bx = (x + if row.is_multiple_of(2) { 0 } else { COL_W / 2 }) % COL_W;
                let by = y % ROW_H;
                let hash = bx.wrapping_mul(7).wrapping_add(by.wrapping_mul(13))
                    .wrapping_add(row.wrapping_mul(31));
                (200u32 + hash % 45) as u8
            };
            pixels[idx]     = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

fn make_tile_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    const TILE: u32 = 30;
    const GROUT: u32 = 2;
    const UNIT: u32 = TILE + GROUT; // 32

    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let xm = x % UNIT;
            let ym = y % UNIT;
            let is_grout = xm >= TILE || ym >= TILE;
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_grout {
                35
            } else {
                let tx = x / UNIT;
                let ty = y / UNIT;
                let hash = tx.wrapping_mul(17).wrapping_add(ty.wrapping_mul(31))
                    .wrapping_add(xm.wrapping_mul(3)).wrapping_add(ym.wrapping_mul(5));
                (185u32 + hash % 55) as u8
            };
            pixels[idx]     = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
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

// Cycles S→E→N→W and returns the first direction with an open neighbour.
fn initial_facing(grid: &[Vec<char>], row: usize, col: usize) -> GridFacing {
    let rows = grid.len() as isize;
    let cols = if grid.is_empty() { 0 } else { grid[0].len() as isize };
    let r = row as isize;
    let c = col as isize;
    let open = |dr: isize, dc: isize| -> bool {
        let (nr, nc) = (r + dr, c + dc);
        nr >= 0 && nc >= 0 && nr < rows && nc < cols && grid[nr as usize][nc as usize] != 'W'
    };
    if open( 1,  0) { return GridFacing::South; }
    if open( 0,  1) { return GridFacing::East;  }
    if open(-1,  0) { return GridFacing::North; }
    if open( 0, -1) { return GridFacing::West;  }
    GridFacing::South
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
        build_app(&mut app, None);
        app.update();
        app
    }

    fn make_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app, None);
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

    #[test]
    fn grid_facing_turn_right_north_gives_east() {
        assert_eq!(GridFacing::North.turn_right(), GridFacing::East);
    }

    #[test]
    fn grid_facing_to_direction_round_trip() {
        let dirs: Vec<_> = [GridFacing::North, GridFacing::East, GridFacing::South, GridFacing::West]
            .iter()
            .map(|&f| f.to_direction())
            .collect();
        for i in 0..dirs.len() {
            for j in (i + 1)..dirs.len() {
                assert_ne!(dirs[i], dirs[j], "facing {i} and {j} map to the same direction");
            }
        }
    }

    #[test]
    fn build_app_with_none_uses_demo_grid() {
        let app = make_playing_app();
        let state = app.world().resource::<GameState>();
        assert_eq!(state.grid.len(), 7);
        assert_eq!(state.grid[0].len(), 7);
    }

    #[test]
    fn initial_facing_prefers_south_when_open() {
        // S at (0,0): south neighbour (1,0) = ' ' → South
        let grid = demo_grid();
        assert_eq!(initial_facing(&grid, 0, 0), GridFacing::South);
    }

    #[test]
    fn initial_facing_skips_south_wall_picks_east() {
        let grid = vec![
            vec!['S', ' '],
            vec!['W', 'W'],
        ];
        assert_eq!(initial_facing(&grid, 0, 0), GridFacing::East);
    }

    #[test]
    fn initial_facing_skips_south_east_picks_north() {
        // (1,1): south=(2,1)=W, east=(1,2)=W, north=(0,1)=' ' → North
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec!['W', 'S', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert_eq!(initial_facing(&grid, 1, 1), GridFacing::North);
    }

    #[test]
    fn initial_facing_skips_south_east_north_picks_west() {
        // (0,1): south=W, east=OOB, north=OOB, west=(0,0)=' ' → West
        let grid = vec![
            vec![' ', 'S'],
            vec!['W', 'W'],
        ];
        assert_eq!(initial_facing(&grid, 0, 1), GridFacing::West);
    }

    #[test]
    fn initial_facing_all_walls_falls_back_to_south() {
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert_eq!(initial_facing(&grid, 0, 1), GridFacing::South);
    }

    #[test]
    fn build_app_with_maze_json_uses_provided_grid() {
        let json = r#"{"grid":[["S"," "," "],[" ","W"," "],[" "," ","F"]]}"#;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app, Some(json));
        app.update();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::Playing);
        app.update();
        let state = app.world().resource::<GameState>();
        assert_eq!(state.grid.len(), 3);
        assert_eq!(state.grid[0].len(), 3);
        assert_eq!(state.game.player_row(), 0);
        assert_eq!(state.game.player_col(), 0);
    }
}
