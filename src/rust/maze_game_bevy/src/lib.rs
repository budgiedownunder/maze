mod hud;
mod images;
mod movement;
mod overlays;
mod palette;
mod state;
mod world;

pub use state::{GameConfig, GameOutcome, GameResult, Landmarks, SkyType, WallType};
pub use world::generate_maze_json;

use bevy::prelude::*;

pub fn build_app(app: &mut App, maze_json: Option<&str>) {
    use crate::hud::{bag, clock, minimap, statusbar};
    use crate::movement::{movement_system, pickup_system, quit_system};
    use crate::overlays::{lose, pause, title, win};
    use crate::state::{AppState, PendingMazeJson, TitleTimer};
    use crate::world::{
        objects::{
            self,
            dead_end::brazier_flicker_system,
            door::{door_animation_system, door_tick_system},
            key_holder::key_holder_system,
        },
        sky, spawn_world,
    };

    // `GameConfig` is the seam the JS host uses (via
    // `maze_game_bevy_wasm::start_with_config`) to drive difficulty / timer /
    // splash title / seed. `init_resource` only inserts the default when the
    // caller didn't already supply one, so a host-provided config is
    // preserved.
    app.init_resource::<GameConfig>();
    app.insert_resource(PendingMazeJson(maze_json.map(String::from)))
        .init_state::<AppState>()
        .insert_resource(TitleTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(AppState::TitleScreen), title::setup_title)
        .add_systems(Update, title::tick_title.run_if(in_state(AppState::TitleScreen)))
        .add_systems(Update, title::title_resize_system.run_if(in_state(AppState::TitleScreen)))
        .add_systems(OnExit(AppState::TitleScreen), title::teardown_title)
        .add_systems(OnEnter(AppState::Playing), spawn_world)
        .add_systems(Update, movement_system.run_if(in_state(AppState::Playing)))
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
        .add_systems(Update, objects::finish::orb::orb_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, brazier_flicker_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, key_holder_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, pickup_system.run_if(in_state(AppState::Playing)))
        // The door countdown runs in `FixedUpdate` for deterministic, frame-rate
        // independent opening; the hinge animation reads the resulting state in
        // `Update` for smooth per-frame motion.
        .add_systems(FixedUpdate, door_tick_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, door_animation_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, bag::bag_hud_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, pause::pause_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, sky::sky_dome_follow_camera.run_if(in_state(AppState::Playing)))
        .add_systems(Update, world::camera_fov_resize_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, quit_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlays::title::TitleEntity;
    use crate::state::{AppState, GameState, GridFacing, SkyType, WallType};
    use crate::world::{
        camera_fov_for_aspect, camera_pos_for, cell_centre,
        decorations::{floor::FloorAccent, wall::WallDecoration},
        demo_grid,
        floor::FloorCell,
        initial_facing,
        objects::{
            dead_end::{BrazierBowl, DeadEndObject},
            door::DoorMarker,
            finish::orb::FinishOrb,
            key_holder::KeyMarker,
        },
        sky::dome::SkyDome,
        walls::WallCell,
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
        use crate::world::objects::dead_end::is_dead_end;
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
        use crate::world::objects::dead_end::{dead_end_object_index, is_dead_end};
        let grid = demo_grid();
        for seed in 0u64..1024 {
            for (r, row) in grid.iter().enumerate() {
                for (c, &cell) in row.iter().enumerate() {
                    // Mirror `spawn_dead_end_object_for_cell`'s exclusions: only
                    // cells that actually receive a landmark are candidates.
                    if matches!(cell, 'S' | 'F' | 'K' | 'D') {
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
        assert_eq!(WallType::Brick.to_kind_index(), WALL_MATERIAL_BRICK);
        assert_eq!(
            WallType::DressedStone.to_kind_index(),
            WALL_MATERIAL_DRESSED_STONE
        );
        assert_eq!(WallType::Wood.to_kind_index(), WALL_MATERIAL_WOOD);
        assert_eq!(
            WallType::Cobblestone.to_kind_index(),
            WALL_MATERIAL_COBBLESTONE
        );
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
}
