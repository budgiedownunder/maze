//! Game configuration — Play 3D presets and seeds.
//!
//! The values in this module are the single source of truth for what each
//! difficulty means: maze dimensions, time limit, RNG seed (fixed per
//! difficulty for leaderboard fairness from day 1), and a minimum solution-path
//! length (mapped to the maze crate's existing `min_spine_length` option so
//! configured mazes are guaranteed non-trivial). All values are reported to the
//! frontends via `GET /api/v1/game/play3d-config?difficulty=…` so the React /
//! MAUI clients never duplicate them.

use serde::{Deserialize, Deserializer, Serialize};

/// Per-difficulty Play 3D preset.
///
/// `seed` is fixed (not minted per request): every Play 3D Easy run plays the
/// same Easy maze. This makes the future leaderboard fair from day 1 without
/// extra same-seed bucketing logic. `?seed=<n>` on the `/game/` URL still
/// overrides this for replay / share variety.
///
/// Every field is `#[serde(default)]` so a partial / incomplete sub-section in
/// `config.toml` degrades gracefully to that field's default rather than
/// failing the *entire* `AppConfig` deserialise (which silently falls back to
/// `AppConfig::default()` — see `AppConfig::load`). A single commented-out line
/// must never take down the whole server config.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Play3dDifficultyConfig {
    /// Number of maze rows. Defaults to 8 when omitted.
    #[serde(default = "default_play3d_rows")]
    pub rows: u32,
    /// Number of maze columns. Defaults to 8 when omitted.
    #[serde(default = "default_play3d_cols")]
    pub cols: u32,
    /// Time limit, in seconds, before the player loses. Defaults to 120 when
    /// omitted.
    #[serde(default = "default_play3d_timer_seconds")]
    pub timer_seconds: u32,
    /// Fixed RNG seed handed to the maze generator. Same `seed` + same `rows`,
    /// `cols`, `min_solution_length` produce the same maze every time.
    /// Defaults to 0 when omitted.
    #[serde(default = "default_play3d_seed")]
    pub seed: u64,
    /// Minimum number of cells along the start-to-finish path. Maps directly
    /// to the maze crate's `min_spine_length` generator option (with the
    /// crate's default `max_retries = 100`) so configured mazes are never
    /// degenerate. The generator returns an error if no draw meets this.
    /// Defaults to 0 (no minimum) when omitted.
    #[serde(default = "default_play3d_min_solution_length")]
    pub min_solution_length: u32,
    /// On-screen pixel size of each minimap cell. Scales the minimap's
    /// physical footprint without changing how much of the maze is visible.
    /// Defaults to 10 (the value the game shipped with) when omitted.
    #[serde(default = "default_minimap_cell_px")]
    pub minimap_cell_px: u32,
    /// Number of minimap cells visible in each direction from the player —
    /// i.e. the minimap shows a `(2 × radius + 1)` square window of the maze.
    /// Larger values reveal more of the maze (useful for big Hard mazes).
    /// Defaults to 5 (the value the game shipped with) when omitted.
    #[serde(default = "default_minimap_radius")]
    pub minimap_radius: u32,
    /// Optional per-difficulty title override for the in-game splash. When
    /// `None`, the parent `[game.play3d].title` is used.
    #[serde(default)]
    pub title: Option<String>,
    /// Free-text label shown in the in-game status bar. One
    /// per difficulty so e.g. Easy / Tricky / Hard. Defaults to "Play".
    #[serde(default = "default_play3d_mode")]
    pub mode: String,
    /// Landmark / spatial-orientation toggles. Each landmark technique
    /// has its own flag here so an operator can enable or disable any
    /// individual technique per difficulty without code changes.
    #[serde(default)]
    pub landmarks: LandmarksConfig,
    /// Atmospheric sky mode. Drives the dome texture (gradient +
    /// clouds + stars) and the paired ambient + directional light
    /// preset in the Bevy game. Default `night`.
    #[serde(default = "default_sky_type")]
    pub sky_type: SkyTypeConfig,
    /// Wall material kind to use when `landmarks.wall_material_variation`
    /// is OFF for this difficulty (the per-cell tinted path). When that
    /// landmark is on, the per-quadrant material variation supersedes
    /// this setting — same bypass model as `landmarks.wall_tint`. Default
    /// `brick` so the no-config and pre-Step-14 behaviour is preserved.
    #[serde(default = "default_wall_type")]
    pub wall_type: WallTypeConfig,
    /// Door open-animation style for this difficulty. Default `swing`.
    #[serde(default = "default_door_style")]
    pub door_style: DoorStyleConfig,
    /// Key-holder style for `'K'` cells this difficulty. Default `pedestal`.
    #[serde(default = "default_key_holder")]
    pub key_holder: KeyHolderStyleConfig,
}

/// Atmospheric sky modes. Wire form (TOML / JSON) is lowercase
/// (`"night" | "sunrise" | "day" | "sunset"`). Unknown values
/// deserialise as `Night` rather than failing the entire `AppConfig`
/// load — same forgiving policy as the rest of this module.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkyTypeConfig {
    #[default]
    Night,
    Sunrise,
    Day,
    Sunset,
}

impl SkyTypeConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Night => "night",
            Self::Sunrise => "sunrise",
            Self::Day => "day",
            Self::Sunset => "sunset",
        }
    }
}

impl<'de> Deserialize<'de> for SkyTypeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "sunrise" => Self::Sunrise,
            "day" => Self::Day,
            "sunset" => Self::Sunset,
            _ => Self::Night,
        })
    }
}

/// Wall texture kind for the per-cell tinted path. Wire form (TOML /
/// JSON) is `snake_case` (`"brick" | "dressed_stone" | "wood" |
/// "cobblestone"`). Unknown values deserialise as `Brick` rather than
/// failing the entire `AppConfig` load — same forgiving policy as
/// [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WallTypeConfig {
    #[default]
    Brick,
    DressedStone,
    Wood,
    Cobblestone,
}

impl WallTypeConfig {
    /// `snake_case` wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Brick => "brick",
            Self::DressedStone => "dressed_stone",
            Self::Wood => "wood",
            Self::Cobblestone => "cobblestone",
        }
    }
}

impl<'de> Deserialize<'de> for WallTypeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "dressed_stone" => Self::DressedStone,
            "wood" => Self::Wood,
            "cobblestone" => Self::Cobblestone,
            _ => Self::Brick,
        })
    }
}

/// Door open-animation style for the 3D game. Wire form (TOML / JSON) is
/// `snake_case` (`"swing" | "slide" | "portcullis" | "dissolve"`). Unknown
/// values deserialise as `Swing` rather than failing the entire `AppConfig`
/// load — same forgiving policy as [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DoorStyleConfig {
    #[default]
    Swing,
    Slide,
    Portcullis,
    Dissolve,
}

impl DoorStyleConfig {
    /// `snake_case` wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Swing => "swing",
            Self::Slide => "slide",
            Self::Portcullis => "portcullis",
            Self::Dissolve => "dissolve",
        }
    }
}

impl<'de> Deserialize<'de> for DoorStyleConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "slide" => Self::Slide,
            "portcullis" => Self::Portcullis,
            "dissolve" => Self::Dissolve,
            _ => Self::Swing,
        })
    }
}

/// Key-holder style for `'K'` cells in the 3D game. Wire form (TOML / JSON) is
/// `snake_case` (`"pedestal" | "chest" | "floating_key"`). Unknown values
/// deserialise as `Pedestal` — same forgiving policy as [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum KeyHolderStyleConfig {
    #[default]
    Pedestal,
    Chest,
    FloatingKey,
}

impl KeyHolderStyleConfig {
    /// `snake_case` wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Pedestal => "pedestal",
            Self::Chest => "chest",
            Self::FloatingKey => "floating_key",
        }
    }
}

impl<'de> Deserialize<'de> for KeyHolderStyleConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "chest" => Self::Chest,
            "floating_key" => Self::FloatingKey,
            _ => Self::Pedestal,
        })
    }
}

/// Per-difficulty landmark / spatial-orientation toggles. New techniques
/// add their own field here (default `true`) as they land — the schema
/// is intentionally a flat record of booleans (or simple values) so
/// `config.toml` reads like `[game.play3d.<difficulty>.landmarks]` with
/// one knob per feature.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LandmarksConfig {
    /// Per-cell wall tint variation. Each passable cell hashes
    /// `(row, col, seed)` to pick one of several emissive variants for
    /// its wall panels — different corridor sections then read as
    /// subtly different shades. Default `true`.
    #[serde(default = "default_landmarks_wall_tint")]
    pub wall_tint: bool,
    /// Dead-end landmark objects — place a single distinctive object
    /// (brazier / urn / pillar / chest) in every dead-end cell, picked
    /// by hashing `(row, col, seed)` so the same maze always shows the
    /// same object in the same dead-end. Default `true`.
    #[serde(default = "default_landmarks_dead_end_objects")]
    pub dead_end_objects: bool,
    /// Sparse wall decorations / posters — a small fraction of wall panels
    /// (currently 1 in 10) get a decorative emissive decoration (vent grate,
    /// faded poster, rune, glowing glass) projected on their inside face.
    /// Decoration placement and kind are seeded so the same maze always
    /// looks the same. Default `true`.
    #[serde(default = "default_landmarks_wall_decorations")]
    pub wall_decorations: bool,
    /// Floor accents at junction cells — every 3- or 4-way junction
    /// (excluding start / finish) gets a single flat accent (moss /
    /// cracked tile / mosaic / sigil) on its floor. Kind is picked by
    /// hashing `(row, col, seed)` so the same maze always shows the same
    /// accent at the same junction. Default `true`.
    #[serde(default = "default_landmarks_floor_accents")]
    pub floor_accents: bool,
    /// Per-quadrant wall material variation — splits the maze into a
    /// 2×2 NW/NE/SW/SE grid and gives each quadrant its own wall
    /// material kind (brick / dressed stone / wood / cobblestone). The
    /// quadrant-to-kind mapping is seed-permuted so different seeds
    /// rotate the assignment. Supersedes `wall_tint` when on — each
    /// quadrant renders with one fixed material kind. Default `true`.
    #[serde(default = "default_landmarks_wall_material_variation")]
    pub wall_material_variation: bool,
}

impl Default for LandmarksConfig {
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

impl Default for Play3dDifficultyConfig {
    fn default() -> Self {
        Self {
            rows: default_play3d_rows(),
            cols: default_play3d_cols(),
            timer_seconds: default_play3d_timer_seconds(),
            seed: default_play3d_seed(),
            min_solution_length: default_play3d_min_solution_length(),
            minimap_cell_px: default_minimap_cell_px(),
            minimap_radius: default_minimap_radius(),
            title: None,
            mode: default_play3d_mode(),
            landmarks: LandmarksConfig::default(),
            sky_type: default_sky_type(),
            wall_type: default_wall_type(),
            door_style: default_door_style(),
            key_holder: default_key_holder(),
        }
    }
}

/// Top-level `[game.play3d]` configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Play3dConfig {
    /// Default title for the in-game splash (overridable per difficulty).
    #[serde(default = "default_play3d_title")]
    pub title: String,
    /// Easy preset. A wholly-omitted `[game.play3d.easy]` table degrades to
    /// `Play3dDifficultyConfig::default()` rather than failing config load.
    #[serde(default)]
    pub easy: Play3dDifficultyConfig,
    /// Tricky preset. Same omitted-section fallback as `easy`.
    #[serde(default)]
    pub tricky: Play3dDifficultyConfig,
    /// Hard preset. Same omitted-section fallback as `easy`.
    #[serde(default)]
    pub hard: Play3dDifficultyConfig,
}

impl Play3dConfig {
    /// Returns the preset for the given difficulty label, or `None` if the
    /// label is not recognised. Comparison is case-insensitive so a client
    /// that capitalises the value (e.g. `?difficulty=Easy`) still resolves.
    pub fn lookup(&self, difficulty: &str) -> Option<&Play3dDifficultyConfig> {
        match difficulty.to_ascii_lowercase().as_str() {
            "easy" => Some(&self.easy),
            "tricky" => Some(&self.tricky),
            "hard" => Some(&self.hard),
            _ => None,
        }
    }

    /// Resolves the title shown on the in-game splash for the given difficulty:
    /// the per-difficulty override if set, otherwise the parent `title`.
    pub fn resolved_title(&self, difficulty: &str) -> String {
        self.lookup(difficulty)
            .and_then(|d| d.title.clone())
            .unwrap_or_else(|| self.title.clone())
    }
}

impl Default for Play3dConfig {
    fn default() -> Self {
        // Defaults align with what the plan documents and what the standalone
        // Bevy game uses when no config is fetched — keeping the no-config /
        // misconfigured path behaving like Step 2 instead of crashing.
        Self {
            title: default_play3d_title(),
            easy: Play3dDifficultyConfig {
                rows: 8,
                cols: 8,
                timer_seconds: 120,
                seed: 8_080_808,
                min_solution_length: 30,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Easy".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
            },
            tricky: Play3dDifficultyConfig {
                rows: 15,
                cols: 15,
                timer_seconds: 240,
                seed: 15_151_515,
                min_solution_length: 90,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Tricky".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
            },
            hard: Play3dDifficultyConfig {
                rows: 25,
                cols: 25,
                timer_seconds: 420,
                seed: 25_252_525,
                min_solution_length: 220,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Hard".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
            },
        }
    }
}

/// Top-level `[game]` configuration. Wraps `play3d` for forward room (other
/// game-related config can land here later).
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GameConfig {
    #[serde(default)]
    pub play3d: Play3dConfig,
}

fn default_play3d_title() -> String {
    "Maze 3D".to_string()
}

// Per-field fallbacks for an incomplete `[game.play3d.<difficulty>]` table.
// Generic, safe, playable values — not tuned per difficulty — because their
// only job is to keep the rest of the config loading when one field is
// missing. A fully-specified config never hits these.
fn default_play3d_rows() -> u32 {
    8
}
fn default_play3d_cols() -> u32 {
    8
}
fn default_play3d_timer_seconds() -> u32 {
    120
}
fn default_play3d_seed() -> u64 {
    0
}
fn default_play3d_min_solution_length() -> u32 {
    0
}

/// The minimap cell pixel size the game shipped with.
fn default_minimap_cell_px() -> u32 {
    10
}

/// The minimap visible-radius the game shipped with.
fn default_minimap_radius() -> u32 {
    5
}

/// Default mode label shown in the in-game status bar when not configured.
fn default_play3d_mode() -> String {
    "Play".to_string()
}

/// Per-cell wall tint variation defaults on. Operators can disable it
/// per difficulty via `[game.play3d.<difficulty>.landmarks] wall_tint = false`.
fn default_landmarks_wall_tint() -> bool {
    true
}

/// Dead-end landmark objects default on. Operators can disable it per
/// difficulty via `[game.play3d.<difficulty>.landmarks] dead_end_objects = false`.
fn default_landmarks_dead_end_objects() -> bool {
    true
}

/// Sparse wall decorations default on. Operators can disable them per
/// difficulty via `[game.play3d.<difficulty>.landmarks] wall_decorations = false`.
fn default_landmarks_wall_decorations() -> bool {
    true
}

/// Floor accents at junction cells default on. Operators can disable them
/// per difficulty via `[game.play3d.<difficulty>.landmarks] floor_accents = false`.
fn default_landmarks_floor_accents() -> bool {
    true
}

/// Per-quadrant wall material variation defaults on. Operators can disable
/// it per difficulty via
/// `[game.play3d.<difficulty>.landmarks] wall_material_variation = false`.
fn default_landmarks_wall_material_variation() -> bool {
    true
}

/// Atmospheric sky mode defaults to night for parity with the pre-Step-10
/// look (the only sky mode that previously existed). Operators override
/// per difficulty via `[game.play3d.<difficulty>] sky_type = "day"`.
fn default_sky_type() -> SkyTypeConfig {
    SkyTypeConfig::Night
}

/// Wall material kind for the per-cell tinted path defaults to brick —
/// parity with the pre-Step-14 hard-coded path. Operators override per
/// difficulty via `[game.play3d.<difficulty>] wall_type = "wood"`.
fn default_wall_type() -> WallTypeConfig {
    WallTypeConfig::Brick
}

/// Door style defaults to swing — the topology-driven look the 3D game shipped
/// with. Operators override per difficulty via
/// `[game.play3d.<difficulty>] door_style = "portcullis"`.
fn default_door_style() -> DoorStyleConfig {
    DoorStyleConfig::Swing
}

/// Key-holder style defaults to pedestal. Operators override per difficulty via
/// `[game.play3d.<difficulty>] key_holder = "chest"`.
fn default_key_holder() -> KeyHolderStyleConfig {
    KeyHolderStyleConfig::Pedestal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_matches_known_difficulties_case_insensitively() {
        let cfg = Play3dConfig::default();
        assert_eq!(cfg.lookup("easy").map(|d| d.rows), Some(8));
        assert_eq!(cfg.lookup("Easy").map(|d| d.rows), Some(8));
        assert_eq!(cfg.lookup("TRICKY").map(|d| d.cols), Some(15));
        assert_eq!(cfg.lookup("hard").map(|d| d.timer_seconds), Some(420));
    }

    #[test]
    fn lookup_returns_none_for_unknown_difficulty() {
        let cfg = Play3dConfig::default();
        assert!(cfg.lookup("banana").is_none());
        assert!(cfg.lookup("").is_none());
    }

    #[test]
    fn resolved_title_falls_back_to_parent_default() {
        let cfg = Play3dConfig::default();
        assert_eq!(cfg.resolved_title("easy"), "Maze 3D");
        assert_eq!(cfg.resolved_title("tricky"), "Maze 3D");
        assert_eq!(cfg.resolved_title("hard"), "Maze 3D");
    }

    #[test]
    fn resolved_title_uses_per_difficulty_override_when_present() {
        let mut cfg = Play3dConfig::default();
        cfg.easy.title = Some("MAZE 3D — EASY".to_string());
        assert_eq!(cfg.resolved_title("easy"), "MAZE 3D — EASY");
        // Other difficulties still fall back to the parent default.
        assert_eq!(cfg.resolved_title("tricky"), "Maze 3D");
    }

    #[test]
    fn resolved_title_falls_back_to_parent_default_for_unknown_difficulty() {
        let cfg = Play3dConfig::default();
        assert_eq!(cfg.resolved_title("banana"), "Maze 3D");
    }

    #[test]
    fn play3d_config_deserialises_from_toml() {
        let toml = r#"
            [play3d]
            title = "Maze 3D Daily"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            minimap_cell_px = 8
            minimap_radius = 7
            title = "MAZE 3D — TRICKY"

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.title, "Maze 3D Daily");
        assert_eq!(cfg.play3d.easy.rows, 6);
        assert_eq!(cfg.play3d.easy.seed, 42);
        assert_eq!(cfg.play3d.tricky.title.as_deref(), Some("MAZE 3D — TRICKY"));
        assert_eq!(cfg.play3d.hard.min_solution_length, 160);
        assert_eq!(cfg.play3d.resolved_title("easy"), "Maze 3D Daily");
        assert_eq!(cfg.play3d.resolved_title("tricky"), "MAZE 3D — TRICKY");
        // Minimap fields: explicit on tricky, defaulted on easy/hard (omitted in toml).
        assert_eq!(cfg.play3d.tricky.minimap_cell_px, 8);
        assert_eq!(cfg.play3d.tricky.minimap_radius, 7);
        assert_eq!(cfg.play3d.easy.minimap_cell_px, 10);
        assert_eq!(cfg.play3d.easy.minimap_radius, 5);
        assert_eq!(cfg.play3d.hard.minimap_cell_px, 10);
        assert_eq!(cfg.play3d.hard.minimap_radius, 5);
    }

    #[test]
    fn landmarks_round_trips_from_toml_and_defaults_to_all_true() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            [play3d.easy.landmarks]
            wall_tint = false
            dead_end_objects = false
            wall_decorations = false
            floor_accents = false
            wall_material_variation = false

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            # landmarks deliberately omitted — must default to all-true

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            [play3d.hard.landmarks]
            # only one toggle present — the others must default to true
            dead_end_objects = false
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        // Easy override disables every toggle.
        assert!(!cfg.play3d.easy.landmarks.wall_tint);
        assert!(!cfg.play3d.easy.landmarks.dead_end_objects);
        assert!(!cfg.play3d.easy.landmarks.wall_decorations);
        assert!(!cfg.play3d.easy.landmarks.floor_accents);
        assert!(!cfg.play3d.easy.landmarks.wall_material_variation);
        // Tricky omits the whole landmarks table → all default true.
        assert!(cfg.play3d.tricky.landmarks.wall_tint);
        assert!(cfg.play3d.tricky.landmarks.dead_end_objects);
        assert!(cfg.play3d.tricky.landmarks.wall_decorations);
        assert!(cfg.play3d.tricky.landmarks.floor_accents);
        assert!(cfg.play3d.tricky.landmarks.wall_material_variation);
        // Hard sets one toggle; the others fall back to the default.
        assert!(cfg.play3d.hard.landmarks.wall_tint);
        assert!(!cfg.play3d.hard.landmarks.dead_end_objects);
        assert!(cfg.play3d.hard.landmarks.wall_decorations);
        assert!(cfg.play3d.hard.landmarks.floor_accents);
        assert!(cfg.play3d.hard.landmarks.wall_material_variation);
        // Built-in Play3dConfig::default() enables every toggle.
        let default = Play3dConfig::default();
        for d in [&default.easy, &default.tricky, &default.hard] {
            assert!(d.landmarks.wall_tint);
            assert!(d.landmarks.dead_end_objects);
            assert!(d.landmarks.wall_decorations);
            assert!(d.landmarks.floor_accents);
            assert!(d.landmarks.wall_material_variation);
        }
    }

    #[test]
    fn sky_type_round_trips_from_toml_and_defaults_to_night() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            sky_type = "day"

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            sky_type = "SUNSET"
            # case-insensitive

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            # sky_type deliberately omitted — defaults to night
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.sky_type, SkyTypeConfig::Day);
        assert_eq!(cfg.play3d.tricky.sky_type, SkyTypeConfig::Sunset);
        assert_eq!(cfg.play3d.hard.sky_type, SkyTypeConfig::Night);
        // Built-in defaults are all Night.
        let default = Play3dConfig::default();
        assert_eq!(default.easy.sky_type, SkyTypeConfig::Night);
        assert_eq!(default.tricky.sky_type, SkyTypeConfig::Night);
        assert_eq!(default.hard.sky_type, SkyTypeConfig::Night);
    }

    #[test]
    fn unknown_sky_type_falls_back_to_night() {
        // Forgiving deserialiser — a typo (or a value from a future
        // version) must not kill config load. Falls back to Night so
        // the player still gets *some* sky rather than no game.
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            sky_type = "thunderstorm"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.sky_type, SkyTypeConfig::Night);
    }

    #[test]
    fn sky_type_as_wire_str_matches_serde_form() {
        // The handler layer surfaces this lowercase string in JSON
        // responses; the WASM client parses it back via
        // SkyType::from_wire_str. The round-trip is symmetric.
        assert_eq!(SkyTypeConfig::Night.as_wire_str(), "night");
        assert_eq!(SkyTypeConfig::Sunrise.as_wire_str(), "sunrise");
        assert_eq!(SkyTypeConfig::Day.as_wire_str(), "day");
        assert_eq!(SkyTypeConfig::Sunset.as_wire_str(), "sunset");
    }

    #[test]
    fn wall_type_round_trips_from_toml_and_defaults_to_brick() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            wall_type = "wood"

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            wall_type = "DRESSED_STONE"
            # case-insensitive

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            # wall_type deliberately omitted — defaults to brick
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.wall_type, WallTypeConfig::Wood);
        assert_eq!(cfg.play3d.tricky.wall_type, WallTypeConfig::DressedStone);
        assert_eq!(cfg.play3d.hard.wall_type, WallTypeConfig::Brick);
        // Built-in defaults are all Brick.
        let default = Play3dConfig::default();
        assert_eq!(default.easy.wall_type, WallTypeConfig::Brick);
        assert_eq!(default.tricky.wall_type, WallTypeConfig::Brick);
        assert_eq!(default.hard.wall_type, WallTypeConfig::Brick);
    }

    #[test]
    fn unknown_wall_type_falls_back_to_brick() {
        // Same forgiving policy as sky_type — a typo must not kill the
        // config load. Falls back to Brick (the pre-Step-14 hard-coded
        // texture).
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            wall_type = "marble"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.wall_type, WallTypeConfig::Brick);
    }

    #[test]
    fn wall_type_as_wire_str_matches_serde_form() {
        assert_eq!(WallTypeConfig::Brick.as_wire_str(), "brick");
        assert_eq!(WallTypeConfig::DressedStone.as_wire_str(), "dressed_stone");
        assert_eq!(WallTypeConfig::Wood.as_wire_str(), "wood");
        assert_eq!(WallTypeConfig::Cobblestone.as_wire_str(), "cobblestone");
    }

    #[test]
    fn door_style_round_trips_from_toml_and_defaults_to_swing() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            door_style = "portcullis"

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            door_style = "DISSOLVE"
            # case-insensitive

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            # door_style deliberately omitted — defaults to swing
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.door_style, DoorStyleConfig::Portcullis);
        assert_eq!(cfg.play3d.tricky.door_style, DoorStyleConfig::Dissolve);
        assert_eq!(cfg.play3d.hard.door_style, DoorStyleConfig::Swing);
        assert_eq!(Play3dConfig::default().easy.door_style, DoorStyleConfig::Swing);
    }

    #[test]
    fn unknown_door_style_falls_back_to_swing() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            door_style = "teleport"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.door_style, DoorStyleConfig::Swing);
    }

    #[test]
    fn key_holder_round_trips_from_toml_and_defaults_to_pedestal() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            key_holder = "chest"

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            key_holder = "FLOATING_KEY"
            # case-insensitive

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            # key_holder deliberately omitted — defaults to pedestal
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.key_holder, KeyHolderStyleConfig::Chest);
        assert_eq!(cfg.play3d.tricky.key_holder, KeyHolderStyleConfig::FloatingKey);
        assert_eq!(cfg.play3d.hard.key_holder, KeyHolderStyleConfig::Pedestal);
        assert_eq!(
            Play3dConfig::default().easy.key_holder,
            KeyHolderStyleConfig::Pedestal
        );
    }

    #[test]
    fn unknown_key_holder_falls_back_to_pedestal() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            key_holder = "vault"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.key_holder, KeyHolderStyleConfig::Pedestal);
    }

    #[test]
    fn door_style_and_key_holder_as_wire_str_match_serde_form() {
        assert_eq!(DoorStyleConfig::Swing.as_wire_str(), "swing");
        assert_eq!(DoorStyleConfig::Slide.as_wire_str(), "slide");
        assert_eq!(DoorStyleConfig::Portcullis.as_wire_str(), "portcullis");
        assert_eq!(DoorStyleConfig::Dissolve.as_wire_str(), "dissolve");
        assert_eq!(KeyHolderStyleConfig::Pedestal.as_wire_str(), "pedestal");
        assert_eq!(KeyHolderStyleConfig::Chest.as_wire_str(), "chest");
        assert_eq!(KeyHolderStyleConfig::FloatingKey.as_wire_str(), "floating_key");
    }

    #[test]
    fn mode_round_trips_from_toml_and_falls_back_to_default_when_omitted() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            mode = "Marathon"

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            # mode deliberately omitted — must fall back to default
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.mode, "Marathon");
        assert_eq!(cfg.play3d.tricky.mode, "Play");
        // Default Play3dConfig still uses the shipped per-difficulty labels.
        let default = Play3dConfig::default();
        assert_eq!(default.easy.mode, "Easy");
        assert_eq!(default.tricky.mode, "Tricky");
        assert_eq!(default.hard.mode, "Hard");
    }

    #[test]
    fn minimap_defaults_match_shipped_values() {
        let cfg = Play3dConfig::default();
        for d in ["easy", "tricky", "hard"] {
            let preset = cfg.lookup(d).unwrap();
            assert_eq!(preset.minimap_cell_px, 10, "{d} cell px");
            assert_eq!(preset.minimap_radius, 5, "{d} radius");
        }
    }

    #[test]
    fn partial_difficulty_section_degrades_gracefully() {
        // Regression: a single omitted field (or a wholly-omitted difficulty
        // sub-section) must NOT fail the deserialise. Previously every
        // `Play3dDifficultyConfig` field was required, so commenting out one
        // line in `config.toml` failed the entire `AppConfig` deserialise,
        // which `AppConfig::load` silently swallows by falling back to
        // `AppConfig::default()` — surfacing later as a confusing
        // "static_dir 'static' does not exist".
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 10
            cols = 10
            timer_seconds = 120
            seed = 999
            # min_solution_length deliberately omitted

            # [play3d.tricky] and [play3d.hard] deliberately omitted entirely
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("partial config must still deserialise");
        // Explicit fields survive.
        assert_eq!(cfg.play3d.easy.rows, 10);
        assert_eq!(cfg.play3d.easy.seed, 999);
        // The omitted field falls back to its default.
        assert_eq!(cfg.play3d.easy.min_solution_length, 0);
        assert_eq!(cfg.play3d.easy.minimap_cell_px, 10);
        // Omitted sub-sections fall back to Play3dDifficultyConfig::default().
        assert_eq!(cfg.play3d.tricky.rows, 8);
        assert_eq!(cfg.play3d.hard.timer_seconds, 120);
    }
}
