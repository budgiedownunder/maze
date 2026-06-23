use super::{build_emissive_material, spawn_with_outline, CommonObjectAssets};
use crate::world::world_y;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Urn body emissive RGB — terracotta brown.
const URN_EMISSIVE: LinearRgba = LinearRgba::new(0.55, 0.30, 0.15, 1.0);
/// Darker terracotta used for the rim and the two pattern bands wrapping
/// the belly. Sits visibly darker than [`URN_EMISSIVE`] so the bands
/// read as a contrasting pattern.
const BAND_EMISSIVE: LinearRgba = LinearRgba::new(0.30, 0.15, 0.07, 1.0);

// Stacked-cylinder vase silhouette: narrow base → wide belly → narrower
// neck → flared rim. Each part is a scaled instance of the shared unit
// cylinder.

/// Base segment (narrow foot).
const BASE_Y: f32 = 0.05;
const BASE_SCALE: Vec3 = Vec3::new(0.30, 0.10, 0.30);

/// Lower-belly segment, swelling outward from the base.
const LOWER_BELLY_Y: f32 = 0.20;
const LOWER_BELLY_SCALE: Vec3 = Vec3::new(0.45, 0.20, 0.45);

/// Belly segment (widest point of the urn).
const BELLY_Y: f32 = 0.40;
const BELLY_SCALE: Vec3 = Vec3::new(0.55, 0.20, 0.55);

/// Upper-belly segment, tapering back inward toward the neck.
const UPPER_BELLY_Y: f32 = 0.575;
const UPPER_BELLY_SCALE: Vec3 = Vec3::new(0.45, 0.15, 0.45);

/// Neck segment (narrowest point above the belly).
const NECK_Y: f32 = 0.725;
const NECK_SCALE: Vec3 = Vec3::new(0.30, 0.15, 0.30);

/// Rim segment (flared lip at the top).
const RIM_Y: f32 = 0.825;
const RIM_SCALE: Vec3 = Vec3::new(0.40, 0.05, 0.40);

/// Pattern band A — wraps the belly low.
const BAND_A_Y: f32 = 0.33;
/// Pattern band B — wraps the belly high.
const BAND_B_Y: f32 = 0.47;
/// Shared belly-pattern band scale. Slightly wider than the belly so the
/// band rim protrudes past the body silhouette and reads as a distinct ring.
const BAND_SCALE: Vec3 = Vec3::new(0.56, 0.025, 0.56);

// ---------- Join rings ----------
//
// At every wider→narrower step (Belly→UpperBelly, UpperBelly→Neck), the
// lower cylinder's top edge is otherwise invisible from eye-level: the
// inverted-hull outline only renders the silhouette, and the top face
// disc reads as the same flat terracotta as the body. A thin darker
// ring at each step makes the join read clearly. Width is matched to
// the LOWER cylinder so the ring sits as a flange on its top face.

/// Join ring at the top of Belly (Belly→UpperBelly join).
const JOIN_RING_BELLY_TOP_Y: f32 = 0.50;
const JOIN_RING_BELLY_TOP_SCALE: Vec3 = Vec3::new(0.56, 0.025, 0.56);

/// Join ring at the top of Upper belly (UpperBelly→Neck join).
const JOIN_RING_UPPER_BELLY_TOP_Y: f32 = 0.65;
const JOIN_RING_UPPER_BELLY_TOP_SCALE: Vec3 = Vec3::new(0.46, 0.025, 0.46);

pub(crate) fn build_urn_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, URN_EMISSIVE)
}

pub(crate) fn build_dark_terracotta_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, BAND_EMISSIVE)
}

pub(crate) fn spawn_urn(commands: &mut Commands, assets: &CommonObjectAssets, x: f32, z: f32, level: usize) {
    let body = assets.urn_mat.clone();
    let band = assets.dark_terracotta_mat.clone();
    // Lift each stacked-cylinder part to its run level; level 0 is the identity.
    let pos = |y: f32| Vec3::new(x, world_y(level, y), z);
    // The urn deliberately skips the inverted-hull outline: the stacked
    // cylinders' vertical silhouettes are slightly offset from each
    // other in radius, and a black outline at each layer's edge fails
    // to connect across the steps — it reads as broken "vertical
    // edging" rather than a single piece. The darker terracotta join
    // rings and belly bands already provide the horizontal contrast
    // needed to delineate the layers.
    let outline = || -> Option<Handle<StandardMaterial>> { None };
    let mesh = || assets.cylinder.clone();

    // Body stack.
    spawn_with_outline(
        commands,
        None,
        mesh(),
        body.clone(),
        outline(),
        Transform::from_translation(pos(BASE_Y)).with_scale(BASE_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        body.clone(),
        outline(),
        Transform::from_translation(pos(LOWER_BELLY_Y)).with_scale(LOWER_BELLY_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        body.clone(),
        outline(),
        Transform::from_translation(pos(BELLY_Y)).with_scale(BELLY_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        body.clone(),
        outline(),
        Transform::from_translation(pos(UPPER_BELLY_Y)).with_scale(UPPER_BELLY_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        body,
        outline(),
        Transform::from_translation(pos(NECK_Y)).with_scale(NECK_SCALE),
        (),
    );

    // Rim + two pattern bands all share the darker terracotta material.
    spawn_with_outline(
        commands,
        None,
        mesh(),
        band.clone(),
        outline(),
        Transform::from_translation(pos(RIM_Y)).with_scale(RIM_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        band.clone(),
        outline(),
        Transform::from_translation(pos(BAND_A_Y)).with_scale(BAND_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        band.clone(),
        outline(),
        Transform::from_translation(pos(BAND_B_Y)).with_scale(BAND_SCALE),
        (),
    );

    // Join rings at the two wider→narrower steps. Without these, the
    // top edge of the wider cylinder reads as a single flat shade and
    // the join with the narrower cylinder above is invisible.
    spawn_with_outline(
        commands,
        None,
        mesh(),
        band.clone(),
        outline(),
        Transform::from_translation(pos(JOIN_RING_BELLY_TOP_Y))
            .with_scale(JOIN_RING_BELLY_TOP_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        mesh(),
        band,
        outline(),
        Transform::from_translation(pos(JOIN_RING_UPPER_BELLY_TOP_Y))
            .with_scale(JOIN_RING_UPPER_BELLY_TOP_SCALE),
        (),
    );
}
