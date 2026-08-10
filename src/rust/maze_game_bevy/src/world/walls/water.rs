//! Non-occluding water cell — a still, faintly glowing pool recessed in a basin
//! below floor level. The cell has no floor tile or grid lines; the recessed
//! surface *is* the bottom of the basin, so adjacent water cells abut edge-to-edge
//! into one continuous sunken sheet. The rim wall up to floor level is drawn by
//! [`super::rim`]. The surface is a clear blue and sits low so, with the wall
//! panels around it suppressed (see [`super::solid::spawn_walls_for_cell`]), the
//! player sees across it to whatever lies in the cells beyond. It is opaque (like
//! lava) so adjacent surfaces meet without alpha-blended seams. [`water_animation_system`]
//! gently undulates the surface and scrolls a tileable ripple texture across it.

use super::rim::RECESS_DEPTH;
use crate::state::GameConfig;
use crate::world::visibility::LevelWindow;
use crate::world::{LevelPlacement, LevelTag, CELL_SIZE};
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Thin vertical extent of the surface sheet — small, so it reads as a flat
/// waterline rather than a block.
const SURFACE_THICKNESS: f32 = 0.04;

/// Y of the surface — recessed [`RECESS_DEPTH`] below the surrounding floor tops
/// (which sit at ≈ 0). The rim skirt ([`super::rim`]) fills the band from this
/// level up to the floor on every edge that meets a non-pool cell.
const SURFACE_Y: f32 = -RECESS_DEPTH;

/// Surface emissive — a clear, saturated blue that reads unmistakably as water
/// under the dim corridor lighting without lighting the walls around it. A
/// tileable ripple texture modulates this so the surface shows moving ripples.
const WATER_EMISSIVE: LinearRgba = LinearRgba::new(0.04, 0.22, 0.70, 1.0);

/// Undulation amplitude (units) — the vertical rise/fall of the surface. Small
/// relative to [`RECESS_DEPTH`] so the water stays well within its basin.
const WAVE_AMP: f32 = 0.04;
/// Spatial frequency of the surface wave (radians/unit) — ≈ a four-cell
/// wavelength, so the undulation flows visibly across a multi-cell pool.
const WAVE_K: f32 = 0.785;
/// Temporal speed of the surface wave (radians/sec) — a slow, calm ripple.
const WAVE_SPEED: f32 = 0.8;

/// Ripple-texture repeats across one cell. Integer so the pattern tiles
/// seamlessly into the neighbouring cell (the ripples stay continuous across a
/// multi-cell pool). Three repeats packs the fine ripples in tighter.
const RIPPLE_UV: Vec2 = Vec2::new(3.0, 3.0);
/// Per-second UV scroll of the ripple texture (u, v) — the ripples drift slowly
/// across the surface, like a light breeze.
const RIPPLE_SCROLL: Vec2 = Vec2::new(0.018, 0.011);

/// Plane-wave frequencies for the water ripple texture — several higher-frequency
/// waves at mixed (often diagonal) directions, so the surface reads as many fine
/// crossing ripples rather than a few broad patches of shade.
const RIPPLE_WAVES: &[(f32, f32)] = &[
    (5.0, 1.0),
    (1.0, 6.0),
    (4.0, 4.0),
    (7.0, 3.0),
    (3.0, 7.0),
];
/// Contrast of the ripple texture around its mid grey.
const RIPPLE_AMP: f32 = 0.24;

/// Marker on a water pool surface. Spawned per non-occluding water `'W'` cell;
/// [`water_animation_system`] queries it to undulate the surface. The stored
/// `base_y` is the pool's level floor Y (`base_level_y[level]`), so the animation
/// keeps the surface at its stacked — possibly pool-gap-lifted — height.
#[derive(Component)]
pub(crate) struct WaterSurface {
    base_y: f32,
}

pub(crate) struct WaterAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_water_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> WaterAssets {
    // Full-cell surface sheet (no border inset) so adjacent pools meet seamlessly.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, SURFACE_THICKNESS, CELL_SIZE)));
    let ripple = images
        .as_mut()
        .map(|imgs| super::ripple_texture(imgs, RIPPLE_WAVES, RIPPLE_AMP));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Opaque (like lava) so adjacent surfaces meet without alpha-blended
            // seams; the player still sees *over* it because it is low and
            // panel-free. The ripple texture (scrolled by water_animation_system)
            // modulates the emissive into moving ripples.
            base_color: crate::palette::EMISSIVE_ONLY_BASE,
            emissive: WATER_EMISSIVE,
            emissive_texture: ripple,
            uv_transform: Affine2::from_scale(RIPPLE_UV),
            ..default()
        })
    });
    WaterAssets { mesh, material }
}

/// Spawns the recessed water pool surface for a cell at the caller-built `surface`
/// transform (its free edges inset off the cell boundary — see
/// [`super::pool_surface_transform`]). The caller spawns the rim
/// ([`super::rim`]); the cell has no separate floor tile.
pub(crate) fn spawn_water(
    commands: &mut Commands,
    assets: &WaterAssets,
    placement: LevelPlacement,
    surface: Transform,
) {
    // Only the level's floor base is stored: the animation re-derives the resting
    // Y from it; the surface's X/Z and edge-inset scale (baked into `surface`) are
    // fixed for its lifetime — the animation rewrites only Y + tilt, leaving the
    // scale intact.
    let base_y = placement.base_y();
    match (assets.mesh.clone(), assets.material.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((WaterSurface { base_y }, placement.tag(), surface, Mesh3d(mesh), MeshMaterial3d(mat)));
        }
        _ => {
            commands.spawn((WaterSurface { base_y }, placement.tag(), surface));
        }
    }
}

/// `Update` system: gently undulates every water surface and scrolls the shared
/// ripple texture. The undulation is phased by each tile's world `(x, z)` (read
/// straight off its transform, only its Y is animated), so adjacent water tiles
/// read as one continuous moving sheet — see [`super::pool_wave`]. The ripple
/// texture is one shared material, so a single UV scroll drifts the ripples
/// across every water tile in step. Mirrors the per-entity transform animation of
/// the enemy / health / door systems.
pub(crate) fn water_animation_system(
    time: Res<Time>,
    config: Res<GameConfig>,
    window: Res<LevelWindow>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut surfaces: Query<(&mut Transform, &WaterSurface, &LevelTag)>,
    surface_mats: Query<&MeshMaterial3d<StandardMaterial>, With<WaterSurface>>,
) {
    let t = time.elapsed_secs();
    // Frozen: the wave's whole cost is the per-surface transform write, so the
    // ablation has to skip the loop rather than write the same value.
    if !config.freeze_wall_animation {
    for (mut tr, surface, tag) in surfaces.iter_mut() {
        // A floor outside the window is neither drawn nor moved: the write alone
        // would re-run transform propagation and re-upload the instance.
        if !window.contains(tag.0) {
            continue;
        }
        let (dy, rot) = super::pool_wave(tr.translation.x, tr.translation.z, t, WAVE_AMP, WAVE_K, WAVE_SPEED);
        tr.translation.y = surface.base_y + SURFACE_Y + dy;
        tr.rotation = rot;
    }
    }
    // Drift the ripple texture. The surfaces share one material, so updating it
    // once (off any surface's handle) ripples them all; only the UV translation
    // changes, preserving the scale set at build time.
    if let (Some(materials), Some(handle)) = (materials.as_mut(), surface_mats.iter().next()) {
        if let Some(mat) = materials.get_mut(&handle.0) {
            mat.uv_transform.translation = RIPPLE_SCROLL * t;
        }
    }
}
