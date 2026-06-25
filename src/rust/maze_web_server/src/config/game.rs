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
    /// Whether the maze perimeter is walled at the grid edge under an **open**
    /// sky. Enclosed skies (`dungeon` / `chamber`) always wall the perimeter
    /// regardless; for open skies, `true` walls the edge (the traditional
    /// enclosed-maze look) and `false` shows the sky past it. Default `true`.
    #[serde(default = "default_perimeter_walls")]
    pub perimeter_walls: bool,
    /// Door open-animation style for this difficulty. Default `swing`.
    #[serde(default = "default_door_style")]
    pub door_style: DoorStyleConfig,
    /// Key-holder style for `'K'` cells this difficulty. Default `pedestal`.
    #[serde(default = "default_key_holder")]
    pub key_holder: KeyHolderStyleConfig,
    /// Number of doors (each paired with one key) the maze generator
    /// auto-places into this difficulty's maze. Doors gate the solution path
    /// and the maze stays solvable (key-aware verified). Clamped to what the
    /// maze can hold. Default 0 = a lock-free maze.
    #[serde(default = "default_play3d_door_count")]
    pub door_count: u32,
    /// Number of decoy doors planted on off-spine branches after the maze
    /// passes the solvability check. A decoy is visually indistinguishable
    /// from a real path door — opening one burns a key the player might have
    /// needed for a real door and (when the spare budget is exhausted)
    /// strands them. Clamped to `MAX_AUTO_DOORS` and to feasibility. Default
    /// 0 = no decoys.
    #[serde(default = "default_play3d_spare_doors")]
    pub spare_doors: u32,
    /// Number of spare keys planted on off-spine branches, giving the player
    /// a budget to burn on decoys before they risk stranding. Default 0.
    #[serde(default = "default_play3d_spare_keys")]
    pub spare_keys: u32,
    /// Number of enemies (`'E'` cells) the generator auto-places on this
    /// difficulty's maze. Clamped to `maze::MAX_ENEMY_COUNT` (= 8) and to
    /// the available eligible cells. Default 0 = no enemies.
    #[serde(default = "default_play3d_enemy_count")]
    pub enemy_count: u32,
    /// Number of health pickups (`'H'` cells) the generator auto-places
    /// on this difficulty's maze. Clamped to `maze::MAX_HEALTH_COUNT`
    /// (= 8) and to the available eligible cells. Default 0 = no
    /// health pickups.
    #[serde(default = "default_play3d_health_count")]
    pub health_count: u32,
    /// Number of treasure cells (`'T'`) the generator auto-places on this
    /// difficulty's maze, dead-end-first and type-weighted. Clamped to
    /// `maze::MAX_TREASURE_COUNT` (= 12) and to the available eligible cells.
    /// Default 0 = no treasure.
    #[serde(default = "default_play3d_treasure_count")]
    pub treasure_count: u32,
    /// Enemy rig kind to spawn at every `'E'` cell. Same rig for every
    /// enemy on a given difficulty (per-enemy variation is deferred to a
    /// later plan). Default `goblin`.
    #[serde(default = "default_enemy_type")]
    pub enemy_type: EnemyTypeConfig,
    /// Health-pickup rig kind to spawn at every `'H'` cell. Default
    /// `heart`.
    #[serde(default = "default_health_style")]
    pub health_style: HealthStyleConfig,
    /// How often each enemy advances one cell, in milliseconds of
    /// real-game time. Lower = harder. Default 1500.
    #[serde(default = "default_play3d_enemy_move_period_ms")]
    pub enemy_move_period_ms: u32,
    /// Player's HP cap for this difficulty. Starting HP is set to this
    /// value (the Bevy `StartConfig` re-uses `max_hp` for `starting_hp`).
    /// Default 3.
    #[serde(default = "default_play3d_max_hp")]
    pub max_hp: u32,
    /// Multi-level run settings (`[game.play3d.<difficulty>.levels]`). A
    /// wholly-omitted table degrades to `LevelsConfig::default()` — a
    /// single-level game, today's behaviour.
    #[serde(default)]
    pub levels: LevelsConfig,
}

/// Atmospheric sky modes. Wire form (TOML / JSON) is lowercase
/// (`"night" | "sunrise" | "day" | "sunset" | "dungeon" | "chamber"`).
/// Unknown values deserialise as `Night` rather than failing the entire
/// `AppConfig` load — same forgiving policy as the rest of this module.
/// `Dungeon` swaps the open sky for a dark-rock ceiling over every cell;
/// `Chamber` caps it instead with a ceiling in the cell's wall material.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkyTypeConfig {
    #[default]
    Night,
    Sunrise,
    Day,
    Sunset,
    Dungeon,
    Chamber,
}

impl SkyTypeConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Night => "night",
            Self::Sunrise => "sunrise",
            Self::Day => "day",
            Self::Sunset => "sunset",
            Self::Dungeon => "dungeon",
            Self::Chamber => "chamber",
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
            "dungeon" => Self::Dungeon,
            "chamber" => Self::Chamber,
            _ => Self::Night,
        })
    }
}

/// Per-maze wall type. Wire form (TOML / JSON) is `snake_case`: the four solid
/// textures (`"brick" | "dressed_stone" | "wood" | "cobblestone"`) plus the three
/// non-occluding types (`"water" | "lava" | "iron_fence"`) — a whole maze can be
/// any of them (every `'W'` cell becomes that type unless a per-cell override
/// says otherwise). Unknown values deserialise as `Brick` rather than failing the
/// entire `AppConfig` load — same forgiving policy as [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WallTypeConfig {
    #[default]
    Brick,
    DressedStone,
    Wood,
    Cobblestone,
    Water,
    Lava,
    IronFence,
}

impl WallTypeConfig {
    /// `snake_case` wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Brick => "brick",
            Self::DressedStone => "dressed_stone",
            Self::Wood => "wood",
            Self::Cobblestone => "cobblestone",
            Self::Water => "water",
            Self::Lava => "lava",
            Self::IronFence => "iron_fence",
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
            "water" => Self::Water,
            "lava" => Self::Lava,
            "iron_fence" => Self::IronFence,
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

/// Interim-finish transition rig for a multi-level run. Wire form (TOML / JSON)
/// is lowercase (`"ladder" | "portal" | "random"`). A fixed value draws that one
/// rig at every interim finish; `random` picks a concrete rig per interim finish
/// cell, seeded off the run. Unknown values deserialise as `Ladder` — same
/// forgiving policy as [`SkyTypeConfig`]. Inert when `levels.count == 1`.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FinishTypeConfig {
    #[default]
    Ladder,
    Portal,
    Random,
}

impl FinishTypeConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Ladder => "ladder",
            Self::Portal => "portal",
            Self::Random => "random",
        }
    }
}

impl<'de> Deserialize<'de> for FinishTypeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "portal" => Self::Portal,
            "random" => Self::Random,
            _ => Self::Ladder,
        })
    }
}

/// How a multi-level run's difficulty changes as the player ascends. Wire form
/// (TOML / JSON) is lowercase (`"same" | "easier" | "harder"`): `easier` =
/// hardest at the bottom, easing as you climb; `harder` = the reverse; `same` =
/// every level equally hard. Enemy count is the lever (footprint stays uniform).
/// Unknown values deserialise as `Easier` — same forgiving policy as
/// [`SkyTypeConfig`]. Inert when `levels.count == 1`.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DifficultyChangeConfig {
    Same,
    #[default]
    Easier,
    Harder,
}

impl DifficultyChangeConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::Easier => "easier",
            Self::Harder => "harder",
        }
    }
}

impl<'de> Deserialize<'de> for DifficultyChangeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "same" => Self::Same,
            "harder" => Self::Harder,
            _ => Self::Easier,
        })
    }
}

/// How a reduced upper level is positioned over the level below in a multi-level
/// run. Wire form (TOML / JSON) is lowercase (`"edge" | "centre" | "random"`):
/// `edge` corner-aligns every layer (zero X/Z offset); `centre` centres each
/// smaller layer over the ground layer; `random` lets the client pick per level.
/// Only meaningful under an open sky (enclosed stacks stay uniform). Unknown
/// values deserialise as `Edge` — same forgiving policy as [`SkyTypeConfig`].
/// Inert when `levels.count == 1`.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayeredAlignmentConfig {
    #[default]
    Edge,
    Centre,
    Random,
}

impl LayeredAlignmentConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Edge => "edge",
            Self::Centre => "centre",
            Self::Random => "random",
        }
    }
}

impl<'de> Deserialize<'de> for LayeredAlignmentConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "centre" | "center" => Self::Centre,
            "random" => Self::Random,
            _ => Self::Edge,
        })
    }
}

/// Enemy rig kind for `'E'` cells in the 3D game. Wire form (TOML / JSON)
/// is lowercase (`"goblin" | "ghost"`). Unknown values deserialise as
/// `Goblin` — same forgiving policy as [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnemyTypeConfig {
    #[default]
    Goblin,
    Ghost,
}

impl EnemyTypeConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Goblin => "goblin",
            Self::Ghost => "ghost",
        }
    }
}

impl<'de> Deserialize<'de> for EnemyTypeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "ghost" => Self::Ghost,
            _ => Self::Goblin,
        })
    }
}

/// Health-pickup rig kind for `'H'` cells in the 3D game. Wire form
/// (TOML / JSON) is lowercase (`"heart" | "potion"`). Unknown values
/// deserialise as `Heart` — same forgiving policy as [`SkyTypeConfig`].
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum HealthStyleConfig {
    #[default]
    Heart,
    Potion,
}

impl HealthStyleConfig {
    /// Lowercase wire string used in JSON responses + TOML values.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Heart => "heart",
            Self::Potion => "potion",
        }
    }
}

impl<'de> Deserialize<'de> for HealthStyleConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "potion" => Self::Potion,
            _ => Self::Heart,
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

/// Upper bound on a multi-level run's level count. Mirrors `maze_game_bevy`'s
/// `MAX_LEVEL_COUNT` (the source of truth for the renderer / generation cap);
/// the server clamps the configured `levels.count` to it before reporting it so
/// a client never has to render more levels than the Bevy game supports.
pub const MAX_LEVEL_COUNT: u32 = 5;

/// Multi-level run settings for a difficulty (`[game.play3d.<difficulty>.levels]`).
///
/// Every field is `#[serde(default)]` so the whole group can be omitted — that
/// degrades to a single-level run (`count == 1`), byte-for-byte today's
/// behaviour. The fields drop the `level_` / `layer_` prefix because the table
/// name already scopes them.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LevelsConfig {
    /// Number of stacked maze levels in a run. `1` = a single-level game (the
    /// default; no transitions). Clamped to [`MAX_LEVEL_COUNT`] when reported.
    #[serde(default = "default_levels_count")]
    pub count: u32,
    /// Interim-finish transition rig. Default `ladder`.
    #[serde(default = "default_finish_type")]
    pub finish_type: FinishTypeConfig,
    /// How difficulty changes as the player ascends. Default `easier` (hardest
    /// at the bottom).
    #[serde(default = "default_difficulty_change")]
    pub difficulty_change: DifficultyChangeConfig,
    /// Whether the player's bag resets at each level. `true` (default) = each
    /// level is self-contained; `false` carries the whole bag forward.
    #[serde(default = "default_reset_bag")]
    pub reset_bag: bool,
    /// How a reduced upper level is positioned over the level below. Default
    /// `edge` (corner-aligned). Only meaningful under an open sky.
    #[serde(default = "default_alignment")]
    pub alignment: LayeredAlignmentConfig,
    /// When `true`, each level's perimeter walls are randomised on / off
    /// independently; when `false` (default) every level uses the difficulty's
    /// `perimeter_walls` setting.
    #[serde(default = "default_perimeter_random")]
    pub perimeter_random: bool,
    /// When `true`, a completed lower level's enemies are despawned once the
    /// player climbs past it (the player only ever ascends, so a completed level is
    /// never revisited); when `false` (default) they idle in place. Only meaningful
    /// when `count > 1`.
    #[serde(default)]
    pub hide_completed_enemies: bool,
    /// Optional scene override for the final (top) level — its own `sky_type` /
    /// `perimeter_walls`, inheriting the base where unset. Only meaningful when
    /// `count > 1`. `[game.play3d.<difficulty>.levels.top]`.
    #[serde(default)]
    pub top: Option<TopLevelConfig>,
}

impl Default for LevelsConfig {
    fn default() -> Self {
        Self {
            count: default_levels_count(),
            finish_type: default_finish_type(),
            difficulty_change: default_difficulty_change(),
            reset_bag: default_reset_bag(),
            alignment: default_alignment(),
            perimeter_random: default_perimeter_random(),
            hide_completed_enemies: false,
            top: None,
        }
    }
}

/// Top-level scene override (`[game.play3d.<difficulty>.levels.top]`). Each field
/// is optional and falls back to the base difficulty's value when unset; the
/// override only applies when `levels.count > 1`.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TopLevelConfig {
    /// Sky type for the top level, overriding the base when set.
    #[serde(default)]
    pub sky_type: Option<SkyTypeConfig>,
    /// Perimeter-walls flag for the top level, overriding the base when set.
    #[serde(default)]
    pub perimeter_walls: Option<bool>,
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
            perimeter_walls: default_perimeter_walls(),
            door_style: default_door_style(),
            key_holder: default_key_holder(),
            door_count: default_play3d_door_count(),
            spare_doors: default_play3d_spare_doors(),
            spare_keys: default_play3d_spare_keys(),
            enemy_count: default_play3d_enemy_count(),
            health_count: default_play3d_health_count(),
            treasure_count: default_play3d_treasure_count(),
            enemy_type: default_enemy_type(),
            health_style: default_health_style(),
            enemy_move_period_ms: default_play3d_enemy_move_period_ms(),
            max_hp: default_play3d_max_hp(),
            levels: LevelsConfig::default(),
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
        // Fallback presets for the no-config / misconfigured path. The
        // `min_solution_length`s are kept comfortably below each grid's geometric
        // minimum path so generation always succeeds with the bushier
        // growing-tree generator (whose spines are shorter than the old
        // recursive-backtracking "rivers"); operators raise them via config.toml.
        Self {
            title: default_play3d_title(),
            easy: Play3dDifficultyConfig {
                rows: 8,
                cols: 8,
                timer_seconds: 120,
                seed: 8_080_808,
                min_solution_length: 12,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Easy".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                perimeter_walls: default_perimeter_walls(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
                door_count: 2,
                spare_doors: 0,
                spare_keys: 0,
                enemy_count: 1,
                health_count: 2,
                treasure_count: 3,
                enemy_type: EnemyTypeConfig::Goblin,
                health_style: HealthStyleConfig::Heart,
                enemy_move_period_ms: 1800,
                max_hp: 3,
                levels: LevelsConfig::default(),
            },
            tricky: Play3dDifficultyConfig {
                rows: 15,
                cols: 15,
                timer_seconds: 240,
                seed: 15_151_515,
                min_solution_length: 24,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Tricky".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                perimeter_walls: default_perimeter_walls(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
                door_count: 3,
                spare_doors: 2,
                spare_keys: 1,
                enemy_count: 3,
                health_count: 3,
                treasure_count: 5,
                enemy_type: EnemyTypeConfig::Goblin,
                health_style: HealthStyleConfig::Heart,
                enemy_move_period_ms: 1500,
                max_hp: 3,
                levels: LevelsConfig {
                    count: 2,
                    ..LevelsConfig::default()
                },
            },
            hard: Play3dDifficultyConfig {
                rows: 25,
                cols: 25,
                timer_seconds: 420,
                seed: 25_252_525,
                min_solution_length: 44,
                minimap_cell_px: default_minimap_cell_px(),
                minimap_radius: default_minimap_radius(),
                title: None,
                mode: "Hard".to_string(),
                landmarks: LandmarksConfig::default(),
                sky_type: default_sky_type(),
                wall_type: default_wall_type(),
                perimeter_walls: default_perimeter_walls(),
                door_style: default_door_style(),
                key_holder: default_key_holder(),
                door_count: 4,
                spare_doors: 3,
                spare_keys: 1,
                enemy_count: 5,
                health_count: 4,
                treasure_count: 8,
                enemy_type: EnemyTypeConfig::Goblin,
                health_style: HealthStyleConfig::Heart,
                enemy_move_period_ms: 1200,
                max_hp: 3,
                levels: LevelsConfig {
                    count: 3,
                    ..LevelsConfig::default()
                },
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
fn default_play3d_door_count() -> u32 {
    0
}
fn default_play3d_spare_doors() -> u32 {
    0
}
fn default_play3d_spare_keys() -> u32 {
    0
}
fn default_play3d_enemy_count() -> u32 {
    0
}
fn default_play3d_health_count() -> u32 {
    0
}
fn default_play3d_treasure_count() -> u32 {
    0
}
fn default_play3d_enemy_move_period_ms() -> u32 {
    1500
}
fn default_play3d_max_hp() -> u32 {
    3
}

/// A run is single-level unless an operator opts in via
/// `[game.play3d.<difficulty>.levels] count = N`.
fn default_levels_count() -> u32 {
    1
}

/// Interim-finish rig defaults to ladder. Operators override per difficulty via
/// `[game.play3d.<difficulty>.levels] finish_type = "portal"`.
fn default_finish_type() -> FinishTypeConfig {
    FinishTypeConfig::Ladder
}

/// Difficulty eases upward by default (hardest at the bottom). Operators
/// override via `[game.play3d.<difficulty>.levels] difficulty_change = "harder"`.
fn default_difficulty_change() -> DifficultyChangeConfig {
    DifficultyChangeConfig::Easier
}

/// The bag resets each level by default. Operators carry it forward via
/// `[game.play3d.<difficulty>.levels] reset_bag = false`.
fn default_reset_bag() -> bool {
    true
}

/// Layers corner-align by default. Operators override via
/// `[game.play3d.<difficulty>.levels] alignment = "centre"`.
fn default_alignment() -> LayeredAlignmentConfig {
    LayeredAlignmentConfig::Edge
}

/// Per-level perimeter randomisation is off by default (every level uses the
/// difficulty's `perimeter_walls`). Operators enable it via
/// `[game.play3d.<difficulty>.levels] perimeter_random = true`.
fn default_perimeter_random() -> bool {
    false
}

/// Enemy rig kind defaults to goblin — the default rig the Bevy game ships
/// with. Operators override per difficulty via
/// `[game.play3d.<difficulty>] enemy_type = "ghost"`.
fn default_enemy_type() -> EnemyTypeConfig {
    EnemyTypeConfig::Goblin
}

/// Health-pickup rig kind defaults to heart — the default rig the Bevy
/// game ships with. Operators override per difficulty via
/// `[game.play3d.<difficulty>] health_style = "potion"`.
fn default_health_style() -> HealthStyleConfig {
    HealthStyleConfig::Heart
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

/// The maze perimeter is walled by default (even under an open sky) — the
/// traditional enclosed-maze look. Operators override per difficulty via
/// `[game.play3d.<difficulty>] perimeter_walls = false`.
fn default_perimeter_walls() -> bool {
    true
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
        assert_eq!(SkyTypeConfig::Dungeon.as_wire_str(), "dungeon");
        assert_eq!(SkyTypeConfig::Chamber.as_wire_str(), "chamber");
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
        // The non-occluding types are also valid whole-maze wall types.
        assert_eq!(WallTypeConfig::Water.as_wire_str(), "water");
        assert_eq!(WallTypeConfig::Lava.as_wire_str(), "lava");
        assert_eq!(WallTypeConfig::IronFence.as_wire_str(), "iron_fence");
    }

    #[test]
    fn non_occluding_wall_types_round_trip_from_toml() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            wall_type = "water"

            [play3d.tricky]
            rows = 8
            cols = 8
            timer_seconds = 50
            seed = 99
            min_solution_length = 16
            wall_type = "lava"

            [play3d.hard]
            rows = 10
            cols = 10
            timer_seconds = 40
            seed = 7
            min_solution_length = 20
            wall_type = "iron_fence"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("valid game config");
        assert_eq!(cfg.play3d.easy.wall_type, WallTypeConfig::Water);
        assert_eq!(cfg.play3d.tricky.wall_type, WallTypeConfig::Lava);
        assert_eq!(cfg.play3d.hard.wall_type, WallTypeConfig::IronFence);
    }

    #[test]
    fn perimeter_walls_round_trips_from_toml_and_defaults_to_true() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            perimeter_walls = false

            [play3d.tricky]
            rows = 8
            cols = 8
            timer_seconds = 50
            seed = 99
            min_solution_length = 16
            # perimeter_walls deliberately omitted — defaults to true

            [play3d.hard]
            rows = 10
            cols = 10
            timer_seconds = 40
            seed = 7
            min_solution_length = 20
            perimeter_walls = true
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("valid game config");
        assert!(!cfg.play3d.easy.perimeter_walls);
        assert!(cfg.play3d.tricky.perimeter_walls); // default
        assert!(cfg.play3d.hard.perimeter_walls);
        // Built-in defaults are all walled.
        let default = Play3dConfig::default();
        assert!(default.easy.perimeter_walls);
        assert!(default.tricky.perimeter_walls);
        assert!(default.hard.perimeter_walls);
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
    fn door_count_round_trips_from_toml_and_defaults_to_zero() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 15
            cols = 15
            timer_seconds = 120
            seed = 42
            min_solution_length = 20
            door_count = 3

            [play3d.tricky]
            rows = 20
            cols = 20
            timer_seconds = 240
            seed = 43
            min_solution_length = 60
            # door_count deliberately omitted — defaults to 0
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.door_count, 3);
        assert_eq!(cfg.play3d.tricky.door_count, 0);
        // The shipped fallback presets seed a few doors per difficulty.
        assert_eq!(Play3dConfig::default().easy.door_count, 2);
        assert_eq!(Play3dConfig::default().hard.door_count, 4);
    }

    #[test]
    fn spare_doors_and_spare_keys_round_trip_from_toml_and_default_to_zero() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 15
            cols = 15
            timer_seconds = 120
            seed = 42
            min_solution_length = 20
            spare_doors = 2
            spare_keys = 1

            [play3d.tricky]
            rows = 20
            cols = 20
            timer_seconds = 240
            seed = 43
            min_solution_length = 60
            # spare_doors / spare_keys deliberately omitted — default to 0
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.spare_doors, 2);
        assert_eq!(cfg.play3d.easy.spare_keys, 1);
        assert_eq!(cfg.play3d.tricky.spare_doors, 0);
        assert_eq!(cfg.play3d.tricky.spare_keys, 0);
        // The shipped fallback presets ramp the strand risk by difficulty.
        let d = Play3dConfig::default();
        assert_eq!((d.easy.spare_doors, d.easy.spare_keys), (0, 0));
        assert_eq!((d.tricky.spare_doors, d.tricky.spare_keys), (2, 1));
        assert_eq!((d.hard.spare_doors, d.hard.spare_keys), (3, 1));
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

    #[test]
    fn unknown_enemy_type_falls_back_to_goblin() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            enemy_type = "dragon"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.enemy_type, EnemyTypeConfig::Goblin);
    }

    #[test]
    fn unknown_health_style_falls_back_to_heart() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            health_style = "elixir"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typo must not fail load");
        assert_eq!(cfg.play3d.easy.health_style, HealthStyleConfig::Heart);
    }

    #[test]
    fn enemy_type_and_health_style_as_wire_str_matches_serde_form() {
        assert_eq!(EnemyTypeConfig::Goblin.as_wire_str(), "goblin");
        assert_eq!(EnemyTypeConfig::Ghost.as_wire_str(), "ghost");
        assert_eq!(HealthStyleConfig::Heart.as_wire_str(), "heart");
        assert_eq!(HealthStyleConfig::Potion.as_wire_str(), "potion");
    }

    #[test]
    fn enemy_and_health_knobs_round_trip_from_toml() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            enemy_count = 4
            health_count = 3
            treasure_count = 6
            enemy_type = "ghost"
            health_style = "potion"
            enemy_move_period_ms = 900
            max_hp = 5
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        let preset = &cfg.play3d.easy;
        assert_eq!(preset.enemy_count, 4);
        assert_eq!(preset.health_count, 3);
        assert_eq!(preset.treasure_count, 6);
        assert_eq!(preset.enemy_type, EnemyTypeConfig::Ghost);
        assert_eq!(preset.health_style, HealthStyleConfig::Potion);
        assert_eq!(preset.enemy_move_period_ms, 900);
        assert_eq!(preset.max_hp, 5);
    }

    #[test]
    fn treasure_count_defaults_to_zero_when_omitted() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.treasure_count, 0);
    }

    #[test]
    fn levels_round_trips_from_toml_and_defaults_to_a_single_level() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            [play3d.easy.levels]
            count = 3
            finish_type = "random"
            difficulty_change = "harder"
            reset_bag = false
            alignment = "centre"
            perimeter_random = true
            hide_completed_enemies = true

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            # levels table omitted entirely — must default to a single level

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            [play3d.hard.levels]
            # only count present — the rest default
            count = 2
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        // Easy overrides every field.
        let easy = &cfg.play3d.easy.levels;
        assert_eq!(easy.count, 3);
        assert_eq!(easy.finish_type, FinishTypeConfig::Random);
        assert_eq!(easy.difficulty_change, DifficultyChangeConfig::Harder);
        assert!(!easy.reset_bag);
        assert_eq!(easy.alignment, LayeredAlignmentConfig::Centre);
        assert!(easy.perimeter_random);
        assert!(easy.hide_completed_enemies);
        assert!(easy.top.is_none());
        // Tricky omits the whole table → single-level defaults.
        let tricky = &cfg.play3d.tricky.levels;
        assert_eq!(tricky.count, 1);
        assert_eq!(tricky.finish_type, FinishTypeConfig::Ladder);
        assert_eq!(tricky.difficulty_change, DifficultyChangeConfig::Easier);
        assert!(tricky.reset_bag);
        assert_eq!(tricky.alignment, LayeredAlignmentConfig::Edge);
        assert!(!tricky.perimeter_random);
        assert!(!tricky.hide_completed_enemies);
        // Hard sets only count; the rest fall back to defaults.
        assert_eq!(cfg.play3d.hard.levels.count, 2);
        assert_eq!(cfg.play3d.hard.levels.finish_type, FinishTypeConfig::Ladder);
        assert!(cfg.play3d.hard.levels.reset_bag);
    }

    #[test]
    fn unknown_levels_enum_values_fall_back_to_defaults() {
        // Forgiving deserialisers — typos (or values from a future version)
        // must not kill config load; they degrade to the documented defaults.
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            [play3d.easy.levels]
            count = 2
            finish_type = "trapdoor"
            difficulty_change = "spicier"
            alignment = "diagonal"
        "#;
        let cfg: GameConfig = toml::from_str(toml).expect("typos must not fail load");
        let easy = &cfg.play3d.easy.levels;
        assert_eq!(easy.finish_type, FinishTypeConfig::Ladder);
        assert_eq!(easy.difficulty_change, DifficultyChangeConfig::Easier);
        assert_eq!(easy.alignment, LayeredAlignmentConfig::Edge);
        // `center` (US spelling) still resolves to Centre.
        let toml = r#"
            [play3d]
            title = "Maze 3D"
            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            [play3d.easy.levels]
            alignment = "center"
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.play3d.easy.levels.alignment, LayeredAlignmentConfig::Centre);
    }

    #[test]
    fn levels_top_override_round_trips_and_defaults_to_none() {
        let toml = r#"
            [play3d]
            title = "Maze 3D"

            [play3d.easy]
            rows = 6
            cols = 6
            timer_seconds = 60
            seed = 42
            min_solution_length = 12
            [play3d.easy.levels]
            count = 3
            [play3d.easy.levels.top]
            sky_type = "day"
            perimeter_walls = false

            [play3d.tricky]
            rows = 12
            cols = 12
            timer_seconds = 180
            seed = 43
            min_solution_length = 60
            [play3d.tricky.levels]
            count = 2
            # no [levels.top] table — override stays None

            [play3d.hard]
            rows = 20
            cols = 20
            timer_seconds = 360
            seed = 44
            min_solution_length = 160
            [play3d.hard.levels.top]
            # present but empty — both fields default to None
        "#;
        let cfg: GameConfig = toml::from_str(toml).unwrap();
        let top = cfg.play3d.easy.levels.top.as_ref().expect("easy has a top override");
        assert_eq!(top.sky_type, Some(SkyTypeConfig::Day));
        assert_eq!(top.perimeter_walls, Some(false));
        // Tricky has no top table at all.
        assert!(cfg.play3d.tricky.levels.top.is_none());
        // Hard's table is present but empty — the struct exists, fields None.
        let hard = cfg.play3d.hard.levels.top.as_ref().expect("hard has an (empty) top table");
        assert!(hard.sky_type.is_none());
        assert!(hard.perimeter_walls.is_none());
    }

    #[test]
    fn levels_enums_as_wire_str_match_serde_form() {
        assert_eq!(FinishTypeConfig::Ladder.as_wire_str(), "ladder");
        assert_eq!(FinishTypeConfig::Portal.as_wire_str(), "portal");
        assert_eq!(FinishTypeConfig::Random.as_wire_str(), "random");
        assert_eq!(DifficultyChangeConfig::Same.as_wire_str(), "same");
        assert_eq!(DifficultyChangeConfig::Easier.as_wire_str(), "easier");
        assert_eq!(DifficultyChangeConfig::Harder.as_wire_str(), "harder");
        assert_eq!(LayeredAlignmentConfig::Edge.as_wire_str(), "edge");
        assert_eq!(LayeredAlignmentConfig::Centre.as_wire_str(), "centre");
        assert_eq!(LayeredAlignmentConfig::Random.as_wire_str(), "random");
    }

    #[test]
    fn default_presets_ramp_the_level_count_by_difficulty() {
        let cfg = Play3dConfig::default();
        assert_eq!(cfg.easy.levels.count, 1);
        assert_eq!(cfg.tricky.levels.count, 2);
        assert_eq!(cfg.hard.levels.count, 3);
        // Everything else stays at the documented single-level defaults.
        assert_eq!(cfg.hard.levels.finish_type, FinishTypeConfig::Ladder);
        assert_eq!(cfg.hard.levels.difficulty_change, DifficultyChangeConfig::Easier);
        assert!(cfg.hard.levels.reset_bag);
        assert_eq!(cfg.hard.levels.alignment, LayeredAlignmentConfig::Edge);
        assert!(!cfg.hard.levels.perimeter_random);
        assert!(cfg.hard.levels.top.is_none());
    }
}
