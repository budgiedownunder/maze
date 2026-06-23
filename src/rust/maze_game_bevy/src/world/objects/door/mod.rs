//! Door panels for `'D'` cells.
//!
//! A door is one or more primitive-rig leaves filling the corridor opening at a
//! `'D'` cell. It carries no maze-blocking geometry of its own — passability is
//! decided entirely by [`maze::MazeGame`], which gates *entry into the cell from
//! every side* until the door opens (the leaves are purely a *view* of the
//! door's [`DoorState`]). Leaves render in the **same wall material as the cell
//! they sit between** (see [`crate::world::walls::wall_kind_for_cell`]); a
//! keyhole marks them, and each is eligible for the same sparse wall decoration
//! as a wall panel.
//!
//! **Style + placement.** [`crate::state::DoorStyle`] selects the open motion:
//! - `Swing` only at a straight corridor (two opposing open edges) → a single
//!   leaf, centred and hinged against the side walls.
//! - Every other case — `Swing` at a non-corridor, and `Slide` / `Portcullis` /
//!   `Dissolve` everywhere — hangs a leaf on **each open edge** (a swing can't
//!   anchor there, so `Swing` falls back to `Slide`). The per-leaf motion is
//!   recorded as a [`DoorMotion`] and applied each frame by
//!   `door_animation_system`.
//!
//! A **non-occluding** wall neighbour (water / lava / iron fence) counts as an
//! open edge here, not a wall: its panel is suppressed, so a leaf must seal that
//! side and a swing has no panel to hinge against — see [`open_for_door`].
//!
//! The motion rigs live in sibling files: [`swing`] (hinge), [`slide`] (drop
//! into the floor), [`portcullis`] (rise into a framed gate), and [`dissolve`]
//! (fade a per-leaf material). The slab is in [`panel`] and the lock in
//! [`keyhole`]. The countdown advances deterministically in the central
//! `FixedUpdate` tick driver (`crate::tick::game_tick_system`); once open a
//! leaf is pinned to its open pose permanently and never despawned.

pub(crate) mod dissolve;
pub(crate) mod keyhole;
pub(crate) mod panel;
pub(crate) mod portcullis;
pub(crate) mod slide;
pub(crate) mod swing;

use crate::state::{DoorStyle, GameConfig, GameState};
use crate::world::decorations::wall::{
    wall_decoration_index, WallDecoration, WallDecorationAssets, DECORATION_OFFSET, DECORATION_Y,
};
use crate::world::walls::{is_non_occluding_wall, wall_kind_for_cell, WallAssets, PANEL_W};
use crate::world::{LevelPlacement, CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use maze::{CellEntity, DoorState};
use panel::DOOR_THICKNESS;
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

/// How a single door leaf opens. Derived from [`DoorStyle`] per leaf (a `Swing`
/// style degrades to `Slide` on per-edge leaves that can't anchor a hinge).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DoorMotion {
    Swing,
    Slide,
    Portcullis,
    Dissolve,
}

/// Marker on a door leaf's pivot entity. The panel, keyhole, and any decoration
/// are children of this entity, so transforming it moves the whole leaf. A door
/// cell may own several leaves (one per open edge); they share the cell's
/// [`DoorState`].
#[derive(Component)]
pub(crate) struct DoorMarker {
    /// Grid cell this leaf belongs to.
    pub(crate) cell: (usize, usize),
    /// Yaw orienting the leaf so its local +X spans the opening and its local
    /// +Z (keyhole / decoration face) points out toward the neighbour.
    closed_yaw: f32,
    /// How this leaf opens.
    motion: DoorMotion,
    /// The leaf's resting (closed) pivot translation, captured at spawn so the
    /// slide / portcullis motions can offset from it (and swing / dissolve hold
    /// it).
    base_translation: Vec3,
    /// Set once a `GameEvent::DoorOpened` has been applied — the door is then
    /// pinned to its fully-open pose permanently and never re-reads its state.
    /// Mutated only via [`DoorMarker::mark_opened`] so the write path stays
    /// explicit and discoverable from outside this module.
    opened: bool,
    /// For [`DoorMotion::Dissolve`] only: the leaf's own cloned, alpha-blended
    /// materials (panel + keyhole plate + keyhole) each paired with its base
    /// emissive, so the whole leaf fades together without touching the shared
    /// wall / keyhole materials. Empty for every other motion.
    dissolve_materials: Vec<(Handle<StandardMaterial>, LinearRgba)>,
}

impl DoorMarker {
    /// Pins the leaf to its fully-open pose permanently. The only path
    /// that flips a closed leaf to opened. Called from the central tick
    /// driver (`crate::tick::game_tick_system`) when a
    /// `GameEvent::DoorOpened` for this leaf's cell fires.
    pub(crate) fn mark_opened(&mut self) {
        self.opened = true;
    }
}

pub(crate) struct DoorAssets {
    panel: panel::PanelAssets,
    keyhole: keyhole::KeyholeAssets,
    /// Unit cuboid for the portcullis frame (posts + lintel).
    cuboid: Option<Handle<Mesh>>,
}

pub(crate) fn build_door_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> DoorAssets {
    DoorAssets {
        panel: panel::build_panel_assets(meshes),
        keyhole: keyhole::build_keyhole_assets(meshes, materials),
        cuboid: meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0))),
    }
}

/// Which axis a door cell's straight corridor runs along.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CorridorAxis {
    /// Passage runs north–south (walls east & west).
    NorthSouth,
    /// Passage runs east–west (walls north & south).
    EastWest,
}

/// Whether the in-bounds neighbour `(nr, nc)` reads as a passage opening for door
/// placement: a passable cell, **or** a non-occluding wall (water / lava / iron
/// fence). A non-occluding cell has no wall panel, so a door leaf must seal that
/// side and a swing has nothing to anchor against — exactly like an open
/// neighbour. Only a *solid* wall counts as a wall here.
fn open_for_door(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    nr: usize,
    nc: usize,
) -> bool {
    grid[nr][nc] != 'W' || is_non_occluding_wall(grid, cell_entities, config, nr, nc)
}

/// The axis of the door cell's straight corridor, or `None` if it isn't one.
///
/// A straight corridor has *solid* walls on both sides of one axis and at least
/// one open passage on the perpendicular axis. Out-of-bounds counts as a wall, so
/// a corridor capped by the grid edge at one end still qualifies — the swing rig
/// only needs the two facing walls to anchor its hinge, so whether the far end is
/// closed by a wall or by the maze boundary is immaterial. A **non-occluding**
/// neighbour (water / lava / iron fence) counts as an *opening*, not a wall — its
/// panel is suppressed, so a swing can't hinge against it (see [`open_for_door`]).
/// Corners, T-/cross-junctions, and open areas return `None` (any third open side
/// disqualifies it).
fn corridor_axis(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) -> Option<CorridorAxis> {
    let rows = grid.len();
    let cols = grid[r].len();
    let n = r > 0 && open_for_door(grid, cell_entities, config, r - 1, c);
    let s = r + 1 < rows && open_for_door(grid, cell_entities, config, r + 1, c);
    let e = c + 1 < cols && open_for_door(grid, cell_entities, config, r, c + 1);
    let w = c > 0 && open_for_door(grid, cell_entities, config, r, c - 1);
    if !e && !w && (n || s) {
        Some(CorridorAxis::NorthSouth)
    } else if !n && !s && (e || w) {
        Some(CorridorAxis::EastWest)
    } else {
        None
    }
}

/// The wall material a leaf borrows: the N/S panel material when its face normal
/// runs along Z, otherwise the E/W material — matching the surrounding wall.
fn leaf_material(
    wall_assets: &WallAssets,
    kind: usize,
    normal_z: bool,
) -> Option<Handle<StandardMaterial>> {
    if normal_z {
        wall_assets.ns.material_mats[kind].clone()
    } else {
        wall_assets.ew.material_mats[kind].clone()
    }
}

/// How a single leaf is positioned and behaves, gathered so [`spawn_leaf`] takes
/// one bundle rather than a long argument list.
struct LeafSpec {
    closed_yaw: f32,
    pivot_translation: Vec3,
    /// Centre of the opening this leaf seals (used for the portcullis frame).
    edge_centre: Vec3,
    panel_mat: Option<Handle<StandardMaterial>>,
    motion: DoorMotion,
    /// Decoration hash face id (the sealed edge's compass id, or the corridor
    /// axis for the single swing leaf).
    face_id: u32,
    /// Swing leaves are seen from both ends of the corridor, so they get a
    /// keyhole on both faces; per-edge leaves only on the outward face.
    keyhole_both_faces: bool,
}

/// Clones `src` into an alpha-blended dissolve material, records it in `out` so
/// the fade can drive it, and returns the clone's handle. Falls back to `src`
/// (unrecorded) when the clone can't be made (no material assets / missing
/// source).
fn clone_or(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    src: &Option<Handle<StandardMaterial>>,
    out: &mut Vec<(Handle<StandardMaterial>, LinearRgba)>,
) -> Option<Handle<StandardMaterial>> {
    if let Some((handle, base)) = dissolve::clone_blend(materials, src) {
        out.push((handle.clone(), base));
        Some(handle)
    } else {
        src.clone()
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_leaf(
    commands: &mut Commands,
    door_assets: &DoorAssets,
    decoration_assets: &WallDecorationAssets,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    r: usize,
    c: usize,
    config: &GameConfig,
    spec: LeafSpec,
) {
    // A dissolve leaf renders the panel AND keyhole with their own alpha-blended
    // clones so the whole leaf can fade without touching the shared materials;
    // every other motion uses the shared materials directly.
    let is_dissolve = spec.motion == DoorMotion::Dissolve;
    let mut dissolve_materials = Vec::new();
    let panel_mat = if is_dissolve {
        clone_or(materials, &spec.panel_mat, &mut dissolve_materials)
    } else {
        spec.panel_mat.clone()
    };
    let (plate_mat, keyhole_mat) = if is_dissolve {
        (
            clone_or(materials, &door_assets.keyhole.plate_handle(), &mut dissolve_materials),
            clone_or(materials, &door_assets.keyhole.keyhole_handle(), &mut dissolve_materials),
        )
    } else {
        (None, None)
    };

    let pivot = commands
        .spawn((
            DoorMarker {
                cell: (r, c),
                closed_yaw: spec.closed_yaw,
                motion: spec.motion,
                base_translation: spec.pivot_translation,
                opened: false,
                dissolve_materials,
            },
            Transform::from_translation(spec.pivot_translation)
                .with_rotation(Quat::from_rotation_y(spec.closed_yaw)),
            Visibility::default(),
        ))
        .id();

    panel::spawn_panel(commands, &door_assets.panel, panel_mat, pivot);
    // Keyhole on the outward (+Z) face — the side the approaching player sees.
    // Dissolve leaves use their cloned (fading) materials; others the shared ones.
    if is_dissolve {
        keyhole::spawn_keyhole_face_with(commands, &door_assets.keyhole, pivot, 1.0, plate_mat, keyhole_mat);
    } else {
        keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, 1.0);
        if spec.keyhole_both_faces {
            keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, -1.0);
        }
    }

    // A portcullis grille rises into a static wall-material frame.
    if spec.motion == DoorMotion::Portcullis {
        portcullis::spawn_frame(
            commands,
            door_assets.cuboid.clone(),
            spec.panel_mat.clone(),
            spec.edge_centre,
            spec.closed_yaw,
        );
    }

    // Eligible for the same sparse, seeded wall decoration as a wall panel, on
    // the outward face and parented to the pivot so it moves with the leaf.
    if config.landmarks.wall_decorations {
        if let Some(decoration_kind) = wall_decoration_index(r, c, spec.face_id, config.seed) {
            if let (Some(mesh), Some(mat)) = (
                decoration_assets.ns_mesh.clone(),
                decoration_assets.mats[decoration_kind as usize].clone(),
            ) {
                let decoration = commands
                    .spawn((
                        WallDecoration,
                        Mesh3d(mesh),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(
                            PANEL_W / 2.0,
                            DECORATION_Y,
                            DOOR_THICKNESS / 2.0 + DECORATION_OFFSET,
                        ),
                    ))
                    .id();
                commands.entity(pivot).add_child(decoration);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_door_for_cell(
    commands: &mut Commands,
    door_assets: &DoorAssets,
    wall_assets: &WallAssets,
    decoration_assets: &WallDecorationAssets,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
    cell_entity: Option<&CellEntity>,
    placement: LevelPlacement,
) {
    if cell != 'D' {
        return;
    }
    // Per-cell `doorStyle` override, else the per-maze default.
    let door_style = super::overrides::resolve_door_style(cell_entity, config.door_style);
    let rows = grid.len();
    let cols = grid[r].len();
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    // The leaf-anchor Y for this level; every edge centre / pivot below derives
    // from it, and the leaf-motion systems offset from the captured base
    // translation, so the whole leaf stays on its stacked floor. Level 0 is the
    // identity.
    let base_y = placement.world_y(0.0);
    let kind = wall_kind_for_cell(r, c, rows, cols, config);

    // A swinging door only reads well between two facing walls, so it's the one
    // special case: a single central leaf when the cell is a straight corridor
    // (including one capped at an end by the grid edge). Everything else seals
    // each open edge with its own leaf.
    let swing_axis = (door_style == DoorStyle::Swing)
        .then(|| corridor_axis(grid, cell_entities, config, r, c))
        .flatten();
    if let Some(axis) = swing_axis {
        let normal_z = axis == CorridorAxis::NorthSouth; // N/S corridor → normal along Z
        let closed_yaw = if normal_z { 0.0 } else { FRAC_PI_2 };
        let edge_centre = Vec3::new(x, base_y, z);
        let pivot_translation =
            edge_centre + Quat::from_rotation_y(closed_yaw) * Vec3::new(-PANEL_W / 2.0, 0.0, 0.0);
        spawn_leaf(
            commands,
            door_assets,
            decoration_assets,
            materials,
            r,
            c,
            config,
            LeafSpec {
                closed_yaw,
                pivot_translation,
                edge_centre,
                panel_mat: leaf_material(wall_assets, kind, normal_z),
                motion: DoorMotion::Swing,
                face_id: if normal_z { 0 } else { 2 },
                keyhole_both_faces: true,
            },
        );
        return;
    }

    // Per-edge leaves. The chosen style's motion applies to each, except a
    // `Swing` style degrades to `Slide` here (no walls to anchor a hinge).
    let motion = match door_style {
        DoorStyle::Swing | DoorStyle::Slide => DoorMotion::Slide,
        DoorStyle::Portcullis => DoorMotion::Portcullis,
        DoorStyle::Dissolve => DoorMotion::Dissolve,
    };
    // A leaf seals each open edge. An edge is "open" if its neighbour is passable
    // OR a non-occluding wall (water / lava / iron fence) — that side has no wall
    // panel, so the door must seal it (see [`open_for_door`]). Out-of-bounds and
    // solid walls are closed.
    // (open?, closed_yaw, edge centre, decoration face id, normal-along-Z?)
    let edges = [
        (r > 0 && open_for_door(grid, cell_entities, config, r - 1, c), PI, Vec3::new(x, base_y, z - HALF_CELL), 0u32, true),
        (r + 1 < rows && open_for_door(grid, cell_entities, config, r + 1, c), 0.0, Vec3::new(x, base_y, z + HALF_CELL), 1, true),
        (c + 1 < cols && open_for_door(grid, cell_entities, config, r, c + 1), FRAC_PI_2, Vec3::new(x + HALF_CELL, base_y, z), 2, false),
        (c > 0 && open_for_door(grid, cell_entities, config, r, c - 1), -FRAC_PI_2, Vec3::new(x - HALF_CELL, base_y, z), 3, false),
    ];
    for (open, closed_yaw, edge_centre, face_id, normal_z) in edges {
        if !open {
            continue;
        }
        let pivot_translation =
            edge_centre + Quat::from_rotation_y(closed_yaw) * Vec3::new(-PANEL_W / 2.0, 0.0, 0.0);
        spawn_leaf(
            commands,
            door_assets,
            decoration_assets,
            materials,
            r,
            c,
            config,
            LeafSpec {
                closed_yaw,
                pivot_translation,
                edge_centre,
                panel_mat: leaf_material(wall_assets, kind, normal_z),
                motion,
                face_id,
                keyhole_both_faces: false,
            },
        );
    }
}

/// Smoothstep easing — the same `t·t·(3 − 2t)` curve [`crate::state::Animation`]
/// uses for camera moves, so a door eases into its open pose.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Door open / close transitions are driven by the central tick system —
// `crate::tick::game_tick_system` — which calls `MazeGame::tick` once per
// `FixedUpdate` and dispatches `GameEvent::DoorOpened` to this module by
// writing the matching `DoorMarker.opened`. Centralising the tick call
// across all event-producing entities (doors, enemies, HP) avoids
// double-stepping the maze.

/// `Update`: drives each leaf from its door's [`DoorState`], dispatching on the
/// leaf's [`DoorMotion`] — swing rotates, slide / portcullis translate, dissolve
/// fades its (own) material. A locked door sits closed; an opening one animates
/// with its progress; an open (or `opened`-pinned) leaf stays fully open.
pub(crate) fn door_animation_system(
    state: Res<GameState>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut doors: Query<(&DoorMarker, &mut Transform)>,
) {
    if doors.is_empty() {
        return;
    }
    let states = state.game.doors();
    for (marker, mut transform) in &mut doors {
        let fraction = if marker.opened {
            1.0
        } else {
            match states
                .iter()
                .find(|(cell, _)| *cell == marker.cell)
                .map(|(_, phase)| *phase)
            {
                Some(DoorState::Open) => 1.0,
                Some(DoorState::Opening { progress }) => smoothstep(progress),
                _ => 0.0,
            }
        };
        *transform = match marker.motion {
            DoorMotion::Swing => {
                swing::leaf_transform(marker.base_translation, marker.closed_yaw, fraction)
            }
            DoorMotion::Slide => {
                slide::leaf_transform(marker.base_translation, marker.closed_yaw, fraction)
            }
            DoorMotion::Portcullis => {
                portcullis::leaf_transform(marker.base_translation, marker.closed_yaw, fraction)
            }
            DoorMotion::Dissolve => {
                // The leaf holds its closed pose; its materials (panel + keyhole)
                // fade instead.
                if let Some(mats) = materials.as_mut() {
                    for (handle, base) in &marker.dissolve_materials {
                        if let Some(mat) = mats.get_mut(handle) {
                            dissolve::apply(mat, *base, fraction);
                        }
                    }
                }
                Transform::from_translation(marker.base_translation)
                    .with_rotation(Quat::from_rotation_y(marker.closed_yaw))
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `corridor_axis` with no per-cell overrides — the common case in the
    /// topology tests (every `'W'` is a plain solid wall).
    fn corridor_axis_plain(grid: &[Vec<char>], r: usize, c: usize) -> Option<CorridorAxis> {
        corridor_axis(grid, &HashMap::new(), &GameConfig::default(), r, c)
    }

    /// A `cell_entities` map with a single water (non-occluding) override at `rc`.
    fn map_with_water(rc: (usize, usize)) -> HashMap<(usize, usize), Vec<CellEntity>> {
        let mut m = HashMap::new();
        m.insert(
            rc,
            vec![serde_json::from_str::<CellEntity>(r#"{"type":"W","wallType":"water"}"#).unwrap()],
        );
        m
    }

    #[test]
    fn smoothstep_endpoints_and_midpoint() {
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn smoothstep_clamps_out_of_range() {
        assert_eq!(smoothstep(-1.0), 0.0);
        assert_eq!(smoothstep(2.0), 1.0);
    }

    #[test]
    fn straight_ns_corridor_uses_single_swing() {
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'W'],
            vec!['W', 'F', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 1, 1), Some(CorridorAxis::NorthSouth));
    }

    #[test]
    fn straight_ew_corridor_uses_single_swing() {
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['S', 'D', 'F'],
            vec!['W', 'W', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 1, 1), Some(CorridorAxis::EastWest));
    }

    #[test]
    fn corner_is_not_a_straight_corridor() {
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'F'],
            vec!['W', 'W', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 1, 1), None);
    }

    #[test]
    fn t_junction_is_not_a_straight_corridor() {
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'F'],
            vec!['W', ' ', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 1, 1), None);
    }

    #[test]
    fn boundary_capped_ns_corridor_is_a_corridor() {
        // `W D W` on the top boundary row, open south into the maze. The north
        // end is capped by the grid edge (counted as a wall), but the door still
        // has its two facing lateral walls, so it is an N–S straight corridor.
        let grid = vec![
            vec!['W', 'D', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 0, 1), Some(CorridorAxis::NorthSouth));
    }

    #[test]
    fn boundary_capped_ew_corridor_is_a_corridor() {
        // Door on the left boundary column, open east, walls north & south. The
        // west end is capped by the grid edge — still an E–W straight corridor.
        let grid = vec![
            vec!['W', 'W'],
            vec!['D', ' '],
            vec!['W', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 1, 0), Some(CorridorAxis::EastWest));
    }

    #[test]
    fn boundary_corner_is_not_a_corridor() {
        // A bend at the grid corner — open south and east (adjacent, not
        // opposing) — must not count as a corridor (it would slide).
        let grid = vec![
            vec!['D', ' '],
            vec![' ', 'W'],
        ];
        assert_eq!(corridor_axis_plain(&grid, 0, 0), None);
    }

    #[test]
    fn non_occluding_lateral_wall_disqualifies_swing() {
        // `W D W` corridor open N–S. With both laterals solid it's a straight
        // corridor (swing). Turning the west lateral into a non-occluding water
        // cell removes that swing anchor (its panel is suppressed), so it's no
        // longer a corridor — it falls back to per-edge leaves.
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec!['W', 'D', 'W'],
            vec!['W', ' ', 'W'],
        ];
        let config = GameConfig::default();
        assert_eq!(
            corridor_axis_plain(&grid, 1, 1),
            Some(CorridorAxis::NorthSouth),
            "plain solid laterals → straight corridor",
        );
        let water = map_with_water((1, 0));
        assert_eq!(
            corridor_axis(&grid, &water, &config, 1, 1),
            None,
            "a non-occluding lateral has no panel to hinge against → not a corridor",
        );
    }

    #[test]
    fn swing_survives_non_occluding_opening_on_perpendicular_axis() {
        // Walls N & S are solid anchors; the corridor runs E–W with the east end
        // passable and the west end a non-occluding water cell. The two solid
        // anchors remain, so it's still a straight E–W corridor (single swing).
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', 'D', ' '],
            vec!['W', 'W', 'W'],
        ];
        let config = GameConfig::default();
        let water = map_with_water((1, 0));
        assert_eq!(
            corridor_axis(&grid, &water, &config, 1, 1),
            Some(CorridorAxis::EastWest),
            "solid N & S anchors + a perpendicular opening → still a swing corridor",
        );
    }
}
