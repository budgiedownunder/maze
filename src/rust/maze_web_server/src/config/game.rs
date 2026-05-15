//! Game configuration — Play 3D presets and seeds.
//!
//! The values in this module are the single source of truth for what each
//! difficulty means: maze dimensions, time limit, RNG seed (fixed per
//! difficulty for leaderboard fairness from day 1), and a minimum solution-path
//! length (mapped to the maze crate's existing `min_spine_length` option so
//! configured mazes are guaranteed non-trivial). All values are reported to the
//! frontends via `GET /api/v1/game/play3d-config?difficulty=…` so the React /
//! MAUI clients never duplicate them.

use serde::{Deserialize, Serialize};

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
