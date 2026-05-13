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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Play3dDifficultyConfig {
    /// Number of maze rows.
    pub rows: u32,
    /// Number of maze columns.
    pub cols: u32,
    /// Time limit, in seconds, before the player loses.
    pub timer_seconds: u32,
    /// Fixed RNG seed handed to the maze generator. Same `seed` + same `rows`,
    /// `cols`, `min_solution_length` produce the same maze every time.
    pub seed: u64,
    /// Minimum number of cells along the start-to-finish path. Maps directly
    /// to the maze crate's `min_spine_length` generator option (with the
    /// crate's default `max_retries = 100`) so configured mazes are never
    /// degenerate. The generator returns an error if no draw meets this.
    pub min_solution_length: u32,
    /// Optional per-difficulty title override for the in-game splash. When
    /// `None`, the parent `[game.play3d].title` is used.
    #[serde(default)]
    pub title: Option<String>,
}

/// Top-level `[game.play3d]` configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Play3dConfig {
    /// Default title for the in-game splash (overridable per difficulty).
    #[serde(default = "default_play3d_title")]
    pub title: String,
    /// Easy preset.
    pub easy: Play3dDifficultyConfig,
    /// Tricky preset.
    pub tricky: Play3dDifficultyConfig,
    /// Hard preset.
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
                title: None,
            },
            tricky: Play3dDifficultyConfig {
                rows: 15,
                cols: 15,
                timer_seconds: 240,
                seed: 15_151_515,
                min_solution_length: 90,
                title: None,
            },
            hard: Play3dDifficultyConfig {
                rows: 25,
                cols: 25,
                timer_seconds: 420,
                seed: 25_252_525,
                min_solution_length: 220,
                title: None,
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
    }
}
