//! The portcullis door rig — a grille that rises straight up out of the
//! opening, set in a wall-material frame (two posts + a lintel beam) so the
//! raised grille reads as a proper gate rather than a slab floating into the
//! (ceiling-less) sky. Like the other sliding rigs it works at any topology.
//! The orchestrator in [`super`] decides when this rig applies and spawns the
//! frame; this module owns the rise motion and the frame geometry.

use crate::world::walls::{PANEL_H, PANEL_W};
use bevy::prelude::*;

/// How far the grille rises when fully open — just past its own height so the
/// bottom edge clears the opening.
pub(crate) const DISTANCE: f32 = PANEL_H * 1.02;

/// Post / lintel depth (units).
const FRAME_THICKNESS: f32 = 0.14;
/// Lintel beam vertical height (units).
const LINTEL_HEIGHT: f32 = 0.22;

/// The grille transform at `fraction` (`0.0` closed … `1.0` open): the grille
/// holds its yaw and rises straight up by up to [`DISTANCE`].
pub(crate) fn leaf_transform(base_translation: Vec3, closed_yaw: f32, fraction: f32) -> Transform {
    Transform::from_translation(base_translation + Vec3::Y * (fraction * DISTANCE))
        .with_rotation(Quat::from_rotation_y(closed_yaw))
}

/// Spawns the static frame — two posts and a lintel beam — around the opening
/// centred at `edge_centre` and oriented by `closed_yaw`, in the cell's wall
/// `material`. The frame does not move; the grille rises behind it.
pub(crate) fn spawn_frame(
    commands: &mut Commands,
    cuboid: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    edge_centre: Vec3,
    closed_yaw: f32,
) {
    let (Some(mesh), Some(mat)) = (cuboid, material) else {
        return;
    };
    let rot = Quat::from_rotation_y(closed_yaw);
    // Local frame: the opening spans X (±PANEL_W/2); posts stand at each end,
    // the lintel bridges the top. Each piece is placed in local space then
    // rotated about the edge centre by `closed_yaw`.
    let piece = |lx: f32, ly: f32, scale: Vec3| {
        Transform::from_translation(edge_centre + rot * Vec3::new(lx, ly, 0.0))
            .with_rotation(rot)
            .with_scale(scale)
    };
    let half = PANEL_W / 2.0;
    let post_scale = Vec3::new(FRAME_THICKNESS, PANEL_H, FRAME_THICKNESS);
    for sign in [-1.0_f32, 1.0] {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            piece(sign * half, PANEL_H / 2.0, post_scale),
        ));
    }
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        piece(
            0.0,
            PANEL_H + LINTEL_HEIGHT / 2.0,
            Vec3::new(PANEL_W + FRAME_THICKNESS * 2.0, LINTEL_HEIGHT, FRAME_THICKNESS),
        ),
    ));
}
