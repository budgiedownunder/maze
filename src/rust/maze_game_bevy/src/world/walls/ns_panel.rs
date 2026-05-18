use super::{
    WallCell, PANEL_H, PANEL_W, WALL_MATERIAL_VARIANTS, WALL_THICKNESS, WALL_TINT_OFFSETS,
    WALL_TINT_VARIANTS,
};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct NsPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    /// Tinted material handles indexed by `[material_kind][tint_index]`
    /// so the per-cell tinted path can render any of the four wall
    /// textures (brick / dressed_stone / wood / cobblestone) under each
    /// of the six per-cell tint variants.
    pub(crate) tinted_mats:
        [[Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS]; WALL_MATERIAL_VARIANTS],
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
    // N/S-facing panels (ahead/behind) — one tinted-material set per
    // wall texture kind, each carrying the WALL_TINT_VARIANTS emissive
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
    NsPanelAssets {
        mesh,
        tinted_mats,
        material_mats,
    }
}

pub(crate) fn spawn_ns_face_tinted(
    commands: &mut Commands,
    assets: &NsPanelAssets,
    kind: usize,
    tint: usize,
    pos: Vec3,
) {
    spawn_ns_panel(
        commands,
        assets.mesh.clone(),
        assets.tinted_mats[kind][tint].clone(),
        pos,
    );
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
