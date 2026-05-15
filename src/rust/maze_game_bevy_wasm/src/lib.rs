use bevy::prelude::*;
use maze_game_bevy::GameConfig;
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

#[wasm_bindgen]
pub fn start_with_maze(maze_json: &str) {
    let mut app = make_app();
    maze_game_bevy::build_app(&mut app, Some(maze_json));
    app.run();
}

/// Shape of the JSON payload accepted by `start_with_config`. Mirrors the
/// `Play3dConfigResponse` from the server endpoint, plus an optional
/// `mazeJson` escape hatch for the `/game/?id=…` path. All other fields are
/// fixed-per-session config (difficulty / dimensions / timer / seed / splash
/// title) handed straight through to Bevy as a `GameConfig` resource.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartConfig {
    #[serde(default)]
    difficulty: Option<String>,
    #[serde(default)]
    rows: u32,
    #[serde(default)]
    cols: u32,
    timer_seconds: f32,
    #[serde(default)]
    seed: u64,
    #[serde(default)]
    min_solution_length: u32,
    #[serde(default = "default_minimap_cell_px")]
    minimap_cell_px: u32,
    #[serde(default = "default_minimap_radius")]
    minimap_radius: u32,
    title: String,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    maze_json: Option<String>,
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

    // Resolve the maze JSON: explicit `mazeJson` wins; otherwise, when
    // dimensions are provided, generate from the seed/min-spine constraint.
    // If neither is provided Bevy falls back to its built-in demo grid.
    let maze_json: Option<String> = if let Some(json) = cfg.maze_json.clone() {
        Some(json)
    } else if cfg.rows > 0 && cfg.cols > 0 {
        Some(
            maze_game_bevy::generate_maze_json(
                cfg.rows,
                cfg.cols,
                cfg.seed,
                cfg.min_solution_length,
            )
            .map_err(|err| JsValue::from_str(&format!("Maze generation failed: {err}")))?,
        )
    } else {
        None
    };

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
    });
    maze_game_bevy::build_app(&mut app, maze_json.as_deref());
    app.run();
    Ok(())
}
