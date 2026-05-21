//! Key-holder objects for `'K'` cells.
//!
//! Each uncollected key (`'K'`) renders as a holder: a short stone pedestal with
//! a glowing key floating, bobbing, and slowly spinning above it (the bob/spin
//! mirrors the finish [`crate::world::objects::finish::orb`]). The holder entity
//! carries [`KeyMarker`]; picking the key up (see
//! [`crate::movement::pickup_system`]) despawns the whole holder.
//!
//! Step 5A ships a single holder style (pedestal). The key itself is built from
//! shared primitives — a round bow, a shaft, and two teeth — so it reads as a
//! key without an asset file.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const PEDESTAL_HEIGHT: f32 = 0.55;
const PEDESTAL_RADIUS: f32 = 0.30;
/// Resting Y of the floating key, comfortably above the pedestal top.
const KEY_REST_Y: f32 = 1.15;
const KEY_BOB_RATE: f32 = 2.0;
const KEY_BOB_AMPLITUDE: f32 = 0.08;
const KEY_SPIN_RATE: f32 = 1.5;

/// Pedestal emissive RGB — neutral dim stone, matching the dead-end landmark
/// palette so the holder reads as carved masonry.
const PEDESTAL_EMISSIVE: LinearRgba = LinearRgba::new(0.16, 0.16, 0.18, 1.0);
/// Key emissive RGB — warm gold, bright enough to act as a small glow source.
const KEY_EMISSIVE: LinearRgba = LinearRgba::new(1.2, 0.95, 0.2, 1.0);

/// Marker on a key holder's root entity, keyed by grid cell. Picking the key up
/// despawns this entity (and its pedestal / key children).
#[derive(Component)]
pub(crate) struct KeyMarker {
    pub(crate) cell: (usize, usize),
}

/// Marker on the floating key group, animated by [`key_holder_system`].
#[derive(Component)]
pub(crate) struct FloatingKey {
    base_y: f32,
}

pub(crate) struct KeyHolderAssets {
    pedestal_mesh: Option<Handle<Mesh>>,
    pedestal_mat: Option<Handle<StandardMaterial>>,
    /// Flattened cylinder used for the key's round bow.
    bow_mesh: Option<Handle<Mesh>>,
    /// Unit cuboid scaled per-piece into the shaft and teeth.
    cuboid_mesh: Option<Handle<Mesh>>,
    key_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_key_holder_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> KeyHolderAssets {
    let pedestal_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(PEDESTAL_RADIUS, PEDESTAL_HEIGHT)));
    let bow_mesh = meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0)));
    let cuboid_mesh = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    let pedestal_mat = build_emissive(materials, PEDESTAL_EMISSIVE);
    let key_mat = build_emissive(materials, KEY_EMISSIVE);
    KeyHolderAssets {
        pedestal_mesh,
        pedestal_mat,
        bow_mesh,
        cuboid_mesh,
        key_mat,
    }
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

pub(crate) fn spawn_key_holder_for_cell(
    commands: &mut Commands,
    assets: &KeyHolderAssets,
    cell: char,
    r: usize,
    c: usize,
) {
    if cell != 'K' {
        return;
    }
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;

    let holder = commands
        .spawn((
            KeyMarker { cell: (r, c) },
            Transform::from_xyz(x, 0.0, z),
            Visibility::default(),
        ))
        .id();

    if let (Some(mesh), Some(mat)) = (assets.pedestal_mesh.clone(), assets.pedestal_mat.clone()) {
        let pedestal = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.0, PEDESTAL_HEIGHT / 2.0, 0.0),
            ))
            .id();
        commands.entity(holder).add_child(pedestal);
    }

    // The floating key group bobs and spins as one; its sub-meshes are children.
    let key_group = commands
        .spawn((
            FloatingKey { base_y: KEY_REST_Y },
            Transform::from_xyz(0.0, KEY_REST_Y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(holder).add_child(key_group);

    if let (Some(bow), Some(cuboid), Some(mat)) = (
        assets.bow_mesh.clone(),
        assets.cuboid_mesh.clone(),
        assets.key_mat.clone(),
    ) {
        // Bow (round head) at the top — flattened disc lying in the X/Z plane.
        let bow_entity = commands
            .spawn((
                Mesh3d(bow.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(0.34, 0.06, 0.34)),
            ))
            .id();
        // Shaft running down from the bow.
        let shaft = commands
            .spawn((
                Mesh3d(cuboid.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.0, -0.05, 0.0).with_scale(Vec3::new(0.08, 0.45, 0.08)),
            ))
            .id();
        // Two teeth jutting from the shaft's lower end.
        let tooth_a = commands
            .spawn((
                Mesh3d(cuboid.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.08, -0.22, 0.0).with_scale(Vec3::new(0.16, 0.06, 0.06)),
            ))
            .id();
        let tooth_b = commands
            .spawn((
                Mesh3d(cuboid),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.06, -0.30, 0.0).with_scale(Vec3::new(0.12, 0.06, 0.06)),
            ))
            .id();
        commands
            .entity(key_group)
            .add_children(&[bow_entity, shaft, tooth_a, tooth_b]);
    }
}

/// `Update`: bobs and spins every floating key. Mirrors the finish orb's
/// bob/rotate so collectibles share a recognisable "pick me up" idle.
pub(crate) fn key_holder_system(time: Res<Time>, mut keys: Query<(&FloatingKey, &mut Transform)>) {
    for (key, mut transform) in &mut keys {
        transform.translation.y =
            key.base_y + KEY_BOB_AMPLITUDE * (time.elapsed_secs() * KEY_BOB_RATE).sin();
        transform.rotate_y(time.delta_secs() * KEY_SPIN_RATE);
    }
}
