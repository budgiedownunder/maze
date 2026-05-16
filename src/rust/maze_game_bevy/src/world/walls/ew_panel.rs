use super::ns_panel::WallMaterialSpec;
use super::{
    WallCell, PANEL_H, PANEL_W, WALL_MATERIAL_VARIANTS, WALL_THICKNESS, WALL_TINT_OFFSETS,
    WALL_TINT_VARIANTS,
};
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct EwPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) tinted_mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS],
    pub(crate) material_mats: [Option<Handle<StandardMaterial>>; WALL_MATERIAL_VARIANTS],
}

pub(crate) fn build_ew_panel_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    material_specs: &[WallMaterialSpec; WALL_MATERIAL_VARIANTS],
) -> EwPanelAssets {
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(WALL_THICKNESS, PANEL_H, PANEL_W)));
    // E/W-facing panels (sides) — slightly darker stone grey for orientation distinction.
    let brick_spec = &material_specs[super::WALL_MATERIAL_BRICK];
    let tinted_mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS] =
        std::array::from_fn(|i| {
            let (dr, dg, db) = WALL_TINT_OFFSETS[i];
            let (br, bg, bb) = brick_spec.emissive;
            materials.as_mut().map(|m| {
                m.add(StandardMaterial {
                    base_color: Color::BLACK,
                    emissive: LinearRgba::new(
                        (br + dr).max(0.0),
                        (bg + dg).max(0.0),
                        (bb + db).max(0.0),
                        1.0,
                    ),
                    emissive_texture: brick_spec.texture.clone(),
                    uv_transform: Affine2::from_scale(brick_spec.uv_scale),
                    ..default()
                })
            })
        });
    let material_mats: [Option<Handle<StandardMaterial>>; WALL_MATERIAL_VARIANTS] =
        std::array::from_fn(|i| {
            let spec = &material_specs[i];
            let (r, g, b) = spec.emissive;
            materials.as_mut().map(|m| {
                m.add(StandardMaterial {
                    base_color: Color::BLACK,
                    emissive: LinearRgba::new(r, g, b, 1.0),
                    emissive_texture: spec.texture.clone(),
                    uv_transform: Affine2::from_scale(spec.uv_scale),
                    ..default()
                })
            })
        });
    EwPanelAssets {
        mesh,
        tinted_mats,
        material_mats,
    }
}

pub(crate) fn spawn_ew_face_tinted(
    commands: &mut Commands,
    assets: &EwPanelAssets,
    tint: usize,
    pos: Vec3,
) {
    spawn_ew_panel(commands, assets.mesh.clone(), assets.tinted_mats[tint].clone(), pos);
}

pub(crate) fn spawn_ew_face_material(
    commands: &mut Commands,
    assets: &EwPanelAssets,
    kind: usize,
    pos: Vec3,
) {
    spawn_ew_panel(commands, assets.mesh.clone(), assets.material_mats[kind].clone(), pos);
}

fn spawn_ew_panel(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
    pos: Vec3,
) {
    match (mesh, mat) {
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
