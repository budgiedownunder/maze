use crate::state::{GameConfig, GameState};
use bevy::prelude::*;
use std::f32::consts::PI;

// Minimap defaults. `MAP_CELL_PX` / `MAP_RADIUS` are the values the game
// shipped with; they seed `GameConfig::default` and are used by the no-config
// (native / demo) path. The configured Play 3D path overrides both per
// difficulty via `GameConfig`. `MAP_MARGIN` (screen-edge inset) is not
// configurable.
pub(crate) const MAP_CELL_PX: u32 = 10;
pub(crate) const MAP_RADIUS: u32 = 5;
const MAP_MARGIN: f32 = 12.0;

const COLOR_MINIMAP_DARK: Color = Color::srgb(0.05, 0.05, 0.05);
const COLOR_MINIMAP_OUTSIDE: Color = Color::srgb(0.0, 0.0, 0.0);
const COLOR_MINIMAP_WALL: Color = Color::srgb(0.55, 0.55, 0.58);
const COLOR_MINIMAP_START: Color = Color::srgb(0.0, 0.65, 0.0);
const COLOR_MINIMAP_FINISH: Color = Color::srgb(1.0, 0.85, 0.1);
const COLOR_MINIMAP_FLOOR: Color = Color::srgb(0.78, 0.92, 0.78);
const COLOR_MINIMAP_PLAYER: Color = Color::srgb(0.95, 0.15, 0.15);

#[derive(Component)]
pub(crate) struct MinimapCamera;

#[derive(Component)]
pub(crate) struct MinimapPlayer;

#[derive(Component)]
pub(crate) struct MinimapBackground;

#[derive(Component)]
pub(crate) struct MinimapCell {
    pub(crate) dr: i32,
    pub(crate) dc: i32,
}

#[derive(Resource)]
pub(crate) struct MinimapConfig {
    pub(crate) center_x: f32,
    pub(crate) center_y: f32,
}

pub(crate) fn spawn_minimap(
    commands: &mut Commands,
    window: &Query<&Window>,
    config: &GameConfig,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    color_materials: &mut Option<ResMut<Assets<ColorMaterial>>>,
) {
    // A `(2·radius + 1)`-square viewport centred on the player. `cell_px` and
    // `radius` come from `GameConfig` (per-difficulty for the configured Play
    // 3D path; the shipped 10 / 5 defaults for the native / demo path). Cell
    // colours update each frame in minimap_system based on fog-of-war state.
    let cell_px = config.minimap_cell_px as f32;
    let radius = config.minimap_radius as i32;
    let view = radius * 2 + 1;
    let map_size = view as f32 * cell_px;
    let (center_x, center_y) = if let Ok(win) = window.single() {
        (
            win.width() / 2.0 - MAP_MARGIN - map_size / 2.0,
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

    // Dark background — tagged so minimap_resize_system can reposition it
    // (along with the cells) when the window size changes.
    commands.spawn((
        MinimapBackground,
        Sprite {
            color: COLOR_MINIMAP_DARK,
            custom_size: Some(Vec2::splat(map_size + 4.0)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y, -0.5),
    ));

    // Fixed grid of viewport sprites — one per slot, initially all dark (unexplored).
    for dr in -radius..=radius {
        for dc in -radius..=radius {
            let sx = center_x + dc as f32 * cell_px;
            let sy = center_y - dr as f32 * cell_px;
            commands.spawn((
                Sprite {
                    color: COLOR_MINIMAP_DARK,
                    custom_size: Some(Vec2::splat(cell_px - 1.0)),
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
        m.add(ColorMaterial { color: COLOR_MINIMAP_PLAYER, ..default() })
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

pub(crate) fn minimap_system(
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
            COLOR_MINIMAP_OUTSIDE
        } else {
            let (r, c) = (mr as usize, mc as usize);
            if !state.explored.contains(&(r, c)) {
                COLOR_MINIMAP_DARK
            } else {
                match state.grid[r][c] {
                    'W' => COLOR_MINIMAP_WALL,
                    'S' => COLOR_MINIMAP_START,
                    'F' => COLOR_MINIMAP_FINISH,
                    _ => COLOR_MINIMAP_FLOOR,
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

pub(crate) fn minimap_resize_system(
    window: Query<&Window>,
    game_config: Res<GameConfig>,
    mut minimap: ResMut<MinimapConfig>,
    mut last_size: Local<(f32, f32)>,
    mut bg: Query<&mut Transform, (With<MinimapBackground>, Without<MinimapCell>)>,
    mut cells: Query<(&MinimapCell, &mut Transform), Without<MinimapBackground>>,
) {
    let Ok(win) = window.single() else { return; };
    let w = win.width();
    let h = win.height();
    if (w - last_size.0).abs() < 0.5 && (h - last_size.1).abs() < 0.5 {
        return;
    }
    *last_size = (w, h);

    let cell_px = game_config.minimap_cell_px as f32;
    let radius = game_config.minimap_radius as i32;
    let view = radius * 2 + 1;
    let map_size = view as f32 * cell_px;
    let center_x = w / 2.0 - MAP_MARGIN - map_size / 2.0;
    let center_y = h / 2.0 - MAP_MARGIN - map_size / 2.0;

    minimap.center_x = center_x;
    minimap.center_y = center_y;

    for mut t in &mut bg {
        t.translation.x = center_x;
        t.translation.y = center_y;
    }
    for (cell, mut t) in &mut cells {
        t.translation.x = center_x + cell.dc as f32 * cell_px;
        t.translation.y = center_y - cell.dr as f32 * cell_px;
    }
    // Player marker is repositioned to (center_x, center_y) each frame by
    // minimap_system, so it picks up the new centre automatically.
}
