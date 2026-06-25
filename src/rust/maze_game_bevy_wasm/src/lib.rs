use bevy::prelude::*;
use maze_game_bevy::{
    DoorStyle, EnemyType, FinishType, GameConfig, HealthStyle, KeyHolderStyle, Landmarks,
    LayeredAlignment, LevelDifficultyChange, PendingLevels, SkyType, WallType,
};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Maze 3D".into(),
            canvas: Some("#bevy-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }));
    app
}

#[wasm_bindgen]
pub fn start() {
    let mut app = make_app();
    maze_game_bevy::build_app(&mut app, None);
    app.run();
}

/// Shape of the JSON payload accepted by `start_with_config`. Mirrors the
/// `Play3dConfigResponse` from the server endpoint, plus an optional
/// `mazeJson` escape hatch for the `/game/?id=…` path. All other fields are
/// fixed-per-session config (difficulty / dimensions / timer / seed / splash
/// title) handed straight through to Bevy as a `GameConfig` resource.
///
/// Every field has a serde default so the host page can send a minimal
/// payload — e.g. the `/game/?id=…` path only sets `mazeJson` + the two
/// fixed overrides for user-edited mazes (`wallType` and
/// `landmarks.wallMaterialVariation`) and lets the rest fall through to
/// the same values as `GameConfig::default()`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartConfig {
    #[serde(default)]
    difficulty: Option<String>,
    #[serde(default)]
    rows: u32,
    #[serde(default)]
    cols: u32,
    #[serde(default = "default_timer_seconds")]
    timer_seconds: f32,
    #[serde(default)]
    seed: u64,
    #[serde(default)]
    min_solution_length: u32,
    #[serde(default = "default_minimap_cell_px")]
    minimap_cell_px: u32,
    #[serde(default = "default_minimap_radius")]
    minimap_radius: u32,
    #[serde(default = "default_title")]
    title: String,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    landmarks: LandmarksStartConfig,
    #[serde(default)]
    sky_type: String,
    #[serde(default)]
    wall_type: String,
    #[serde(default = "default_perimeter_walls")]
    perimeter_walls: bool,
    #[serde(default)]
    door_style: String,
    #[serde(default)]
    key_holder: String,
    #[serde(default)]
    door_count: u32,
    #[serde(default)]
    spare_doors: u32,
    #[serde(default)]
    spare_keys: u32,
    #[serde(default)]
    enemy_count: u32,
    #[serde(default)]
    health_count: u32,
    #[serde(default)]
    treasure_count: u32,
    #[serde(default = "default_enemy_move_period_ms")]
    enemy_move_period_ms: f32,
    #[serde(default = "default_enemy_damage")]
    enemy_damage: u32,
    #[serde(default = "default_max_hp")]
    max_hp: u32,
    #[serde(default = "default_starting_hp")]
    starting_hp: u32,
    #[serde(default = "default_enemy_type")]
    enemy_type: String,
    #[serde(default = "default_health_style")]
    health_style: String,
    #[serde(default)]
    maze_json: Option<String>,
    /// Multi-level run settings. Mirrors the server's `levels` response group.
    /// A `count` of 1 (the default) is a single-level game and every other field
    /// is inert.
    #[serde(default)]
    levels: LevelsStartConfig,
}

/// Shape of the nested `levels` object in the host JSON payload — the
/// multi-level run settings. Mirrors the server's `LevelsResponse`
/// field-for-field (the host page forwards the whole play3d-config response, so
/// the nested object arrives intact); kept as a separate type so it defaults
/// field-wise. The server may also send `perimeterRandom` / `top`, which serde
/// ignores here until a later step consumes them.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LevelsStartConfig {
    #[serde(default = "default_levels_count")]
    count: u32,
    #[serde(default = "default_finish_type")]
    finish_type: String,
    #[serde(default = "default_difficulty_change")]
    difficulty_change: String,
    #[serde(default = "default_reset_bag")]
    reset_bag: bool,
    #[serde(default = "default_alignment")]
    alignment: String,
    #[serde(default)]
    hide_completed_enemies: bool,
    #[serde(default)]
    perimeter_random: bool,
    #[serde(default)]
    top: Option<TopStartConfig>,
}

/// Optional final-level scene override in the host's `levels` object — its own
/// `skyType` / `perimeterWalls`, each absent to inherit the base difficulty.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopStartConfig {
    #[serde(default)]
    sky_type: Option<String>,
    #[serde(default)]
    perimeter_walls: Option<bool>,
}

impl Default for LevelsStartConfig {
    fn default() -> Self {
        Self {
            count: default_levels_count(),
            finish_type: default_finish_type(),
            difficulty_change: default_difficulty_change(),
            reset_bag: default_reset_bag(),
            alignment: default_alignment(),
            hide_completed_enemies: false,
            perimeter_random: false,
            top: None,
        }
    }
}

fn default_levels_count() -> u32 {
    1
}

fn default_finish_type() -> String {
    "ladder".to_string()
}

fn default_difficulty_change() -> String {
    "easier".to_string()
}

fn default_reset_bag() -> bool {
    true
}

fn default_alignment() -> String {
    "edge".to_string()
}

/// Defaults for the per-game tuning knobs. Match the values in
/// `MazeGameOptions::default()` so omitting them from the host payload
/// (e.g. the bare `/game/?id=` path) yields the same behaviour the maze
/// crate would have used on its own.
fn default_enemy_move_period_ms() -> f32 {
    1500.0
}

fn default_enemy_damage() -> u32 {
    1
}

fn default_max_hp() -> u32 {
    3
}

fn default_starting_hp() -> u32 {
    3
}

fn default_enemy_type() -> String {
    "goblin".to_string()
}

fn default_health_style() -> String {
    "heart".to_string()
}

/// The maze is walled at its perimeter by default (even under an open sky), so a
/// host payload that omits the flag gets the traditional enclosed-maze look.
fn default_perimeter_walls() -> bool {
    true
}

/// Shape of the nested `landmarks` object in the host JSON payload —
/// per-difficulty toggles for the landmark / spatial-orientation
/// features. Mirrors the server's `LandmarksResponse` field-for-field;
/// kept as a separate type so we can default it field-wise.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LandmarksStartConfig {
    #[serde(default = "default_landmarks_wall_tint")]
    wall_tint: bool,
    #[serde(default = "default_landmarks_dead_end_objects")]
    dead_end_objects: bool,
    #[serde(default = "default_landmarks_wall_decorations")]
    wall_decorations: bool,
    #[serde(default = "default_landmarks_floor_accents")]
    floor_accents: bool,
    #[serde(default = "default_landmarks_wall_material_variation")]
    wall_material_variation: bool,
}

impl Default for LandmarksStartConfig {
    fn default() -> Self {
        Self {
            wall_tint: default_landmarks_wall_tint(),
            dead_end_objects: default_landmarks_dead_end_objects(),
            wall_decorations: default_landmarks_wall_decorations(),
            floor_accents: default_landmarks_floor_accents(),
            wall_material_variation: default_landmarks_wall_material_variation(),
        }
    }
}

fn default_landmarks_wall_tint() -> bool {
    true
}

fn default_landmarks_dead_end_objects() -> bool {
    true
}

fn default_landmarks_wall_decorations() -> bool {
    true
}

fn default_landmarks_floor_accents() -> bool {
    true
}

fn default_landmarks_wall_material_variation() -> bool {
    true
}

/// The minimap cell pixel size the game shipped with — used when the host
/// payload omits `minimapCellPx` (e.g. an older `/game/index.html`).
fn default_minimap_cell_px() -> u32 {
    10
}

/// The minimap visible-radius the game shipped with — used when the host
/// payload omits `minimapRadius`.
fn default_minimap_radius() -> u32 {
    5
}

/// Timer the standalone game ships with when no preset / no override is
/// supplied. Matches `GameConfig::default().timer_seconds` so omitting
/// `timerSeconds` from the host payload yields the same value Bevy would
/// have used on its own — single source of truth lives in `state.rs`,
/// this is the wasm-boundary echo of it.
fn default_timer_seconds() -> f32 {
    60.0
}

/// Splash title the standalone game ships with when no preset / no
/// override is supplied. Mirrors `GameConfig::default().title`.
fn default_title() -> String {
    "MAZE 3D".to_string()
}

/// Start the Bevy game with a host-supplied session config. Called by
/// `/game/index.html` after it fetches the preset from
/// `GET /api/v1/game/play3d-config?difficulty=…` (with optional `?seed=`
/// override).
///
/// Generation happens here, *before* Bevy enters `AppState::Playing`, so any
/// failure (e.g. `min_solution_length` too high for the chosen seed) surfaces
/// as a `JsValue` error the HTML host can render in `#loading`. Doing it
/// inside Bevy would leave the rest of the schedule expecting a `GameState`
/// that was never inserted, and panic.
#[wasm_bindgen]
pub fn start_with_config(json: &str) -> Result<(), JsValue> {
    let cfg: StartConfig = serde_json::from_str(json)
        .map_err(|err| JsValue::from_str(&format!("Invalid start_with_config payload: {err}")))?;

    // Resolve the maze source. Explicit `mazeJson` always wins (the saved-maze
    // path — single-level). Otherwise, when dimensions are provided, generate:
    // a multi-level run (`levels.count > 1`) builds the whole stack up front and
    // is injected via `PendingLevels`; a single-level run generates one maze. With
    // neither, Bevy falls back to its built-in demo grid.
    let mut maze_json: Option<String> = None;
    let mut pending_levels: Option<Vec<String>> = None;
    if let Some(json) = cfg.maze_json.clone() {
        maze_json = Some(json);
    } else if cfg.rows > 0 && cfg.cols > 0 {
        if cfg.levels.count > 1 {
            pending_levels = Some(
                maze_game_bevy::generate_level_maze_jsons(
                    cfg.rows,
                    cfg.cols,
                    cfg.seed,
                    cfg.min_solution_length,
                    cfg.door_count,
                    cfg.spare_doors,
                    cfg.spare_keys,
                    cfg.enemy_count,
                    cfg.health_count,
                    cfg.treasure_count,
                    cfg.levels.count,
                    LevelDifficultyChange::from_wire_str(&cfg.levels.difficulty_change),
                )
                .map_err(|err| JsValue::from_str(&format!("Maze generation failed: {err}")))?,
            );
        } else {
            maze_json = Some(
                maze_game_bevy::generate_maze_json(
                    cfg.rows,
                    cfg.cols,
                    cfg.seed,
                    cfg.min_solution_length,
                    cfg.door_count,
                    cfg.spare_doors,
                    cfg.spare_keys,
                    cfg.enemy_count,
                    cfg.health_count,
                    cfg.treasure_count,
                )
                .map_err(|err| JsValue::from_str(&format!("Maze generation failed: {err}")))?,
            );
        }
    }

    let mut app = make_app();
    app.insert_resource(GameConfig {
        difficulty: cfg.difficulty,
        rows: cfg.rows,
        cols: cfg.cols,
        timer_seconds: cfg.timer_seconds,
        seed: cfg.seed,
        min_solution_length: cfg.min_solution_length,
        minimap_cell_px: cfg.minimap_cell_px,
        minimap_radius: cfg.minimap_radius,
        title: cfg.title,
        mode: cfg.mode,
        landmarks: Landmarks {
            wall_tint: cfg.landmarks.wall_tint,
            dead_end_objects: cfg.landmarks.dead_end_objects,
            wall_decorations: cfg.landmarks.wall_decorations,
            floor_accents: cfg.landmarks.floor_accents,
            wall_material_variation: cfg.landmarks.wall_material_variation,
        },
        sky_type: SkyType::from_wire_str(&cfg.sky_type),
        wall_type: WallType::from_wire_str(&cfg.wall_type),
        perimeter_walls: cfg.perimeter_walls,
        door_style: DoorStyle::from_wire_str(&cfg.door_style),
        key_holder: KeyHolderStyle::from_wire_str(&cfg.key_holder),
        enemy_move_period_ms: cfg.enemy_move_period_ms,
        enemy_damage: cfg.enemy_damage,
        max_hp: cfg.max_hp,
        starting_hp: cfg.starting_hp,
        enemy_type: EnemyType::from_wire_str(&cfg.enemy_type),
        health_style: HealthStyle::from_wire_str(&cfg.health_style),
        layered_alignment: LayeredAlignment::from_wire_str(&cfg.levels.alignment),
        finish_type: FinishType::from_wire_str(&cfg.levels.finish_type),
        reset_bag_between_levels: cfg.levels.reset_bag,
        hide_completed_enemies: cfg.levels.hide_completed_enemies,
        top_sky_type: cfg.levels.top.as_ref().and_then(|t| t.sky_type.as_deref()).map(SkyType::from_wire_str),
        top_perimeter_walls: cfg.levels.top.as_ref().and_then(|t| t.perimeter_walls),
        perimeter_random: cfg.levels.perimeter_random,
    });
    // A multi-level run is fed in via `PendingLevels`, which `spawn_world` reads
    // ahead of the single `PendingMazeJson` — the same seam the native demos use.
    if let Some(levels) = pending_levels {
        app.insert_resource(PendingLevels(levels));
    }
    maze_game_bevy::build_app(&mut app, maze_json.as_deref());
    app.run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_config_omitted_fields_take_defaults() {
        // The /game/?id=<id> host-page path now sends a minimal payload
        // containing only the maze JSON plus the two user-edited-maze
        // overrides. Every other field must deserialise to the same
        // value `GameConfig::default()` would have used — otherwise the
        // user-edited maze would silently inherit non-default values.
        let json = r#"{
            "wallType": "brick",
            "landmarks": { "wallMaterialVariation": false },
            "mazeJson": "{\"grid\":[[\"S\",\"F\"]]}"
        }"#;
        let cfg: StartConfig = serde_json::from_str(json).expect("minimal payload must parse");
        assert_eq!(cfg.timer_seconds, 60.0);
        assert_eq!(cfg.title, "MAZE 3D");
        assert_eq!(cfg.rows, 0);
        assert_eq!(cfg.cols, 0);
        assert_eq!(cfg.seed, 0);
        assert_eq!(cfg.min_solution_length, 0);
        assert_eq!(cfg.minimap_cell_px, 10);
        assert_eq!(cfg.minimap_radius, 5);
        assert_eq!(cfg.mode, "");
        assert_eq!(cfg.sky_type, "");
        assert_eq!(cfg.wall_type, "brick");
        // Omitting the perimeter flag defaults to walled (the enclosed-maze look).
        assert!(cfg.perimeter_walls);
        assert_eq!(cfg.door_count, 0);
        assert_eq!(cfg.spare_doors, 0);
        assert_eq!(cfg.spare_keys, 0);
        assert_eq!(cfg.enemy_count, 0);
        assert_eq!(cfg.health_count, 0);
        assert_eq!(cfg.treasure_count, 0);
        assert_eq!(cfg.enemy_move_period_ms, 1500.0);
        assert_eq!(cfg.enemy_damage, 1);
        assert_eq!(cfg.max_hp, 3);
        assert_eq!(cfg.starting_hp, 3);
        assert_eq!(cfg.enemy_type, "goblin");
        assert_eq!(cfg.health_style, "heart");
        assert!(cfg.difficulty.is_none());
        assert!(cfg.maze_json.is_some());
        // The single landmark override must take effect; the rest fall
        // back to true.
        assert!(cfg.landmarks.wall_tint);
        assert!(cfg.landmarks.dead_end_objects);
        assert!(cfg.landmarks.wall_decorations);
        assert!(cfg.landmarks.floor_accents);
        assert!(!cfg.landmarks.wall_material_variation);
    }

    #[test]
    fn start_config_can_disable_wall_tint_and_material_variation_together() {
        // The /game/?id=<id> path sends both landmarks toggles in one
        // payload — make sure both flip and the rest stay defaulted.
        let json = r#"{
            "wallType": "brick",
            "landmarks": { "wallTint": false, "wallMaterialVariation": false },
            "mazeJson": "{\"grid\":[[\"S\",\"F\"]]}"
        }"#;
        let cfg: StartConfig = serde_json::from_str(json).expect("payload must parse");
        assert!(!cfg.landmarks.wall_tint);
        assert!(!cfg.landmarks.wall_material_variation);
        // Other landmarks still default to true.
        assert!(cfg.landmarks.dead_end_objects);
        assert!(cfg.landmarks.wall_decorations);
        assert!(cfg.landmarks.floor_accents);
    }

    #[test]
    fn start_config_levels_default_to_a_single_level_when_omitted() {
        // The saved-maze and single-level difficulty paths send no `levels`
        // object; it must default to a single-level run with the documented
        // per-field defaults.
        let json = r#"{ "mazeJson": "{\"grid\":[[\"S\",\"F\"]]}" }"#;
        let cfg: StartConfig = serde_json::from_str(json).expect("payload must parse");
        assert_eq!(cfg.levels.count, 1);
        assert_eq!(cfg.levels.finish_type, "ladder");
        assert_eq!(cfg.levels.difficulty_change, "easier");
        assert!(cfg.levels.reset_bag);
        assert_eq!(cfg.levels.alignment, "edge");
        assert!(!cfg.levels.hide_completed_enemies);
        assert!(!cfg.levels.perimeter_random);
        assert!(cfg.levels.top.is_none());
    }

    #[test]
    fn start_config_parses_a_multi_level_levels_group() {
        // A curated multi-level preset forwards the server's whole response, so
        // the nested `levels` object arrives intact, including `perimeterRandom`
        // and the optional `top` scene override.
        let json = r#"{
            "rows": 9,
            "cols": 9,
            "levels": {
                "count": 3,
                "finishType": "random",
                "difficultyChange": "harder",
                "resetBag": false,
                "alignment": "centre",
                "perimeterRandom": true,
                "hideCompletedEnemies": true,
                "top": { "skyType": "day", "perimeterWalls": false }
            }
        }"#;
        let cfg: StartConfig = serde_json::from_str(json).expect("payload must parse");
        assert_eq!(cfg.levels.count, 3);
        assert_eq!(cfg.levels.finish_type, "random");
        assert_eq!(cfg.levels.difficulty_change, "harder");
        assert!(!cfg.levels.reset_bag);
        assert_eq!(cfg.levels.alignment, "centre");
        assert!(cfg.levels.hide_completed_enemies);
        assert!(cfg.levels.perimeter_random);
        let top = cfg.levels.top.as_ref().expect("top override parsed");
        assert_eq!(top.sky_type.as_deref(), Some("day"));
        assert_eq!(top.perimeter_walls, Some(false));
        // The wire strings resolve to the Bevy enums the GameConfig uses.
        assert_eq!(FinishType::from_wire_str(&cfg.levels.finish_type), FinishType::Random);
        assert_eq!(
            LayeredAlignment::from_wire_str(&cfg.levels.alignment),
            LayeredAlignment::Centre,
        );
        assert_eq!(
            LevelDifficultyChange::from_wire_str(&cfg.levels.difficulty_change),
            LevelDifficultyChange::Harder,
        );
    }
}
