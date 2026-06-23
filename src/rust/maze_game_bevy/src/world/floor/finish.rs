use super::{FloorAssets, FloorCell};
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Finish-cell emissive RGB — bright cool grey, paired with the warm
/// gold orb hovering above.
const FINISH_EMISSIVE: LinearRgba = LinearRgba::new(0.8, 0.8, 0.8, 1.0);
/// UV repeat across the cell, matched to the regular floor tile.
const FINISH_UV_SCALE: Vec2 = Vec2::new(2.0, 2.0);

#[derive(Component)]
pub(crate) struct FinishCell;

pub(crate) fn build_finish_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tile_tex: &Option<Handle<Image>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: FINISH_EMISSIVE,
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(FINISH_UV_SCALE),
            ..default()
        })
    })
}

pub(crate) fn spawn_finish(
    commands: &mut Commands,
    assets: &FloorAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(0.0);
    match (assets.floor_mesh.clone(), assets.finish_mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FinishCell,
                FloorCell,
                Transform::from_xyz(x, y, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((FinishCell, FloorCell, Transform::from_xyz(x, y, z)));
        }
    }
}
