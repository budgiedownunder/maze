use super::bake::{BakedRig, RigBuilder, UnitMeshes};
use super::{build_emissive_material, CommonObjectAssets};
use bevy::prelude::*;
use std::f32::consts::TAU;

// ---------- Tuning constants ----------

/// Pillar emissive RGB — pale marble grey.
const PILLAR_EMISSIVE: LinearRgba = LinearRgba::new(0.70, 0.70, 0.65, 1.0);
/// Groove emissive RGB — darker marble for the perimeter flutes so they
/// read as recessed channels around the base and capital.
const GROOVE_EMISSIVE: LinearRgba = LinearRgba::new(0.40, 0.40, 0.37, 1.0);
/// Outline base colour for the pillar — same linear values as
/// [`GROOVE_EMISSIVE`] so the inverted-hull outline matches the
/// recessed flute colour around the base and capital instead of the
/// default black used by the other props.
pub(crate) const OUTLINE_BASE_COLOR: Color = Color::linear_rgba(0.40, 0.40, 0.37, 1.0);

/// Base (lower disc) world-Y centre and scale.
const BASE_Y: f32 = 0.075;
const BASE_SCALE: Vec3 = Vec3::new(0.55, 0.15, 0.55);

/// Shaft world-Y centre and scale (thin column between base + capital).
const SHAFT_Y: f32 = 0.75;
const SHAFT_SCALE: Vec3 = Vec3::new(0.35, 1.20, 0.35);

/// Capital (upper disc) world-Y centre and scale.
const CAPITAL_Y: f32 = 1.425;
const CAPITAL_SCALE: Vec3 = Vec3::new(0.55, 0.15, 0.55);

/// Apex height of the pillar at full (dead-end) height — the top of the capital
/// disc. Multiply by the spawn's `height_scale` for the effective apex.
pub(crate) const TOP_Y: f32 = CAPITAL_Y + CAPITAL_SCALE.y * 0.5;

/// Vertical scale applied to the whole rig when the pillar is reused as a key
/// holder's pedestal — half the dead-end landmark height, so the floating key
/// sits at a comfortable eye level rather than high overhead.
pub(crate) const KEYHOLDER_HEIGHT_SCALE: f32 = 0.5;

/// Number of perimeter grooves around the base (lower frequency).
const BASE_GROOVE_COUNT: u32 = 8;
/// Number of perimeter grooves around the capital (higher frequency).
const CAPITAL_GROOVE_COUNT: u32 = 12;
/// Scale of each individual groove cuboid `(x, y, z)`. Thin square
/// cross-section, matched height to the base/capital disc.
const GROOVE_SCALE: Vec3 = Vec3::new(0.04, 0.15, 0.04);
/// Radial distance from the pillar centre at which grooves sit. Matches
/// the base/capital disc edge: `BASE_SCALE.x * 0.5` for a unit cylinder
/// scaled to `BASE_SCALE.x` diameter.
const GROOVE_RADIUS: f32 = 0.275;

/// Join ring at the top of the base where the shaft sits on it. Without
/// this, the shaft's bottom edge (hidden inside the base's top face)
/// has no visible silhouette and the join blends into the marble. The
/// ring sits on the base's top surface in the darker groove colour.
const BASE_JOIN_RING_Y: f32 = 0.15;
const BASE_JOIN_RING_SCALE: Vec3 = Vec3::new(0.40, 0.025, 0.40);

// Rig slots — one combined mesh per material. The pillar's outline is the grey
// `pillar_outline_mat` (matching the groove colour) rather than the default
// black, so the rim around each disc / shaft reads as continuous with the
// perimeter flutes rather than as a hard cartoon edge.
const BODY: usize = 0;
const GROOVE: usize = 1;
const OUTLINE: usize = 2;

/// Bakes the pillar rig in its local frame, at **full** height: base + shaft +
/// capital, the perimeter grooves around the two discs, and the join ring.
pub(crate) fn build_pillar_rig(
    prims: &UnitMeshes,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    outline_mat: &Option<Handle<StandardMaterial>>,
) -> BakedRig {
    let mut rig = RigBuilder::new(&[
        build_emissive_material(materials, PILLAR_EMISSIVE),
        build_emissive_material(materials, GROOVE_EMISSIVE),
        outline_mat.clone(),
    ]);
    let pose = |x: f32, y: f32, z: f32, scale: Vec3| {
        Transform::from_xyz(x, y, z).with_scale(scale)
    };

    // Base + shaft + capital — the columnar silhouette.
    rig.add_with_outline(BODY, OUTLINE, &prims.cylinder, pose(0.0, BASE_Y, 0.0, BASE_SCALE));
    rig.add_with_outline(BODY, OUTLINE, &prims.cylinder, pose(0.0, SHAFT_Y, 0.0, SHAFT_SCALE));
    rig.add_with_outline(BODY, OUTLINE, &prims.cylinder, pose(0.0, CAPITAL_Y, 0.0, CAPITAL_SCALE));

    // Perimeter grooves around the base + capital. Spaced evenly via
    // `(cos(angle), sin(angle)) * GROOVE_RADIUS`, with one cuboid per angle.
    for (y, count) in [(BASE_Y, BASE_GROOVE_COUNT), (CAPITAL_Y, CAPITAL_GROOVE_COUNT)] {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * TAU;
            let (gx, gz) = (angle.cos() * GROOVE_RADIUS, angle.sin() * GROOVE_RADIUS);
            rig.add_with_outline(GROOVE, OUTLINE, &prims.cuboid, pose(gx, y, gz, GROOVE_SCALE));
        }
    }

    // Join ring on top of the base around the shaft's foot. Width is wider than
    // the shaft (0.175 radius) but narrower than the base (0.275 radius), so it
    // sits as a darker flange exactly at the join.
    rig.add_with_outline(
        GROOVE,
        OUTLINE,
        &prims.cylinder,
        pose(0.0, BASE_JOIN_RING_Y, 0.0, BASE_JOIN_RING_SCALE),
    );

    rig.finish(meshes)
}

/// Spawns the pillar at `(x, z)` on its level's floor, scaled to `height_scale`
/// of the full landmark — `1.0` for a dead-end landmark, [`KEYHOLDER_HEIGHT_SCALE`]
/// for the key holder's pedestal.
///
/// The height is a **vertical scale on the whole rig**, not a rebuild: scaling
/// the baked rig by `(1, h, 1)` moves each part's centre and squashes its extent
/// by exactly `h` while leaving the footprint alone, which is what the rig
/// wanted from a height variant — so one baked pillar serves both callers.
pub(crate) fn spawn_pillar(
    commands: &mut Commands,
    assets: &CommonObjectAssets,
    x: f32,
    z: f32,
    height_scale: f32,
    base_y: f32,
) {
    assets.pillar.spawn(
        commands,
        Transform::from_xyz(x, base_y, z).with_scale(Vec3::new(1.0, height_scale, 1.0)),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::super::test_support::entities_spawned;
    use super::super::build_common_object_assets;
    use super::*;

    #[test]
    fn a_pillar_costs_one_entity_per_material() {
        let assets = build_common_object_assets(&mut None, &mut None);
        let count = entities_spawned(|commands| {
            spawn_pillar(commands, &assets, 0.0, 0.0, 1.0, 0.0);
        });
        assert_eq!(count, 3, "marble body, darker grooves, one outline shell");
    }
}
