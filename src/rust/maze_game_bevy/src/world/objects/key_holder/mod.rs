//! Key-holder objects for `'K'` cells.
//!
//! Each uncollected key (`'K'`) renders as a glowing key floating, bobbing, and
//! slowly spinning (the bob/spin mirrors the finish
//! [`crate::world::objects::finish::orb`]) above a base chosen by
//! [`crate::state::KeyHolderStyle`]: a broken **pillar**, a wooden **chest**, or
//! **nothing** (the key floats alone). The base rigs are the shared
//! [`crate::world::objects::common`] props (the same ones the dead-end landmarks
//! use); the key floats a fixed clearance above each rig's `TOP_Y`.
//!
//! The base prop is spawned free-standing — only the floating key sits under the
//! [`KeyMarker`] holder. Walking onto the key auto-collects it:
//! [`crate::tick::game_tick_system`] tags the holder with [`CollectingKey`] on
//! the `KeyCollected` event, and [`key_collection_system`] plays a brief
//! rise-and-shrink flourish before despawning it — leaving the base prop behind
//! as an emptied holder.
//!
//! The key itself is built from shared primitives — a ringed bow (a torus), a
//! shaft, and two teeth — each paired with a black inverted-hull outline so the
//! parts read distinctly.

use super::common::{self, CommonObjectAssets};
use crate::state::KeyHolderStyle;
use crate::world::{world_y, CELL_SIZE};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};

// ---------- Tuning constants ----------

/// Eye-level resting Y (world height, from the cell floor) of the floating key
/// when it has no base, or a base short enough not to push it higher. A little
/// below the camera eye height (~1.7) so the whole key sits dead-ahead in view
/// when you enter the cell, without tilting down.
const KEY_REST_Y: f32 = 1.4;
/// Vertical clearance kept between a base prop's apex and the floating key, so
/// the key reads as hovering above (rather than embedded in) a tall base.
const KEY_CLEARANCE: f32 = 0.45;
/// Uniform scale applied to the whole floating-key group so the key reads at a
/// sensible size relative to the door keyholes.
const KEY_SCALE: f32 = 0.5;
const KEY_BOB_RATE: f32 = 2.0;
const KEY_BOB_AMPLITUDE: f32 = 0.08;
const KEY_SPIN_RATE: f32 = 1.5;

/// Duration of the key-collection flourish, in seconds.
const KEY_COLLECT_DURATION: f32 = 0.35;
/// How far (world units) the holder rises over the collection flourish as it
/// shrinks away — reads as the key being whisked up to the player.
const KEY_COLLECT_RISE: f32 = 1.2;
/// Spin rate (rad/s) during collection — faster than the idle spin for a
/// "snatched away" feel.
const KEY_COLLECT_SPIN_RATE: f32 = 12.0;

/// Key emissive RGB — warm gold, bright enough to act as a small glow source.
const KEY_EMISSIVE: LinearRgba = LinearRgba::new(1.2, 0.95, 0.2, 1.0);

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

/// Marker on a key holder's root entity, keyed by grid cell. The root owns the
/// floating key (its child) but NOT the base prop, which stands free. Collecting
/// the key tags this entity with [`CollectingKey`], which despawns it (and the
/// key) once the collection flourish finishes, leaving the base prop behind.
#[derive(Component)]
pub(crate) struct KeyMarker {
    pub(crate) cell: (usize, usize),
    /// Run level this holder sits on. The holder root rests at this level's
    /// floor and [`key_collection_system`] rewrites its absolute Y during the
    /// rise flourish, so it must re-apply the level offset. (The floating key is
    /// a child in the holder's local frame, so its idle bob needs no offset.)
    pub(crate) level: usize,
}

/// Tags a key holder whose key was just auto-collected. [`key_collection_system`]
/// rises and shrinks the holder over [`KEY_COLLECT_DURATION`], then despawns it.
#[derive(Component, Default)]
pub(crate) struct CollectingKey {
    elapsed: f32,
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

/// Key-specific assets. The shaft / teeth / outline reuse
/// [`CommonObjectAssets`]; only the round bow and the spark sphere (plus the
/// two gold materials) are unique to the key.
pub(crate) struct KeyHolderAssets {
    /// Torus forming the key's round bow — a ring with a hole, matching the bag
    /// icon. Pre-sized, so it's spawned with only a rotation (no scale).
    bow_mesh: Option<Handle<Mesh>>,
    /// Tiny sphere for the radiating sparks.
    spark_mesh: Option<Handle<Mesh>>,
    key_mat: Option<Handle<StandardMaterial>>,
    spark_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_key_holder_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> KeyHolderAssets {
    // Torus::new(inner_hole_radius, outer_radius) — a ring with a hole through
    // it, like the bag icon's bow.
    let bow_mesh = meshes.as_mut().map(|m| m.add(Torus::new(0.085, 0.18)));
    let spark_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(SPARK_MESH_RADIUS)));
    KeyHolderAssets {
        bow_mesh,
        spark_mesh,
        key_mat: common::build_emissive_material(materials, KEY_EMISSIVE),
        spark_mat: common::build_emissive_material(materials, SPARK_EMISSIVE),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_key_holder_for_cell(
    commands: &mut Commands,
    key_assets: &KeyHolderAssets,
    common_assets: &CommonObjectAssets,
    style: KeyHolderStyle,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    level: usize,
) {
    if cell != 'K' {
        return;
    }
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;

    // Holder root — owns the floating key (added below), positioned at the cell
    // floor (lifted to its level) so the collection flourish can rise it from the
    // level's floor.
    let holder = commands
        .spawn((
            KeyMarker { cell: (r, c), level },
            Transform::from_xyz(x, world_y(level, 0.0), z),
            Visibility::default(),
        ))
        .id();

    // Spawn the chosen base prop free-standing (so it remains as an emptied
    // holder after the key is collected), and note its apex so the key can
    // float a fixed clearance above it.
    let base_top = match style {
        KeyHolderStyle::Pedestal => {
            let h = common::pillar::KEYHOLDER_HEIGHT_SCALE;
            common::pillar::spawn_pillar(commands, common_assets, x, z, h, level);
            common::pillar::TOP_Y * h
        }
        KeyHolderStyle::Chest => {
            let yaw = common::yaw_toward_open_neighbour(grid, r, c);
            common::chest::spawn_chest(commands, common_assets, x, z, yaw, common::chest::ChestLid::Closed, level);
            common::chest::TOP_Y
        }
        KeyHolderStyle::FloatingKey => 0.0, // key floats alone, no base
    };
    let rest_y = KEY_REST_Y.max(base_top + KEY_CLEARANCE);

    spawn_floating_key(commands, key_assets, common_assets, holder, rest_y);
}

fn spawn_floating_key(
    commands: &mut Commands,
    key_assets: &KeyHolderAssets,
    common_assets: &CommonObjectAssets,
    holder: Entity,
    rest_y: f32,
) {
    // The floating key group bobs and spins as one (in the holder's local
    // frame); its sub-meshes, glow, and sparks are children. A uniform scale
    // keeps the key sensibly sized relative to the door keyholes.
    let key_group = commands
        .spawn((
            FloatingKey { base_y: rest_y },
            Transform::from_xyz(0.0, rest_y, 0.0).with_scale(Vec3::splat(KEY_SCALE)),
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
        (key_assets.spark_mesh.clone(), key_assets.spark_mat.clone())
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

    let mat = key_assets.key_mat.clone();
    let outline = common_assets.outline_mat.clone();
    // Bow (round head) at the top — a ring with a hole, standing upright in the
    // key's own plane (rotated 90° about X from the torus's default flat pose)
    // so the hole faces the player as the key spins.
    common::spawn_with_outline(
        commands,
        Some(key_group),
        key_assets.bow_mesh.clone(),
        mat.clone(),
        outline.clone(),
        Transform::from_xyz(0.0, 0.27, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        (),
    );
    // Shaft running down from the bow — the shared unit cylinder, scaled thin so
    // its round cross-section matches the circular lock / keyhole.
    common::spawn_with_outline(
        commands,
        Some(key_group),
        common_assets.cylinder.clone(),
        mat.clone(),
        outline.clone(),
        Transform::from_xyz(0.0, -0.05, 0.0).with_scale(Vec3::new(0.08, 0.45, 0.08)),
        (),
    );
    // Two teeth jutting from the shaft's lower end — the shared unit cuboid.
    common::spawn_with_outline(
        commands,
        Some(key_group),
        common_assets.cuboid.clone(),
        mat.clone(),
        outline.clone(),
        Transform::from_xyz(0.08, -0.22, 0.0).with_scale(Vec3::new(0.16, 0.06, 0.06)),
        (),
    );
    common::spawn_with_outline(
        commands,
        Some(key_group),
        common_assets.cuboid.clone(),
        mat,
        outline,
        Transform::from_xyz(0.06, -0.30, 0.0).with_scale(Vec3::new(0.12, 0.06, 0.06)),
        (),
    );
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

/// `Update`: plays the key-collection flourish on any holder tagged
/// [`CollectingKey`] — the holder (and its child key) rises while shrinking to
/// nothing and spinning faster — then despawns it when the animation completes.
/// The base prop is a separate free-standing entity, so it stays behind as an
/// emptied holder. The idle bob/spin on the child [`FloatingKey`] keeps running
/// in local space, so the key tumbles as it's whisked up.
pub(crate) fn key_collection_system(
    mut commands: Commands,
    time: Res<Time>,
    mut holders: Query<(Entity, &mut CollectingKey, &mut Transform, &KeyMarker)>,
) {
    let dt = time.delta_secs();
    for (entity, mut collecting, mut transform, marker) in &mut holders {
        collecting.elapsed += dt;
        let progress = (collecting.elapsed / KEY_COLLECT_DURATION).min(1.0);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Ease-out so the key leaps up quickly then settles into nothing.
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
        transform.translation.y = world_y(marker.level, KEY_COLLECT_RISE * eased);
        transform.scale = Vec3::splat(1.0 - eased);
        transform.rotate_y(dt * KEY_COLLECT_SPIN_RATE);
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
