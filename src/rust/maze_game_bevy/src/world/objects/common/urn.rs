use super::bake::{BakedRig, RigBuilder, UnitMeshes};
use super::{build_emissive_material, CommonObjectAssets};
use crate::world::LevelTag;
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

// Rig slots — one combined mesh per material. The urn deliberately has no
// outline slot: the stacked cylinders' vertical silhouettes are slightly offset
// from each other in radius, and a black outline at each layer's edge fails to
// connect across the steps — it reads as broken "vertical edging" rather than a
// single piece. The darker terracotta join rings and belly bands already provide
// the horizontal contrast needed to delineate the layers.
const BODY: usize = 0;
const BAND: usize = 1;

/// Bakes the urn rig in its local frame: a stacked-cylinder vase silhouette,
/// with the rim, two belly bands and two join rings in the darker terracotta.
pub(crate) fn build_urn_rig(
    prims: &UnitMeshes,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> BakedRig {
    let mut rig = RigBuilder::new(&[
        build_emissive_material(materials, URN_EMISSIVE),
        build_emissive_material(materials, BAND_EMISSIVE),
    ]);
    let mut add = |slot: usize, y: f32, scale: Vec3| {
        rig.add(slot, &prims.cylinder, Transform::from_xyz(0.0, y, 0.0).with_scale(scale));
    };

    // Body stack.
    add(BODY, BASE_Y, BASE_SCALE);
    add(BODY, LOWER_BELLY_Y, LOWER_BELLY_SCALE);
    add(BODY, BELLY_Y, BELLY_SCALE);
    add(BODY, UPPER_BELLY_Y, UPPER_BELLY_SCALE);
    add(BODY, NECK_Y, NECK_SCALE);

    // Rim + two pattern bands.
    add(BAND, RIM_Y, RIM_SCALE);
    add(BAND, BAND_A_Y, BAND_SCALE);
    add(BAND, BAND_B_Y, BAND_SCALE);

    // Join rings at the two wider→narrower steps. Without these, the top edge of
    // the wider cylinder reads as a single flat shade and the join with the
    // narrower cylinder above is invisible.
    add(BAND, JOIN_RING_BELLY_TOP_Y, JOIN_RING_BELLY_TOP_SCALE);
    add(BAND, JOIN_RING_UPPER_BELLY_TOP_Y, JOIN_RING_UPPER_BELLY_TOP_SCALE);

    rig.finish(meshes)
}

pub(crate) fn spawn_urn(commands: &mut Commands, assets: &CommonObjectAssets, x: f32, z: f32, base_y: f32, tag: LevelTag) {
    // `base_y` lifts the rig to its run level's floor; level 0 is `0.0`.
    assets.urn.spawn(commands, Transform::from_xyz(x, base_y, z), None, Some(tag));
}

#[cfg(test)]
mod tests {
    use super::super::test_support::entities_spawned;
    use super::super::build_common_object_assets;
    use super::*;

    #[test]
    fn an_urn_costs_one_entity_per_material() {
        let assets = build_common_object_assets(&mut None, &mut None);
        let count = entities_spawned(|commands| {
            spawn_urn(commands, &assets, 0.0, 0.0, 0.0, LevelTag(0));
        });
        assert_eq!(count, 2, "the terracotta body and the darker bands");
    }
}
