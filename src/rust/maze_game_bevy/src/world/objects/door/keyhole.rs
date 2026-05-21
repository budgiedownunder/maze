//! The door keyhole — a brass lock plate with a dark cone-plus-disc cutout,
//! built like the chest's keyhole ([`crate::world::objects::dead_end::chest`])
//! so the panel obviously reads as a (locked) door even though it borrows the
//! surrounding wall's material. Spawned on both faces so it's visible from
//! either approach.

use super::panel::DOOR_THICKNESS;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::walls::{PANEL_W, PANEL_Y};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

/// Brass lock-plate emissive — a warm metallic so the keyhole stands out
/// against the door panel whatever wall material it borrows.
const LOCK_PLATE_EMISSIVE: LinearRgba = LinearRgba::new(0.55, 0.42, 0.16, 1.0);
/// Keyhole emissive — pure black, paired with the BLACK base colour so the
/// cone + disc render as a flat dark cutout regardless of corridor lighting.
const KEYHOLE_EMISSIVE: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 1.0);

/// Keyhole position along the door span (`0` = hinge, `PANEL_W` = free edge) —
/// near the latch edge, like a real lock.
const KEYHOLE_X: f32 = PANEL_W * 0.82;
/// Keyhole centre height (waist height, below the panel centre).
const KEYHOLE_Y: f32 = PANEL_Y - 0.25;
/// Lock-plate scale — a slim rectangle on the door face.
const LOCK_PLATE_SCALE: Vec3 = Vec3::new(0.22, 0.42, 0.03);
/// Keyhole cone scale — flattened in the face normal so it reads as a flat
/// triangle widening downward.
const KEYHOLE_CONE_SCALE: Vec3 = Vec3::new(0.10, 0.10, 0.01);
/// Keyhole disc scale — the flat round top of the keyhole.
const KEYHOLE_DISC_SCALE: Vec3 = Vec3::new(0.08, 0.005, 0.08);
/// Outward offsets (from the panel centre plane) for the plate / cone / disc,
/// each just proud of the previous to avoid z-fighting.
const LOCK_PLATE_Z: f32 = DOOR_THICKNESS / 2.0 + 0.015;
const KEYHOLE_CONE_Z: f32 = LOCK_PLATE_Z + 0.02;
const KEYHOLE_DISC_Z: f32 = KEYHOLE_CONE_Z + 0.012;
/// The disc sits above the cone centre so the round hole tops the widening slot.
const KEYHOLE_DISC_DY: f32 = 0.05;

pub(crate) struct KeyholeAssets {
    /// Unit cuboid, scaled per-instance into the lock plate.
    cuboid_mesh: Option<Handle<Mesh>>,
    /// Unit cone for the keyhole's downward-widening slot.
    cone_mesh: Option<Handle<Mesh>>,
    /// Unit cylinder for the keyhole's round top (flattened into a disc).
    disc_mesh: Option<Handle<Mesh>>,
    plate_mat: Option<Handle<StandardMaterial>>,
    keyhole_mat: Option<Handle<StandardMaterial>>,
}

fn build_emissive(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive,
            ..default()
        })
    })
}

pub(crate) fn build_keyhole_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> KeyholeAssets {
    KeyholeAssets {
        cuboid_mesh: meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0))),
        cone_mesh: meshes.as_mut().map(|m| m.add(Cone::new(0.5, 1.0))),
        disc_mesh: meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0))),
        plate_mat: build_emissive(materials, LOCK_PLATE_EMISSIVE),
        keyhole_mat: build_emissive(materials, KEYHOLE_EMISSIVE),
    }
}

impl KeyholeAssets {
    /// The shared lock-plate material handle (for cloning into a per-leaf
    /// alpha-blended copy on dissolve doors).
    pub(crate) fn plate_handle(&self) -> Option<Handle<StandardMaterial>> {
        self.plate_mat.clone()
    }

    /// The shared keyhole (cone + disc) material handle.
    pub(crate) fn keyhole_handle(&self) -> Option<Handle<StandardMaterial>> {
        self.keyhole_mat.clone()
    }
}

/// Spawns a lock plate + keyhole on one face of the door using the keyhole's
/// shared materials. `sign` selects the face: `+1.0` for local `+Z`, `-1.0` for
/// `-Z`.
pub(crate) fn spawn_keyhole_face(
    commands: &mut Commands,
    assets: &KeyholeAssets,
    pivot: Entity,
    sign: f32,
) {
    spawn_keyhole_face_with(
        commands,
        assets,
        pivot,
        sign,
        assets.plate_mat.clone(),
        assets.keyhole_mat.clone(),
    );
}

/// As [`spawn_keyhole_face`] but with caller-supplied plate / keyhole materials,
/// so a dissolve leaf can pass its own cloned, alpha-blended copies that fade
/// with the panel. The plate + keyhole (cone + disc) become children of `pivot`.
pub(crate) fn spawn_keyhole_face_with(
    commands: &mut Commands,
    assets: &KeyholeAssets,
    pivot: Entity,
    sign: f32,
    plate_mat: Option<Handle<StandardMaterial>>,
    keyhole_mat: Option<Handle<StandardMaterial>>,
) {
    if let (Some(mesh), Some(mat)) = (assets.cuboid_mesh.clone(), plate_mat) {
        let plate = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(KEYHOLE_X, KEYHOLE_Y, sign * LOCK_PLATE_Z)
                    .with_scale(LOCK_PLATE_SCALE),
            ))
            .id();
        commands.entity(pivot).add_child(plate);
    }
    if let (Some(mesh), Some(mat)) = (assets.cone_mesh.clone(), keyhole_mat.clone()) {
        let cone = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(KEYHOLE_X, KEYHOLE_Y, sign * KEYHOLE_CONE_Z)
                    .with_scale(KEYHOLE_CONE_SCALE),
            ))
            .id();
        commands.entity(pivot).add_child(cone);
    }
    if let (Some(mesh), Some(mat)) = (assets.disc_mesh.clone(), keyhole_mat) {
        // Cylinder rotated 90° about X so the disc faces along the door normal.
        let disc = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(KEYHOLE_X, KEYHOLE_Y + KEYHOLE_DISC_DY, sign * KEYHOLE_DISC_Z)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(KEYHOLE_DISC_SCALE),
            ))
            .id();
        commands.entity(pivot).add_child(disc);
    }
}
