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
    /// Atmospheric sky mode for this session. Determines the dome
    /// texture (gradient + clouds + stars) and a paired ambient +
    /// directional light preset. Default `Night`.
    pub sky_type: SkyType,
    /// Wall texture kind used by the per-cell tinted path (the path
    /// taken when [`Landmarks::wall_material_variation`] is `false`).
    /// When `wall_material_variation` is `true`, the per-quadrant
    /// material variation supersedes this setting — same bypass model
    /// as [`Landmarks::wall_tint`]. Default `Brick` so the pre-Step-14
    /// hard-coded look is preserved.
    pub wall_type: WallType,
}

/// Atmospheric sky modes. Each variant maps to a procedurally generated
/// dome texture + a paired light preset (see `world/sky`). Default is
/// `Night` so a missing or unrecognised config value preserves the
/// pre-Step-10 visual.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkyType {
    #[default]
    Night,
    Sunrise,
    Day,
    Sunset,
}

impl SkyType {
    /// Lowercase wire form, matching the JSON / TOML strings the server
    /// emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Night => "night",
            Self::Sunrise => "sunrise",
            Self::Day => "day",
            Self::Sunset => "sunset",
        }
    }

    /// Parses a wire string into a [`SkyType`]. Unknown values fall
    /// back to [`SkyType::Night`] rather than failing — the same
    /// design as the config-layer deserialiser, so a stale client +
    /// fresh server (or vice versa) keeps working with a sane default
    /// instead of crashing.
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "sunrise" => Self::Sunrise,
            "day" => Self::Day,
            "sunset" => Self::Sunset,
            _ => Self::Night,
        }
    }
}

/// Wall texture kinds for the per-cell tinted path. Each variant maps
/// to one of the `WALL_MATERIAL_*` indices in [`crate::world::walls`].
/// Default is `Brick` so a missing or unrecognised wire value preserves
/// the pre-Step-14 hard-coded look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WallType {
    #[default]
    Brick,
    DressedStone,
    Wood,
    Cobblestone,
}

impl WallType {
    /// `snake_case` wire form, matching the JSON / TOML strings the
    /// server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Brick => "brick",
            Self::DressedStone => "dressed_stone",
            Self::Wood => "wood",
            Self::Cobblestone => "cobblestone",
        }
    }

    /// Parses a wire string into a [`WallType`]. Unknown values fall
    /// back to [`WallType::Brick`] — same forgiving policy as
    /// [`SkyType::from_wire_str`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "dressed_stone" => Self::DressedStone,
            "wood" => Self::Wood,
            "cobblestone" => Self::Cobblestone,
            _ => Self::Brick,
        }
    }

    /// Returns the matching `WALL_MATERIAL_*` index used by
    /// [`crate::world::walls`]. Single source of truth so no call site
    /// hard-codes the integer mapping.
    pub fn to_kind_index(self) -> usize {
        match self {
            Self::Brick => crate::world::walls::WALL_MATERIAL_BRICK,
            Self::DressedStone => crate::world::walls::WALL_MATERIAL_DRESSED_STONE,
            Self::Wood => crate::world::walls::WALL_MATERIAL_WOOD,
            Self::Cobblestone => crate::world::walls::WALL_MATERIAL_COBBLESTONE,
        }
    }
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
    /// Sparse wall decorations — when `true`, ~1 in 10 wall panels gets a
    /// decorative element (vent grate, faded poster, rune, glowing glass)
    /// projected on the inside face. Both placement and decoration kind are
    /// seeded.
    pub wall_decorations: bool,
    /// Floor accents at junction cells — when `true`, every 3- or 4-way
    /// junction cell (excluding start / finish) gets a single flat accent
    /// (moss / cracked tile / mosaic / sigil) on its floor, kind picked by
    /// hashing `(row, col, seed)`. Reinforces "this is a decision point"
    /// memory.
    pub floor_accents: bool,
    /// Per-quadrant wall material variation — when `true`, the maze is
    /// split into a 2×2 NW/NE/SW/SE grid and each quadrant renders with
    /// its own wall material (brick / dressed stone / wood / cobblestone),
    /// seed-permuted so different seeds rotate the quadrant-to-kind
    /// mapping. Supersedes [`Self::wall_tint`] when on: each quadrant gets
    /// one fixed material kind and the per-cell tint hash is bypassed.
    pub wall_material_variation: bool,
}

impl Default for Landmarks {
    fn default() -> Self {
        Self {
            wall_tint: true,
            dead_end_objects: true,
            wall_decorations: true,
            floor_accents: true,
            wall_material_variation: true,
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
            sky_type: SkyType::default(),
            wall_type: WallType::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn anim(elapsed: f32, duration: f32) -> Animation {
        Animation {
            start_pos: Vec3::ZERO,
            target_pos: Vec3::splat(4.0),
            start_yaw: 0.0,
            target_yaw: 8.0,
            elapsed,
            duration,
        }
    }

    #[test]
    fn animation_at_start_is_at_start_position() {
        let a = anim(0.0, 1.0);
        assert_eq!(a.current_pos(), Vec3::ZERO);
        assert_eq!(a.current_yaw(), 0.0);
    }

    #[test]
    fn animation_at_end_is_at_target() {
        let a = anim(1.0, 1.0);
        assert_eq!(a.current_pos(), Vec3::splat(4.0));
        assert_eq!(a.current_yaw(), 8.0);
    }

    #[test]
    fn animation_at_midpoint_is_halfway() {
        // Smoothstep at t=0.5 evaluates to 0.5 exactly: t*t*(3 - 2t) = 0.5.
        let a = anim(0.5, 1.0);
        assert!((a.current_pos().x - 2.0).abs() < 1e-6);
        assert!((a.current_yaw() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn animation_overshoot_clamps_to_target() {
        let a = anim(5.0, 1.0);
        assert_eq!(a.current_pos(), Vec3::splat(4.0));
        assert_eq!(a.current_yaw(), 8.0);
    }
}
