mod hud;
mod images;
mod movement;
mod outcome;
mod overlays;
mod palette;
mod state;
mod tick;
mod transition;
mod world;

pub use state::{
    DoorStyle, EnemyType, FinishType, GameConfig, GameOutcome, GameResult, HealthStyle,
    KeyHolderStyle, Landmarks, LayeredAlignment, PendingLevels, SkyType, TreasureStyle, WallType,
};
pub use world::gallery::validate_demo_env;
pub use world::{
    generate_level_maze_jsons, generate_maze_json, LevelDifficultyChange, MAX_LEVEL_COUNT,
};

use bevy::prelude::*;

pub fn build_app(app: &mut App, maze_json: Option<&str>) {
    use crate::hud::{bag, clock, hp, level, minimap, score, statusbar};
    use crate::movement::{movement_system, quit_system};
    use crate::outcome::outcome_watcher_system;
    use crate::overlays::{lose, pause, title, win};
    use crate::state::{AppState, PendingMazeJson, TitleTimer};
    use crate::tick::{damage_flash_system, game_tick_system};
    use crate::world::{
        objects::{
            self,
            common::brazier::brazier_flicker_system,
            door::door_animation_system,
            enemy::{despawn_completed_level_enemies_system, enemy_animation_system, ghost::ghost_hem_wave_system},
            health::health_animation_system,
            key_holder::{key_collection_system, key_holder_system, key_sparks_system},
            treasure::{treasure_collection_system, treasure_sparkle_system},
        },
        sky, spawn_world,
        walls::{
            lava::{lava_animation_system, lava_steam_system},
            water::water_animation_system,
        },
    };

    // `GameConfig` is the seam the JS host uses (via
    // `maze_game_bevy_wasm::start_with_config`) to drive difficulty / timer /
    // splash title / seed. `init_resource` only inserts the default when the
    // caller didn't already supply one, so a host-provided config is
    // preserved.
    app.init_resource::<GameConfig>();
    app.insert_resource(PendingMazeJson(maze_json.map(String::from)))
        .init_state::<AppState>()
        .insert_resource(TitleTimer(Timer::from_seconds(3.0, TimerMode::Once)))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(AppState::TitleScreen), title::setup_title)
        .add_systems(Update, title::tick_title.run_if(in_state(AppState::TitleScreen)))
        .add_systems(Update, title::update_title_countdown.run_if(in_state(AppState::TitleScreen)))
        .add_systems(Update, title::title_resize_system.run_if(in_state(AppState::TitleScreen)))
        .add_systems(OnExit(AppState::TitleScreen), title::teardown_title)
        .add_systems(OnEnter(AppState::Playing), spawn_world)
        .add_systems(Update, movement_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, crate::transition::transition_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, crate::transition::transition_fx_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, world::floor::hatch::hatch_close_watcher.run_if(in_state(AppState::Playing)))
        .add_systems(Update, world::floor::hatch::hatch_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, outcome_watcher_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, win::win_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, win::leaf_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, clock::tick_clock_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, clock::clock_text_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, clock::clock_flash_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, statusbar::statusbar_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, lose::lose_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, lose::rain_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, lose::lightning_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, minimap::minimap_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, minimap::minimap_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, minimap::minimap_dimensions_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, minimap::minimap_dimensions_update_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, objects::finish::orb::orb_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, objects::finish::portal::portal_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, brazier_flicker_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, key_holder_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, key_sparks_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, key_collection_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, treasure_sparkle_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, treasure_collection_system.run_if(in_state(AppState::Playing)))
        // The single game-state tick driver runs in `FixedUpdate` for
        // deterministic, frame-rate independent stepping (doors, enemies,
        // HP arithmetic). Per-entity animation systems read the resulting
        // state in `Update` for smooth per-frame motion.
        .add_systems(FixedUpdate, game_tick_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, door_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, enemy_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, despawn_completed_level_enemies_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, ghost_hem_wave_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, health_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, water_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, lava_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, lava_steam_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, bag::bag_hud_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, hp::hp_hud_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, score::score_hud_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, level::level_indicator_system.run_if(in_state(AppState::Playing)))
        // Damage flash runs through pause/lost so an in-flight flash from
        // the last live tick finishes fading rather than freezing on screen.
        .add_systems(Update, damage_flash_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, pause::pause_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, sky::sky_dome_follow_camera.run_if(in_state(AppState::Playing)))
        .add_systems(Update, world::camera_fov_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, quit_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlays::title::TitleEntity;
    use crate::state::{AppState, GameState, GridFacing, SkyType, TreasureStyle, WallType};
    use crate::world::{
        camera_fov_for_aspect, camera_pos_for, cell_centre,
        decorations::{floor::FloorAccent, wall::WallDecoration},
        demo_grid,
        floor::FloorCell,
        initial_facing,
        objects::{
            common::brazier::BrazierBowl,
            dead_end::DeadEndObject,
            door::DoorMarker,
            enemy::EnemyMarker,
            finish::{ladder::FinishLadder, orb::FinishOrb, portal::FinishPortal},
            health::HealthMarker,
            key_holder::KeyMarker,
            treasure::{TreasureLoot, TreasureMarker},
        },
        roof::RoofCell,
        sky::dome::SkyDome,
        walls::{
            iron_fence::IronFenceBars,
            lava::{LavaRock, LavaSurface},
            rim::PoolRim,
            water::WaterSurface,
            WallCell,
        },
        CAMERA_EDGE_OFFSET, CAMERA_FOV_REFERENCE_ASPECT, CAMERA_FOV_VERTICAL_MAX_RADIANS,
        CAMERA_FOV_VERTICAL_RADIANS,
    };
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

    fn make_playing_app_with(maze_json: &str) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app, Some(maze_json));
        app.update(); // OnEnter(TitleScreen) runs
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update(); // OnExit(TitleScreen) + OnEnter(Playing) run
        app
    }

    /// A playing app with both a custom maze JSON and a custom `GameConfig`
    /// (`build_app`'s `init_resource` keeps the pre-inserted config).
    fn make_playing_app_with_maze_and_config(maze_json: &str, config: GameConfig) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(config);
        build_app(&mut app, Some(maze_json));
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app
    }

    /// A playing app whose run is the given stacked level set (bottom level
    /// first), supplied via the `PendingLevels` override so the multi-level
    /// rendering path can be exercised headlessly without the native env var.
    fn make_playing_app_with_levels(levels: &[&str]) -> App {
        make_playing_app_with_levels_and_config(levels, GameConfig::default())
    }

    /// As [`make_playing_app_with_levels`] but with a custom `GameConfig` (e.g. a
    /// `layered_alignment`), so the alignment-dependent rendering can be checked.
    fn make_playing_app_with_levels_and_config(levels: &[&str], config: GameConfig) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(config);
        app.insert_resource(crate::state::PendingLevels(
            levels.iter().map(|s| s.to_string()).collect(),
        ));
        build_app(&mut app, None);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
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
    fn minimap_dimensions_readout_matches_demo_grid() {
        use crate::hud::minimap::MinimapDimensions;
        let mut app = make_playing_app();
        let grid = demo_grid();
        let expected = format!("{} x {}", grid[0].len(), grid.len());
        let labels: Vec<String> = app
            .world_mut()
            .query_filtered::<&Text2d, With<MinimapDimensions>>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert_eq!(labels, vec![expected], "one dimensions readout, cols x rows");
    }

    #[test]
    fn minimap_dimensions_readout_follows_the_active_level() {
        use crate::hud::minimap::MinimapDimensions;
        use crate::state::{GameConfig, GameState, MultiLevelRun};
        // Two differently-sized levels: 1×2 ("2 x 1") then 1×3 ("3 x 1").
        let l0 = r#"{"grid":[["S","F"]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[l0, l1]);

        let read = |app: &mut App| -> String {
            app.world_mut()
                .query_filtered::<&Text2d, With<MinimapDimensions>>()
                .iter(app.world())
                .next()
                .map(|t| t.0.clone())
                .unwrap_or_default()
        };
        assert_eq!(read(&mut app), "2 x 1", "bottom level dims");

        // Swap to the taller-footprint level 1; the readout should follow.
        app.world_mut().resource_scope(|world, mut state: Mut<GameState>| {
            world.resource_scope(|world, mut run: Mut<MultiLevelRun>| {
                let config = world.resource::<GameConfig>().clone();
                crate::world::advance_to_next_level(&mut state, &mut run, &config);
            });
        });
        app.update();
        assert_eq!(read(&mut app), "3 x 1", "readout follows the active level");
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
    fn no_roof_for_open_sky() {
        // The default sky (Night) is open-air — no ceiling panels.
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&RoofCell>().iter(app.world()).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn dungeon_sky_caps_every_passable_cell() {
        let mut app = make_playing_app_with_config(GameConfig {
            sky_type: SkyType::Dungeon,
            ..GameConfig::default()
        });
        let count = app.world_mut().query::<&RoofCell>().iter(app.world()).count();
        let grid = demo_grid();
        let expected = grid.iter().flat_map(|r| r.iter()).filter(|&&c| c != 'W').count();
        assert_eq!(count, expected, "dungeon sky caps every passable cell");
    }

    #[test]
    fn chamber_sky_caps_every_passable_cell() {
        // Chamber is the other roofed sky type — same per-cell ceiling coverage
        // as dungeon, only the material differs (cell's wall material).
        let mut app = make_playing_app_with_config(GameConfig {
            sky_type: SkyType::Chamber,
            ..GameConfig::default()
        });
        let count = app.world_mut().query::<&RoofCell>().iter(app.world()).count();
        let grid = demo_grid();
        let expected = grid.iter().flat_map(|r| r.iter()).filter(|&&c| c != 'W').count();
        assert_eq!(count, expected, "chamber sky caps every passable cell");
    }

    #[test]
    fn corridor_door_is_a_single_swing_leaf() {
        // One key at (0,1) and one door at (0,2). The door's open edges are on
        // opposing sides (key to the west, finish to the east) — a straight
        // corridor — so it renders as a single swinging leaf. Markers spawn even
        // under MinimalPlugins (no mesh/material assets), so this asserts the
        // topology dispatch regardless of rendering.
        let mut app = make_playing_app_with(r#"{"grid":[["S","K","D","F"]]}"#);
        let keys = app.world_mut().query::<&KeyMarker>().iter(app.world()).count();
        let doors = app.world_mut().query::<&DoorMarker>().iter(app.world()).count();
        assert_eq!(keys, 1, "expected one key holder");
        assert_eq!(doors, 1, "a straight-corridor door is a single leaf");
    }

    #[test]
    fn junction_door_seals_each_open_edge() {
        // A door cell with three open neighbours (N=start, S=open, E=finish;
        // W=wall) is not a straight corridor, so it seals each open edge with its
        // own (sliding) leaf — three in total.
        let mut app = make_playing_app_with(
            r#"{"grid":[["W","S","W"],["W","D","F"],["W"," ","W"]]}"#,
        );
        let doors = app.world_mut().query::<&DoorMarker>().iter(app.world()).count();
        assert_eq!(doors, 3, "a 3-open door cell seals each open edge");
    }

    #[test]
    fn key_cell_has_no_dead_end_object() {
        // A key sitting in a dead-end must show its holder, not a brazier/chest.
        // Grid: a vertical stub where (2,1) is a dead-end holding a key.
        let mut app = make_playing_app_with(
            r#"{"grid":[["S"," ","F"],["W","K","W"],["W","W","W"]]}"#,
        );
        let dead_end = app
            .world_mut()
            .query::<&DeadEndObject>()
            .iter(app.world())
            .count();
        let keys = app.world_mut().query::<&KeyMarker>().iter(app.world()).count();
        assert_eq!(keys, 1, "expected the key holder");
        assert_eq!(dead_end, 0, "key cell must not also get a dead-end object");
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
        let dirs: Vec<_> =
            [GridFacing::North, GridFacing::East, GridFacing::South, GridFacing::West]
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
        let demo = demo_grid();
        assert_eq!(state.grid.len(), demo.len());
        assert_eq!(state.grid[0].len(), demo[0].len());
    }

    #[test]
    fn initial_facing_prefers_south_when_open() {
        // South open → faced first (the S→E→N→W cycle starts at South).
        let grid = vec![vec!['S'], vec![' ']];
        assert_eq!(initial_facing(&grid, 0, 0), GridFacing::South);
    }

    #[test]
    fn initial_facing_skips_south_wall_picks_east() {
        let grid = vec![vec!['S', ' '], vec!['W', 'W']];
        assert_eq!(initial_facing(&grid, 0, 0), GridFacing::East);
    }

    #[test]
    fn initial_facing_skips_south_east_picks_north() {
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec!['W', 'S', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert_eq!(initial_facing(&grid, 1, 1), GridFacing::North);
    }

    #[test]
    fn initial_facing_skips_south_east_north_picks_west() {
        let grid = vec![vec![' ', 'S'], vec!['W', 'W']];
        assert_eq!(initial_facing(&grid, 0, 1), GridFacing::West);
    }

    #[test]
    fn initial_facing_all_walls_falls_back_to_south() {
        let grid = vec![vec!['W', 'S', 'W'], vec!['W', 'W', 'W']];
        assert_eq!(initial_facing(&grid, 0, 1), GridFacing::South);
    }

    #[test]
    fn initial_pitch_is_zero() {
        let app = make_playing_app();
        assert_eq!(app.world().resource::<GameState>().visual_pitch, 0.0);
    }

    #[test]
    fn initial_camera_offset_back_from_cell_centre() {
        // The camera should spawn at the back-edge of the start cell
        // relative to its facing — i.e. `cell_centre + back_vec * OFFSET`,
        // NOT at the cell centre. Verifies the new `camera_pos_for`
        // helper is wired into spawn_world.
        let app = make_playing_app();
        let state = app.world().resource::<GameState>();
        let start_row = state.game.player_row();
        let start_col = state.game.player_col();
        let centre = cell_centre(start_row, start_col);
        let visual = state.visual_pos;
        let delta = visual - centre;
        // Distance from centre equals the offset, within float tolerance.
        let dist = delta.length();
        assert!(
            (dist - CAMERA_EDGE_OFFSET).abs() < 1e-4,
            "camera offset from cell centre = {dist}, expected ~{CAMERA_EDGE_OFFSET}"
        );
        // Y is unchanged (cell_centre puts Y at EYE_HEIGHT, camera_pos_for
        // shifts only X/Z).
        assert!(delta.y.abs() < 1e-4, "camera Y should match cell centre");
    }

    #[test]
    fn camera_fov_at_reference_aspect_returns_base() {
        let f = camera_fov_for_aspect(CAMERA_FOV_REFERENCE_ASPECT);
        assert!((f - CAMERA_FOV_VERTICAL_RADIANS).abs() < 1e-5);
    }

    #[test]
    fn camera_fov_above_reference_aspect_returns_base() {
        // Ultrawide-style viewports should NOT shrink vertical FOV —
        // Bevy widens horizontal naturally; the function just returns
        // the base vertical unchanged.
        let f = camera_fov_for_aspect(21.0 / 9.0);
        assert!((f - CAMERA_FOV_VERTICAL_RADIANS).abs() < 1e-5);
    }

    #[test]
    fn camera_fov_below_reference_aspect_grows_vertical() {
        // Square viewport: aspect 1.0 < 16/9. Vertical FOV must grow.
        let f = camera_fov_for_aspect(1.0);
        assert!(
            f > CAMERA_FOV_VERTICAL_RADIANS,
            "expected v_fov > base, got {f}"
        );
        // Horizontal FOV should match the reference: tan(h/2) = tan(v/2)*aspect.
        let h_at_target = 2.0 * ((f * 0.5).tan() * 1.0).atan();
        let h_at_ref =
            2.0 * ((CAMERA_FOV_VERTICAL_RADIANS * 0.5).tan() * CAMERA_FOV_REFERENCE_ASPECT).atan();
        assert!(
            (h_at_target - h_at_ref).abs() < 1e-4,
            "horizontal FOV not preserved: {h_at_target} vs {h_at_ref}"
        );
    }

    #[test]
    fn camera_fov_extreme_portrait_is_capped() {
        // Phone portrait ~9:19.5 (aspect ~0.46). Without the cap the
        // formula would demand ~130° vertical — we cap at the constant.
        let f = camera_fov_for_aspect(9.0 / 19.5);
        assert!((f - CAMERA_FOV_VERTICAL_MAX_RADIANS).abs() < 1e-5);
    }

    #[test]
    fn camera_fov_invalid_aspect_falls_back_to_base() {
        // Zero / negative aspect (e.g. window minimised) must not panic
        // or produce NaN — fall back to the base.
        assert_eq!(camera_fov_for_aspect(0.0), CAMERA_FOV_VERTICAL_RADIANS);
        assert_eq!(camera_fov_for_aspect(-1.0), CAMERA_FOV_VERTICAL_RADIANS);
    }

    #[test]
    fn camera_pos_for_back_vector_matches_yaw() {
        // For each cardinal facing, the camera should sit on the OPPOSITE
        // side of cell centre from the direction the camera is looking.
        // Bevy camera default forward = -Z; after Quat::from_rotation_y(yaw),
        // forward = (-sin(yaw), 0, -cos(yaw)) — so back = (sin(yaw), 0, cos(yaw)).
        let (r, c) = (3usize, 5usize);
        let centre = cell_centre(r, c);
        for facing in [
            GridFacing::North,
            GridFacing::East,
            GridFacing::South,
            GridFacing::West,
        ] {
            let yaw = facing.to_yaw();
            let pos = camera_pos_for(r, c, yaw);
            let expected =
                centre + bevy::math::Vec3::new(yaw.sin(), 0.0, yaw.cos()) * CAMERA_EDGE_OFFSET;
            let diff = (pos - expected).length();
            assert!(diff < 1e-5, "{facing:?}: pos {pos} != expected {expected}");
        }
    }

    #[test]
    fn build_app_with_maze_json_uses_provided_grid() {
        let json = r#"{"grid":[["S"," "," "],[" ","W"," "],[" "," ","F"]]}"#;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        build_app(&mut app, Some(json));
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        let state = app.world().resource::<GameState>();
        assert_eq!(state.grid.len(), 3);
        assert_eq!(state.grid[0].len(), 3);
        assert_eq!(state.game.player_row(), 0);
        assert_eq!(state.game.player_col(), 0);
    }

    /// Builds a playing app with a custom `GameConfig` pre-inserted so the
    /// landmark toggles can be exercised end-to-end. `build_app` calls
    /// `init_resource::<GameConfig>()` which is a no-op when the resource
    /// is already present, so the caller's config survives.
    fn make_playing_app_with_config(config: GameConfig) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(config);
        build_app(&mut app, None);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app
    }

    #[test]
    fn playing_spawns_one_finish_orb_for_demo_grid() {
        // demo_grid has exactly one 'F' cell, so spawn_finish_for_cell's
        // 'F' predicate should produce exactly one FinishOrb entity.
        let mut app = make_playing_app();
        let count = app.world_mut().query::<&FinishOrb>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn finish_orb_despawns_on_win() {
        // Reaching the finish cell flips `state.won = true`; `orb_system`
        // then despawns the FinishOrb so its near-floor position
        // doesn't render as a perspective-stretched ellipse at the
        // close, off-axis viewing angle the player ends up at.
        let mut app = make_playing_app();
        assert_eq!(
            app.world_mut().query::<&FinishOrb>().iter(app.world()).count(),
            1,
            "sanity: orb should exist before win"
        );
        app.world_mut().resource_mut::<GameState>().won = true;
        app.update();
        assert_eq!(
            app.world_mut().query::<&FinishOrb>().iter(app.world()).count(),
            0,
            "FinishOrb should be despawned after win"
        );
    }

    #[test]
    fn multi_level_run_stacks_each_level_a_level_height_higher() {
        // Three identical 1×2 levels (a start + a finish). Every passable cell
        // spawns one floor tile, so each level contributes two — lifted to its
        // own `world_y(level, 0.0)` band.
        let level = r#"{"grid":[["S","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[level, level, level]);
        let mut query = app.world_mut().query::<(&FloorCell, &Transform)>();
        let ys: Vec<f32> = query.iter(app.world()).map(|(_, t)| t.translation.y).collect();
        assert_eq!(ys.len(), 6, "two floor tiles × three levels");
        for level in 0..3 {
            let expected = crate::world::world_y(level, 0.0);
            let n = ys.iter().filter(|y| (**y - expected).abs() < 1e-3).count();
            assert_eq!(n, 2, "level {level} keeps its two floor tiles at y = {expected}");
        }
    }

    #[test]
    fn multi_level_run_has_a_single_finish_orb_on_the_top_level() {
        // Two levels, each with a finish: only the top (final) level keeps the
        // orb — the interim finish omits it.
        let level = r#"{"grid":[["S","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[level, level]);
        let mut query = app.world_mut().query::<(&FinishOrb, &Transform)>();
        let orbs: Vec<f32> = query.iter(app.world()).map(|(_, t)| t.translation.y).collect();
        assert_eq!(orbs.len(), 1, "only the top level keeps the finish orb");
        assert!(
            orbs[0] > crate::world::LEVEL_HEIGHT,
            "the orb rides the upper level, a full LEVEL_HEIGHT up — got y = {}",
            orbs[0],
        );
    }

    #[test]
    fn interim_finish_draws_the_transition_rig_chosen_by_finish_type() {
        use crate::state::FinishType;
        // Vertically-aligned levels: the upper level's start `(0,1)` sits directly
        // above the lower level's finish `(0,1)`, so a ladder is allowed.
        let lower = r#"{"grid":[["S","F"]]}"#;
        let upper = r#"{"grid":[["F","S"]]}"#;

        // Default finish type is Ladder: the interim (bottom) finish gets a ladder
        // rig (several sub-meshes), no portal; the top keeps the orb.
        let mut ladder = make_playing_app_with_levels(&[lower, upper]);
        assert!(
            ladder.world_mut().query::<&FinishLadder>().iter(ladder.world()).count() > 0,
            "the interim finish should draw a ladder rig by default when a start is above",
        );
        assert_eq!(
            ladder.world_mut().query::<&FinishPortal>().iter(ladder.world()).count(),
            0,
            "no portal when finish_type is Ladder and a ladder is allowed",
        );
        assert_eq!(
            ladder.world_mut().query::<&FinishOrb>().iter(ladder.world()).count(),
            1,
            "the final level still keeps its orb",
        );

        // FinishType::Portal swaps the interim rig to a portal (one body), no ladder.
        let mut portal = make_playing_app_with_levels_and_config(
            &[lower, upper],
            GameConfig { finish_type: FinishType::Portal, ..GameConfig::default() },
        );
        assert_eq!(
            portal.world_mut().query::<&FinishPortal>().iter(portal.world()).count(),
            1,
            "the interim finish should draw a single portal body",
        );
        assert_eq!(
            portal.world_mut().query::<&FinishLadder>().iter(portal.world()).count(),
            0,
            "no ladder when finish_type is Portal",
        );
    }

    #[test]
    fn a_ladder_finish_with_no_start_above_falls_back_to_a_portal() {
        // The upper level's start `(0,0)` is NOT above the lower finish `(0,1)`, so
        // even a Ladder finish type must fall back to a portal (you can't climb to
        // nothing). This is the rule that a portal can always replace a ladder.
        let lower = r#"{"grid":[["S","F"]]}"#;
        let upper = r#"{"grid":[["S","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[lower, upper]); // default = Ladder
        assert_eq!(
            app.world_mut().query::<&FinishLadder>().iter(app.world()).count(),
            0,
            "a ladder with no start above must not be drawn",
        );
        assert_eq!(
            app.world_mut().query::<&FinishPortal>().iter(app.world()).count(),
            1,
            "it falls back to a portal instead",
        );
    }

    #[test]
    fn hatch_spawns_in_the_start_above_a_ladder_finish_only() {
        use crate::world::floor::hatch::LevelHatch;
        // Aligned: l1's start (0,1) sits above l0's ladder finish (0,1), so level 1's
        // start cell carries a hatch (and the bottom level never does).
        let lower = r#"{"grid":[["S","F"]]}"#;
        let upper = r#"{"grid":[["F","S"]]}"#;
        let mut ladder = make_playing_app_with_levels(&[lower, upper]);
        let levels: Vec<usize> = ladder
            .world_mut()
            .query::<&LevelHatch>()
            .iter(ladder.world())
            .map(|h| h.level)
            .collect();
        assert_eq!(levels, vec![1], "one hatch, on the level above the ladder finish");

        // Misaligned → portal finish → no hatch anywhere.
        let mut portal =
            make_playing_app_with_levels(&[r#"{"grid":[["S","F"]]}"#, r#"{"grid":[["S","F"]]}"#]);
        assert_eq!(
            portal.world_mut().query::<&LevelHatch>().iter(portal.world()).count(),
            0,
            "a portal finish needs no hatch above it",
        );
    }

    #[test]
    fn hatch_closes_when_the_player_climbs_onto_its_level() {
        use crate::state::{GameConfig, GameState, MultiLevelRun};
        use crate::world::floor::hatch::LevelHatch;
        let lower = r#"{"grid":[["S","F"]]}"#;
        let upper = r#"{"grid":[["F","S"]]}"#;
        let mut app = make_playing_app_with_levels(&[lower, upper]);

        let closing = |app: &mut App| -> bool {
            app.world_mut()
                .query::<&LevelHatch>()
                .iter(app.world())
                .next()
                .map(|h| h.closing)
                .unwrap_or(false)
        };
        assert!(!closing(&mut app), "the hatch starts open");

        // Climb onto level 1, then let the close watcher run.
        app.world_mut().resource_scope(|world, mut state: Mut<GameState>| {
            world.resource_scope(|world, mut run: Mut<MultiLevelRun>| {
                let config = world.resource::<GameConfig>().clone();
                crate::world::advance_to_next_level(&mut state, &mut run, &config);
            });
        });
        app.update();
        assert!(closing(&mut app), "the hatch closes once the player arrives on its level");
    }

    #[test]
    fn single_level_game_keeps_the_orb_and_draws_no_transition_rig() {
        // A one-level game is its own final level, so it keeps the orb and never
        // draws a transition rig — unchanged from before multi-level runs.
        let mut app = make_playing_app_with(r#"{"grid":[["S","F"]]}"#);
        assert_eq!(app.world_mut().query::<&FinishOrb>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&FinishLadder>().iter(app.world()).count(), 0);
        assert_eq!(app.world_mut().query::<&FinishPortal>().iter(app.world()).count(), 0);
    }

    #[test]
    fn centre_alignment_shifts_a_smaller_upper_level_in_x_relative_to_edge() {
        use crate::state::LayeredAlignment;
        // A 1×9 bottom level + a 1×5 upper level (a smaller grid). Under `Edge`
        // the upper level's floor sits at the origin corner; under `Centre` it
        // shifts in by (9-5)/2 = 2 cells, so the whole demo reads as a centred
        // stack — and, crucially, the geometry honours the configured alignment.
        let l0 = r#"{"grid":[["S"," "," "," "," "," "," "," ","F"]]}"#;
        let l1 = r#"{"grid":[["S"," "," "," ","F"]]}"#;

        // Minimum X of the upper level's floor tiles (those a `LEVEL_HEIGHT` up).
        fn upper_floor_min_x(app: &mut App) -> f32 {
            let mut query = app.world_mut().query::<(&FloorCell, &Transform)>();
            query
                .iter(app.world())
                .map(|(_, t)| t.translation)
                .filter(|p| p.y > crate::world::LEVEL_HEIGHT / 2.0)
                .map(|p| p.x)
                .fold(f32::INFINITY, f32::min)
        }

        let mut edge = make_playing_app_with_levels_and_config(
            &[l0, l1],
            GameConfig { layered_alignment: LayeredAlignment::Edge, ..GameConfig::default() },
        );
        let mut centre = make_playing_app_with_levels_and_config(
            &[l0, l1],
            GameConfig { layered_alignment: LayeredAlignment::Centre, ..GameConfig::default() },
        );

        let shift = upper_floor_min_x(&mut centre) - upper_floor_min_x(&mut edge);
        assert!(
            (shift - 2.0 * crate::world::CELL_SIZE).abs() < 1e-3,
            "Centre should shift the 1×5 level in by 2 cells vs Edge, got {shift}",
        );
    }

    #[test]
    fn floor_and_wall_counts_scale_with_the_level_count() {
        // A wall-bearing footprint rendered as one level, then as three. Every
        // level renders the same geometry (only its Y offset differs), so the
        // floor and wall counts scale exactly with the level count.
        let level = r#"{"grid":[["S","W"," "],[" ","W","F"]]}"#;
        let single = {
            let mut app = make_playing_app_with_levels(&[level]);
            let floors = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
            let walls = app.world_mut().query::<&WallCell>().iter(app.world()).count();
            (floors, walls)
        };
        let triple = {
            let mut app = make_playing_app_with_levels(&[level, level, level]);
            let floors = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
            let walls = app.world_mut().query::<&WallCell>().iter(app.world()).count();
            (floors, walls)
        };
        assert!(single.1 > 0, "sanity: the footprint actually has wall panels");
        assert_eq!(triple.0, single.0 * 3, "floor tiles scale with the level count");
        assert_eq!(triple.1, single.1 * 3, "wall panels scale with the level count");
    }

    #[test]
    fn wall_decorations_toggle_off_suppresses_spawns() {
        let mut app = make_playing_app_with_config(GameConfig {
            landmarks: Landmarks {
                wall_decorations: false,
                ..Landmarks::default()
            },
            ..GameConfig::default()
        });
        let count = app
            .world_mut()
            .query::<&WallDecoration>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn demo_grid_is_well_formed() {
        use crate::world::demo_grid;
        use maze::is_dead_end;
        use std::collections::{HashSet, VecDeque};

        let grid = demo_grid();
        let rows = grid.len();
        let cols = grid[0].len();

        let count = |target: char| grid.iter().flatten().filter(|&&c| c == target).count();
        assert_eq!(count('S'), 1, "exactly one start");
        assert_eq!(count('F'), 1, "exactly one finish");
        assert!(count('K') >= 1, "at least one key");
        assert!(count('D') >= 1, "at least one door");

        let find = |target: char| {
            grid.iter()
                .enumerate()
                .find_map(|(r, row)| row.iter().position(|&c| c == target).map(|c| (r, c)))
                .unwrap()
        };
        let start = find('S');

        // BFS from the start. `doors_passable = false` models locked doors
        // (treats 'D' as a wall); `true` models every door open.
        let reachable = |doors_passable: bool| -> HashSet<(usize, usize)> {
            let mut seen = HashSet::new();
            let mut queue = VecDeque::new();
            seen.insert(start);
            queue.push_back(start);
            while let Some((r, c)) = queue.pop_front() {
                let mut neighbours = Vec::new();
                if r > 0 {
                    neighbours.push((r - 1, c));
                }
                if r + 1 < rows {
                    neighbours.push((r + 1, c));
                }
                if c > 0 {
                    neighbours.push((r, c - 1));
                }
                if c + 1 < cols {
                    neighbours.push((r, c + 1));
                }
                for (nr, nc) in neighbours {
                    let ch = grid[nr][nc];
                    let passable = ch != 'W' && (doors_passable || ch != 'D');
                    if passable && seen.insert((nr, nc)) {
                        queue.push_back((nr, nc));
                    }
                }
            }
            seen
        };

        let finish = find('F');
        let key = find('K');
        let locked = reachable(false);
        let unlocked = reachable(true);

        // The door genuinely gates the finish, and the key is obtainable first.
        assert!(!locked.contains(&finish), "finish must be gated by the door");
        assert!(unlocked.contains(&finish), "finish reachable once the door opens");
        assert!(locked.contains(&key), "key reachable while the door is locked");

        // Several dead-ends remain for landmark objects (S/F/K/D excluded).
        let mut dead_ends = 0;
        for (r, row) in grid.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                if !matches!(cell, 'S' | 'F' | 'K' | 'D') && is_dead_end(&grid, r, c) {
                    dead_ends += 1;
                }
            }
        }
        assert!(dead_ends >= 6, "expected >= 6 landmark dead-ends, got {dead_ends}");
    }

    #[test]
    fn demo_grid_contains_a_decoy_door() {
        // The demo grid must expose both a real path door AND a decoy: the
        // player burning their lone key on the decoy is the on-ramp to
        // experiencing the `Stranded` lose surface without going through the
        // full Play-3D config flow. Structural check: exactly one of the
        // grid's `'D'` cells lies on the lock-blind S→F path; any others are
        // off-spine decoys.
        use crate::world::demo_grid;
        use maze::MazeGame;
        use std::collections::{HashMap, HashSet, VecDeque};

        let grid = demo_grid();
        let rows = grid.len();
        let cols = grid[0].len();

        // Find S, F, and all D cells.
        let find_one = |target: char| {
            grid.iter()
                .enumerate()
                .find_map(|(r, row)| row.iter().position(|&c| c == target).map(|c| (r, c)))
                .unwrap()
        };
        let start = find_one('S');
        let finish = find_one('F');
        let doors: HashSet<(usize, usize)> = grid
            .iter()
            .enumerate()
            .flat_map(|(r, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, &c)| c == 'D')
                    .map(move |(c, _)| (r, c))
            })
            .collect();
        assert!(
            doors.len() >= 2,
            "demo grid needs at least one real path door + one decoy, got {}",
            doors.len()
        );

        // Lock-blind BFS from S → F (every non-'W' cell passable) with parent
        // pointers, so we can read the shortest-path doors back.
        let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut visited: HashSet<(usize, usize)> = HashSet::new();
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();
        visited.insert(start);
        q.push_back(start);
        while let Some((r, c)) = q.pop_front() {
            if (r, c) == finish {
                break;
            }
            let mut neighbours = Vec::with_capacity(4);
            if r > 0 {
                neighbours.push((r - 1, c));
            }
            if r + 1 < rows {
                neighbours.push((r + 1, c));
            }
            if c > 0 {
                neighbours.push((r, c - 1));
            }
            if c + 1 < cols {
                neighbours.push((r, c + 1));
            }
            for n in neighbours {
                if grid[n.0][n.1] == 'W' {
                    continue;
                }
                if visited.insert(n) {
                    parent.insert(n, (r, c));
                    q.push_back(n);
                }
            }
        }
        assert!(visited.contains(&finish), "finish must be lock-blind reachable");

        // Walk back from F to find which D cells lie on the spine.
        let mut spine_doors: HashSet<(usize, usize)> = HashSet::new();
        let mut cur = finish;
        while cur != start {
            if doors.contains(&cur) {
                spine_doors.insert(cur);
            }
            cur = parent[&cur];
        }
        assert_eq!(
            spine_doors.len(),
            1,
            "demo grid should have exactly one real path door on the spine, got {}",
            spine_doors.len()
        );
        let decoy_count = doors.len() - spine_doors.len();
        assert!(
            decoy_count >= 1,
            "demo grid should have at least one off-spine decoy, got {decoy_count}"
        );

        // Cross-check with the actual `MazeGame` runtime: at construction it
        // identifies the spine doors via the same algorithm and exposes the
        // count via `path_doors_remaining_closed` (indirectly, via the lose
        // semantics). We can verify here that constructing a `MazeGame` from
        // the demo grid succeeds and reports neither complete nor lost — a
        // cheap end-to-end smoke check that the demo grid is a valid
        // starting state for the strand scenario.
        let json = crate::world::grid_to_json(&grid);
        let game = MazeGame::from_json(&json).expect("demo grid loads as a game");
        assert!(!game.is_complete(), "fresh game must not be complete");
        assert!(!game.is_lost(), "fresh game must not be lost");
    }

    #[test]
    fn playing_spawns_brazier_bowl_marker() {
        // The demo grid contains several dead-end cells, and the
        // dead-end-object hash is deterministic — so at least one of the
        // four landmark kinds will be spawned. We can't guarantee a
        // brazier from any single dead-end (the hash picks one of four),
        // so sweep for a seed that forces kind 0 (brazier) on some
        // landmark dead-end.
        let mut app = make_playing_app_with_config(GameConfig {
            seed: brazier_forcing_seed(),
            ..GameConfig::default()
        });
        let count = app.world_mut().query::<&BrazierBowl>().iter(app.world()).count();
        assert!(
            count >= 1,
            "expected at least one BrazierBowl marker, got {count}"
        );
    }

    /// Returns a seed value that hashes at least one demo-grid dead-end
    /// cell into the brazier branch (`dead_end_object_index == 0`). The
    /// demo grid's dead-end cells are determined by `is_dead_end`; we
    /// sweep candidate seeds until we find one that yields a brazier so
    /// the smoke test isn't sensitive to changes in the hash constants.
    fn brazier_forcing_seed() -> u64 {
        use crate::world::demo_grid;
        use crate::world::objects::dead_end::dead_end_object_index;
        use maze::is_dead_end;
        let grid = demo_grid();
        for seed in 0u64..1024 {
            for (r, row) in grid.iter().enumerate() {
                for (c, &cell) in row.iter().enumerate() {
                    // Mirror `spawn_dead_end_object_for_cell`'s exclusions: only
                    // cells that actually receive a landmark are candidates.
                    if matches!(cell, 'S' | 'F' | 'K' | 'D' | 'E' | 'H' | 'T') {
                        continue;
                    }
                    if is_dead_end(&grid, r, c) && dead_end_object_index(r, c, seed) == 0 {
                        return seed;
                    }
                }
            }
        }
        panic!("no seed in 0..1024 produces a brazier on the demo grid");
    }

    #[test]
    fn dead_end_objects_toggle_off_suppresses_spawns() {
        let mut app = make_playing_app_with_config(GameConfig {
            landmarks: Landmarks {
                dead_end_objects: false,
                ..Landmarks::default()
            },
            ..GameConfig::default()
        });
        let count = app
            .world_mut()
            .query::<&DeadEndObject>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn floor_accents_toggle_off_suppresses_spawns() {
        let mut app = make_playing_app_with_config(GameConfig {
            landmarks: Landmarks {
                floor_accents: false,
                ..Landmarks::default()
            },
            ..GameConfig::default()
        });
        let count = app
            .world_mut()
            .query::<&FloorAccent>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    /// Smoke-tests `spawn_sky` for every [`SkyType`] variant.
    ///
    /// Each sky type must spawn exactly one [`SkyDome`] entity and at
    /// least one [`AmbientLight`] — a regression guard against a
    /// missing branch in the dispatch in `world/sky/mod.rs`.
    fn assert_sky_spawns_dome_and_light(sky_type: SkyType) {
        let mut app = make_playing_app_with_config(GameConfig {
            sky_type,
            ..GameConfig::default()
        });
        let dome_count = app.world_mut().query::<&SkyDome>().iter(app.world()).count();
        assert_eq!(dome_count, 1, "{sky_type:?}: expected exactly one SkyDome");
        let ambient_count = app
            .world_mut()
            .query::<&AmbientLight>()
            .iter(app.world())
            .count();
        assert!(
            ambient_count >= 1,
            "{sky_type:?}: expected at least one AmbientLight, got {ambient_count}"
        );
        let directional_count = app
            .world_mut()
            .query::<&DirectionalLight>()
            .iter(app.world())
            .count();
        assert!(
            directional_count >= 1,
            "{sky_type:?}: expected at least one DirectionalLight, got {directional_count}"
        );
    }

    #[test]
    fn night_sky_spawns_dome_and_lights() {
        assert_sky_spawns_dome_and_light(SkyType::Night);
    }

    #[test]
    fn sunrise_sky_spawns_dome_and_lights() {
        assert_sky_spawns_dome_and_light(SkyType::Sunrise);
    }

    #[test]
    fn day_sky_spawns_dome_and_lights() {
        assert_sky_spawns_dome_and_light(SkyType::Day);
    }

    #[test]
    fn sunset_sky_spawns_dome_and_lights() {
        assert_sky_spawns_dome_and_light(SkyType::Sunset);
    }

    #[test]
    fn dungeon_sky_spawns_dome_and_lights() {
        // The dungeon caps cells with a ceiling but still spawns a (near-black)
        // dome behind it and a dim ambient + overhead light, so the shared
        // assertion holds.
        assert_sky_spawns_dome_and_light(SkyType::Dungeon);
    }

    #[test]
    fn chamber_sky_spawns_dome_and_lights() {
        assert_sky_spawns_dome_and_light(SkyType::Chamber);
    }

    #[test]
    fn default_sky_type_is_night() {
        assert_eq!(GameConfig::default().sky_type, SkyType::Night);
    }

    #[test]
    fn sky_type_wire_round_trip() {
        for st in [
            SkyType::Night,
            SkyType::Sunrise,
            SkyType::Day,
            SkyType::Sunset,
            SkyType::Dungeon,
            SkyType::Chamber,
        ] {
            assert_eq!(SkyType::from_wire_str(st.as_wire_str()), st);
        }
        // Unknown values fall back to Night.
        assert_eq!(SkyType::from_wire_str("typo"), SkyType::Night);
        assert_eq!(SkyType::from_wire_str(""), SkyType::Night);
        // Case-insensitive.
        assert_eq!(SkyType::from_wire_str("DAY"), SkyType::Day);
        assert_eq!(SkyType::from_wire_str("SunSet"), SkyType::Sunset);
    }

    #[test]
    fn only_dungeon_and_chamber_are_enclosed() {
        assert!(SkyType::Dungeon.is_enclosed());
        assert!(SkyType::Chamber.is_enclosed());
        for st in [SkyType::Night, SkyType::Sunrise, SkyType::Day, SkyType::Sunset] {
            assert!(!st.is_enclosed(), "{st:?} is open-air");
        }
    }

    #[test]
    fn default_perimeter_walls_is_true() {
        // The maze is walled at its perimeter by default (open-sky mazes too).
        assert!(GameConfig::default().perimeter_walls);
    }

    #[test]
    fn default_wall_type_is_brick() {
        assert_eq!(GameConfig::default().wall_type, WallType::Brick);
    }

    #[test]
    fn wall_type_wire_round_trip() {
        for wt in [
            WallType::Brick,
            WallType::DressedStone,
            WallType::Wood,
            WallType::Cobblestone,
        ] {
            assert_eq!(WallType::from_wire_str(wt.as_wire_str()), wt);
        }
        // Unknown values fall back to Brick.
        assert_eq!(WallType::from_wire_str("typo"), WallType::Brick);
        assert_eq!(WallType::from_wire_str(""), WallType::Brick);
        // Case-insensitive.
        assert_eq!(WallType::from_wire_str("WOOD"), WallType::Wood);
        assert_eq!(
            WallType::from_wire_str("Dressed_Stone"),
            WallType::DressedStone
        );
    }

    #[test]
    fn wall_type_to_kind_index_matches_wall_material_constants() {
        use crate::world::walls::{
            WALL_MATERIAL_BRICK, WALL_MATERIAL_COBBLESTONE, WALL_MATERIAL_DRESSED_STONE,
            WALL_MATERIAL_WOOD,
        };
        assert_eq!(WallType::Brick.to_kind_index(), Some(WALL_MATERIAL_BRICK));
        assert_eq!(
            WallType::DressedStone.to_kind_index(),
            Some(WALL_MATERIAL_DRESSED_STONE)
        );
        assert_eq!(WallType::Wood.to_kind_index(), Some(WALL_MATERIAL_WOOD));
        assert_eq!(
            WallType::Cobblestone.to_kind_index(),
            Some(WALL_MATERIAL_COBBLESTONE)
        );
        // Non-occluding types have no panel material.
        assert_eq!(WallType::Water.to_kind_index(), None);
        assert_eq!(WallType::Lava.to_kind_index(), None);
        assert_eq!(WallType::IronFence.to_kind_index(), None);
    }

    #[test]
    fn only_special_wall_types_are_non_occluding() {
        // The four solid textures occlude; the three special types don't —
        // exactly the inverse of `to_kind_index` being `Some`.
        for wt in [
            WallType::Brick,
            WallType::DressedStone,
            WallType::Wood,
            WallType::Cobblestone,
        ] {
            assert!(!wt.is_non_occluding(), "{wt:?} should occlude");
            assert!(wt.to_kind_index().is_some());
        }
        for wt in [WallType::Water, WallType::Lava, WallType::IronFence] {
            assert!(wt.is_non_occluding(), "{wt:?} should be non-occluding");
            assert!(wt.to_kind_index().is_none());
        }
    }

    #[test]
    fn wall_material_variation_toggle_off_uses_tint_path() {
        // The dispatch branch in `spawn_walls_for_cell` should produce the
        // same wall-panel count whether material variation is on (per-quadrant
        // materials path) or off (tinted path). Sanity check that neither
        // path drops faces — a regression would surface as a mismatch.
        let mut app_on = make_playing_app_with_config(GameConfig {
            landmarks: Landmarks {
                wall_material_variation: true,
                ..Landmarks::default()
            },
            ..GameConfig::default()
        });
        let count_on = app_on
            .world_mut()
            .query::<&WallCell>()
            .iter(app_on.world())
            .count();

        let mut app_off = make_playing_app_with_config(GameConfig {
            landmarks: Landmarks {
                wall_material_variation: false,
                ..Landmarks::default()
            },
            ..GameConfig::default()
        });
        let count_off = app_off
            .world_mut()
            .query::<&WallCell>()
            .iter(app_off.world())
            .count();

        assert_eq!(count_on, count_off);
        assert_eq!(count_on, expected_wall_panel_count(&demo_grid()));
    }

    // ── enemies, health pickups, HP HUD (Bevy parity with maze crate) ─────────

    #[test]
    fn enemy_marker_spawned_per_e_cell() {
        let mut app = make_playing_app();
        let grid = demo_grid();
        let expected = grid.iter().flatten().filter(|&&c| c == 'E').count();
        let count = app
            .world_mut()
            .query::<&EnemyMarker>()
            .iter(app.world())
            .count();
        assert_eq!(count, expected);
        assert!(count >= 1, "demo grid must contain at least one 'E' cell");
    }

    #[test]
    fn enemy_marker_ids_align_with_row_major_scan() {
        // EnemyMarker.id is assigned by the row-major counter in
        // spawn_world; maze::Enemy.id is assigned by the same row-major
        // scan inside MazeGame::from_json_with_options. The two must
        // match so enemy_animation_system can correlate them.
        let mut app = make_playing_app();
        let mut marker_ids: Vec<u32> = app
            .world_mut()
            .query::<&EnemyMarker>()
            .iter(app.world())
            .map(|m| m.id)
            .collect();
        marker_ids.sort();
        let game = app.world().resource::<GameState>().game.enemies();
        let mut runtime_ids: Vec<u32> = game.iter().map(|e| e.id).collect();
        runtime_ids.sort();
        assert_eq!(marker_ids, runtime_ids);
    }

    #[test]
    fn every_levels_enemies_spawn_with_real_per_level_ids() {
        // Level 0 has two enemies, level 1 has one. EVERY level (not just the live
        // bottom one) must spawn its enemies with real per-level row-major ids — no
        // `u32::MAX` frozen-scenery stub — so each level's enemies come alive when
        // it becomes the current level.
        let l0 = r#"{"grid":[["S","E","E","F"]]}"#;
        let l1 = r#"{"grid":[["S","E","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[l0, l1]);
        let mut ids_by_level: std::collections::HashMap<usize, Vec<u32>> = std::collections::HashMap::new();
        for marker in app.world_mut().query::<&EnemyMarker>().iter(app.world()) {
            assert_ne!(marker.id, u32::MAX, "no enemy keeps the old frozen-scenery stub id");
            ids_by_level.entry(marker.placement.level).or_default().push(marker.id);
        }
        for ids in ids_by_level.values_mut() {
            ids.sort();
        }
        assert_eq!(ids_by_level.get(&0).cloned().unwrap_or_default(), vec![0, 1], "level 0 enemies are ids 0,1");
        assert_eq!(ids_by_level.get(&1).cloned().unwrap_or_default(), vec![0], "level 1 enemy is id 0 (per-level)");
    }

    #[test]
    fn an_off_level_enemy_holds_its_own_cell_and_is_not_hijacked() {
        // Level 0's enemy (id 0) sits at cell (0,1); level 1's enemy (id 0) sits at
        // a DIFFERENT cell (0,0). While we're on level 0, the level-1 marker must
        // hold its OWN spawn cell (0,0) → world (1.0, 1.0), not be dragged to
        // level-0's enemy at (0,1) → world x = 3.0 by the colliding id (the enemy
        // analogue of the 6a cross-level door bug).
        let l0 = r#"{"grid":[["S","E","F"]]}"#;
        let l1 = r#"{"grid":[["E","S","F"]]}"#;
        let mut app = make_playing_app_with_levels(&[l0, l1]);
        let (x, z) = app
            .world_mut()
            .query::<(&EnemyMarker, &Transform)>()
            .iter(app.world())
            .find(|(m, _)| m.placement.level == 1)
            .map(|(_, t)| (t.translation.x, t.translation.z))
            .expect("level 1 has an enemy marker");
        assert!(
            (x - 1.0).abs() < 1e-3 && (z - 1.0).abs() < 1e-3,
            "level-1 enemy held its own cell (0,0) → (1.0, 1.0); got ({x}, {z})"
        );
    }

    #[test]
    fn hide_completed_enemies_despawns_lower_levels_on_ascend_only_when_enabled() {
        use crate::state::MultiLevelRun;
        let l0 = r#"{"grid":[["S","E","F"]]}"#;
        let l1 = r#"{"grid":[["S","E","F"]]}"#;

        // Climb from level 0 to level 1 (swaps the live game) and run a frame.
        fn ascend(app: &mut App) {
            let config = app.world().resource::<GameConfig>().clone();
            app.world_mut().resource_scope(|world, mut state: Mut<GameState>| {
                world.resource_scope(|_world, mut run: Mut<MultiLevelRun>| {
                    crate::world::advance_to_next_level(&mut state, &mut run, &config);
                });
            });
            app.update();
        }
        fn level0_enemies(app: &mut App) -> usize {
            app.world_mut()
                .query::<&EnemyMarker>()
                .iter(app.world())
                .filter(|m| m.placement.level == 0)
                .count()
        }

        // Flag ON → level 0's enemy is freed once we've climbed past it.
        let mut app_on = make_playing_app_with_levels_and_config(
            &[l0, l1],
            GameConfig { hide_completed_enemies: true, ..GameConfig::default() },
        );
        assert_eq!(level0_enemies(&mut app_on), 1, "level 0 enemy present before ascending");
        ascend(&mut app_on);
        assert_eq!(level0_enemies(&mut app_on), 0, "flag on: level 0 enemy despawned on ascend");

        // Flag OFF (default) → the completed level's enemy stays (idles in place).
        let mut app_off = make_playing_app_with_levels_and_config(&[l0, l1], GameConfig::default());
        ascend(&mut app_off);
        assert_eq!(level0_enemies(&mut app_off), 1, "flag off: level 0 enemy retained");
    }

    #[test]
    fn health_marker_spawned_per_h_cell() {
        let mut app = make_playing_app();
        let grid = demo_grid();
        let expected = grid.iter().flatten().filter(|&&c| c == 'H').count();
        let count = app
            .world_mut()
            .query::<&HealthMarker>()
            .iter(app.world())
            .count();
        assert_eq!(count, expected);
        assert!(count >= 1, "demo grid must contain at least one 'H' cell");
    }

    #[test]
    fn hp_hud_spawns_max_hp_heart_icons() {
        let mut app = make_playing_app();
        let count = app
            .world_mut()
            .query::<&crate::hud::hp::HpHeartIcon>()
            .iter(app.world())
            .count() as u32;
        let max_hp = app.world().resource::<GameState>().game.max_hp();
        assert_eq!(count, max_hp);
    }

    #[test]
    fn demo_grid_is_well_formed_with_enemy_and_health() {
        // Extends `demo_grid_is_well_formed` for the new vocabulary —
        // exactly one 'E' and one 'H' so the smoke-test exercises both
        // rigs without confusing the player with a horde.
        let grid = demo_grid();
        let count = |target: char| grid.iter().flatten().filter(|&&c| c == target).count();
        assert_eq!(count('E'), 1, "exactly one enemy spawn");
        assert_eq!(count('H'), 1, "exactly one health pickup");
    }

    // ── EnemyType / HealthStyle wire round-trip + rig dispatch ────────────────

    #[test]
    fn enemy_type_wire_round_trip() {
        for variant in [EnemyType::Goblin, EnemyType::Ghost] {
            assert_eq!(EnemyType::from_wire_str(variant.as_wire_str()), variant);
        }
    }

    #[test]
    fn enemy_type_unknown_wire_string_falls_back_to_goblin() {
        assert_eq!(EnemyType::from_wire_str(""), EnemyType::Goblin);
        assert_eq!(EnemyType::from_wire_str("totally-unknown"), EnemyType::Goblin);
    }

    #[test]
    fn health_style_wire_round_trip() {
        for variant in [HealthStyle::Heart, HealthStyle::Potion] {
            assert_eq!(HealthStyle::from_wire_str(variant.as_wire_str()), variant);
        }
    }

    #[test]
    fn health_style_unknown_wire_string_falls_back_to_heart() {
        assert_eq!(HealthStyle::from_wire_str(""), HealthStyle::Heart);
        assert_eq!(
            HealthStyle::from_wire_str("totally-unknown"),
            HealthStyle::Heart,
        );
    }

    #[test]
    fn playing_with_ghost_enemy_type_spawns_ghost_tag() {
        // `GhostTag` is a zero-cost unit marker on the root entity of
        // every ghost rig — present regardless of whether the
        // asset-bearing child entities spawned, so headless tests (no
        // mesh / material plugins) can distinguish the rig.
        use crate::world::objects::enemy::ghost::GhostTag;
        let goblin_tag_count = {
            let mut g = make_playing_app();
            g.world_mut().query::<&GhostTag>().iter(g.world()).count()
        };
        assert_eq!(
            goblin_tag_count, 0,
            "Goblin rig must not spawn any GhostTag entities",
        );
        let mut ghost_app = make_playing_app_with_config(GameConfig {
            enemy_type: EnemyType::Ghost,
            ..GameConfig::default()
        });
        let ghost_tag_count = ghost_app
            .world_mut()
            .query::<&GhostTag>()
            .iter(ghost_app.world())
            .count();
        let grid = demo_grid();
        let expected = grid.iter().flatten().filter(|&&c| c == 'E').count();
        assert_eq!(
            ghost_tag_count, expected,
            "Ghost rig must spawn one GhostTag per 'E' cell",
        );
    }

    #[test]
    fn per_cell_ghost_override_spawns_ghost_under_goblin_default() {
        // The maze's default enemy rig is Goblin (default `GameConfig`), but the
        // single `'E'` cell carries a `ghost` per-cell override. The override —
        // not the config default — must drive the spawned rig.
        use crate::world::objects::enemy::ghost::GhostTag;
        let json = r#"{"grid":[["S",[{"type":"E","enemyType":"ghost"}],"F"]]}"#;
        let mut app = make_playing_app_with(json);
        let ghost_tag_count = app
            .world_mut()
            .query::<&GhostTag>()
            .iter(app.world())
            .count();
        assert_eq!(
            ghost_tag_count, 1,
            "a per-cell ghost override must spawn a ghost rig even when the maze default is Goblin",
        );
    }

    // ── Treasure ('T' cells: open chest + collectible loot) ──────────────────

    #[test]
    fn treasure_marker_spawned_per_t_cell() {
        let mut app = make_playing_app();
        let grid = demo_grid();
        let expected = grid.iter().flatten().filter(|&&c| c == 'T').count();
        let count = app
            .world_mut()
            .query::<&TreasureMarker>()
            .iter(app.world())
            .count();
        assert_eq!(count, expected);
        assert!(count >= 1, "demo grid must contain at least one 'T' cell");
    }

    #[test]
    fn treasure_cell_has_no_dead_end_object() {
        // A treasure sitting in a dead-end shows its open chest + loot, not a
        // brazier/urn/pillar/chest landmark — treasure takes precedence.
        let mut app = make_playing_app_with(
            r#"{"grid":[["S"," ","F"],["W","T","W"],["W","W","W"]]}"#,
        );
        let dead_end = app
            .world_mut()
            .query::<&DeadEndObject>()
            .iter(app.world())
            .count();
        let treasure = app
            .world_mut()
            .query::<&TreasureMarker>()
            .iter(app.world())
            .count();
        assert_eq!(treasure, 1, "expected the treasure marker");
        assert_eq!(dead_end, 0, "treasure cell must not also get a dead-end object");
    }

    #[test]
    fn treasure_loot_family_matches_style() {
        // Each loot pile is baked into shared meshes: the coin styles
        // (Silver / Gold) into one combined mesh; the gem styles
        // (Diamonds / Jewels) into one mesh per colour group. So a style
        // override that switches families changes the baked-mesh (TreasureLoot)
        // count from 1 to one-per-group. This confirms the per-cell override is
        // wired into the spawn dispatch (the four-way resolution itself is
        // unit-tested in `objects::overrides`).
        let loot = |json: &str| {
            let mut app = make_playing_app_with(json);
            app.world_mut().query::<&TreasureLoot>().iter(app.world()).count()
        };
        let coins = loot(r#"{"grid":[["S","T","F"]]}"#); // bare 'T' → Silver coins
        assert_eq!(coins, 1, "coin loot bakes to a single combined mesh");
        let gems = loot(r#"{"grid":[["S",[{"type":"T","style":"diamonds"}],"F"]]}"#);
        assert_eq!(gems, 4, "gem loot bakes to one combined mesh per colour group");
    }

    #[test]
    fn treasure_style_wire_round_trip() {
        for variant in [
            TreasureStyle::Silver,
            TreasureStyle::Gold,
            TreasureStyle::Diamonds,
            TreasureStyle::Jewels,
        ] {
            assert_eq!(TreasureStyle::from_wire_str(variant.as_wire_str()), variant);
        }
        // Unknown values fall back to Silver.
        assert_eq!(TreasureStyle::from_wire_str(""), TreasureStyle::Silver);
        assert_eq!(TreasureStyle::from_wire_str("totally-unknown"), TreasureStyle::Silver);
        // Case-insensitive.
        assert_eq!(TreasureStyle::from_wire_str("GOLD"), TreasureStyle::Gold);
    }

    // ── Non-occluding wall types (water / lava / iron fence) ──────────────────

    #[test]
    fn water_override_renders_surface_and_no_floor_tile() {
        // The 'W' cell at (1,1) carries a water override. It renders one water
        // surface and NO floor tile (the pool serves as the floor); only the
        // three passable cells get floor tiles.
        let json = r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"water"}],"W"]]}"#;
        let mut app = make_playing_app_with(json);
        let water = app.world_mut().query::<&WaterSurface>().iter(app.world()).count();
        let floors = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
        assert_eq!(water, 1, "one water surface");
        assert_eq!(floors, 3, "only the three passable cells get floor tiles");
    }

    #[test]
    fn lava_override_renders_surface_and_no_floor_tile() {
        let json = r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"lava"}],"W"]]}"#;
        let mut app = make_playing_app_with(json);
        let lava = app.world_mut().query::<&LavaSurface>().iter(app.world()).count();
        let floors = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
        assert_eq!(lava, 1, "one lava surface");
        assert_eq!(floors, 3, "only the three passable cells get floor tiles");
    }

    #[test]
    fn iron_fence_override_renders_bars_over_a_floor_tile() {
        // Unlike the pools, the iron fence stands on a normal floor — so the
        // four floor tiles are the three passable cells plus the fence's own.
        let json =
            r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"iron_fence"}],"W"]]}"#;
        let mut app = make_playing_app_with(json);
        let bars = app.world_mut().query::<&IronFenceBars>().iter(app.world()).count();
        let floors = app.world_mut().query::<&FloorCell>().iter(app.world()).count();
        assert_eq!(bars, 1, "one iron-fence lattice");
        assert_eq!(floors, 4, "three passable cells + the iron-fence floor tile");
    }

    #[test]
    fn non_occluding_neighbour_suppresses_shared_panels() {
        // A centre cell ringed by four passable neighbours. As a solid wall it
        // draws four panels (one per open neighbour). As a non-occluding water
        // cell those four shared panels are suppressed, and the water cell —
        // having no solid neighbours and no grid-edge faces — draws none, so the
        // world has exactly four fewer wall panels.
        let solid = r#"{"grid":[["S"," "," "],[" ","W"," "],[" "," ","F"]]}"#;
        let water =
            r#"{"grid":[["S"," "," "],[" ",[{"type":"W","wallType":"water"}]," "],[" "," ","F"]]}"#;
        let panels = |json: &str| {
            let mut app = make_playing_app_with(json);
            app.world_mut().query::<&WallCell>().iter(app.world()).count()
        };
        assert_eq!(panels(solid), panels(water) + 4);
    }

    #[test]
    fn non_occluding_edge_cell_draws_no_outer_wall_when_perimeter_open() {
        // Open sky with perimeter walls off: the corner water cell (two grid edges)
        // draws no *solid wall* panel (the sky shows past its edge; its low rim
        // frames it instead), and its two open neighbours suppress their panels
        // toward it. Replacing it with a solid wall makes those two neighbours each
        // draw a panel, so the solid variant has two MORE wall panels.
        let solid = r#"{"grid":[["S"," "],["F","W"]]}"#;
        let water = r#"{"grid":[["S"," "],["F",[{"type":"W","wallType":"water"}]]]}"#;
        let panels = |json: &str| {
            let config = GameConfig {
                perimeter_walls: false,
                ..GameConfig::default()
            };
            let mut app = make_playing_app_with_maze_and_config(json, config);
            app.world_mut().query::<&WallCell>().iter(app.world()).count()
        };
        assert_eq!(panels(solid), panels(water) + 2);
    }

    #[test]
    fn no_wall_decorations_at_open_edges_without_perimeter_walls() {
        // An all-passable open-sky maze with perimeter walls off has no wall panels
        // at all, so no wall decorations spawn — otherwise they'd float in mid-air
        // at the boundary where the wall would have been.
        let json = r#"{"grid":[["S"," "," "," "," ","F"]]}"#;
        let config = GameConfig {
            perimeter_walls: false,
            ..GameConfig::default()
        };
        let mut app = make_playing_app_with_maze_and_config(json, config);
        let count = app.world_mut().query::<&WallDecoration>().iter(app.world()).count();
        assert_eq!(count, 0, "open edges (no panel) must carry no floating decorations");
    }

    #[test]
    fn perimeter_walls_restore_edge_decorations() {
        // Walling the perimeter makes those grid-edge panels (and so their
        // decorations) appear again. The 1/10 placement hash makes any single seed
        // unreliable, so find one that decorates with the perimeter walled, then
        // confirm the same seed places none when the perimeter is open.
        let json = r#"{"grid":[["S"," "," "," "," ","F"]]}"#;
        let decorations = |seed: u64, perimeter_walls: bool| {
            let config = GameConfig {
                seed,
                perimeter_walls,
                ..GameConfig::default()
            };
            let mut app = make_playing_app_with_maze_and_config(json, config);
            app.world_mut().query::<&WallDecoration>().iter(app.world()).count()
        };
        let seed = (0u64..256)
            .find(|&s| decorations(s, true) > 0)
            .expect("some seed decorates the walled edges");
        assert!(decorations(seed, true) > 0);
        assert_eq!(decorations(seed, false), 0, "same seed: open edges carry none");
    }

    #[test]
    fn pool_rim_walls_the_maze_edge() {
        // A water cell at the top-right corner: two of its edges are the grid
        // boundary and two face open cells. All four are rimmed (the maze perimeter
        // is framed, not left open), so the cell gets four rim skirts.
        let json = r#"{"grid":[["S",[{"type":"W","wallType":"water"}]],["F"," "]]}"#;
        let mut app = make_playing_app_with(json);
        let rims = app.world_mut().query::<&PoolRim>().iter(app.world()).count();
        assert_eq!(rims, 4, "an edge pool is rimmed on its grid-boundary sides too");
    }

    #[test]
    fn non_occluding_side_reshapes_a_corridor_door() {
        // A door in a straight N–S corridor is a single swing leaf. Turning one
        // lateral wall into a non-occluding water cell removes the swing anchor
        // (its panel is suppressed) AND opens that side, so the door instead seals
        // each open edge with its own leaf — the two passable ends plus the water
        // side — three leaves in all.
        let swing = r#"{"grid":[["W","S","W"],["W","D","W"],["W","F","W"]]}"#;
        let water =
            r#"{"grid":[["W","S","W"],[[{"type":"W","wallType":"water"}],"D","W"],["W","F","W"]]}"#;
        let leaves = |json: &str| {
            let mut app = make_playing_app_with(json);
            app.world_mut().query::<&DoorMarker>().iter(app.world()).count()
        };
        assert_eq!(leaves(swing), 1, "straight corridor → single swing leaf");
        assert_eq!(
            leaves(water), 3,
            "non-occluding lateral → per-edge leaves on the two ends + the water side",
        );
    }

    #[test]
    fn upper_level_door_holds_when_a_same_cell_live_door_opens() {
        use maze::Direction;
        // Level 0 (live): door at (1,1), key at (2,1), start (2,0), finish (0,2).
        // Level 1: ALSO a door at (1,1) (same grid coords). Opening level 0's door
        // must NOT disturb level 1's leaf — a regression guard for the cross-level
        // door bug where an upper-level leaf slid down into the live doorway.
        let l0 = r#"{"grid":[[" "," ","F"],[" ","D"," "],["S","K"," "]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"],[" ","D"," "],[" "," "," "]]}"#;
        let mut app = make_playing_app_with_levels(&[l0, l1]);

        // The upper (level-1) leaves spawn near y = LEVEL_HEIGHT (3); the live
        // (level-0) leaves near y = 0. Capture the upper leaves' resting heights.
        let upper_ys = |app: &mut App| -> Vec<f32> {
            let mut v: Vec<f32> = app
                .world_mut()
                .query::<(&DoorMarker, &Transform)>()
                .iter(app.world())
                .map(|(_, t)| t.translation.y)
                .filter(|y| *y > 1.5)
                .collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v
        };
        let before = upper_ys(&mut app);
        assert!(!before.is_empty(), "level 1 has door leaves up at LEVEL_HEIGHT");

        {
            let mut state = app.world_mut().resource_mut::<GameState>();
            state.game.move_player(Direction::Right); // collect the key
            state.game.move_player(Direction::Up); // walk into the level-0 door → unlock
            let _ = state.game.tick(10_000.0); // Opening → Open
        }
        app.update(); // door_animation_system runs

        let after = upper_ys(&mut app);
        assert_eq!(
            before, after,
            "the level-1 door leaves must stay put when the level-0 door at the same (row, col) opens",
        );
    }

    #[test]
    fn a_vertically_sliding_door_leaf_hides_when_it_would_intrude_on_an_adjacent_level() {
        use crate::state::DoorStyle;
        use maze::Direction;
        // `S K D F` straight corridor: collect the key, walk into the door, open it.
        let maze = r#"{"grid":[["S","K","D","F"]]}"#;
        let upper = r#"{"grid":[["S"," "," ","F"]]}"#;
        let open_door = |app: &mut App| {
            {
                let mut state = app.world_mut().resource_mut::<GameState>();
                state.game.move_player(Direction::Right); // collect the key
                state.game.move_player(Direction::Right); // into the door → unlock
                let _ = state.game.tick(10_000.0); // Opening → Open
            }
            app.update(); // door_animation_system sets visibility
        };
        let hidden = |app: &mut App| {
            app.world_mut()
                .query::<(&DoorMarker, &Visibility)>()
                .iter(app.world())
                .filter(|(_, v)| matches!(v, Visibility::Hidden))
                .count()
        };

        // Single-level portcullis: rises into open sky (no level above) → stays visible.
        let cfg = GameConfig { door_style: DoorStyle::Portcullis, ..GameConfig::default() };
        let mut app = make_playing_app_with_maze_and_config(maze, cfg);
        open_door(&mut app);
        assert_eq!(hidden(&mut app), 0, "a top-level portcullis stays visibly raised");

        // Two-level portcullis on the bottom level: a level sits above → hidden when open.
        let cfg = GameConfig { door_style: DoorStyle::Portcullis, ..GameConfig::default() };
        let mut app = make_playing_app_with_levels_and_config(&[maze, upper], cfg);
        open_door(&mut app);
        assert!(hidden(&mut app) > 0, "an intruding portcullis hides when open");

        // Two-level SLIDE on the bottom level: no level below → stays visible.
        let cfg = GameConfig { door_style: DoorStyle::Slide, ..GameConfig::default() };
        let mut app = make_playing_app_with_levels_and_config(&[maze, upper], cfg);
        open_door(&mut app);
        assert_eq!(hidden(&mut app), 0, "a bottom-level slide has no level below → stays visible");
    }

    #[test]
    fn an_upper_pool_level_is_lifted_and_its_underside_sealed() {
        use crate::world::{UndersideSeal, LEVEL_HEIGHT, POOL_GAP};

        // A pool-free two-level run is NOT lifted: no underside seals.
        let plain0 = r#"{"grid":[["S"," ","F"],[" "," "," "]]}"#;
        let plain1 = r#"{"grid":[["S"," ","F"],[" "," "," "]]}"#;
        let mut app = make_playing_app_with_levels(&[plain0, plain1]);
        assert_eq!(
            app.world_mut().query::<&UndersideSeal>().iter(app.world()).count(),
            0,
            "a pool-free stack needs no underside seal and isn't lifted",
        );

        // Level 1 carries a lava pool → that level is lifted by POOL_GAP and every
        // cell's underside is sealed, so from below the pool reads like any cell.
        let l0 = r#"{"grid":[["S"," ","F"],[" "," "," "]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"lava"}],"W"]]}"#;
        let mut app = make_playing_app_with_levels(&[l0, l1]);
        assert!(
            app.world_mut().query::<&UndersideSeal>().iter(app.world()).count() > 0,
            "the pool-bearing upper level seals its cells' undersides",
        );

        // The lava surface rests at the LIFTED level-1 floor (LEVEL_HEIGHT +
        // POOL_GAP), recessed RECESS_DEPTH (0.3) below it — so its basin sits well
        // above the level-0 wall-top (LEVEL_HEIGHT), not poking into the level below.
        let surf_y = app
            .world_mut()
            .query::<(&LavaSurface, &Transform)>()
            .iter(app.world())
            .map(|(_, t)| t.translation.y)
            .next()
            .expect("a lava surface on the upper level");
        let lifted_floor = LEVEL_HEIGHT + POOL_GAP;
        assert!(
            (surf_y - (lifted_floor - 0.3)).abs() < 0.15,
            "lava surface {surf_y} should rest just below the lifted floor {lifted_floor}",
        );
        assert!(
            surf_y > LEVEL_HEIGHT,
            "the pool surface must sit above the level-0 wall-top ({LEVEL_HEIGHT})",
        );
    }

    #[test]
    fn support_poles_brace_a_floating_upper_level_at_its_corners() {
        use crate::world::support_pole::SupportPole;
        let l0 = r#"{"grid":[["S"," ","F"],[" ","W"," "],[" "," "," "]]}"#;
        let l1 = r#"{"grid":[["S"," ","F"],[" ","W"," "],[" "," "," "]]}"#;

        // Lava walls + open perimeter: nothing solid holds up level 1, so it's
        // braced at all four corners (none over a solid wall or a perimeter wall).
        let floating = GameConfig {
            wall_type: WallType::Lava,
            perimeter_walls: false,
            ..GameConfig::default()
        };
        let mut app = make_playing_app_with_levels_and_config(&[l0, l1], floating);
        assert_eq!(
            app.world_mut().query::<&SupportPole>().iter(app.world()).count(),
            4,
            "a floating, no-solid-walls upper level is braced at all four corners",
        );

        // A solid (brick) perimeter wall carries the same-size upper level's edges,
        // so its corners need no poles.
        let walled = GameConfig {
            wall_type: WallType::Brick,
            perimeter_walls: true,
            ..GameConfig::default()
        };
        let mut app2 = make_playing_app_with_levels_and_config(&[l0, l1], walled);
        assert_eq!(
            app2.world_mut().query::<&SupportPole>().iter(app2.world()).count(),
            0,
            "a solid perimeter carries the corners, so no poles",
        );
    }

    #[test]
    fn pool_rim_skirts_every_non_pool_edge() {
        // A lone water cell ringed by four passable cells gets a rim skirt on each
        // of its four edges (the recess wall up to floor level).
        let json = r#"{"grid":[["S"," "," "],[" ",[{"type":"W","wallType":"water"}]," "],[" "," ","F"]]}"#;
        let mut app = make_playing_app_with(json);
        let rims = app.world_mut().query::<&PoolRim>().iter(app.world()).count();
        assert_eq!(rims, 4, "four non-pool edges → four rim skirts");
    }

    #[test]
    fn lava_cell_spawns_bobbing_rocks() {
        // A single lava cell is well under the global rock budget, so it seeds the
        // full per-cell base of rocks that the lava animation system bobs through
        // the surface.
        let json = r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"lava"}],"W"]]}"#;
        let mut app = make_playing_app_with(json);
        let rocks = app.world_mut().query::<&LavaRock>().iter(app.world()).count();
        assert_eq!(rocks, 2, "one lava cell (under budget) seeds two rocks");
    }

    #[test]
    fn pool_animation_systems_displace_the_surfaces() {
        // After entering Playing, the water/lava animation systems have run, so
        // each surface carries the position-phased wave's tilt rather than the
        // identity rotation it was spawned with — a regression guard that the
        // systems are wired into `build_app` and reach the pool surfaces.
        let water = r#"{"grid":[["S"," ","F"],["W",[{"type":"W","wallType":"water"}],"W"]]}"#;
        let mut app = make_playing_app_with(water);
        let rot = app
            .world_mut()
            .query_filtered::<&Transform, With<WaterSurface>>()
            .iter(app.world())
            .next()
            .expect("a water surface")
            .rotation;
        assert!(
            rot.angle_between(Quat::IDENTITY) > 1e-5,
            "water_animation_system should have tilted the surface",
        );
    }

    #[test]
    fn adjacent_pools_share_no_rim() {
        // Two side-by-side water cells: each rims its three outward edges, but the
        // shared edge between them is left open so they read as one continuous
        // basin — six skirts, not eight.
        let json = r#"{"grid":[["S"," "," "," "],[" ",[{"type":"W","wallType":"water"}],[{"type":"W","wallType":"water"}]," "],[" "," "," ","F"]]}"#;
        let mut app = make_playing_app_with(json);
        let rims = app.world_mut().query::<&PoolRim>().iter(app.world()).count();
        assert_eq!(rims, 6, "the shared same-type pool edge carries no rim");
    }

    #[test]
    fn adjacent_different_pools_are_divided_by_a_rim() {
        // A water cell beside a lava cell: the border between *different* pool
        // types is walled (a skirt from each side) so they don't read as merged —
        // so every edge is rimmed: four skirts each, eight in all.
        let json = r#"{"grid":[["S"," "," "," "],[" ",[{"type":"W","wallType":"water"}],[{"type":"W","wallType":"lava"}]," "],[" "," "," ","F"]]}"#;
        let mut app = make_playing_app_with(json);
        let rims = app.world_mut().query::<&PoolRim>().iter(app.world()).count();
        assert_eq!(rims, 8, "a water↔lava border is walled on both sides");
    }
}
