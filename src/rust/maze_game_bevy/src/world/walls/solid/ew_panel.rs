use super::ns_panel::WallMaterialSpec;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::LevelTag;
use crate::world::walls::{
    WallCell, PANEL_H, PANEL_W, WALL_MATERIAL_VARIANTS, WALL_THICKNESS, WALL_TINT_OFFSETS,
    WALL_TINT_VARIANTS,
};
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct EwPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    /// Tinted material handles indexed by `[material_kind][tint_index]`.
    /// Same shape as [`super::ns_panel::NsPanelAssets::tinted_mats`].
    pub(crate) tinted_mats:
        [[Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS]; WALL_MATERIAL_VARIANTS],
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
    // E/W-facing panels (sides) — one tinted-material set per wall
    // texture kind, each carrying the WALL_TINT_VARIANTS emissive
    // variants. The active (kind, tint) pair is picked at spawn time
    // from `GameConfig.wall_type` + the per-cell tint hash.
    let tinted_mats: [[Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS];
        WALL_MATERIAL_VARIANTS] = std::array::from_fn(|kind| {
        let spec = &material_specs[kind];
        std::array::from_fn(|tint| {
            let (dr, dg, db) = WALL_TINT_OFFSETS[tint];
            let (br, bg, bb) = spec.emissive;
            materials.as_mut().map(|m| {
                m.add(StandardMaterial {
                    base_color: EMISSIVE_ONLY_BASE,
                    emissive: LinearRgba::new(
                        (br + dr).max(0.0),
                        (bg + dg).max(0.0),
                        (bb + db).max(0.0),
                        1.0,
                    ),
                    emissive_texture: spec.texture.clone(),
                    uv_transform: Affine2::from_scale(spec.uv_scale),
                    ..default()
                })
            })
        })
    });
    let material_mats: [Option<Handle<StandardMaterial>>; WALL_MATERIAL_VARIANTS] =
        std::array::from_fn(|i| {
            let spec = &material_specs[i];
            let (r, g, b) = spec.emissive;
            materials.as_mut().map(|m| {
                m.add(StandardMaterial {
                    base_color: EMISSIVE_ONLY_BASE,
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
    tag: LevelTag,
    assets: &EwPanelAssets,
    kind: usize,
    tint: usize,
    pos: Vec3,
) {
    spawn_ew_panel(
        commands,
        tag,
        assets.mesh.clone(),
        assets.tinted_mats[kind][tint].clone(),
        pos,
    );
}

pub(crate) fn spawn_ew_face_material(
    commands: &mut Commands,
    tag: LevelTag,
    assets: &EwPanelAssets,
    kind: usize,
    pos: Vec3,
) {
    spawn_ew_panel(commands, tag, assets.mesh.clone(), assets.material_mats[kind].clone(), pos);
}

fn spawn_ew_panel(
    commands: &mut Commands,
    tag: LevelTag,
    mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
    pos: Vec3,
) {
    match (mesh, mat) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                WallCell,
                tag,
                Transform::from_translation(pos),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((WallCell, tag, Transform::from_translation(pos)));
        }
    }
}
