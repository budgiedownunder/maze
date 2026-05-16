use super::{WallCell, PANEL_H, PANEL_W, WALL_THICKNESS, WALL_TINT_OFFSETS, WALL_TINT_VARIANTS};
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct EwPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS],
}

pub(crate) fn build_ew_panel_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    brick_tex: &Option<Handle<Image>>,
) -> EwPanelAssets {
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(WALL_THICKNESS, PANEL_H, PANEL_W)));
    // E/W-facing panels (sides) — slightly darker stone grey for orientation distinction.
    let mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS] = std::array::from_fn(|i| {
        let (dr, dg, db) = WALL_TINT_OFFSETS[i];
        materials.as_mut().map(|m| {
            m.add(StandardMaterial {
                base_color: Color::BLACK,
                emissive: LinearRgba::new(
                    (0.14 + dr).max(0.0),
                    (0.14 + dg).max(0.0),
                    (0.16 + db).max(0.0),
                    1.0,
                ),
                emissive_texture: brick_tex.clone(),
                uv_transform: Affine2::from_scale(Vec2::new(3.0, 5.0)),
                ..default()
            })
        })
    });
    EwPanelAssets { mesh, mats }
}

pub(crate) fn spawn_ew_face(
    commands: &mut Commands,
    assets: &EwPanelAssets,
    tint: usize,
    pos: Vec3,
) {
    match (assets.mesh.clone(), assets.mats[tint].clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                WallCell,
                Transform::from_translation(pos),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((WallCell, Transform::from_translation(pos)));
        }
    }
}
