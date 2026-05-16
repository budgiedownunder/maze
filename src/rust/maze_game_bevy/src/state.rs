use bevy::prelude::*;
use maze::{Direction, MazeGame};
use std::collections::HashSet;
use std::f32::consts::PI;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub(crate) enum AppState {
    #[default]
    TitleScreen,
    Playing,
}

#[derive(Resource)]
pub(crate) struct PendingMazeJson(pub(crate) Option<String>);

#[derive(Resource)]
pub(crate) struct TitleTimer(pub(crate) Timer);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum GridFacing {
    North,
    East,
    South,
    West,
}

impl GridFacing {
    pub(crate) fn turn_left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    pub(crate) fn turn_right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub(crate) fn to_direction(self) -> Direction {
        match self {
            Self::North => Direction::Up,
            Self::East => Direction::Right,
            Self::South => Direction::Down,
            Self::West => Direction::Left,
        }
    }

    pub(crate) fn to_yaw(self) -> f32 {
        match self {
            Self::North => 0.0,
            Self::East => PI + PI / 2.0,
            Self::South => PI,
            Self::West => PI / 2.0,
        }
    }
}

pub(crate) struct Animation {
    pub(crate) start_pos: Vec3,
    pub(crate) target_pos: Vec3,
    pub(crate) start_yaw: f32,
    pub(crate) target_yaw: f32,
    pub(crate) elapsed: f32,
    pub(crate) duration: f32,
}

impl Animation {
    fn progress(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub(crate) fn current_pos(&self) -> Vec3 {
        self.start_pos.lerp(self.target_pos, self.progress())
    }

    pub(crate) fn current_yaw(&self) -> f32 {
        self.start_yaw + (self.target_yaw - self.start_yaw) * self.progress()
    }
}

#[derive(Resource)]
pub(crate) struct GameState {
    pub(crate) game: MazeGame,
    pub(crate) grid: Vec<Vec<char>>,
    pub(crate) facing: GridFacing,
    pub(crate) visual_pos: Vec3,
    pub(crate) visual_yaw: f32,
    pub(crate) visual_pitch: f32,
    pub(crate) anim: Option<Animation>,
    pub(crate) explored: HashSet<(usize, usize)>,
    pub(crate) won: bool,
    pub(crate) lost: bool,
    pub(crate) paused: bool,
}

#[derive(Resource)]
pub(crate) struct GameClock {
    pub(crate) remaining_secs: f32,
    pub(crate) elapsed_secs: f32,
    pub(crate) last_displayed_secs: i32,
}

/// Per-session game configuration handed down from the JS host (via
/// `maze_game_bevy_wasm::start_with_config`). When no host config is provided
/// (native `cargo run`, or the bare wasm `start()` path), `Default` produces
/// values that preserve the Step 2 hardcoded behaviour: a 60-second clock,
/// the splash title "MAZE 3D", and `rows`/`cols == 0` so `spawn_world` knows
/// to fall back to the built-in demo grid instead of running the maze
/// generator.
#[derive(Resource, Clone, Debug)]
pub struct GameConfig {
    pub difficulty: Option<String>,
    pub rows: u32,
    pub cols: u32,
    pub timer_seconds: f32,
    pub seed: u64,
    pub min_solution_length: u32,
    /// On-screen pixel size of each minimap cell.
    pub minimap_cell_px: u32,
    /// Minimap cells visible in each direction from the player.
    pub minimap_radius: u32,
    pub title: String,
    /// Free-text label shown in the in-game status bar
    pub mode: String,
    /// Per-difficulty landmark / spatial-orientation toggles. Each
    /// landmark technique has its own flag here so a build can disable
    /// any individual technique at runtime via the server config.
    pub landmarks: Landmarks,
}

/// Toggle bag for the spatial-orientation landmark techniques. Each new
/// landmark sub-step adds one field (default `true`). The host populates
/// this from `[game.play3d.<difficulty>.landmarks]` in the server config.
#[derive(Clone, Debug)]
pub struct Landmarks {
    /// Per-cell wall tint variation — when `false`, every cell uses the
    /// base wall material variant (reproduces the pre-5A look).
    pub wall_tint: bool,
    /// Dead-end landmark objects — when `true`, every dead-end cell
    /// (passable cell with exactly one open neighbour, excluding start
    /// and finish) gets a single distinctive object picked by hashing
    /// `(row, col, seed)`. When `false`, dead-ends render bare.
    pub dead_end_objects: bool,
    /// Sparse wall decorations — when `true`, ~1 in 8 wall panels gets a
    /// decorative element (vent grate, faded poster, rune, window glow)
    /// projected on the inside face. Both placement and decoration kind are
    /// seeded.
    pub wall_decorations: bool,
}

impl Default for Landmarks {
    fn default() -> Self {
        Self {
            wall_tint: true,
            dead_end_objects: true,
            wall_decorations: true,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            difficulty: None,
            rows: 0,
            cols: 0,
            timer_seconds: 60.0,
            seed: 0,
            min_solution_length: 0,
            minimap_cell_px: crate::hud::minimap::MAP_CELL_PX,
            minimap_radius: crate::hud::minimap::MAP_RADIUS,
            title: "MAZE 3D".to_string(),
            mode: "Play".to_string(),
            landmarks: Landmarks::default(),
        }
    }
}

/// Outcome of a 3D-game session, as reported to the JS host on completion.
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GameOutcome {
    Win,
    Lose,
}

/// Extensible result payload dispatched to the JS host as the `detail` of a
/// `maze-game-result` CustomEvent on `window`. Fixed fields cover the metrics
/// known up-front (outcome, elapsed time, maze dimensions, optional difficulty);
/// `extras` is an open map so future per-feature metrics (moves, hint count,
/// deaths, etc.) can be added without breaking the contract.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub outcome: GameOutcome,
    pub elapsed_ms: u64,
    pub difficulty: Option<String>,
    pub rows: u32,
    pub cols: u32,
    /// Seed of the maze actually played. `None` for the no-config / demo path
    /// where no seed was ever supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub extras: std::collections::BTreeMap<String, serde_json::Value>,
}

/// Dispatches a `maze-game-result` CustomEvent on the browser `window` so the
/// hosting page (React `/game/` wrapper, or the MAUI WebView) can react to the
/// outcome. Wrapped behind `#[cfg(target_arch = "wasm32")]` so native builds
/// (cargo run -p maze_game_bevy) compile without any browser dependencies.
#[cfg(target_arch = "wasm32")]
pub(crate) fn dispatch_game_result(result: &GameResult) {
    use wasm_bindgen::JsValue;
    let Ok(json) = serde_json::to_string(result) else {
        return;
    };
    let Some(window) = web_sys::window() else {
        return;
    };
    let detail = js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL);
    let init = web_sys::CustomEventInit::new();
    init.set_detail(&detail);
    if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("maze-game-result", &init) {
        let _ = window.dispatch_event(&event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn dispatch_game_result(_result: &GameResult) {
    // Native builds have no browser host — nothing to do. Kept as an empty
    // function so call sites can stay platform-agnostic.
}

/// Dispatches a `maze-game-paused` CustomEvent on the browser `window` so the
/// container page/application can swap its pause/play state in sync with the Bevy
/// game state. Detail is `{ "paused": bool }`.
#[cfg(target_arch = "wasm32")]
pub(crate) fn dispatch_pause_state(paused: bool) {
    use wasm_bindgen::JsValue;
    let Some(window) = web_sys::window() else {
        return;
    };
    let json = format!("{{\"paused\":{}}}", paused);
    let detail = js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL);
    let init = web_sys::CustomEventInit::new();
    init.set_detail(&detail);
    if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("maze-game-paused", &init) {
        let _ = window.dispatch_event(&event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn dispatch_pause_state(_paused: bool) {}
