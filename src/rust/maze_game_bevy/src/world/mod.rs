pub(crate) mod decorations;
pub(crate) mod floor;
pub(crate) mod objects;
pub(crate) mod roof;
pub(crate) mod sky;
pub(crate) mod textures;
pub(crate) mod walls;

use crate::hud;
use crate::overlays::pause;
use crate::state::{GameClock, GameConfig, GameState, GridFacing, PendingMazeJson};
use bevy::prelude::*;
use maze::{GenerationAlgorithm, Generator, GeneratorOptions, MazeGame};
use std::collections::HashSet;

pub(crate) const CELL_SIZE: f32 = 2.0;
pub(crate) const HALF_CELL: f32 = CELL_SIZE / 2.0;
const EYE_HEIGHT: f32 = 1.7;

/// How far back from the cell centre — in the direction OPPOSITE the
/// player's current facing — the camera is positioned. A non-zero shift
/// lets the player see further into the cell ahead and brings
/// perpendicular openings (left/right corridors) into a glancing angle
/// inside the camera's FOV instead of sitting at 90° off-axis. With
/// `CELL_SIZE = 2.0` the back wall sits at `HALF_CELL - WALL_THICKNESS/2
/// ≈ 0.975` from centre, so 0.7 leaves ~0.275 clearance behind the
/// camera — still well clear of the near plane.
pub(crate) const CAMERA_EDGE_OFFSET: f32 = 0.7;

/// Vertical field of view (radians) for the player camera at the
/// reference aspect ratio. Bevy's default `PerspectiveProjection` uses
/// π/4 ≈ 45° vertical, which gives ~73° horizontal at 16:9 — too
/// narrow for perpendicular openings to register without turning the
/// head. π/3 ≈ 60° vertical gives ~91° horizontal at 16:9, the
/// FPS-typical range. On viewports narrower than the reference
/// (e.g. phone portrait, tall windows) the vertical FOV grows past
/// this value to keep the horizontal FOV constant — see
/// [`camera_fov_for_aspect`].
pub(crate) const CAMERA_FOV_VERTICAL_RADIANS: f32 = std::f32::consts::PI / 3.0;

/// Reference viewport aspect ratio (width / height) at which the
/// vertical FOV equals [`CAMERA_FOV_VERTICAL_RADIANS`]. Below this
/// aspect, the vertical FOV grows so the horizontal FOV stays constant
/// — classic "Hor+" scaling. 16:9 is the desktop / landscape phone
/// reference.
pub(crate) const CAMERA_FOV_REFERENCE_ASPECT: f32 = 16.0 / 9.0;

/// Upper bound on the vertical FOV used by the Hor+ scaling. Without
/// a cap, a phone-portrait viewport (~9:19.5) would need ~130°
/// vertical to keep the reference horizontal FOV — fisheye territory
/// that reads as queasy at the screen edges. The cap accepts some
/// horizontal-FOV loss on very narrow viewports in exchange for a
/// watchable scene. 100° is wide-but-not-fisheye.
pub(crate) const CAMERA_FOV_VERTICAL_MAX_RADIANS: f32 =
    std::f32::consts::PI * 100.0 / 180.0;

/// Returns the vertical FOV (radians) the camera should use at the
/// given viewport aspect ratio (`width / height`). Wide viewports get
/// the base vertical FOV unchanged (Bevy widens horizontal FOV
/// naturally). Narrow viewports get a grown vertical FOV that keeps
/// the horizontal FOV equal to the value seen at
/// [`CAMERA_FOV_REFERENCE_ASPECT`], capped at
/// [`CAMERA_FOV_VERTICAL_MAX_RADIANS`].
///
/// Math: for a perspective projection,
/// `tan(h_fov / 2) = tan(v_fov / 2) · aspect`. Hold the LHS constant at
/// its value at the reference aspect and solve for the new v_fov.
pub(crate) fn camera_fov_for_aspect(aspect: f32) -> f32 {
    if aspect <= 0.0 || aspect >= CAMERA_FOV_REFERENCE_ASPECT {
        return CAMERA_FOV_VERTICAL_RADIANS;
    }
    let ref_half_h_tan = (CAMERA_FOV_VERTICAL_RADIANS * 0.5).tan() * CAMERA_FOV_REFERENCE_ASPECT;
    let target_v_fov = 2.0 * (ref_half_h_tan / aspect).atan();
    target_v_fov.min(CAMERA_FOV_VERTICAL_MAX_RADIANS)
}

/// Generates a maze using the seeded `maze::Generator` and returns its grid
/// serialised as the JSON form `MazeGame::from_json` accepts (`{"grid":[…]}`).
///
/// Intended for the JS host (`maze_game_bevy_wasm::start_with_config`) so that
/// generation failures surface up the call stack — and into the browser's
/// `#loading` error overlay — *before* Bevy enters `AppState::Playing`, where
/// the rest of the game systems would otherwise panic on missing resources.
pub fn generate_maze_json(
    rows: u32,
    cols: u32,
    seed: u64,
    min_solution_length: u32,
    door_count: u32,
) -> Result<String, String> {
    let options = GeneratorOptions {
        row_count: rows as usize,
        col_count: cols as usize,
        algorithm: GenerationAlgorithm::RecursiveBacktracking,
        start: None,
        finish: None,
        min_spine_length: Some(min_solution_length as usize),
        max_retries: None,
        branch_from_finish: None,
        seed: Some(seed),
        door_count: Some(door_count as usize),
        spare_doors: None,
        spare_keys: None,
    };
    let maze = Generator { options }
        .generate()
        .map_err(|err| err.to_string())?;
    Ok(grid_to_json(&maze.definition.grid))
}

/// Built-in fallback maze for the native binary and the bare wasm `start()`
/// path (the React `/game/` flow always supplies a real maze). An 11×11
/// perfect maze chosen to exercise the full feature set: a key (`K` at `(1,3)`)
/// sits along a corridor, the real path door (`D` at `(8,9)`) is the *only*
/// cell adjacent to the finish so it gates `F` outright, a **decoy** door
/// (`D` at `(2,9)`) hangs off the top-right branch (so a player tempted to
/// burn their only key on it strands themselves), and several further
/// dead-ends pick up landmark objects (brazier / urn / pillar / chest). The
/// intended solve is: collect the key, navigate the spine down and east,
/// then hold against the real door to open it before reaching the finish.
/// See `demo_grid_is_well_formed` for the structural guarantees this layout
/// upholds.
pub(crate) fn demo_grid() -> Vec<Vec<char>> {
    vec![
        vec!['W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W'],
        vec!['W', 'S', ' ', 'K', ' ', ' ', ' ', ' ', ' ', ' ', 'W'],
        vec!['W', ' ', 'W', 'W', 'W', ' ', 'W', 'W', 'W', 'D', 'W'],
        vec!['W', ' ', ' ', ' ', 'W', ' ', ' ', ' ', 'W', ' ', 'W'],
        vec!['W', ' ', 'W', 'W', 'W', ' ', 'W', 'W', 'W', 'W', 'W'],
        vec!['W', ' ', 'W', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 'W'],
        vec!['W', ' ', 'W', 'W', 'W', ' ', 'W', 'W', 'W', 'W', 'W'],
        vec!['W', ' ', ' ', ' ', 'W', ' ', ' ', ' ', 'W', 'F', 'W'],
        vec!['W', ' ', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'D', 'W'],
        vec!['W', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 'W'],
        vec!['W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W'],
    ]
}

pub(crate) fn grid_to_json(grid: &[Vec<char>]) -> String {
    let rows: Vec<String> = grid
        .iter()
        .map(|row| {
            let cols: Vec<String> = row.iter().map(|c| format!("\"{}\"", c)).collect();
            format!("[{}]", cols.join(","))
        })
        .collect();
    format!("{{\"grid\":[{}]}}", rows.join(","))
}

pub(crate) fn lcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 32) as f32) / 4294967296.0
}

/// Cycles S→E→N→W and returns the first direction with an open neighbour.
pub(crate) fn initial_facing(grid: &[Vec<char>], row: usize, col: usize) -> GridFacing {
    let rows = grid.len() as isize;
    let cols = if grid.is_empty() { 0 } else { grid[0].len() as isize };
    let r = row as isize;
    let c = col as isize;
    let open = |dr: isize, dc: isize| -> bool {
        let (nr, nc) = (r + dr, c + dc);
        nr >= 0 && nc >= 0 && nr < rows && nc < cols && grid[nr as usize][nc as usize] != 'W'
    };
    if open(1, 0) {
        return GridFacing::South;
    }
    if open(0, 1) {
        return GridFacing::East;
    }
    if open(-1, 0) {
        return GridFacing::North;
    }
    if open(0, -1) {
        return GridFacing::West;
    }
    GridFacing::South
}

pub(crate) fn cell_centre(row: usize, col: usize) -> Vec3 {
    Vec3::new(
        col as f32 * CELL_SIZE + 1.0,
        EYE_HEIGHT,
        row as f32 * CELL_SIZE + 1.0,
    )
}

/// Camera world position for a player at `(row, col)` looking along
/// `yaw`. Returns the cell centre shifted by [`CAMERA_EDGE_OFFSET`] in
/// the direction OPPOSITE the camera's forward, so the camera sits near
/// the back edge of the cell relative to its current facing.
///
/// Bevy's default camera forward is `-Z`; after `Quat::from_rotation_y(yaw)`
/// the forward becomes `(-sin(yaw), 0, -cos(yaw))`, so the back-vector
/// is `(sin(yaw), 0, cos(yaw))`.
pub(crate) fn camera_pos_for(row: usize, col: usize, yaw: f32) -> Vec3 {
    cell_centre(row, col) + Vec3::new(yaw.sin(), 0.0, yaw.cos()) * CAMERA_EDGE_OFFSET
}

pub(crate) fn explore_cell(
    explored: &mut HashSet<(usize, usize)>,
    grid: &[Vec<char>],
    row: usize,
    col: usize,
) {
    explore_cell_raw(
        explored,
        grid.len(),
        if grid.is_empty() { 0 } else { grid[0].len() },
        row,
        col,
    );
}

pub(crate) fn explore_cell_raw(
    explored: &mut HashSet<(usize, usize)>,
    nrows: usize,
    ncols: usize,
    row: usize,
    col: usize,
) {
    explored.insert((row, col));
    if row > 0 {
        explored.insert((row - 1, col));
    }
    if row + 1 < nrows {
        explored.insert((row + 1, col));
    }
    if col > 0 {
        explored.insert((row, col - 1));
    }
    if col + 1 < ncols {
        explored.insert((row, col + 1));
    }
}

fn spawn_camera(commands: &mut Commands, start_pos: Vec3, start_yaw: f32) {
    commands.spawn((
        Camera3d::default(),
        // Explicit Projection so we can override Bevy's narrow default
        // vertical FOV — see CAMERA_FOV_VERTICAL_RADIANS for rationale.
        // The aspect-dependent FOV adjustment lives in
        // `camera_fov_resize_system` (Hor+ scaling) so this initial
        // value is just the wide-viewport value.
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_VERTICAL_RADIANS,
            ..default()
        }),
        Transform::from_translation(start_pos).with_rotation(Quat::from_rotation_y(start_yaw)),
    ));
}

/// Each frame, set the camera's vertical FOV from the current window
/// aspect via [`camera_fov_for_aspect`]. Bevy's `PerspectiveProjection`
/// is locked to vertical FOV — its horizontal is derived from
/// `vfov × aspect_ratio` — so on a narrow viewport the horizontal view
/// shrinks. This system implements Hor+ scaling: at aspects narrower
/// than [`CAMERA_FOV_REFERENCE_ASPECT`] it grows the vertical FOV to
/// preserve the horizontal FOV, capped at
/// [`CAMERA_FOV_VERTICAL_MAX_RADIANS`].
pub(crate) fn camera_fov_resize_system(
    windows: Query<&Window>,
    mut cameras: Query<&mut Projection, With<Camera3d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (w, h) = (window.resolution.width(), window.resolution.height());
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let target_fov = camera_fov_for_aspect(w / h);
    for mut projection in cameras.iter_mut() {
        if let Projection::Perspective(persp) = &mut *projection {
            if (persp.fov - target_fov).abs() > 1e-4 {
                persp.fov = target_fov;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_world(
    mut commands: Commands,
    pending: Res<PendingMazeJson>,
    config: Res<GameConfig>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut color_materials: Option<ResMut<Assets<ColorMaterial>>>,
    mut images: Option<ResMut<Assets<Image>>>,
    window: Query<&Window>,
) {
    // Maze source: either the JS host pre-generated / supplied JSON via
    // `PendingMazeJson` (the `/game/?id=…` or `/game/?difficulty=…` paths), or
    // we fall back to the built-in demo grid (the native / no-config path).
    // Generation failures are surfaced before we ever reach here — see
    // `generate_maze_json` and `maze_game_bevy_wasm::start_with_config`.
    let (game, grid) = match pending.0.as_deref() {
        Some(json) => {
            let game = MazeGame::from_json(json).expect("maze JSON was validated by the REST API");
            let grid = game.grid().to_vec();
            (game, grid)
        }
        None => {
            let grid = demo_grid();
            let json = grid_to_json(&grid);
            (
                MazeGame::from_json(&json).expect("demo grid is hardcoded and always valid"),
                grid,
            )
        }
    };

    let start_row = game.player_row();
    let start_col = game.player_col();
    let facing = initial_facing(&grid, start_row, start_col);
    let start_yaw = facing.to_yaw();
    let start_pos = camera_pos_for(start_row, start_col, start_yaw);

    let mut explored = HashSet::new();
    explore_cell(&mut explored, &grid, start_row, start_col);

    commands.insert_resource(GameState {
        game,
        grid: grid.clone(),
        facing,
        visual_pos: start_pos,
        visual_yaw: start_yaw,
        visual_pitch: 0.0,
        anim: None,
        explored,
        won: false,
        lost: false,
        paused: false,
        can_pickup: false,
    });

    // Timer comes from `GameConfig.timer_seconds`. The default (60 s, see
    // `GameConfig::default`) is what the no-config / demo path uses, so this
    // single source covers both the configured Play 3D session and the
    // fallback.
    commands.insert_resource(GameClock {
        remaining_secs: config.timer_seconds.max(0.0),
        elapsed_secs: 0.0,
        last_displayed_secs: -1,
    });

    spawn_camera(&mut commands, start_pos, start_yaw);
    sky::spawn_sky(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        &config,
    );

    let wall_assets = walls::build_wall_assets(&mut meshes, &mut materials, &mut images);
    let floor_assets = floor::build_floor_assets(&mut meshes, &mut materials, &mut images);
    let decoration_assets =
        decorations::build_decoration_assets(&mut meshes, &mut materials, &mut images);
    let object_assets = objects::build_object_assets(&mut meshes, &mut materials);
    let roof_assets = roof::build_roof_assets(&mut meshes, &mut materials, &mut images, &config);

    for (r, row) in grid.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == 'W' {
                continue;
            }
            walls::spawn_walls_for_cell(&mut commands, &wall_assets, &grid, r, c, &config);
            decorations::spawn_decorations_for_cell(
                &mut commands,
                &decoration_assets,
                &grid,
                cell,
                r,
                c,
                &config,
            );
            floor::spawn_floor_for_cell(&mut commands, &floor_assets, &grid, cell, r, c);
            objects::spawn_objects_for_cell(
                &mut commands,
                &object_assets,
                &grid,
                cell,
                r,
                c,
                &config,
            );
            // Doors are spawned here (not inside `spawn_objects_for_cell`)
            // because the panel borrows the cell's wall material from
            // `wall_assets`.
            objects::door::spawn_door_for_cell(
                &mut commands,
                &object_assets.door,
                &wall_assets,
                &decoration_assets.wall,
                &mut materials,
                &grid,
                cell,
                r,
                c,
                &config,
            );
            roof::spawn_roof_for_cell(&mut commands, &roof_assets, &wall_assets, &grid, r, c, &config);
        }
    }

    hud::minimap::spawn_minimap(
        &mut commands,
        &window,
        &config,
        &mut meshes,
        &mut color_materials,
    );
    hud::clock::spawn_clock_hud(&mut commands, &window);
    hud::statusbar::spawn_statusbar(&mut commands, &window, &config);
    hud::bag::spawn_bag_hud(&mut commands, &window, &mut images);
    pause::spawn_paused_overlay(&mut commands);
}
