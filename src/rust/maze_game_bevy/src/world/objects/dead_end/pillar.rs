use super::{build_emissive_material, spawn_with_outline, DeadEndAssets};
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
/// default black used by the other dead-end objects.
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

pub(crate) fn build_pillar_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, PILLAR_EMISSIVE)
}

pub(crate) fn build_groove_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, GROOVE_EMISSIVE)
}

pub(crate) fn spawn_pillar(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    let body = assets.pillar_mat.clone();
    let groove = assets.groove_mat.clone();
    // Pillar uses the grey `pillar_outline_mat` (matches the groove
    // colour) instead of the default black outline, so the rim around
    // each disc/shaft reads as continuous with the perimeter flutes
    // rather than as a hard cartoon edge.
    let outline = || assets.pillar_outline_mat.clone();

    // Base + shaft + capital — the columnar silhouette.
    spawn_with_outline(
        commands,
        assets.cylinder.clone(),
        body.clone(),
        outline(),
        Transform::from_translation(Vec3::new(x, BASE_Y, z)).with_scale(BASE_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        assets.cylinder.clone(),
        body.clone(),
        outline(),
        Transform::from_translation(Vec3::new(x, SHAFT_Y, z)).with_scale(SHAFT_SCALE),
        (),
    );
    spawn_with_outline(
        commands,
        assets.cylinder.clone(),
        body,
        outline(),
        Transform::from_translation(Vec3::new(x, CAPITAL_Y, z)).with_scale(CAPITAL_SCALE),
        (),
    );

    // Perimeter grooves around the base + capital. Spaced evenly via
    // `(cos(angle), sin(angle)) * GROOVE_RADIUS`, with one cuboid per
    // angle.
    spawn_grooves(
        commands,
        assets,
        groove.clone(),
        x,
        z,
        BASE_Y,
        BASE_GROOVE_COUNT,
    );
    spawn_grooves(
        commands,
        assets,
        groove.clone(),
        x,
        z,
        CAPITAL_Y,
        CAPITAL_GROOVE_COUNT,
    );

    // Join ring on top of the base around the shaft's foot. Width is
    // wider than the shaft (0.175 radius) but narrower than the base
    // (0.275 radius), so it sits as a darker flange exactly at the join.
    spawn_with_outline(
        commands,
        assets.cylinder.clone(),
        groove,
        assets.pillar_outline_mat.clone(),
        Transform::from_translation(Vec3::new(x, BASE_JOIN_RING_Y, z))
            .with_scale(BASE_JOIN_RING_SCALE),
        (),
    );
}

fn spawn_grooves(
    commands: &mut Commands,
    assets: &DeadEndAssets,
    groove_mat: Option<Handle<StandardMaterial>>,
    x: f32,
    z: f32,
    y: f32,
    count: u32,
) {
    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU;
        let gx = x + angle.cos() * GROOVE_RADIUS;
        let gz = z + angle.sin() * GROOVE_RADIUS;
        spawn_with_outline(
            commands,
            assets.cuboid.clone(),
            groove_mat.clone(),
            assets.pillar_outline_mat.clone(),
            Transform::from_translation(Vec3::new(gx, y, gz)).with_scale(GROOVE_SCALE),
            (),
        );
    }
}
