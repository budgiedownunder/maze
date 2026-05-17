pub(crate) mod decorations;
pub(crate) mod floor;
pub(crate) mod objects;
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
    };
    let maze = Generator { options }
        .generate()
        .map_err(|err| err.to_string())?;
    Ok(grid_to_json(&maze.definition.grid))
}

pub(crate) fn demo_grid() -> Vec<Vec<char>> {
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
        Transform::from_translation(start_pos).with_rotation(Quat::from_rotation_y(start_yaw)),
    ));
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
        visual_pitch: 0.0,
        anim: None,
        explored,
        won: false,
        lost: false,
        paused: false,
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
    pause::spawn_paused_overlay(&mut commands);
}
