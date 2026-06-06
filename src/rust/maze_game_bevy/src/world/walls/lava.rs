//! Non-occluding lava cell — a molten pool recessed in a basin below floor level.
//! The cell has no floor tile or grid lines; the recessed surface *is* the bottom
//! of the basin, so adjacent lava cells abut edge-to-edge into one continuous
//! flow. The rim wall up to floor level is drawn by [`super::rim`]. The surface
//! glows a hot orange and sits low, so — with the wall panels around it suppressed
//! (see [`super::solid::spawn_walls_for_cell`]) — the player sees across it.
//!
//! The bubbling motion and the small dark rocks that rise and fall on the
//! surface are added by the water/lava animation system in a later step; this
//! module draws the static glowing surface.

use super::rim::RECESS_DEPTH;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Thin vertical extent of the surface sheet — small, so it reads as a flat
/// molten surface rather than a block. Matches the water surface.
const SURFACE_THICKNESS: f32 = 0.04;

/// Y of the surface — recessed [`RECESS_DEPTH`] below the surrounding floor tops
/// (≈ 0), matching the water surface so the two pool types sit at the same level.
/// The rim skirt ([`super::rim`]) fills the band up to the floor.
const SURFACE_Y: f32 = -RECESS_DEPTH;

/// Surface emissive — a hot orange (a strong green component lifts it from
/// red toward orange), brighter than the water so molten rock reads as a light
/// source against the dim corridors.
const LAVA_EMISSIVE: LinearRgba = LinearRgba::new(1.35, 0.55, 0.06, 1.0);

/// Marker on a lava pool surface. Spawned per non-occluding lava `'W'` cell;
/// the lava-animation system (a later step) queries it to bubble the surface.
#[derive(Component)]
pub(crate) struct LavaSurface;

pub(crate) struct LavaAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_lava_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> LavaAssets {
    // Full-cell slab (no border inset) so adjacent pools meet seamlessly.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, SURFACE_THICKNESS, CELL_SIZE)));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Lava is opaque — the molten surface hides what is beneath it (but
            // the player still sees *over* it because it is low and panel-free).
            base_color: crate::palette::EMISSIVE_ONLY_BASE,
            emissive: LAVA_EMISSIVE,
            ..default()
        })
    });
    LavaAssets { mesh, material }
}

/// Spawns the recessed lava pool surface filling cell `(r, c)`. The caller spawns
/// the rim ([`super::rim`]); the cell has no separate floor tile.
pub(crate) fn spawn_lava(commands: &mut Commands, assets: &LavaAssets, r: usize, c: usize) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    match (assets.mesh.clone(), assets.material.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                LavaSurface,
                Transform::from_xyz(x, SURFACE_Y, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((LavaSurface, Transform::from_xyz(x, SURFACE_Y, z)));
        }
    }
}
