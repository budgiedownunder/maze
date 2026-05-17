use super::{
    WallCell, PANEL_H, PANEL_W, WALL_MATERIAL_VARIANTS, WALL_THICKNESS, WALL_TINT_OFFSETS,
    WALL_TINT_VARIANTS,
};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct NsPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) tinted_mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS],
    pub(crate) material_mats: [Option<Handle<StandardMaterial>>; WALL_MATERIAL_VARIANTS],
}

/// Per-wall-material emissive base + UV scale, paired with the matching
/// texture handle. One entry per `WALL_MATERIAL_*` index (brick, dressed
/// stone, wood, cobblestone). Values picked at design time so each material
/// reads as visually distinct under the dim corridor lighting.
pub(crate) struct WallMaterialSpec<'a> {
    pub(crate) texture: &'a Option<Handle<Image>>,
    pub(crate) emissive: (f32, f32, f32),
    pub(crate) uv_scale: Vec2,
}

pub(crate) fn build_ns_panel_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    material_specs: &[WallMaterialSpec; WALL_MATERIAL_VARIANTS],
) -> NsPanelAssets {
    // PANEL_W / PANEL_H inset panels from their edges by BORDER_GAP, creating dark gap
    // lines between adjacent coplanar panels and between walls and the floor.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(PANEL_W, PANEL_H, WALL_THICKNESS)));
    // emissive: LinearRgba writes directly to the framebuffer without sRGB conversion or
    // lighting interaction. base_color: BLACK ensures PBR diffuse contributes nothing.
    // N/S-facing panels (ahead/behind) — cool stone grey, with WALL_TINT_VARIANTS
    // emissive variants for per-cell tint variation (picked in the spawn loop).
    let brick_spec = &material_specs[super::WALL_MATERIAL_BRICK];
    let tinted_mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS] =
        std::array::from_fn(|i| {
            let (dr, dg, db) = WALL_TINT_OFFSETS[i];
            let (br, bg, bb) = brick_spec.emissive;
            materials.as_mut().map(|m| {
                m.add(StandardMaterial {
                    base_color: EMISSIVE_ONLY_BASE,
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
                    base_color: EMISSIVE_ONLY_BASE,
                    emissive: LinearRgba::new(r, g, b, 1.0),
                    emissive_texture: spec.texture.clone(),
                    uv_transform: Affine2::from_scale(spec.uv_scale),
                    ..default()
                })
            })
        });
    NsPanelAssets {
        mesh,
        tinted_mats,
        material_mats,
    }
}

pub(crate) fn spawn_ns_face_tinted(
    commands: &mut Commands,
    assets: &NsPanelAssets,
    tint: usize,
    pos: Vec3,
) {
    spawn_ns_panel(commands, assets.mesh.clone(), assets.tinted_mats[tint].clone(), pos);
}

pub(crate) fn spawn_ns_face_material(
    commands: &mut Commands,
    assets: &NsPanelAssets,
    kind: usize,
    pos: Vec3,
) {
    spawn_ns_panel(commands, assets.mesh.clone(), assets.material_mats[kind].clone(), pos);
}

fn spawn_ns_panel(
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
