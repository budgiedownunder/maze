//! Key-holder objects for `'K'` cells.
//!
//! Each uncollected key (`'K'`) renders as a glowing key floating, bobbing, and
//! slowly spinning (the bob/spin mirrors the finish
//! [`crate::world::objects::finish::orb`]) above a base chosen by
//! [`crate::state::KeyHolderStyle`]: a stone **pedestal**, a wooden **chest**, or
//! **nothing** (the key floats alone). The holder entity carries [`KeyMarker`];
//! picking the key up (see [`crate::movement::pickup_system`]) despawns the whole
//! holder.
//!
//! The key itself is built from shared primitives — a ringed bow (a torus), a
//! shaft, and two teeth — each paired with a black inverted-hull outline so the
//! parts read distinctly.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::KeyHolderStyle;
use crate::world::CELL_SIZE;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::f32::consts::{FRAC_PI_2, TAU};

// ---------- Tuning constants ----------

const PEDESTAL_HEIGHT: f32 = 0.55;
const PEDESTAL_RADIUS: f32 = 0.30;
/// Resting Y (world height, measured from the cell floor) of the floating key.
/// Set a little below the camera eye height (~1.7) — roughly half a key-length
/// down from eye level — so the whole key sits dead-ahead in view when you enter
/// the cell, without tilting down, whether it floats alone or hovers above a
/// pedestal / chest.
const KEY_REST_Y: f32 = 1.4;
/// Uniform scale applied to the whole floating-key group so the key reads at a
/// sensible size relative to the door keyholes (it was previously oversized).
const KEY_SCALE: f32 = 0.5;
const KEY_BOB_RATE: f32 = 2.0;
const KEY_BOB_AMPLITUDE: f32 = 0.08;
const KEY_SPIN_RATE: f32 = 1.5;

/// Pedestal emissive RGB — neutral dim stone, matching the dead-end landmark
/// palette so the holder reads as carved masonry.
const PEDESTAL_EMISSIVE: LinearRgba = LinearRgba::new(0.16, 0.16, 0.18, 1.0);
/// Key emissive RGB — warm gold, bright enough to act as a small glow source.
const KEY_EMISSIVE: LinearRgba = LinearRgba::new(1.2, 0.95, 0.2, 1.0);
/// Chest body / lid emissive RGB — dark wood brown (matches the dead-end chest).
const CHEST_BODY_EMISSIVE: LinearRgba = LinearRgba::new(0.40, 0.25, 0.10, 1.0);
const CHEST_LID_EMISSIVE: LinearRgba = LinearRgba::new(0.30, 0.18, 0.07, 1.0);

/// Inverted-hull outline scale for the sub-meshes — a slightly larger black
/// shell so each part reads as distinct instead of merging into one shape,
/// matching the dead-end landmark outline trick.
const OUTLINE_SCALE: f32 = 1.06;

/// Warm point-light glow at the key, giving it an enchanted feel. No shadows —
/// it's a small accent, and several keys shouldn't each cast a shadow map.
const GLOW_COLOR: Color = Color::srgb(1.0, 0.85, 0.45);
const GLOW_INTENSITY: f32 = 30_000.0;
const GLOW_RADIUS: f32 = 0.2;

/// Radiating sparks: a few tiny emissive spheres that fly outward from the key
/// and shrink away, looping on staggered phases — a cheap "magical" shimmer (no
/// particle system). They're children of the key group, so they spin with it.
const SPARK_COUNT: usize = 6;
const SPARK_MESH_RADIUS: f32 = 0.045;
const SPARK_MAX_RADIUS: f32 = 0.6;
const SPARK_RATE: f32 = 0.6;
const SPARK_EMISSIVE: LinearRgba = LinearRgba::new(1.6, 1.35, 0.7, 1.0);

/// Marker on a key holder's root entity, keyed by grid cell. Picking the key up
/// despawns this entity (and its base / key children).
#[derive(Component)]
pub(crate) struct KeyMarker {
    pub(crate) cell: (usize, usize),
}

/// Marker on the floating key group, animated by [`key_holder_system`].
#[derive(Component)]
pub(crate) struct FloatingKey {
    base_y: f32,
}

/// One radiating spark around a floating key, animated by [`key_sparks_system`].
#[derive(Component)]
pub(crate) struct Spark {
    /// Outward unit direction the spark travels.
    dir: Vec3,
    /// Phase offset (`0.0..1.0`) so the sparks don't all pulse in lockstep.
    phase: f32,
}

pub(crate) struct KeyHolderAssets {
    pedestal_mesh: Option<Handle<Mesh>>,
    pedestal_mat: Option<Handle<StandardMaterial>>,
    /// Torus forming the key's round bow — a ring with a hole, matching the bag
    /// icon. Pre-sized, so it's spawned with only a rotation (no scale).
    bow_mesh: Option<Handle<Mesh>>,
    /// Unit cuboid scaled per-piece into the teeth and chest body / lid.
    cuboid_mesh: Option<Handle<Mesh>>,
    /// Unit cylinder for the key's shaft — a round cross-section matching the
    /// circular lock / keyhole.
    shaft_mesh: Option<Handle<Mesh>>,
    /// Tiny sphere for the radiating sparks.
    spark_mesh: Option<Handle<Mesh>>,
    key_mat: Option<Handle<StandardMaterial>>,
    chest_body_mat: Option<Handle<StandardMaterial>>,
    chest_lid_mat: Option<Handle<StandardMaterial>>,
    spark_mat: Option<Handle<StandardMaterial>>,
    /// Black inverted-hull outline material shared by the sub-meshes.
    outline_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_key_holder_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> KeyHolderAssets {
    let pedestal_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(PEDESTAL_RADIUS, PEDESTAL_HEIGHT)));
    // Torus::new(inner_hole_radius, outer_radius) — a ring with a hole through
    // it, like the bag icon's bow.
    let bow_mesh = meshes.as_mut().map(|m| m.add(Torus::new(0.085, 0.18)));
    let cuboid_mesh = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    let shaft_mesh = meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0)));
    let spark_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(SPARK_MESH_RADIUS)));
    KeyHolderAssets {
        pedestal_mesh,
        pedestal_mat: build_emissive(materials, PEDESTAL_EMISSIVE),
        bow_mesh,
        cuboid_mesh,
        shaft_mesh,
        spark_mesh,
        key_mat: build_emissive(materials, KEY_EMISSIVE),
        chest_body_mat: build_emissive(materials, CHEST_BODY_EMISSIVE),
        chest_lid_mat: build_emissive(materials, CHEST_LID_EMISSIVE),
        spark_mat: build_emissive(materials, SPARK_EMISSIVE),
        outline_mat: materials.as_mut().map(|m| {
            m.add(StandardMaterial {
                base_color: Color::BLACK,
                unlit: true,
                // Render only the back faces of the enlarged shell so just a thin
                // dark rim pokes past each silhouette (inverted-hull outline).
                cull_mode: Some(Face::Front),
                ..default()
            })
        }),
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
    style: KeyHolderStyle,
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

    match style {
        KeyHolderStyle::Pedestal => spawn_pedestal(commands, assets, holder),
        KeyHolderStyle::Chest => spawn_chest(commands, assets, holder),
        KeyHolderStyle::FloatingKey => {} // key floats alone, no base
    }
    spawn_floating_key(commands, assets, holder);
}

fn spawn_pedestal(commands: &mut Commands, assets: &KeyHolderAssets, holder: Entity) {
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
}

fn spawn_chest(commands: &mut Commands, assets: &KeyHolderAssets, holder: Entity) {
    let Some(cuboid) = assets.cuboid_mesh.clone() else {
        return;
    };
    let outline = assets.outline_mat.clone();
    // Body: a wide low box on the floor.
    if let Some(body_mat) = assets.chest_body_mat.clone() {
        spawn_outlined(
            commands,
            holder,
            cuboid.clone(),
            body_mat,
            outline.clone(),
            Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::new(0.72, 0.4, 0.5)),
        );
    }
    // Lid: a slightly wider slab sitting on top.
    if let Some(lid_mat) = assets.chest_lid_mat.clone() {
        spawn_outlined(
            commands,
            holder,
            cuboid,
            lid_mat,
            outline,
            Transform::from_xyz(0.0, 0.46, 0.0).with_scale(Vec3::new(0.76, 0.12, 0.54)),
        );
    }
}

fn spawn_floating_key(commands: &mut Commands, assets: &KeyHolderAssets, holder: Entity) {
    // The floating key group bobs and spins as one; its sub-meshes, glow, and
    // sparks are children. A uniform scale keeps the key sensibly sized relative
    // to the door keyholes.
    let key_group = commands
        .spawn((
            FloatingKey { base_y: KEY_REST_Y },
            Transform::from_xyz(0.0, KEY_REST_Y, 0.0).with_scale(Vec3::splat(KEY_SCALE)),
            Visibility::default(),
        ))
        .id();
    commands.entity(holder).add_child(key_group);

    // Enchanted glow — a warm point light at the key.
    let glow = commands
        .spawn((
            PointLight {
                color: GLOW_COLOR,
                intensity: GLOW_INTENSITY,
                radius: GLOW_RADIUS,
                shadows_enabled: false,
                ..default()
            },
            Transform::default(),
        ))
        .id();
    commands.entity(key_group).add_child(glow);

    // Radiating sparks — a ring of tiny emissive spheres flung outward and
    // shrinking on staggered phases (see `key_sparks_system`).
    if let (Some(spark_mesh), Some(spark_mat)) =
        (assets.spark_mesh.clone(), assets.spark_mat.clone())
    {
        for i in 0..SPARK_COUNT {
            let angle = i as f32 / SPARK_COUNT as f32 * TAU;
            let dir = Vec3::new(angle.cos(), (i as f32 * 1.7).sin() * 0.5, angle.sin()).normalize();
            let spark = commands
                .spawn((
                    Spark {
                        dir,
                        phase: i as f32 / SPARK_COUNT as f32,
                    },
                    Mesh3d(spark_mesh.clone()),
                    MeshMaterial3d(spark_mat.clone()),
                    Transform::default(),
                ))
                .id();
            commands.entity(key_group).add_child(spark);
        }
    }

    if let (Some(bow), Some(shaft), Some(cuboid), Some(mat)) = (
        assets.bow_mesh.clone(),
        assets.shaft_mesh.clone(),
        assets.cuboid_mesh.clone(),
        assets.key_mat.clone(),
    ) {
        let outline = assets.outline_mat.clone();
        // Bow (round head) at the top — a ring with a hole, standing upright in
        // the key's own plane (rotated 90° about X from the torus's default flat
        // pose) so the hole faces the player as the key spins.
        spawn_outlined(
            commands,
            key_group,
            bow,
            mat.clone(),
            outline.clone(),
            Transform::from_xyz(0.0, 0.27, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        );
        // Shaft running down from the bow — a cylinder so its round cross-section
        // matches the circular lock / keyhole.
        spawn_outlined(
            commands,
            key_group,
            shaft,
            mat.clone(),
            outline.clone(),
            Transform::from_xyz(0.0, -0.05, 0.0).with_scale(Vec3::new(0.08, 0.45, 0.08)),
        );
        // Two teeth jutting from the shaft's lower end.
        spawn_outlined(
            commands,
            key_group,
            cuboid.clone(),
            mat.clone(),
            outline.clone(),
            Transform::from_xyz(0.08, -0.22, 0.0).with_scale(Vec3::new(0.16, 0.06, 0.06)),
        );
        spawn_outlined(
            commands,
            key_group,
            cuboid,
            mat,
            outline,
            Transform::from_xyz(0.06, -0.30, 0.0).with_scale(Vec3::new(0.12, 0.06, 0.06)),
        );
    }
}

/// Spawns one sub-mesh as a child of `parent`, paired with a slightly larger
/// black inverted-hull outline sibling so the part reads as distinct from its
/// neighbours rather than merging into one shape.
fn spawn_outlined(
    commands: &mut Commands,
    parent: Entity,
    mesh: Handle<Mesh>,
    body_mat: Handle<StandardMaterial>,
    outline_mat: Option<Handle<StandardMaterial>>,
    transform: Transform,
) {
    let body = commands
        .spawn((Mesh3d(mesh.clone()), MeshMaterial3d(body_mat), transform))
        .id();
    commands.entity(parent).add_child(body);
    if let Some(outline) = outline_mat {
        let outline_xform = Transform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale * OUTLINE_SCALE,
        };
        let edge = commands
            .spawn((Mesh3d(mesh), MeshMaterial3d(outline), outline_xform))
            .id();
        commands.entity(parent).add_child(edge);
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

/// `Update`: animates the radiating sparks — each flies outward from the key
/// along its direction and shrinks to nothing, looping on its phase, for a cheap
/// magical shimmer.
pub(crate) fn key_sparks_system(time: Res<Time>, mut sparks: Query<(&Spark, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (spark, mut transform) in &mut sparks {
        let cycle = (t * SPARK_RATE + spark.phase).fract();
        transform.translation = spark.dir * (SPARK_MAX_RADIUS * cycle);
        transform.scale = Vec3::splat(1.0 - cycle);
    }
}
