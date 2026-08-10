//! Potion health-pickup rig — a small glowing vial: a narrow neck cap
//! atop a wider rounded body. Hovers above the cell with the same gentle
//! pulse + slow spin as the Heart rig so the two variants share an idle
//! motion vocabulary.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{icosphere, CELL_SIZE, LevelPlacement};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Bottle body sphere radius (units).
const BODY_RADIUS: f32 = 0.18;
/// Bottle neck dimensions — narrower than the body, taller than wide so
/// the silhouette reads as a vial rather than a sphere.
const NECK_RADIUS: f32 = 0.06;
const NECK_HEIGHT: f32 = 0.10;
/// Vertical offset of the neck centre above the body's centre — chosen
/// so the bottom of the neck just sits on top of the body's upper rim.
const NECK_OFFSET_Y: f32 = BODY_RADIUS + NECK_HEIGHT / 2.0 - 0.02;
/// Stopper cap dimensions — a small flat cylinder capping the neck.
const STOPPER_RADIUS: f32 = NECK_RADIUS + 0.015;
const STOPPER_HEIGHT: f32 = 0.04;
const STOPPER_OFFSET_Y: f32 = NECK_OFFSET_Y + NECK_HEIGHT / 2.0 + STOPPER_HEIGHT / 2.0;

/// Resting Y position above the floor — same as the heart so both
/// variants read at the same height.
const POTION_BASE_Y: f32 = 0.9;

/// Bottle body emissive — bright green so it reads as a healing potion.
const BODY_EMISSIVE: LinearRgba = LinearRgba::new(0.20, 1.30, 0.30, 1.0);
/// Translucent-glass alpha for the body — high enough to read clearly
/// but low enough that the player sees a hint of the liquid behind it.
const BODY_ALPHA: f32 = 0.65;
/// Neck and stopper emissive — muted cork brown.
const NECK_EMISSIVE: LinearRgba = LinearRgba::new(0.45, 0.30, 0.10, 1.0);


pub(crate) struct PotionAssets {
    body_mesh: Option<Handle<Mesh>>,
    neck_mesh: Option<Handle<Mesh>>,
    stopper_mesh: Option<Handle<Mesh>>,
    body_mat: Option<Handle<StandardMaterial>>,
    neck_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_potion_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> PotionAssets {
    let body_mesh = meshes.as_mut().map(|m| m.add(icosphere(BODY_RADIUS, 2)));
    let neck_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(NECK_RADIUS, NECK_HEIGHT)));
    let stopper_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(STOPPER_RADIUS, STOPPER_HEIGHT)));
    let body_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, BODY_ALPHA),
            emissive: BODY_EMISSIVE,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    });
    let neck_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: NECK_EMISSIVE,
            ..default()
        })
    });
    PotionAssets {
        body_mesh,
        neck_mesh,
        stopper_mesh,
        body_mat,
        neck_mat,
    }
}

/// Spawns the Potion entity hierarchy for the `'H'` cell at `(r, c)`.
/// The marker carries the cell coordinate so the tick driver can despawn
/// this entity on `GameEvent::PlayerHealed { cell, .. }`.
pub(crate) fn spawn_potion(commands: &mut Commands, assets: &PotionAssets, r: usize, c: usize, placement: LevelPlacement) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let root = commands
        .spawn((
            super::HealthMarker { cell: (r, c), level: placement.level },
            placement.tag(),
            Transform::from_xyz(x, placement.world_y(POTION_BASE_Y), z),
            Visibility::default(),
        ))
        .id();
    let (
        Some(body_mesh),
        Some(neck_mesh),
        Some(stopper_mesh),
        Some(body_mat),
        Some(neck_mat),
    ) = (
        assets.body_mesh.clone(),
        assets.neck_mesh.clone(),
        assets.stopper_mesh.clone(),
        assets.body_mat.clone(),
        assets.neck_mat.clone(),
    ) else {
        return;
    };
    commands.entity(root).with_children(|parent| {
        // Bottle body at root origin.
        parent.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_mat),
            Transform::default(),
        ));
        // Neck just above the body.
        parent.spawn((
            Mesh3d(neck_mesh),
            MeshMaterial3d(neck_mat.clone()),
            Transform::from_xyz(0.0, NECK_OFFSET_Y, 0.0),
        ));
        // Stopper cap atop the neck.
        parent.spawn((
            Mesh3d(stopper_mesh),
            MeshMaterial3d(neck_mat),
            Transform::from_xyz(0.0, STOPPER_OFFSET_Y, 0.0),
        ));
    });
}

// Idle animation (scale pulse + Y-spin) is driven uniformly across
// every `HealthMarker` by `super::health_animation_system` — see
// [`super`] module docs.
