//! Door panels for `'D'` cells.
//!
//! A door is a primitive-rig leaf (or leaves) that fills the corridor opening at
//! a `'D'` cell. It carries no maze-blocking geometry of its own — passability
//! is decided entirely by [`maze::MazeGame`], which gates *entry into the cell
//! from every side* until the door opens (the leaves are purely a *view* of the
//! door's [`DoorState`]). Leaves render in the **same wall material as the cell
//! they sit between** (see [`crate::world::walls::wall_kind_for_cell`]); a
//! keyhole marks them as a door, and each is eligible for the same sparse wall
//! decoration as a wall panel.
//!
//! Two rigs, picked from the cell's topology:
//! - **Straight corridor** (exactly two open edges on opposing sides): a single
//!   swinging leaf, centred in the cell, hinged against a side wall — the
//!   familiar door swing, which only reads well when anchored between two facing
//!   walls.
//! - **Anything else** (corner, T-junction, open area, dead-end stub): a leaf on
//!   each open edge that **slides down into the floor**. A swing would sweep
//!   awkwardly through the open space beside the opening; a sliding leaf needs no
//!   side anchor and seals the edge cleanly.
//!
//! The hinge angle / slide offset is driven each frame from the door's
//! [`DoorState`] (`door_animation_system`); the underlying state advances
//! deterministically in the `FixedUpdate` tick (`door_tick_system`). Once a door
//! has finished opening it is pinned to its fully-open pose permanently — leaves
//! are never despawned. The pieces live in sibling files — the slab in
//! [`panel`], the keyhole in [`keyhole`], and the two rigs' open motions in
//! [`swing`] and [`slide`].

pub(crate) mod keyhole;
pub(crate) mod panel;
pub(crate) mod slide;
pub(crate) mod swing;

use crate::state::{GameConfig, GameState};
use crate::world::decorations::wall::{
    wall_decoration_index, WallDecoration, WallDecorationAssets, DECORATION_OFFSET, DECORATION_Y,
};
use crate::world::walls::{wall_kind_for_cell, WallAssets, PANEL_W};
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use maze::{DoorState, GameEvent};
use panel::DOOR_THICKNESS;
use std::f32::consts::{FRAC_PI_2, PI};

/// Marker on a door leaf's pivot entity. The panel, keyhole, and any decoration
/// are children of this entity, so transforming it moves the whole leaf. A door
/// cell may own several leaves (one per open edge); they share the cell's
/// [`DoorState`].
#[derive(Component)]
pub(crate) struct DoorMarker {
    /// Grid cell this leaf belongs to.
    pub(crate) cell: (usize, usize),
    /// Yaw orienting the leaf so its local +X spans the opening and its local +Z
    /// (keyhole / decoration face) points out toward the neighbour.
    closed_yaw: f32,
    /// `true` if this leaf retracts by sliding into the floor; `false` if it
    /// swings on a hinge.
    slides: bool,
    /// The leaf's resting (closed) pivot translation, captured at spawn so the
    /// slide animation can offset from it (and the swing animation can hold it).
    base_translation: Vec3,
    /// Set once a [`GameEvent::DoorOpened`] has been applied — the door is then
    /// pinned to its fully-open pose permanently and never re-reads its state.
    opened: bool,
}

pub(crate) struct DoorAssets {
    panel: panel::PanelAssets,
    keyhole: keyhole::KeyholeAssets,
}

pub(crate) fn build_door_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> DoorAssets {
    DoorAssets {
        panel: panel::build_panel_assets(meshes),
        keyhole: keyhole::build_keyhole_assets(meshes, materials),
    }
}

/// `true` when the door cell is a straight corridor — exactly two open edges on
/// OPPOSING sides (N+S or E+W). Such a door renders as a single swinging leaf;
/// every other topology uses per-edge sliding leaves. Out-of-bounds counts as a
/// wall.
fn is_straight_corridor(grid: &[Vec<char>], r: usize, c: usize) -> bool {
    let rows = grid.len();
    let cols = grid[r].len();
    let n = r > 0 && grid[r - 1][c] != 'W';
    let s = r + 1 < rows && grid[r + 1][c] != 'W';
    let e = c + 1 < cols && grid[r][c + 1] != 'W';
    let w = c > 0 && grid[r][c - 1] != 'W';
    (n && s && !e && !w) || (e && w && !n && !s)
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
    panel_mat: Option<Handle<StandardMaterial>>,
    slides: bool,
    /// Decoration hash face id (compass id of the sealed edge, or the corridor
    /// axis for the single swing leaf).
    face_id: u32,
    /// Swing leaves are seen from both ends of the corridor, so they get a
    /// keyhole on both faces; sliding leaves only on the outward face.
    keyhole_both_faces: bool,
}

fn spawn_leaf(
    commands: &mut Commands,
    door_assets: &DoorAssets,
    decoration_assets: &WallDecorationAssets,
    r: usize,
    c: usize,
    config: &GameConfig,
    spec: LeafSpec,
) {
    let pivot = commands
        .spawn((
            DoorMarker {
                cell: (r, c),
                closed_yaw: spec.closed_yaw,
                slides: spec.slides,
                base_translation: spec.pivot_translation,
                opened: false,
            },
            Transform::from_translation(spec.pivot_translation)
                .with_rotation(Quat::from_rotation_y(spec.closed_yaw)),
            Visibility::default(),
        ))
        .id();

    panel::spawn_panel(commands, &door_assets.panel, spec.panel_mat, pivot);
    // Keyhole on the outward (+Z) face — the side the approaching player sees.
    keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, 1.0);
    if spec.keyhole_both_faces {
        keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, -1.0);
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
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    if cell != 'D' {
        return;
    }
    let rows = grid.len();
    let cols = grid[r].len();
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let kind = wall_kind_for_cell(r, c, rows, cols, config);

    if is_straight_corridor(grid, r, c) {
        // Single swinging leaf, centred in the cell and spanning the corridor.
        // The corridor's side walls anchor the hinge so the leaf swings flush.
        let normal_z = r > 0 && grid[r - 1][c] != 'W'; // N/S corridor → normal along Z
        let closed_yaw = if normal_z { 0.0 } else { FRAC_PI_2 };
        let pivot_translation = Vec3::new(x, 0.0, z)
            + Quat::from_rotation_y(closed_yaw) * Vec3::new(-PANEL_W / 2.0, 0.0, 0.0);
        spawn_leaf(
            commands,
            door_assets,
            decoration_assets,
            r,
            c,
            config,
            LeafSpec {
                closed_yaw,
                pivot_translation,
                panel_mat: leaf_material(wall_assets, kind, normal_z),
                slides: false,
                face_id: if normal_z { 0 } else { 2 },
                keyhole_both_faces: true,
            },
        );
        return;
    }

    // Not a clean opposing-wall corridor: a swing would sweep through the open
    // space beside the opening. Seal each open edge with a leaf that slides down
    // into the floor, positioned on that edge.
    //
    // (open?, closed_yaw, edge centre, decoration face id, normal-along-Z?)
    let edges = [
        (r > 0 && grid[r - 1][c] != 'W', PI, Vec3::new(x, 0.0, z - HALF_CELL), 0u32, true),
        (r + 1 < rows && grid[r + 1][c] != 'W', 0.0, Vec3::new(x, 0.0, z + HALF_CELL), 1, true),
        (c + 1 < cols && grid[r][c + 1] != 'W', FRAC_PI_2, Vec3::new(x + HALF_CELL, 0.0, z), 2, false),
        (c > 0 && grid[r][c - 1] != 'W', -FRAC_PI_2, Vec3::new(x - HALF_CELL, 0.0, z), 3, false),
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
            r,
            c,
            config,
            LeafSpec {
                closed_yaw,
                pivot_translation,
                panel_mat: leaf_material(wall_assets, kind, normal_z),
                slides: true,
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

/// `FixedUpdate`: advances the deterministic door countdown via
/// [`maze::MazeGame::tick`] and applies the resulting events. A
/// [`GameEvent::DoorOpened`] pins every leaf of the matching door to its open
/// pose permanently (the `opened` flag), so it never re-reads its state or
/// re-locks.
pub(crate) fn door_tick_system(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut markers: Query<&mut DoorMarker>,
) {
    if state.paused || state.won || state.lost {
        return;
    }
    let dt_ms = time.delta_secs() * 1000.0;
    for event in state.game.tick(dt_ms) {
        let GameEvent::DoorOpened { cell } = event;
        for mut marker in &mut markers {
            if marker.cell == cell {
                marker.opened = true;
            }
        }
    }
}

/// `Update`: drives each leaf from its door's [`DoorState`] — a locked door sits
/// closed; an opening door swings/slides smoothly with its progress; an open (or
/// `opened`-pinned) leaf stays fully retracted.
pub(crate) fn door_animation_system(
    state: Res<GameState>,
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
        *transform = if marker.slides {
            slide::leaf_transform(marker.base_translation, marker.closed_yaw, fraction)
        } else {
            swing::leaf_transform(marker.base_translation, marker.closed_yaw, fraction)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(is_straight_corridor(&grid, 1, 1));
    }

    #[test]
    fn straight_ew_corridor_uses_single_swing() {
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['S', 'D', 'F'],
            vec!['W', 'W', 'W'],
        ];
        assert!(is_straight_corridor(&grid, 1, 1));
    }

    #[test]
    fn corner_is_not_a_straight_corridor() {
        // (1,1) has north (S) and east (F) open — adjacent, not opposing.
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'F'],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_straight_corridor(&grid, 1, 1));
    }

    #[test]
    fn t_junction_is_not_a_straight_corridor() {
        // (1,1) has north (S), south (open) and east (F) open — three edges.
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'F'],
            vec!['W', ' ', 'W'],
        ];
        assert!(!is_straight_corridor(&grid, 1, 1));
    }
}
