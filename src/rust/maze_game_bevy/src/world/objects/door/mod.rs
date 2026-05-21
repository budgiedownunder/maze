//! Door panels for `'D'` cells.
//!
//! A door is a primitive-rig slab that fills the corridor opening at a `'D'`
//! cell. It carries no maze-blocking geometry of its own — passability is
//! decided entirely by [`maze::MazeGame`] (the slab is purely a *view* of the
//! door's [`DoorState`]). The panel renders in the **same wall material as the
//! cell it sits between** (see [`crate::world::walls::wall_kind_for_cell`]), so a
//! closed door reads as part of the architecture; a keyhole marks it as a door,
//! and a door is eligible for the same sparse wall decoration as any wall panel.
//!
//! This module is the orchestrator: it owns the door's runtime marker, asset
//! bundle, per-cell spawn, and the tick / animation systems. The rendered
//! pieces live in sibling files — the swinging slab in [`panel`], the keyhole in
//! [`keyhole`] — mirroring the per-object split under
//! [`crate::world::objects::dead_end`].
//!
//! Step 5A ships a single rig — a slab on a vertical hinge that swings open. The
//! hinge angle is driven each frame from the door's [`DoorState`]
//! (`door_animation_system`); the underlying state advances deterministically in
//! the `FixedUpdate` tick (`door_tick_system`). Once a door has finished opening
//! it is pinned to its fully-open pose permanently — the entity is never
//! despawned.

pub(crate) mod keyhole;
pub(crate) mod panel;

use crate::state::{GameConfig, GameState};
use crate::world::decorations::wall::{
    wall_decoration_index, WallDecoration, WallDecorationAssets, DECORATION_OFFSET, DECORATION_Y,
};
use crate::world::walls::{wall_kind_for_cell, WallAssets, PANEL_W};
use crate::world::CELL_SIZE;
use bevy::prelude::*;
use maze::{DoorState, GameEvent};
use panel::DOOR_THICKNESS;
use std::f32::consts::FRAC_PI_2;

/// How far the door swings around its hinge when fully open (radians). Slightly
/// past 90° so the open slab tucks fully out of the corridor cross-section.
const SWING_OPEN_ANGLE: f32 = std::f32::consts::PI * 100.0 / 180.0;

/// Marker on a door's hinge-pivot entity. The panel, keyhole, and any
/// decoration are children of this entity, so rotating it swings the whole
/// door.
#[derive(Component)]
pub(crate) struct DoorMarker {
    /// Grid cell this door occupies.
    pub(crate) cell: (usize, usize),
    /// Base yaw encoding the doorway orientation (`0` for a door across an
    /// N/S corridor, `π/2` for an E/W corridor). The swing animation adds onto
    /// this.
    closed_yaw: f32,
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

/// `true` when the door blocks travel along the N/S axis — i.e. the cell has an
/// open neighbour to the north or south. Such a door spans the cell's X extent
/// (its face normal points along Z); otherwise it spans Z (normal along X).
fn door_is_ns_passage(grid: &[Vec<char>], r: usize, c: usize) -> bool {
    let rows = grid.len();
    let north = r > 0 && grid[r - 1][c] != 'W';
    let south = r + 1 < rows && grid[r + 1][c] != 'W';
    north || south
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

    // Orientation: a door across an N/S corridor spans X (face normal along Z),
    // so it borrows the N/S wall material; one across an E/W corridor spans Z
    // and borrows the E/W material. Either way the kind matches the cell's
    // surrounding walls.
    let ns = door_is_ns_passage(grid, r, c);
    let closed_yaw = if ns { 0.0 } else { FRAC_PI_2 };
    let kind = wall_kind_for_cell(r, c, rows, cols, config);
    let panel_mat = if ns {
        wall_assets.ns.material_mats[kind].clone()
    } else {
        wall_assets.ew.material_mats[kind].clone()
    };

    // Hinge pivot at one edge of the doorway (a side wall of the corridor). The
    // panel hangs from the pivot's local +X, spanning the opening when closed.
    let hinge_local = Vec3::new(-PANEL_W / 2.0, 0.0, 0.0);
    let pivot_translation = Vec3::new(x, 0.0, z) + Quat::from_rotation_y(closed_yaw) * hinge_local;
    let pivot = commands
        .spawn((
            DoorMarker {
                cell: (r, c),
                closed_yaw,
                opened: false,
            },
            Transform::from_translation(pivot_translation)
                .with_rotation(Quat::from_rotation_y(closed_yaw)),
            Visibility::default(),
        ))
        .id();

    panel::spawn_panel(commands, &door_assets.panel, panel_mat, pivot);

    // Obvious keyhole on both faces so the panel reads as a (locked) door.
    keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, 1.0);
    keyhole::spawn_keyhole_face(commands, &door_assets.keyhole, pivot, -1.0);

    // A door is eligible for the same sparse, seeded wall decoration as any wall
    // panel (vent / poster / rune / glass), projected on its visible face and
    // parented to the pivot so it swings with the door. The face id differs by
    // orientation so N/S and E/W doors don't hash identically.
    if config.landmarks.wall_decorations {
        let face_id = if ns { 0 } else { 2 };
        if let Some(decoration_kind) = wall_decoration_index(r, c, face_id, config.seed) {
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

/// Smoothstep easing — the same `t·t·(3 − 2t)` curve [`crate::state::Animation`]
/// uses for camera moves, so the door swing decelerates into its open pose.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `FixedUpdate`: advances the deterministic door countdown via
/// [`maze::MazeGame::tick`] and applies the resulting events. A
/// [`GameEvent::DoorOpened`] pins the matching door to its open pose
/// permanently (the `opened` flag), so it never re-reads its state or re-locks.
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

/// `Update`: drives each door's hinge rotation from its [`DoorState`]. A locked
/// door sits closed; an opening door swings smoothly with its progress; an open
/// (or `opened`-pinned) door stays fully open.
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
        transform.rotation = Quat::from_rotation_y(marker.closed_yaw + fraction * SWING_OPEN_ANGLE);
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
    fn ns_corridor_door_spans_x() {
        // Vertical corridor: north & south of (1,1) are open, east & west walls.
        let grid = vec![
            vec!['W', 'S', 'W'],
            vec!['W', 'D', 'W'],
            vec!['W', 'F', 'W'],
        ];
        assert!(door_is_ns_passage(&grid, 1, 1));
    }

    #[test]
    fn ew_corridor_door_spans_z() {
        // Horizontal corridor: east & west of (1,1) are open, north & south walls.
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['S', 'D', 'F'],
            vec!['W', 'W', 'W'],
        ];
        assert!(!door_is_ns_passage(&grid, 1, 1));
    }
}
