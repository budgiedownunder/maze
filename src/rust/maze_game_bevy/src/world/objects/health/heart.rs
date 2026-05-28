//! Heart health-pickup rig — a procedural red heart (two upper sphere
//! lobes + a downward-pointing cone tip) that hovers above the cell with
//! a gentle pulse animation. Matches the red-heart icon used by the 2D
//! editor and 2D game so the visual reads consistently across surfaces.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::CELL_SIZE;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

// ---------- Tuning constants ----------

/// Sphere radius for each upper "lobe" of the heart (units).
const LOBE_RADIUS: f32 = 0.18;
/// Horizontal lobe offset from the heart's centre (units).
const LOBE_OFFSET_X: f32 = 0.13;
/// Vertical lobe offset above the heart's centre (units).
const LOBE_OFFSET_Y: f32 = 0.10;

/// Downward-pointing cone — the heart's "tip" / point.
const TIP_RADIUS: f32 = 0.26;
const TIP_HEIGHT: f32 = 0.45;
/// Vertical offset of the tip's centre below the heart's centre (units).
const TIP_OFFSET_Y: f32 = -0.20;

/// Resting Y position above the floor — keeps the pickup at roughly
/// player-shoulder height so it's clearly visible from the camera.
const HEART_BASE_Y: f32 = 0.9;

/// Heart emissive — bright red so it reads instantly as health.
const HEART_EMISSIVE: LinearRgba = LinearRgba::new(1.4, 0.15, 0.15, 1.0);


pub(crate) struct HeartAssets {
    lobe_mesh: Option<Handle<Mesh>>,
    tip_mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_heart_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> HeartAssets {
    let lobe_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(LOBE_RADIUS)));
    let tip_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cone { radius: TIP_RADIUS, height: TIP_HEIGHT }));
    let mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: HEART_EMISSIVE,
            ..default()
        })
    });
    HeartAssets {
        lobe_mesh,
        tip_mesh,
        mat,
    }
}

/// Spawns the Heart entity hierarchy for the `'H'` cell at `(r, c)`. The
/// marker carries the cell coordinate so the tick driver can despawn this
/// entity on `GameEvent::PlayerHealed { cell, .. }`.
pub(crate) fn spawn_heart(commands: &mut Commands, assets: &HeartAssets, r: usize, c: usize) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let root = commands
        .spawn((
            super::HealthMarker { cell: (r, c) },
            Transform::from_xyz(x, HEART_BASE_Y, z),
            Visibility::default(),
        ))
        .id();
    if let (Some(lobe), Some(tip), Some(mat)) = (
        assets.lobe_mesh.clone(),
        assets.tip_mesh.clone(),
        assets.mat.clone(),
    ) {
        commands.entity(root).with_children(|parent| {
            // Two upper lobes — children inherit the root's transform.
            parent.spawn((
                Mesh3d(lobe.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(-LOBE_OFFSET_X, LOBE_OFFSET_Y, 0.0),
            ));
            parent.spawn((
                Mesh3d(lobe),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(LOBE_OFFSET_X, LOBE_OFFSET_Y, 0.0),
            ));
            // Tip cone — Bevy's Cone primitive points along +Y by default,
            // so rotate it 180° around X to point downward, and shift down.
            parent.spawn((
                Mesh3d(tip),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.0, TIP_OFFSET_Y, 0.0)
                    .with_rotation(Quat::from_rotation_x(2.0 * FRAC_PI_2)),
            ));
        });
    }
}

// Idle animation (scale pulse + Y-spin) is driven uniformly across
// every `HealthMarker` by `super::health_animation_system` — see
// [`super`] module docs.
