use super::{build_emissive_material, spawn_with_outline, CommonObjectAssets};
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::world_y;
use bevy::prelude::*;
use std::f32::consts::TAU;

// ---------- Tuning constants ----------

/// Stone column emissive RGB — neutral grey.
const STONE_EMISSIVE: LinearRgba = LinearRgba::new(0.45, 0.45, 0.45, 1.0);
/// Glowing bowl emissive RGB — over-bright orange that picks up the
/// bloom pipeline at distance. Driven by [`flicker_factor`] each frame.
pub(crate) const GLOW_EMISSIVE: LinearRgba = LinearRgba::new(1.4, 0.7, 0.15, 1.0);
/// Halo emissive RGB — a stronger, redder orange than the bowl so the
/// halo reads as the brighter fringe around the flickering glow. The
/// halo material is steady (not flickered) on purpose so the eye keeps
/// a stable reference while the bowl modulates.
const HALO_EMISSIVE: LinearRgba = LinearRgba::new(1.8, 0.9, 0.25, 1.0);

/// Column world-Y centre (mesh is a unit cylinder, scaled to
/// [`COLUMN_SCALE`]; centre is half the column's scaled height).
const COLUMN_Y: f32 = 0.40;
/// Column scale `(x, y, z)`. Y is the column height; X and Z are equal
/// (round cylinder cross-section).
const COLUMN_SCALE: Vec3 = Vec3::new(0.40, 0.80, 0.40);

/// Bowl world-Y centre (sits on top of the column).
const BOWL_Y: f32 = 0.85;
/// Bowl scale `(x, y, z)`. Wider than column; short height for a shallow bowl.
const BOWL_SCALE: Vec3 = Vec3::new(0.50, 0.15, 0.50);

/// Halo world-Y centre (a thin glowing disc just above the bowl).
const HALO_Y: f32 = 0.95;
/// Halo scale `(x, y, z)`. Wider than the bowl so the halo fringe
/// extends past the bowl's silhouette.
const HALO_SCALE: Vec3 = Vec3::new(0.62, 0.05, 0.62);

/// Sin-flicker base rate (cycles per second).
pub(crate) const FLICKER_RATE_HZ: f32 = 1.5;
/// Sin-flicker maximum amplitude (fraction of base emissive). The phase
/// term spans ~[-1.4, 1.4] so the actual modulation amplitude is this
/// value × 0.7.
pub(crate) const FLICKER_AMPLITUDE: f32 = 0.25;
/// Detune ratio of the secondary sine relative to the primary, picked
/// to break visible periodicity without using a PRNG.
pub(crate) const FLICKER_DETUNE_RATIO: f32 = 1.7;

/// Marker for brazier bowls. Queried by [`brazier_flicker_system`] to
/// modulate the shared glow material each frame. Halo entities don't
/// carry this marker — the halo uses a separate, steady material that
/// frames the flickering bowl.
#[derive(Component)]
pub(crate) struct BrazierBowl;

pub(crate) fn build_stone_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: STONE_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn build_glow_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, GLOW_EMISSIVE)
}

pub(crate) fn build_halo_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, HALO_EMISSIVE)
}

/// Returns the per-frame multiplier applied to [`GLOW_EMISSIVE`] for the
/// brazier bowl. Pure function so unit tests can sweep `t` without
/// booting Bevy; the live [`brazier_flicker_system`] reads this each frame.
pub(crate) fn flicker_factor(t: f32) -> f32 {
    let primary = (t * TAU * FLICKER_RATE_HZ).sin();
    let detuned = (t * TAU * FLICKER_RATE_HZ * FLICKER_DETUNE_RATIO).sin();
    let phase = primary + 0.4 * detuned;
    1.0 + FLICKER_AMPLITUDE * phase * 0.5
}

pub(crate) fn spawn_brazier(commands: &mut Commands, assets: &CommonObjectAssets, x: f32, z: f32, level: usize) {
    // Lift each part to its run level; level 0 is the identity.
    let pos = |y: f32| Vec3::new(x, world_y(level, y), z);
    // Stone column — steady, no flicker.
    spawn_with_outline(
        commands,
        None,
        assets.cylinder.clone(),
        assets.stone_mat.clone(),
        assets.outline_mat.clone(),
        Transform::from_translation(pos(COLUMN_Y)).with_scale(COLUMN_SCALE),
        (),
    );
    // Glowing bowl — carries the `BrazierBowl` marker so the flicker
    // system can find its shared glow material handle.
    spawn_with_outline(
        commands,
        None,
        assets.cylinder.clone(),
        assets.glow_mat.clone(),
        assets.outline_mat.clone(),
        Transform::from_translation(pos(BOWL_Y)).with_scale(BOWL_SCALE),
        BrazierBowl,
    );
    // Halo — a thin disc of stronger orange just above the bowl. Steady
    // (no flicker marker) so it frames the bowl's modulation rather
    // than competing with it.
    spawn_with_outline(
        commands,
        None,
        assets.cylinder.clone(),
        assets.halo_mat.clone(),
        assets.outline_mat.clone(),
        Transform::from_translation(pos(HALO_Y)).with_scale(HALO_SCALE),
        (),
    );
}

/// Modulates the shared brazier glow material's emissive each frame
/// with two slightly detuned sine waves, giving a non-uniform flicker
/// without an explicit PRNG. Every brazier in the maze shares the same
/// `glow_mat` handle, so a single material update animates them all in
/// lockstep — finding any [`BrazierBowl`] entity is enough to get the
/// handle.
pub(crate) fn brazier_flicker_system(
    time: Res<Time>,
    bowls: Query<&MeshMaterial3d<StandardMaterial>, With<BrazierBowl>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    // `Assets<StandardMaterial>` only exists when the PBR / asset plugins are
    // loaded. Tests using `MinimalPlugins` don't have it, so the parameter is
    // `Option<ResMut<…>>` and the system no-ops.
    let Some(mut materials) = materials else { return };
    let Some(handle) = bowls.iter().next() else {
        return;
    };
    let Some(mat) = materials.get_mut(&handle.0) else {
        return;
    };
    let factor = flicker_factor(time.elapsed_secs());
    mat.emissive = LinearRgba::new(
        GLOW_EMISSIVE.red * factor,
        GLOW_EMISSIVE.green * factor,
        GLOW_EMISSIVE.blue * factor,
        1.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flicker_factor_stays_within_amplitude_envelope() {
        // The phase term `sin(a) + 0.4 * sin(b)` is bounded in [-1.4, 1.4];
        // `factor = 1 + AMPLITUDE * phase * 0.5` is therefore bounded in
        // `[1 - 0.7*AMP, 1 + 0.7*AMP]`. Sweep `t` to spot-check.
        let max_phase = 1.4_f32;
        let upper = 1.0 + FLICKER_AMPLITUDE * max_phase * 0.5;
        let lower = 1.0 - FLICKER_AMPLITUDE * max_phase * 0.5;
        for i in 0..2000 {
            let t = i as f32 * 0.013;
            let f = flicker_factor(t);
            assert!(
                f <= upper + 1e-4 && f >= lower - 1e-4,
                "factor {f} out of envelope [{lower}, {upper}] at t={t}"
            );
        }
    }
}
