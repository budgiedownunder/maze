use super::{FloorAssets, FloorCell};
use crate::world::CELL_SIZE;
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) fn build_tile_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tile_tex: &Option<Handle<Image>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.12, 0.12, 0.12, 1.0),
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(Vec2::new(2.0, 2.0)),
            ..default()
        })
    })
}

pub(crate) fn spawn_tile(commands: &mut Commands, assets: &FloorAssets, r: usize, c: usize) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    match (assets.floor_mesh.clone(), assets.tile_mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FloorCell,
                Transform::from_xyz(x, 0.0, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((FloorCell, Transform::from_xyz(x, 0.0, z)));
        }
    }
}
