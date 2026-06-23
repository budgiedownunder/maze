use super::{FloorAssets, FloorCell};
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{world_y, CELL_SIZE};
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Floor tile emissive RGB (cool stone grey).
const TILE_EMISSIVE: LinearRgba = LinearRgba::new(0.12, 0.12, 0.12, 1.0);
/// UV repeat across one floor cell — the tile texture tiles twice per
/// cell on each axis.
const TILE_UV_SCALE: Vec2 = Vec2::new(2.0, 2.0);

pub(crate) fn build_tile_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tile_tex: &Option<Handle<Image>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: TILE_EMISSIVE,
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(TILE_UV_SCALE),
            ..default()
        })
    })
}

pub(crate) fn spawn_tile(
    commands: &mut Commands,
    assets: &FloorAssets,
    r: usize,
    c: usize,
    level: usize,
) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let y = world_y(level, 0.0);
    match (assets.floor_mesh.clone(), assets.tile_mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FloorCell,
                Transform::from_xyz(x, y, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((FloorCell, Transform::from_xyz(x, y, z)));
        }
    }
}
