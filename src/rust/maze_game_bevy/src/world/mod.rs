pub(crate) mod decorations;
pub(crate) mod floor;
pub(crate) mod gallery;
pub(crate) mod levels;
pub(crate) mod objects;
pub(crate) mod roof;
pub(crate) mod sky;
pub(crate) mod textures;
pub(crate) mod walls;

pub use levels::{generate_level_maze_jsons, LevelDifficultyChange, MAX_LEVEL_COUNT};

use crate::hud;
use crate::overlays::pause;
use crate::state::{
    GameClock, GameConfig, GameState, GridFacing, LayeredAlignment, MultiLevelRun, PendingLevels,
    PendingMazeJson, WallType,
};
use bevy::prelude::*;
use maze::{CellEntity, GenerationAlgorithm, Generator, GeneratorOptions, MazeGame, MazeGameOptions};
use std::collections::{HashMap, HashSet};

pub(crate) const CELL_SIZE: f32 = 2.0;
pub(crate) const HALF_CELL: f32 = CELL_SIZE / 2.0;
const EYE_HEIGHT: f32 = 1.7;

/// Vertical gap between stacked levels in a multi-level run. Equal to the wall
/// height so a level's floor sits exactly at the top of the walls of the level
/// below — on the ceiling for a roofed level. Level 0 is the bottom.
pub(crate) const LEVEL_HEIGHT: f32 = crate::world::walls::WALL_HEIGHT;

/// Maps a level-local Y coordinate into world space for the given level index:
/// `y + level * LEVEL_HEIGHT`. Used everywhere a level's geometry sets a Y — at
/// spawn and in the per-frame animation systems — so every level is built the
/// same way, just lifted by its offset. Level 0 is the identity.
pub(crate) fn world_y(level: usize, y: f32) -> f32 {
    y + level as f32 * LEVEL_HEIGHT
}

/// Where a level sits in world space: its index (for the Y lift) plus the X/Z
/// centring offset its grid gets under the run's [`LayeredAlignment`]. Threaded
/// through the spawn helpers in place of a bare `level`, so a level's geometry is
/// built the same way everywhere — local cell coordinates wrapped in
/// [`Self::world_x`] / [`Self::world_y`] / [`Self::world_z`]. The bottom (base)
/// level always has a zero X/Z offset, so single-level games and `Edge` stacks
/// are byte-identical to the pre-Step-8 render.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LevelPlacement {
    pub(crate) level: usize,
    offset_x: f32,
    offset_z: f32,
}

impl LevelPlacement {
    /// The placement for `level`, whose grid is `rows × cols`, stacked over a
    /// bottom level of `base_rows × base_cols` under `alignment`. `Edge` keeps the
    /// X/Z offset at zero (corner-aligned); `Centre` shifts a smaller grid in by
    /// half the size difference so it sits centred over the bottom. (`x` maps to
    /// columns, `z` to rows.)
    pub(crate) fn for_level(
        level: usize,
        rows: usize,
        cols: usize,
        base_rows: usize,
        base_cols: usize,
        alignment: LayeredAlignment,
    ) -> Self {
        let (offset_x, offset_z) = match alignment {
            LayeredAlignment::Edge => (0.0, 0.0),
            LayeredAlignment::Centre => (
                base_cols.saturating_sub(cols) as f32 / 2.0 * CELL_SIZE,
                base_rows.saturating_sub(rows) as f32 / 2.0 * CELL_SIZE,
            ),
        };
        Self {
            level,
            offset_x,
            offset_z,
        }
    }

    /// Maps a cell-local X (column centre) into world space for this level.
    pub(crate) fn world_x(&self, x: f32) -> f32 {
        x + self.offset_x
    }

    /// Maps a cell-local Y into world space for this level (the `LEVEL_HEIGHT` lift).
    pub(crate) fn world_y(&self, y: f32) -> f32 {
        world_y(self.level, y)
    }

    /// Maps a cell-local Z (row centre) into world space for this level.
    pub(crate) fn world_z(&self, z: f32) -> f32 {
        z + self.offset_z
    }

    /// World-space offset added to a ground-level camera position to lift + centre
    /// it onto this level (X/Z centring + the level's Y).
    pub(crate) fn camera_offset(&self) -> Vec3 {
        Vec3::new(self.offset_x, world_y(self.level, 0.0), self.offset_z)
    }
}

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
///
/// The argument list mirrors the JS host's `StartConfig` fields one-for-one
/// rather than introducing an intermediate parameter struct — the host already
/// destructures the JSON payload into local variables and forwarding them
/// positionally is the most direct mapping.
#[allow(clippy::too_many_arguments)]
pub fn generate_maze_json(
    rows: u32,
    cols: u32,
    seed: u64,
    min_solution_length: u32,
    door_count: u32,
    spare_doors: u32,
    spare_keys: u32,
    enemy_count: u32,
    health_count: u32,
    treasure_count: u32,
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
        spare_doors: Some(spare_doors as usize),
        spare_keys: Some(spare_keys as usize),
        enemy_count: Some(enemy_count as usize),
        health_count: Some(health_count as usize),
        treasure_count: Some(treasure_count as usize),
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
/// A bare treasure (`T` at `(3,3)`) sits in a dead-end off the start corridor —
/// playable from `cargo run` and demonstrating that treasure takes precedence
/// over the dead-end landmark prop. See `demo_grid_is_well_formed` for the
/// structural guarantees this layout upholds.
pub(crate) fn demo_grid() -> Vec<Vec<char>> {
    vec![
        vec!['W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W', 'W'],
        vec!['W', 'S', ' ', 'K', ' ', 'E', ' ', 'H', ' ', ' ', 'W'],
        vec!['W', ' ', 'W', 'W', 'W', ' ', 'W', 'W', 'W', 'D', 'W'],
        vec!['W', ' ', ' ', 'T', 'W', ' ', ' ', ' ', 'W', ' ', 'W'],
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

/// Advances a multi-level run to the next level on an interim finish: banks the
/// completed level's score + treasure into the run totals, builds the next
/// level's game (carrying the player's HP, and the whole bag when the run is
/// configured not to reset it), and resets the per-level view state to the new
/// level's start. The caller must ensure the run is not already on its final
/// level ([`MultiLevelRun::is_final`]).
///
/// This performs the *logical* swap only — the active game, grid, and player
/// camera move to the new level instantly. The stacked-world rendering and the
/// climb/transition animation are layered on separately.
pub(crate) fn advance_to_next_level(
    state: &mut GameState,
    run: &mut MultiLevelRun,
    config: &GameConfig,
) {
    // Bank the completed level's contribution to the run totals before the
    // live game is replaced.
    run.banked_score += state.game.score();
    merge_treasure(&mut run.carried_treasure, &state.game.collected_treasure());

    let next_index = run.current_level + 1;
    let carried_hp = state.game.hp();
    let carried_bag = state.game.bag().to_vec();

    let opts = MazeGameOptions {
        enemy_move_period_ms: Some(config.enemy_move_period_ms),
        enemy_damage: Some(config.enemy_damage),
        max_hp: Some(config.max_hp),
        starting_hp: Some(carried_hp),
    };
    let mut next_game = MazeGame::from_json_with_options(&run.levels[next_index], opts)
        .expect("multi-level run holds maze JSON produced by the generator");
    if !run.reset_bag_between_levels {
        next_game.seed_carried_bag(carried_bag);
    }

    let grid = next_game.grid().to_vec();
    let start_row = next_game.player_row();
    let start_col = next_game.player_col();
    let facing = initial_facing(&grid, start_row, start_col);
    let start_yaw = facing.to_yaw();
    let start_pos = camera_pos_for(start_row, start_col, start_yaw);

    let mut explored = HashSet::new();
    explore_cell(&mut explored, &grid, start_row, start_col);

    state.game = next_game;
    state.grid = grid;
    state.facing = facing;
    state.visual_pos = start_pos;
    state.visual_yaw = start_yaw;
    state.visual_pitch = 0.0;
    state.anim = None;
    state.explored = explored;
    state.damage_flash_timer = 0.0;
    // Lift + centre the camera onto the new level. The move animation stays in the
    // level's local frame; `movement_system` adds this offset when writing the
    // camera transform, so reaching an interim finish takes the player up onto the
    // next level (and centred over it under `Centre` alignment). A smooth climb
    // animation is a later refinement — this is the placement snap.
    let placement = LevelPlacement::for_level(
        next_index,
        state.grid.len(),
        state.grid.first().map_or(0, |row| row.len()),
        run.base_dims.0,
        run.base_dims.1,
        config.layered_alignment,
    );
    state.camera_offset = placement.camera_offset();
    run.current_level = next_index;
}

/// Folds a level's per-style treasure counts into the run's cumulative tally,
/// preserving the (ascending-value) order `collected_treasure` returns.
fn merge_treasure(acc: &mut Vec<(maze::TreasureStyle, u32)>, add: &[(maze::TreasureStyle, u32)]) {
    for &(style, count) in add {
        if count == 0 {
            continue;
        }
        if let Some(entry) = acc.iter_mut().find(|(s, _)| *s == style) {
            entry.1 += count;
        } else {
            acc.push((style, count));
        }
    }
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

/// Immutable per-session render assets shared by every level's geometry. Bundled
/// so [`spawn_level`] takes one reference instead of a long argument list.
struct LevelRenderAssets<'a> {
    wall: &'a walls::WallAssets,
    nonoccluding: &'a walls::NonOccludingAssets,
    floor: &'a floor::FloorAssets,
    decoration: &'a decorations::DecorationAssets,
    object: &'a objects::ObjectAssets,
    roof: &'a roof::RoofAssets,
}

/// Renders one level's full geometry at its `placement` (the level's Y lift +
/// X/Z centring offset, threaded through every spawn helper). `is_final` keeps the
/// finish orb only on the top level — interim finishes omit it (a transition rig
/// replaces it later). `is_live` marks the level whose enemies are driven by the
/// live `MazeGame` in `GameState`; every other level's enemies are static
/// scenery, so they get a non-matching id and `enemy_animation_system` leaves
/// them at their spawn pose.
#[allow(clippy::too_many_arguments)]
fn spawn_level(
    commands: &mut Commands,
    assets: &LevelRenderAssets,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    placement: LevelPlacement,
    is_final: bool,
    is_live: bool,
) {
    // Sparkle rays each treasure chest gets — the same count for every chest in
    // this level (so they look uniform), with the total bounded for treasure-dense
    // levels (the additive sparkle overdraw is what overwhelms a mobile GPU; the
    // per-chest point light is comparatively cheap and always kept). The global
    // across-levels budget is a later refinement; per level matches the
    // single-maze behaviour. See rays_per_chest.
    let treasure_rays =
        objects::treasure::rays_per_chest(grid.iter().flatten().filter(|&&ch| ch == 'T').count());
    // Row-major scan order matches `MazeGame`'s enemy-id assignment, so bumping
    // this per `'E'` keeps the live level's `EnemyMarker.id` aligned with the
    // runtime `maze::Enemy.id`.
    let mut enemy_id: u32 = 0;
    for (r, row) in grid.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            let cell_entity = cell_entities.get(&(r, c)).and_then(|v| v.first());
            if cell == 'W' {
                // A solid wall renders nothing itself — the adjacent open cell
                // draws the panel. A non-occluding wall (water / lava / iron
                // fence) is un-skipped: it renders its in-cell geometry plus the
                // panels facing any solid-wall neighbours (panels toward open /
                // non-occluding neighbours and the grid edge are suppressed in
                // `spawn_walls_for_cell`). Water / lava pools double as the floor;
                // the iron fence stands on a normal tile.
                let wall_type = objects::overrides::resolve_wall_type(cell_entity, config.wall_type);
                if !wall_type.is_non_occluding() {
                    continue;
                }
                walls::spawn_walls_for_cell(commands, assets.wall, grid, cell_entities, r, c, config, placement);
                walls::spawn_non_occluding_for_cell(commands, assets.nonoccluding, grid, cell_entities, config, wall_type, r, c, placement);
                if matches!(wall_type, WallType::IronFence) {
                    floor::tile::spawn_tile(commands, assets.floor, r, c, placement);
                }
                roof::spawn_roof_for_cell(commands, assets.roof, assets.wall, grid, r, c, config, placement);
                continue;
            }
            walls::spawn_walls_for_cell(commands, assets.wall, grid, cell_entities, r, c, config, placement);
            decorations::spawn_decorations_for_cell(commands, assets.decoration, grid, cell_entities, cell, r, c, config, placement);
            floor::spawn_floor_for_cell(commands, assets.floor, grid, cell, r, c, placement);
            // A static level's enemies never match a live runtime enemy, so they
            // get a non-matching id and stand frozen as scenery.
            let spawn_enemy_id = if is_live { enemy_id } else { u32::MAX };
            objects::spawn_objects_for_cell(commands, assets.object, grid, cell, r, c, config, cell_entity, spawn_enemy_id, treasure_rays, placement, is_final);
            if cell == 'E' {
                enemy_id += 1;
            }
            // Doors are spawned here (not inside `spawn_objects_for_cell`)
            // because the panel borrows the cell's wall material from
            // `wall_assets`.
            objects::door::spawn_door_for_cell(commands, &assets.object.door, assets.wall, &assets.decoration.wall, materials, grid, cell_entities, cell, r, c, config, cell_entity, placement);
            roof::spawn_roof_for_cell(commands, assets.roof, assets.wall, grid, r, c, config, placement);
        }
    }
}

/// Hand-built level set for the `MAZE_DEMO=multilevel_edge` / `multilevel_centre`
/// native demos — a walkable stack for verifying the stacked rendering under each
/// layer alignment. A **shrinking open-platform pyramid**: an open `9×9` platform
/// at the bottom (live), a `5×5` above it, a `3×3` on top — each a genuinely
/// smaller grid (not a padded one), so with the demo's open perimeter (see
/// `spawn_world`) every platform's edge shows sky instead of a wall and you can
/// look up past the lower platforms to the ones above. `multilevel_edge` stacks
/// the grids to a common corner; `multilevel_centre` centres each smaller grid
/// over the bottom level (a centred pyramid). The run's single finish **orb is on
/// the far corner of the top `3×3`**, in the open, so it reads from below.
/// Collectible cells are kept off each other's `(row, col)` across levels
/// (bottom's outside the upper footprints) so the live game's collection events
/// never disturb an upper level's matching marker. (Under `centre` the
/// start/finish cells aren't world-aligned vertically, so the camera hops across
/// as it snaps up — proper ladder-vertical placement is dedicated follow-on work.)
fn multilevel_demo_levels() -> Vec<String> {
    let build = |rows: &[&str]| -> Vec<Vec<char>> {
        rows.iter().map(|row| row.chars().collect()).collect()
    };
    // Bottom: 9×9 open platform (live). Climbs at F(2,2); objects sit OUTSIDE the
    // 5×5/3×3 upper footprints (rows/cols ≥ 5) so they never collide with an
    // upper level's marker cell.
    let bottom = build(&[
        "         ",
        "         ",
        "  F      ",
        "         ",
        "         ",
        "         ",
        "  K   E  ",
        "       S ",
        "    T    ",
    ]);
    // Middle: 5×5 open platform. S(2,2) sits above the bottom's F(2,2); F(1,1)
    // below the top's S. Health + treasure kept off the top's 3×3 footprint.
    let middle = build(&[
        "     ",
        " F  H",
        "  S  ",
        "     ",
        " T   ",
    ]);
    // Top: 3×3 open platform. S(1,1) sits above the middle's F(1,1); F(2,2) — the
    // orb — is the far corner, in the open, the easiest to spot from below.
    let top = build(&[
        "   ",
        " S ",
        "  F",
    ]);
    [bottom, middle, top]
        .iter()
        .map(|grid| grid_to_json(grid))
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_world(
    mut commands: Commands,
    pending: Res<PendingMazeJson>,
    pending_levels: Option<Res<PendingLevels>>,
    config: Res<GameConfig>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut color_materials: Option<ResMut<Assets<ColorMaterial>>>,
    mut images: Option<ResMut<Assets<Image>>>,
    window: Query<&Window>,
) {
    // Maze sources for the run. The bottom (live) level is `levels[0]`; any levels
    // above it are stacked as static geometry until the player climbs to them (the
    // run-state machine swaps the live game on a transition). The JS host supplies
    // one maze today — multi-level generation feeds in later — so the only
    // multi-level case here is the native `MAZE_DEMO=multilevel` hand-built stack
    // for verifying the stacked rendering. Generation failures are surfaced before
    // we ever reach here — see `generate_maze_json` and
    // `maze_game_bevy_wasm::start_with_config`.
    let game_opts = MazeGameOptions {
        enemy_move_period_ms: Some(config.enemy_move_period_ms),
        enemy_damage: Some(config.enemy_damage),
        max_hp: Some(config.max_hp),
        starting_hp: Some(config.starting_hp),
    };
    // `MAZE_DEMO=<focus>` native run — a rig showroom or the multi-level stack;
    // both relax the round timer so there's no pressure while inspecting. These
    // env demos are native-runtime only: under `cfg(test)` they are forced off so
    // a headless test always uses the maze it supplies (or the built-in demo
    // grid), regardless of a developer's shell `MAZE_DEMO` — otherwise running
    // `cargo test` with it set would swap the maze out from under the assertions.
    let gallery_focus = if cfg!(test) {
        None
    } else {
        gallery::requested_focus()
    };
    // A non-empty `PendingLevels` override supplies the whole run directly (a
    // multi-level host launch, or the rendering tests). Otherwise the native
    // `MAZE_DEMO=multilevel` env var selects the hand-built demo stack.
    let injected_levels = pending_levels
        .as_ref()
        .map(|p| p.0.clone())
        .filter(|levels| !levels.is_empty());
    // `MAZE_DEMO=multilevel_edge` / `multilevel_centre` both select the hand-built
    // demo stack (same grids); they differ only in the layer alignment, so each
    // alignment can be walked and verified. `Some(alignment)` when one is active.
    let multilevel_demo: Option<LayeredAlignment> = if cfg!(test)
        || injected_levels.is_some()
        || pending.0.is_some()
        || gallery_focus.is_some()
    {
        None
    } else {
        match std::env::var("MAZE_DEMO").as_deref() {
            Ok("multilevel_edge") => Some(LayeredAlignment::Edge),
            Ok("multilevel_centre") => Some(LayeredAlignment::Centre),
            _ => None,
        }
    };
    let levels: Vec<String> = if let Some(levels) = injected_levels {
        levels
    } else if let Some(json) = pending.0.as_deref() {
        vec![json.to_string()]
    } else if let Some(focus) = gallery_focus.as_deref() {
        // Native-only — the web/WASM path always supplies a maze via
        // `PendingMazeJson`. The gallery places every entity rig beside its default.
        vec![gallery::json(focus)]
    } else if multilevel_demo.is_some() {
        multilevel_demo_levels()
    } else {
        vec![grid_to_json(&demo_grid())]
    };

    // The multilevel demo opens the perimeter so the stack is genuinely
    // see-through (decision 8) and applies the demo's chosen layer alignment
    // (`edge` corner-stacks, `centre` centres each smaller level). Demo-only;
    // every other launch keeps its configured perimeter + alignment.
    let config: GameConfig = if let Some(alignment) = multilevel_demo {
        GameConfig {
            perimeter_walls: false,
            layered_alignment: alignment,
            ..(*config).clone()
        }
    } else {
        (*config).clone()
    };
    // Replace the resource with this (possibly demo-overridden) config so the rest
    // of the game agrees with the render — `advance_to_next_level` reads the
    // `GameConfig` resource to compute each level's camera placement, so it must
    // see the same `layered_alignment` the geometry was built with. A no-op for
    // non-demo launches (the value is an exact clone of the existing resource).
    commands.insert_resource(config.clone());

    // The bottom level is the live game in `GameState`; build it with the session
    // options. Its per-cell rig overrides (sparse) are cloned out before the game
    // is moved into `GameState`, for the spawn scan to pick per-cell rigs.
    let game = MazeGame::from_json_with_options(&levels[0], game_opts)
        .expect("maze JSON is host-validated or a hardcoded demo, so it always parses");
    let grid = game.grid().to_vec();
    let cell_entities = game.cell_entities().clone();

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
        damage_flash_timer: 0.0,
        // The player starts on the bottom level (placement offset zero); advancing
        // a level lifts + centres the camera (see `advance_to_next_level`).
        camera_offset: Vec3::ZERO,
    });

    // Timer comes from `GameConfig.timer_seconds`. The default (60 s, see
    // `GameConfig::default`) is what the no-config / demo path uses, so this
    // single source covers both the configured Play 3D session and the
    // fallback. The rig galleries get a long timer so there's no time pressure
    // while inspecting the rigs.
    commands.insert_resource(GameClock {
        remaining_secs: if gallery_focus.is_some() || multilevel_demo.is_some() {
            3600.0
        } else {
            config.timer_seconds.max(0.0)
        },
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
    let nonoccluding_assets =
        walls::build_non_occluding_assets(&mut meshes, &mut materials, &mut images);
    // The lava-steam emitter runs every frame and needs its wisp assets after
    // spawn_world returns, so they live in a resource rather than a local bundle.
    commands.insert_resource(walls::lava::build_lava_steam_assets(&mut meshes, &mut materials));
    let floor_assets = floor::build_floor_assets(&mut meshes, &mut materials, &mut images);
    let decoration_assets =
        decorations::build_decoration_assets(&mut meshes, &mut materials, &mut images);
    let object_assets = objects::build_object_assets(&mut meshes, &mut materials, &mut images);
    let roof_assets = roof::build_roof_assets(&mut meshes, &mut materials, &mut images, &config);

    // Render every level, stacked on the Y axis (and, under `Centre` alignment,
    // its smaller grids centred on X/Z). The bottom level (index 0) is live — its
    // enemies track the runtime `MazeGame` in `GameState`; every level above is
    // static geometry until the player climbs to it. Only the top (final) level
    // keeps the finish orb; interim finishes omit it (a transition rig replaces it
    // later). A single-level game is a one-element loop, so its render is unchanged.
    let level_assets = LevelRenderAssets {
        wall: &wall_assets,
        nonoccluding: &nonoccluding_assets,
        floor: &floor_assets,
        decoration: &decoration_assets,
        object: &object_assets,
        roof: &roof_assets,
    };
    // The bottom level's footprint is the reference the upper levels' `Centre`
    // offsets are measured against.
    let base_dims = (grid.len(), grid.first().map_or(0, |row| row.len()));
    let level_count = levels.len();
    for (level, level_json) in levels.iter().enumerate() {
        let is_final = level + 1 == level_count;
        if level == 0 {
            // The live level reuses the already-parsed grid + per-cell overrides.
            let placement = LevelPlacement::for_level(
                0,
                base_dims.0,
                base_dims.1,
                base_dims.0,
                base_dims.1,
                config.layered_alignment,
            );
            spawn_level(&mut commands, &level_assets, &mut materials, &grid, &cell_entities, &config, placement, is_final, true);
        } else {
            // Upper levels need only their grid + per-cell overrides for the static
            // geometry; the game options don't affect either, so parse without them.
            let level_game = MazeGame::from_json(level_json)
                .expect("multi-level maze JSON is host-validated or a hardcoded demo");
            let level_grid = level_game.grid().to_vec();
            let level_cells = level_game.cell_entities().clone();
            let placement = LevelPlacement::for_level(
                level,
                level_grid.len(),
                level_grid.first().map_or(0, |row| row.len()),
                base_dims.0,
                base_dims.1,
                config.layered_alignment,
            );
            spawn_level(&mut commands, &level_assets, &mut materials, &level_grid, &level_cells, &config, placement, is_final, false);
        }
    }

    hud::minimap::spawn_minimap(
        &mut commands,
        &window,
        &config,
        (grid.len(), grid.first().map_or(0, |row| row.len())),
        &mut meshes,
        &mut color_materials,
        &mut images,
    );
    hud::clock::spawn_clock_hud(&mut commands, &window);
    hud::score::spawn_score_hud(&mut commands, &window);
    hud::statusbar::spawn_statusbar(&mut commands, &window, &config);
    hud::bag::spawn_bag_hud(&mut commands, &window, &mut images);
    hud::hp::spawn_hp_hud(
        &mut commands,
        &window,
        &mut images,
        config.max_hp,
        config.starting_hp,
    );

    // Record the run state and spawn the level indicator (a no-op for a
    // single-level run). The bottom level's maze is already live in `GameState`;
    // `MultiLevelRun` holds every level's JSON plus the per-level totals + the
    // level index for the indicator and the win/transition decision.
    let mut run = MultiLevelRun::new(levels);
    // The native multilevel demos carry the bag forward between levels so the
    // carry behaviour is visible — the bottom level's key stays in the bag as you
    // climb. Every other run keeps the default (bag resets each level).
    if multilevel_demo.is_some() {
        run.reset_bag_between_levels = false;
    }
    hud::level::spawn_level_indicator(&mut commands, &window, &run);
    commands.insert_resource(run);

    pause::spawn_paused_overlay(&mut commands);
}

#[cfg(test)]
mod multi_level_tests {
    use super::{advance_to_next_level, camera_pos_for, explore_cell, initial_facing, merge_treasure};
    use crate::state::{GameConfig, GameState, MultiLevelRun};
    use maze::{Direction, MazeGame, TreasureStyle};
    use std::collections::HashSet;

    /// Builds a `GameState` from a maze JSON the same way `spawn_world` does,
    /// so `advance_to_next_level` can be exercised without a Bevy app.
    fn state_from(json: &str) -> GameState {
        let game = MazeGame::from_json(json).expect("valid maze JSON");
        let grid = game.grid().to_vec();
        let row = game.player_row();
        let col = game.player_col();
        let facing = initial_facing(&grid, row, col);
        let visual_yaw = facing.to_yaw();
        let visual_pos = camera_pos_for(row, col, visual_yaw);
        let mut explored = HashSet::new();
        explore_cell(&mut explored, &grid, row, col);
        GameState {
            game,
            grid,
            facing,
            visual_pos,
            visual_yaw,
            visual_pitch: 0.0,
            anim: None,
            explored,
            won: false,
            lost: false,
            paused: false,
            damage_flash_timer: 0.0,
            camera_offset: bevy::prelude::Vec3::ZERO,
        }
    }

    fn run_of(levels: &[&str], reset_bag: bool) -> MultiLevelRun {
        let mut run = MultiLevelRun::new(levels.iter().map(|s| s.to_string()).collect());
        run.reset_bag_between_levels = reset_bag;
        run
    }

    #[test]
    fn run_methods_report_count_finality_and_cumulative_score() {
        let mut run = run_of(&["a", "b"], true);
        run.banked_score = 10;
        assert_eq!(run.level_count(), 2);
        assert!(!run.is_final(), "level 0 of 2 is not final");
        assert_eq!(run.cumulative_score(5), 15);
        run.current_level = 1;
        assert!(run.is_final(), "level 1 of 2 is final");
    }

    #[test]
    fn advance_banks_score_and_swaps_to_the_next_level() {
        let l0 = r#"{"grid":[["S","K","F"]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut state = state_from(l0);
        state.game.move_player(Direction::Right); // collect the key → score 1
        assert_eq!(state.game.score(), 1);

        let mut run = run_of(&[l0, l1], true);
        advance_to_next_level(&mut state, &mut run, &GameConfig::default());

        assert_eq!(run.current_level, 1);
        assert_eq!(run.banked_score, 1, "the completed level's score is banked");
        assert_eq!(state.grid, vec![vec!['S', ' ', 'F']], "swapped to level 1's grid");
        assert_eq!((state.game.player_row(), state.game.player_col()), (0, 0));
        // Same-footprint levels → no X/Z centring, just the Y lift onto level 1.
        assert_eq!(
            state.camera_offset,
            bevy::prelude::Vec3::new(0.0, crate::world::LEVEL_HEIGHT, 0.0),
            "the camera is lifted onto level 1",
        );
    }

    #[test]
    fn advance_resets_the_bag_by_default() {
        let l0 = r#"{"grid":[["S","K","F"]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut state = state_from(l0);
        state.game.move_player(Direction::Right); // bag now holds the key
        assert_eq!(state.game.bag().len(), 1);

        let mut run = run_of(&[l0, l1], true); // reset_bag = true
        advance_to_next_level(&mut state, &mut run, &GameConfig::default());
        assert!(state.game.bag().is_empty(), "default resets the bag each level");
    }

    #[test]
    fn advance_carries_the_bag_when_configured() {
        let l0 = r#"{"grid":[["S","K","F"]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut state = state_from(l0);
        state.game.move_player(Direction::Right); // bag now holds the key

        let mut run = run_of(&[l0, l1], false); // reset_bag = false → carry
        advance_to_next_level(&mut state, &mut run, &GameConfig::default());
        assert_eq!(state.game.bag().len(), 1, "the carried bag seeds the next level");
    }

    #[test]
    fn advance_folds_collected_treasure_into_the_run_tally() {
        let l0 = r#"{"grid":[["S","T","F"]]}"#; // bare 'T' → Silver, value 50
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut state = state_from(l0);
        state.game.move_player(Direction::Right); // collect the treasure

        let mut run = run_of(&[l0, l1], true);
        advance_to_next_level(&mut state, &mut run, &GameConfig::default());

        assert_eq!(run.carried_treasure, vec![(TreasureStyle::Silver, 1)]);
        assert_eq!(run.banked_score, 50, "silver's reward value is banked");
    }

    #[test]
    fn level_placement_offsets_match_alignment() {
        use crate::state::LayeredAlignment;
        use crate::world::{LevelPlacement, CELL_SIZE, LEVEL_HEIGHT};
        use bevy::prelude::Vec3;

        // Edge: never any X/Z offset, just the Y lift.
        let edge = LevelPlacement::for_level(2, 5, 5, 9, 9, LayeredAlignment::Edge);
        assert_eq!(edge.world_x(1.0), 1.0);
        assert_eq!(edge.world_z(1.0), 1.0);
        assert_eq!(edge.camera_offset(), Vec3::new(0.0, 2.0 * LEVEL_HEIGHT, 0.0));

        // Centre: a 5×5 grid centred in a 9×9 base shifts in by (9-5)/2 = 2 cells.
        let centre = LevelPlacement::for_level(1, 5, 5, 9, 9, LayeredAlignment::Centre);
        let shift = 2.0 * CELL_SIZE;
        assert_eq!(centre.world_x(1.0), 1.0 + shift);
        assert_eq!(centre.world_z(1.0), 1.0 + shift);
        assert_eq!(centre.camera_offset(), Vec3::new(shift, LEVEL_HEIGHT, shift));

        // The base level (same dims) has zero X/Z offset under either mode.
        let base = LevelPlacement::for_level(0, 9, 9, 9, 9, LayeredAlignment::Centre);
        assert_eq!(base.camera_offset(), Vec3::ZERO);
    }

    #[test]
    fn advance_under_centre_alignment_centres_the_camera_over_a_smaller_level() {
        use crate::state::LayeredAlignment;
        use crate::world::{CELL_SIZE, LEVEL_HEIGHT};
        use bevy::prelude::Vec3;
        // Bottom 1×5, next level 1×3 (smaller). Under `Centre`, the 1×3 is shifted
        // in by (5-3)/2 = 1 cell in X (cols); rows match, so no Z shift.
        let l0 = r#"{"grid":[["S"," "," "," ","F"]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut state = state_from(l0);
        let mut run = run_of(&[l0, l1], true);
        let config = GameConfig {
            layered_alignment: LayeredAlignment::Centre,
            ..GameConfig::default()
        };
        advance_to_next_level(&mut state, &mut run, &config);
        assert_eq!(state.camera_offset, Vec3::new(CELL_SIZE, LEVEL_HEIGHT, 0.0));
    }

    #[test]
    fn multilevel_demo_levels_taper() {
        // The native `MAZE_DEMO=multilevel` stack: at least two levels, each a
        // parseable maze with its own start + finish, and a strictly shrinking
        // (square) grid as you climb — the open-platform pyramid.
        let levels = super::multilevel_demo_levels();
        assert!(levels.len() >= 2, "the demo stacks at least two levels");
        let mut prev_dim: Option<usize> = None;
        for json in &levels {
            let game = MazeGame::from_json(json).expect("each demo level parses");
            let grid = game.grid();
            assert_eq!(grid.len(), grid[0].len(), "each demo level is square");
            let dim = grid.len();
            if let Some(prev) = prev_dim {
                assert!(dim < prev, "each level up is a strictly smaller grid ({dim} < {prev})");
            }
            prev_dim = Some(dim);
        }
    }

    #[test]
    fn merge_treasure_sums_by_style_and_skips_zero_counts() {
        let mut acc = vec![(TreasureStyle::Silver, 1)];
        merge_treasure(
            &mut acc,
            &[
                (TreasureStyle::Silver, 2),
                (TreasureStyle::Gold, 1),
                (TreasureStyle::Diamonds, 0),
            ],
        );
        assert_eq!(acc, vec![(TreasureStyle::Silver, 3), (TreasureStyle::Gold, 1)]);
    }
}
