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

/// Optional override for the whole run's level set, bottom level first. When
/// present (and non-empty) it supplies the multi-level run directly, taking
/// precedence over the single [`PendingMazeJson`] and the native demos — the seam
/// a multi-level host launch (and the rendering tests) feed levels through.
#[derive(Resource, Default)]
pub(crate) struct PendingLevels(pub(crate) Vec<String>);

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
    /// Remaining milliseconds for the red damage-flash overlay. `0.0` when
    /// no flash is active. Set on `PlayerDamaged` event and decremented by
    /// the damage-flash system; the overlay fades its alpha proportionally.
    pub(crate) damage_flash_timer: f32,
    /// World-space offset the camera is shifted by for the level currently being
    /// played: the level's Y lift plus its X/Z centring offset (see
    /// [`crate::world::LevelPlacement::camera_offset`]). The move animation stays
    /// in the level's local (ground) frame; `movement_system` adds this when
    /// writing the camera transform, so advancing a level lifts + centres the
    /// camera onto it. `Vec3::ZERO` on the bottom level, so single-level games
    /// (and `Edge` stacks) are unchanged.
    pub(crate) camera_offset: Vec3,
}

#[derive(Resource)]
pub(crate) struct GameClock {
    pub(crate) remaining_secs: f32,
    pub(crate) elapsed_secs: f32,
    pub(crate) last_displayed_secs: i32,
}

/// Run-level state for a multi-level game: the per-level maze JSON, the active
/// level index, and the totals that carry across levels. A single-level game
/// is just a run with one level — every multi-level branch collapses to the
/// current single-level behaviour, so a 1-level run is byte-for-byte identical
/// to a non-multi-level game.
#[derive(Resource)]
pub(crate) struct MultiLevelRun {
    /// Per-level maze JSON, bottom level (index 0) first. The active level's
    /// maze is also live in [`GameState::game`]; this holds every level so the
    /// run can build the next one on a transition.
    pub(crate) levels: Vec<String>,
    /// Index (into [`Self::levels`]) of the level currently being played.
    pub(crate) current_level: usize,
    /// Sum of the score of every level completed so far. The live level's
    /// score is added on top for the running total (see
    /// [`Self::cumulative_score`]).
    pub(crate) banked_score: u64,
    /// Per-style treasure collected on completed levels, ordered as
    /// [`maze::MazeGame::collected_treasure`]. The live level's counts are
    /// added on top for display.
    pub(crate) carried_treasure: Vec<(maze::TreasureStyle, u32)>,
    /// When `true` (the default) each level starts with an empty bag; when
    /// `false` the player's whole bag carries from one level into the next.
    pub(crate) reset_bag_between_levels: bool,
    /// `(rows, cols)` of the bottom level's grid — the reference footprint the
    /// upper levels' `Centre` X/Z offsets are measured against. Used on a level
    /// transition to compute the new level's camera placement.
    pub(crate) base_dims: (usize, usize),
}

impl MultiLevelRun {
    /// A run over the given per-level maze JSONs (bottom level first). A
    /// one-element `levels` is a single-level game — the shape `spawn_world`
    /// inserts for every non-multi-level game; every cross-level total starts
    /// empty and the bag resets by default.
    pub(crate) fn new(levels: Vec<String>) -> Self {
        // The bottom level's footprint anchors the upper levels' `Centre` offsets.
        let base_dims = levels
            .first()
            .and_then(|json| MazeGame::from_json(json).ok())
            .map(|game| {
                let grid = game.grid();
                (grid.len(), grid.first().map_or(0, |row| row.len()))
            })
            .unwrap_or((0, 0));
        Self {
            levels,
            current_level: 0,
            banked_score: 0,
            carried_treasure: Vec::new(),
            reset_bag_between_levels: true,
            base_dims,
        }
    }

    /// Total number of levels in the run.
    pub(crate) fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Whether the active level is the last one — reaching its finish wins the
    /// run rather than transitioning to another level.
    pub(crate) fn is_final(&self) -> bool {
        self.current_level + 1 >= self.levels.len()
    }

    /// The running total score: every completed level's banked score plus the
    /// live level's current score.
    pub(crate) fn cumulative_score(&self, live_score: u64) -> u64 {
        self.banked_score + live_score
    }

    /// The run's cumulative collected treasure for display: the banked
    /// `carried_treasure` from completed levels merged with the live level's
    /// `collected` counts (per style, in carried-then-live order). Each level's
    /// `MazeGame` only tracks its own treasure, so the HUD reads this to keep the
    /// treasure chips accumulating across the run rather than resetting on every
    /// level transition.
    pub(crate) fn cumulative_treasure(
        &self,
        live: &[(maze::TreasureStyle, u32)],
    ) -> Vec<(maze::TreasureStyle, u32)> {
        let mut acc = self.carried_treasure.clone();
        for &(style, count) in live {
            if count == 0 {
                continue;
            }
            if let Some(entry) = acc.iter_mut().find(|(s, _)| *s == style) {
                entry.1 += count;
            } else {
                acc.push((style, count));
            }
        }
        acc
    }
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
    /// Whether the maze perimeter is walled at the grid edge under an **open**
    /// sky. Enclosed skies (`Dungeon` / `Chamber`) always wall the perimeter
    /// regardless of this flag; for open skies, `true` walls the edge (the
    /// traditional enclosed-maze look) and `false` shows the skybox past it (a
    /// pool's low rim / a fence's bars still frame the edge). Default `true`.
    /// Solid border `'W'` cells are unaffected either way (their grid-edge faces
    /// are never drawn).
    pub perimeter_walls: bool,
    /// Door open-animation style. Default `Swing`.
    pub door_style: DoorStyle,
    /// Key-holder appearance for `'K'` cells. Default `Pedestal`.
    pub key_holder: KeyHolderStyle,
    /// Move period for enemies (ms of accumulated `tick(dt_ms)` per cell
    /// advance). Threaded into `MazeGameOptions::enemy_move_period_ms`.
    /// Default `1500.0` (matches the maze crate's default).
    pub enemy_move_period_ms: f32,
    /// Damage each enemy deals on same-cell collision. Threaded into
    /// `MazeGameOptions::enemy_damage`. Default `1`.
    pub enemy_damage: u32,
    /// Maximum player HP — also the heal cap. Threaded into
    /// `MazeGameOptions::max_hp`. Default `3`.
    pub max_hp: u32,
    /// Starting player HP. Threaded into `MazeGameOptions::starting_hp`
    /// (clamped to `[1, max_hp]` inside the maze crate). Default `3`
    /// (= `max_hp` → start at full health).
    pub starting_hp: u32,
    /// Visual variant used for every enemy in the session. Default
    /// `Goblin`. The AI and damage mechanics are identical across
    /// variants — only the spawned rig differs.
    pub enemy_type: EnemyType,
    /// Visual variant used for every health pickup in the session.
    /// Default `Heart`. The auto-pickup + heal mechanics are identical
    /// across variants — only the spawned rig differs.
    pub health_style: HealthStyle,
    /// How smaller upper levels of a multi-level run are positioned over the
    /// bottom level — `Edge` (common corner, zero offset) or `Centre`. Only
    /// meaningful for an open-sky multi-level stack; a no-op for single-level
    /// games (level 0 has zero offset under either mode). Default `Edge`.
    pub layered_alignment: LayeredAlignment,
}

/// Atmospheric sky modes. Each variant maps to a procedurally generated
/// dome texture + a paired light preset (see `world/sky`). Default is
/// `Night` so a missing or unrecognised config value preserves the
/// pre-Step-10 visual. `Dungeon` and `Chamber` are the odd ones out: instead
/// of an open sky they cap every passable cell with a ceiling and dim the
/// lighting, for an enclosed feel. `Dungeon` uses a hewn dark-rock ceiling;
/// `Chamber` uses the cell's own wall material, reading as a finished, built
/// interior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkyType {
    #[default]
    Night,
    Sunrise,
    Day,
    Sunset,
    Dungeon,
    Chamber,
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
            Self::Dungeon => "dungeon",
            Self::Chamber => "chamber",
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
            "dungeon" => Self::Dungeon,
            "chamber" => Self::Chamber,
            _ => Self::Night,
        }
    }

    /// Whether this is an enclosed (roofed-over) world — `Dungeon` or `Chamber`.
    /// The open-air modes show the skybox; the enclosed modes cap every cell with a
    /// ceiling and wall the maze in, so a non-occluding cell at the grid edge is
    /// walled (floor upwards) rather than opened to the sky.
    pub fn is_enclosed(self) -> bool {
        matches!(self, Self::Dungeon | Self::Chamber)
    }
}

/// Wall types. The four solid-wall textures (`Brick` / `DressedStone` / `Wood` /
/// `Cobblestone`) each map to a `WALL_MATERIAL_*` index for the panel material;
/// the three special types (`Water` / `Lava` / `IronFence`) are **non-occluding**
/// and render their own in-cell geometry (a floor-level pool, or see-through
/// bars) instead of a solid panel. Shares its wire vocabulary with the
/// `data_model` `WallType` and the per-cell `wallType` override. Default is
/// `Brick` so a missing or unrecognised wire value preserves the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WallType {
    #[default]
    Brick,
    DressedStone,
    Wood,
    Cobblestone,
    Water,
    Lava,
    IronFence,
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
            Self::Water => "water",
            Self::Lava => "lava",
            Self::IronFence => "iron_fence",
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
            "water" => Self::Water,
            "lava" => Self::Lava,
            "iron_fence" => Self::IronFence,
            _ => Self::Brick,
        }
    }

    /// The matching `WALL_MATERIAL_*` index for a solid-wall texture, or `None`
    /// for a non-occluding type (which has no panel material). Single source of
    /// truth so no call site hard-codes the integer mapping.
    pub fn to_kind_index(self) -> Option<usize> {
        match self {
            Self::Brick => Some(crate::world::walls::WALL_MATERIAL_BRICK),
            Self::DressedStone => Some(crate::world::walls::WALL_MATERIAL_DRESSED_STONE),
            Self::Wood => Some(crate::world::walls::WALL_MATERIAL_WOOD),
            Self::Cobblestone => Some(crate::world::walls::WALL_MATERIAL_COBBLESTONE),
            Self::Water | Self::Lava | Self::IronFence => None,
        }
    }

    /// Whether this wall type renders as see-through in-cell geometry (a
    /// floor-level pool, or vertical bars) instead of a solid occluding panel.
    /// A non-occluding `'W'` cell is un-skipped in the spawn loop so its
    /// geometry is drawn, and the wall panel between it and any open or
    /// non-occluding neighbour is suppressed so the region reads as continuous
    /// and the player can see across it. The exact inverse of
    /// [`to_kind_index`](Self::to_kind_index) being `Some`.
    pub fn is_non_occluding(self) -> bool {
        matches!(self, Self::Water | Self::Lava | Self::IronFence)
    }
}

/// Door open-animation styles. `Swing` only applies to a straight-corridor
/// door (a single leaf hinged between the side walls); at any other topology —
/// and for the other styles at every topology — a leaf is hung on each open
/// edge, and `Swing` degrades to `Slide` there because a swing needs walls to
/// anchor against. Default `Swing` preserves the topology-driven look the game
/// shipped with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DoorStyle {
    #[default]
    Swing,
    Slide,
    Portcullis,
    Dissolve,
}

impl DoorStyle {
    /// `snake_case` wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Swing => "swing",
            Self::Slide => "slide",
            Self::Portcullis => "portcullis",
            Self::Dissolve => "dissolve",
        }
    }

    /// Parses a wire string into a [`DoorStyle`]. Unknown values fall back to
    /// [`DoorStyle::Swing`] — same forgiving policy as [`WallType::from_wire_str`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "slide" => Self::Slide,
            "portcullis" => Self::Portcullis,
            "dissolve" => Self::Dissolve,
            _ => Self::Swing,
        }
    }
}

/// Key-holder styles for `'K'` cells. Default `Pedestal` preserves the shipped
/// look. (These variants may later distinguish key *types*, not just looks.)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyHolderStyle {
    #[default]
    Pedestal,
    Chest,
    FloatingKey,
}

impl KeyHolderStyle {
    /// `snake_case` wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Pedestal => "pedestal",
            Self::Chest => "chest",
            Self::FloatingKey => "floating_key",
        }
    }

    /// Parses a wire string into a [`KeyHolderStyle`]. Unknown values fall back
    /// to [`KeyHolderStyle::Pedestal`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "chest" => Self::Chest,
            "floating_key" => Self::FloatingKey,
            _ => Self::Pedestal,
        }
    }
}

/// Enemy visual variants. Both variants use the same AI / damage
/// mechanics — only the spawned rig differs. Default `Goblin`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnemyType {
    #[default]
    Goblin,
    Ghost,
}

impl EnemyType {
    /// Lowercase wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Goblin => "goblin",
            Self::Ghost => "ghost",
        }
    }

    /// Parses a wire string into an [`EnemyType`]. Unknown values fall
    /// back to [`EnemyType::Goblin`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "ghost" => Self::Ghost,
            _ => Self::Goblin,
        }
    }
}

/// Health-pickup visual variants. Both variants use the same auto-pickup
/// + heal mechanics — only the spawned rig differs. Default `Heart`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HealthStyle {
    #[default]
    Heart,
    Potion,
}

impl HealthStyle {
    /// Lowercase wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Heart => "heart",
            Self::Potion => "potion",
        }
    }

    /// Parses a wire string into a [`HealthStyle`]. Unknown values fall
    /// back to [`HealthStyle::Heart`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "potion" => Self::Potion,
            _ => Self::Heart,
        }
    }
}

/// Treasure visual variants for `'T'` cells. The style is purely cosmetic — the
/// reward value (engine-side) is independent — and is chosen per cell from its
/// `style` override, falling back to `Silver`. Default `Silver`. Shares its wire
/// vocabulary with the `data_model` `TreasureStyle`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreasureStyle {
    #[default]
    Silver,
    Gold,
    Diamonds,
    Jewels,
}

impl TreasureStyle {
    /// Lowercase wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Silver => "silver",
            Self::Gold => "gold",
            Self::Diamonds => "diamonds",
            Self::Jewels => "jewels",
        }
    }

    /// Parses a wire string into a [`TreasureStyle`]. Unknown values fall back
    /// to [`TreasureStyle::Silver`].
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "gold" => Self::Gold,
            "diamonds" => Self::Diamonds,
            "jewels" => Self::Jewels,
            _ => Self::Silver,
        }
    }
}

/// How the layers of a multi-level run are positioned over one another when the
/// upper levels are smaller mazes than the bottom (only meaningful for an
/// open-sky stack — a roofed stack seals each level so alignment is invisible).
/// `Edge` stacks every layer to a common origin corner (zero X/Z offset — the
/// historical behaviour); `Centre` centres each smaller layer over the bottom
/// layer. Default `Edge` so single-level games and uniform stacks are unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayeredAlignment {
    #[default]
    Edge,
    Centre,
}

impl LayeredAlignment {
    /// Lowercase wire form, matching the JSON / TOML strings the server emits.
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Edge => "edge",
            Self::Centre => "centre",
        }
    }

    /// Parses a wire string into a [`LayeredAlignment`]. Unknown values fall back
    /// to [`LayeredAlignment::Edge`] — same forgiving policy as the other config
    /// enums. Accepts the US spelling `center` as well.
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "centre" | "center" => Self::Centre,
            _ => Self::Edge,
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
            perimeter_walls: true,
            door_style: DoorStyle::default(),
            key_holder: KeyHolderStyle::default(),
            enemy_move_period_ms: 1500.0,
            enemy_damage: 1,
            max_hp: 3,
            starting_hp: 3,
            enemy_type: EnemyType::default(),
            health_style: HealthStyle::default(),
            layered_alignment: LayeredAlignment::default(),
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
    pub score: u64,
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
    fn game_result_serializes_score_as_camel_case_number() {
        let result = GameResult {
            outcome: GameOutcome::Win,
            elapsed_ms: 83_456,
            score: 5,
            difficulty: None,
            rows: 3,
            cols: 3,
            seed: None,
            extras: std::collections::BTreeMap::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"score\":5"), "score missing from {json}");
        assert!(json.contains("\"elapsedMs\":83456"));
    }

    #[test]
    fn cumulative_treasure_merges_banked_with_live_so_chips_persist_across_levels() {
        use maze::TreasureStyle;
        // A run that banked silver + gold on completed levels, now on a level
        // where the player has picked up more gold and some diamonds.
        let mut run = MultiLevelRun::new(vec!["a".into(), "b".into()]);
        run.carried_treasure = vec![(TreasureStyle::Silver, 2), (TreasureStyle::Gold, 1)];
        let live = [(TreasureStyle::Gold, 1), (TreasureStyle::Diamonds, 3)];

        let cumulative = run.cumulative_treasure(&live);

        // Banked styles persist (so the chips don't reset on a level transition),
        // the live gold folds into the banked gold, and the new diamonds append.
        assert_eq!(
            cumulative,
            vec![
                (TreasureStyle::Silver, 2),
                (TreasureStyle::Gold, 2),
                (TreasureStyle::Diamonds, 3),
            ],
        );
        // Zero-count live entries never add a chip.
        assert_eq!(
            run.cumulative_treasure(&[(TreasureStyle::Jewels, 0)]),
            run.carried_treasure,
        );
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
