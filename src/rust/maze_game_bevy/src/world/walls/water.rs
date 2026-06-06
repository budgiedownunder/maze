//! Non-occluding water cell — a still, faintly glowing pool recessed in a basin
//! below floor level. The cell has no floor tile or grid lines; the recessed
//! surface *is* the bottom of the basin, so adjacent water cells abut edge-to-edge
//! into one continuous sunken sheet. The rim wall up to floor level is drawn by
//! [`super::rim`]. The surface is a clear blue and sits low so, with the wall
//! panels around it suppressed (see [`super::solid::spawn_walls_for_cell`]), the
//! player sees across it to whatever lies in the cells beyond. It is opaque (like
//! lava) so adjacent surfaces meet without alpha-blended seams.

use super::rim::RECESS_DEPTH;
use crate::world::CELL_SIZE;
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
/// under the dim corridor lighting without lighting the walls around it.
const WATER_EMISSIVE: LinearRgba = LinearRgba::new(0.04, 0.22, 0.70, 1.0);

/// Marker on a water pool surface. Spawned per non-occluding water `'W'` cell;
/// the water-animation system (a later step) queries it to undulate the surface.
#[derive(Component)]
pub(crate) struct WaterSurface;

pub(crate) struct WaterAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_water_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> WaterAssets {
    // Full-cell surface sheet (no border inset) so adjacent pools meet seamlessly.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, SURFACE_THICKNESS, CELL_SIZE)));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Opaque (like lava) so adjacent surfaces meet without alpha-blended
            // seams; the player still sees *over* it because it is low and
            // panel-free.
            base_color: crate::palette::EMISSIVE_ONLY_BASE,
            emissive: WATER_EMISSIVE,
            ..default()
        })
    });
    WaterAssets { mesh, material }
}

/// Spawns the recessed water pool surface filling cell `(r, c)`. The caller
/// spawns the rim ([`super::rim`]); the cell has no separate floor tile.
pub(crate) fn spawn_water(commands: &mut Commands, assets: &WaterAssets, r: usize, c: usize) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    match (assets.mesh.clone(), assets.material.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                WaterSurface,
                Transform::from_xyz(x, SURFACE_Y, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((WaterSurface, Transform::from_xyz(x, SURFACE_Y, z)));
        }
    }
}
