use crate::images::make_image;
use crate::state::{GameConfig, GameState, MultiLevelRun, WallType};
use crate::world::objects::overrides::resolve_wall_type;
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;
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
// Non-occluding wall types read distinctly from the neutral solid-wall grey so
// the player can spot pools / fences on the map. Solid wall textures keep
// `COLOR_MINIMAP_WALL`.
const COLOR_MINIMAP_WATER: Color = Color::srgb(0.18, 0.45, 0.92);
const COLOR_MINIMAP_LAVA: Color = Color::srgb(1.0, 0.42, 0.08);

// Maze-dimensions readout, anchored as a strip just below the minimap. Muted
// fill + text (echoing the status-bar mode label) so it reads as quiet
// secondary info rather than competing with the minimap for attention.
const COLOR_MINIMAP_DIM_BG: Color = Color::srgba(0.10, 0.10, 0.14, 0.80);
const COLOR_MINIMAP_DIM_TEXT: Color = Color::srgb(0.67, 0.60, 0.92);
const MINIMAP_DIM_FONT: f32 = 18.0;
pub(crate) const MINIMAP_DIM_STRIP_H: f32 = 22.0;
const MINIMAP_DIM_GAP: f32 = 2.0;
/// Screen-edge inset and muted palette, shared with the diagnostics readout that
/// sits directly beneath this strip so the whole top-right column reads as one
/// block rather than two unrelated panels.
pub(crate) const MINIMAP_EDGE_MARGIN: f32 = MAP_MARGIN;
pub(crate) const MINIMAP_PANEL_BG: Color = COLOR_MINIMAP_DIM_BG;
pub(crate) const MINIMAP_PANEL_TEXT: Color = COLOR_MINIMAP_DIM_TEXT;

/// How a minimap cell should render: a flat colour, or the iron-fence look —
/// the steel-teal base overlaid with thin black vertical bars.
#[derive(Debug, PartialEq)]
enum CellLook {
    Flat(Color),
    IronBars,
}

/// The minimap look for an in-grid, explored cell `(r, c)`. A `'W'` cell reads its
/// per-cell wall-type override (resolved against `default_wall_type`), so water /
/// lava show as distinct flat colours and iron-fence shows its barred look, while
/// solid-wall textures keep the neutral wall grey. `default_wall_type` is the
/// level's effective wall type — the per-maze `wall_type`, or, under
/// `wall_type = "random"`, the type rolled for the level currently shown — so the
/// minimap matches the 3D view rather than the placeholder default.
fn explored_cell_look(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    default_wall_type: WallType,
    r: usize,
    c: usize,
) -> CellLook {
    match grid[r][c] {
        'W' => {
            let entity = cell_entities.get(&(r, c)).and_then(|v| v.first());
            match resolve_wall_type(entity, default_wall_type) {
                WallType::Water => CellLook::Flat(COLOR_MINIMAP_WATER),
                WallType::Lava => CellLook::Flat(COLOR_MINIMAP_LAVA),
                WallType::IronFence => CellLook::IronBars,
                _ => CellLook::Flat(COLOR_MINIMAP_WALL),
            }
        }
        'S' => CellLook::Flat(COLOR_MINIMAP_START),
        'F' => CellLook::Flat(COLOR_MINIMAP_FINISH),
        _ => CellLook::Flat(COLOR_MINIMAP_FLOOR),
    }
}

/// Builds the iron-fence minimap texture: the solid-wall grey base with thin
/// black vertical bars, so a fenced cell reads as a barred wall on the map. Fully
/// opaque, so it renders with a white tint (the texture carries the colour).
fn make_iron_bars_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 16;
    const H: u32 = 8;
    let s = COLOR_MINIMAP_WALL.to_srgba();
    let base = [
        (s.red * 255.0) as u8,
        (s.green * 255.0) as u8,
        (s.blue * 255.0) as u8,
        255,
    ];
    let bar = [0u8, 0, 0, 255]; // black bars
    // Three thin (2 px) bars spread across the cell.
    let is_bar = |x: u32| matches!(x, 3 | 4 | 8 | 9 | 13 | 14);
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let rgba = if is_bar(x) { bar } else { base };
            let idx = ((y * W + x) * 4) as usize;
            pixels[idx..idx + 4].copy_from_slice(&rgba);
        }
    }
    images.add(make_image(W, H, pixels))
}

#[derive(Component)]
pub(crate) struct MinimapCamera;

#[derive(Component)]
pub(crate) struct MinimapPlayer;

#[derive(Component)]
pub(crate) struct MinimapBackground;

#[derive(Component)]
pub(crate) struct MinimapDimensions;

#[derive(Component)]
pub(crate) struct MinimapCell {
    pub(crate) dr: i32,
    pub(crate) dc: i32,
}

#[derive(Resource)]
pub(crate) struct MinimapConfig {
    pub(crate) center_x: f32,
    pub(crate) center_y: f32,
    /// Iron-fence bars texture (steel-teal + black vertical bars), swapped onto a
    /// cell's sprite when it shows an iron fence. `None` when no image assets are
    /// available (headless tests) — fenced cells fall back to a flat colour.
    pub(crate) iron_bars: Option<Handle<Image>>,
}

pub(crate) fn spawn_minimap(
    commands: &mut Commands,
    window: &Query<&Window>,
    config: &GameConfig,
    // `(rows, cols)` of the complete maze grid, shown in the minimap footer.
    dims: (usize, usize),
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    color_materials: &mut Option<ResMut<Assets<ColorMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
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

    let iron_bars = images.as_mut().map(|imgs| make_iron_bars_texture(imgs));
    commands.insert_resource(MinimapConfig { center_x, center_y, iron_bars });

    // Overlay camera — does not clear the colour buffer so the 3D scene shows through.
    let mut camera = commands.spawn((
        Camera2d,
        Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
        MinimapCamera,
    ));
    // Kept in step with the 3D camera: two views of one window disagreeing about
    // sample count is its own render cost, and would muddy any measurement taken
    // against the override.
    if let Some(msaa) = crate::render::msaa_override(config.msaa_samples) {
        camera.insert(msaa);
    }

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

    // Maze-dimensions footer ("{cols} x {rows}") on a dark strip directly below
    // the minimap. Both nodes are tagged `MinimapDimensions` so
    // `minimap_dimensions_resize_system` keeps them under the panel on resize.
    let (rows, cols) = dims;
    let strip_y = minimap_dimensions_y(center_y, map_size);
    commands.spawn((
        MinimapDimensions,
        Sprite {
            color: COLOR_MINIMAP_DIM_BG,
            custom_size: Some(Vec2::new(map_size + 4.0, MINIMAP_DIM_STRIP_H)),
            ..default()
        },
        Transform::from_xyz(center_x, strip_y, 8.8),
    ));
    commands.spawn((
        MinimapDimensions,
        Text2d::new(dimensions_label(rows, cols)),
        TextFont { font_size: MINIMAP_DIM_FONT, ..default() },
        TextColor(COLOR_MINIMAP_DIM_TEXT),
        Transform::from_xyz(center_x, strip_y, 9.0),
    ));
}

/// The minimap-footer label for a maze of `rows` × `cols` — rendered width ×
/// height, so a 6-row, 5-column grid reads "5 x 6".
pub(crate) fn dimensions_label(rows: usize, cols: usize) -> String {
    format!("{cols} x {rows}")
}

/// Keeps the dimensions footer in step with the level currently being played:
/// each level of a multi-level run can have a different footprint, so the readout
/// follows `state.grid` rather than staying on the bottom level's size. Updated
/// only when the label actually changes (i.e. on a level transition).
pub(crate) fn minimap_dimensions_update_system(
    state: Res<GameState>,
    mut text: Query<&mut Text2d, With<MinimapDimensions>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let label = dimensions_label(
        state.grid.len(),
        state.grid.first().map_or(0, |row| row.len()),
    );
    if text.0 != label {
        text.0 = label;
    }
}

/// The y of the dimensions strip: centred just below the minimap's dark
/// background (which is `map_size + 4` tall, centred on `center_y`), with a
/// small gap.
pub(crate) fn minimap_dimensions_y(center_y: f32, map_size: f32) -> f32 {
    center_y - (map_size + 4.0) / 2.0 - MINIMAP_DIM_GAP - MINIMAP_DIM_STRIP_H / 2.0
}

pub(crate) fn minimap_system(
    state: Res<GameState>,
    game_config: Res<GameConfig>,
    run: Res<MultiLevelRun>,
    config: Res<MinimapConfig>,
    mut cells: Query<(&MinimapCell, &mut Sprite)>,
    mut player_q: Query<&mut Transform, With<MinimapPlayer>>,
) {
    let pr = state.game.player_row() as i32;
    let pc = state.game.player_col() as i32;
    let nrows = state.grid.len() as i32;
    let ncols = if state.grid.is_empty() { 0 } else { state.grid[0].len() as i32 };
    // The wall type for the level currently shown — rolled per level under
    // `wall_type = "random"` so the minimap colours match the 3D walls.
    let level_wall_type = if game_config.wall_type_random {
        WallType::random_for_level(run.current_level, game_config.seed)
    } else {
        game_config.wall_type
    };

    for (cell, mut sprite) in &mut cells {
        let mr = pr + cell.dr;
        let mc = pc + cell.dc;
        let look = if mr < 0 || mc < 0 || mr >= nrows || mc >= ncols {
            // Outside grid boundary
            CellLook::Flat(COLOR_MINIMAP_OUTSIDE)
        } else {
            let (r, c) = (mr as usize, mc as usize);
            if !state.explored.contains(&(r, c)) {
                CellLook::Flat(COLOR_MINIMAP_DARK)
            } else {
                explored_cell_look(&state.grid, state.game.cell_entities(), level_wall_type, r, c)
            }
        };
        match look {
            // Flat cells use the default (white) texture tinted by the colour.
            CellLook::Flat(color) => {
                sprite.image = Handle::default();
                sprite.color = color;
            }
            // Iron fence swaps in the bars texture (which carries its own colour),
            // falling back to a flat colour when no image assets exist.
            CellLook::IronBars => match &config.iron_bars {
                Some(handle) => {
                    sprite.image = handle.clone();
                    sprite.color = Color::WHITE;
                }
                None => {
                    sprite.image = Handle::default();
                    sprite.color = COLOR_MINIMAP_WALL;
                }
            },
        }
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

pub(crate) fn minimap_dimensions_resize_system(
    window: Query<&Window>,
    game_config: Res<GameConfig>,
    mut last_size: Local<(f32, f32)>,
    mut dims: Query<&mut Transform, With<MinimapDimensions>>,
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
    let map_size = (radius * 2 + 1) as f32 * cell_px;
    let center_x = w / 2.0 - MAP_MARGIN - map_size / 2.0;
    let center_y = h / 2.0 - MAP_MARGIN - map_size / 2.0;
    let strip_y = minimap_dimensions_y(center_y, map_size);

    for mut t in &mut dims {
        t.translation.x = center_x;
        t.translation.y = strip_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(json: &str) -> CellEntity {
        serde_json::from_str(json).expect("valid cell-entity JSON")
    }

    #[test]
    fn dimensions_label_is_width_by_height() {
        // cols × rows: a 6-row, 5-column maze reads "5 x 6".
        assert_eq!(dimensions_label(6, 5), "5 x 6");
        assert_eq!(dimensions_label(10, 10), "10 x 10");
        assert_eq!(dimensions_label(1, 20), "20 x 1");
    }

    #[test]
    fn minimap_distinguishes_special_wall_types() {
        // A row of four 'W' cells: water / lava / iron-fence overrides + a plain
        // (solid) wall. Water/lava are distinct flat colours, iron-fence is the
        // barred look, and the plain wall keeps the neutral wall grey.
        let grid = vec![vec!['W', 'W', 'W', 'W']];
        let mut ce: HashMap<(usize, usize), Vec<CellEntity>> = HashMap::new();
        ce.insert((0, 0), vec![entity(r#"{"type":"W","wallType":"water"}"#)]);
        ce.insert((0, 1), vec![entity(r#"{"type":"W","wallType":"lava"}"#)]);
        ce.insert((0, 2), vec![entity(r#"{"type":"W","wallType":"iron_fence"}"#)]);
        // (0, 3) has no override → the level default (a solid wall here).
        let d = WallType::Brick;
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 0), CellLook::Flat(COLOR_MINIMAP_WATER));
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 1), CellLook::Flat(COLOR_MINIMAP_LAVA));
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 2), CellLook::IronBars);
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 3), CellLook::Flat(COLOR_MINIMAP_WALL));
        // A level whose rolled default is lava paints an un-overridden 'W' as lava.
        assert_eq!(
            explored_cell_look(&grid, &ce, WallType::Lava, 0, 3),
            CellLook::Flat(COLOR_MINIMAP_LAVA),
        );
    }

    #[test]
    fn minimap_colours_non_wall_cells_by_char() {
        let ce = HashMap::new();
        let grid = vec![vec!['S', 'F', ' ']];
        let d = WallType::Brick;
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 0), CellLook::Flat(COLOR_MINIMAP_START));
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 1), CellLook::Flat(COLOR_MINIMAP_FINISH));
        assert_eq!(explored_cell_look(&grid, &ce, d, 0, 2), CellLook::Flat(COLOR_MINIMAP_FLOOR));
    }
}
