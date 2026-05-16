use super::{WallCell, PANEL_H, PANEL_W, WALL_THICKNESS, WALL_TINT_OFFSETS, WALL_TINT_VARIANTS};
use bevy::math::Affine2;
use bevy::prelude::*;

pub(crate) struct NsPanelAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS],
}

pub(crate) fn build_ns_panel_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    brick_tex: &Option<Handle<Image>>,
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
    let mats: [Option<Handle<StandardMaterial>>; WALL_TINT_VARIANTS] = std::array::from_fn(|i| {
        let (dr, dg, db) = WALL_TINT_OFFSETS[i];
        materials.as_mut().map(|m| {
            m.add(StandardMaterial {
                base_color: Color::BLACK,
                emissive: LinearRgba::new(
                    (0.38 + dr).max(0.0),
                    (0.38 + dg).max(0.0),
                    (0.40 + db).max(0.0),
                    1.0,
                ),
                emissive_texture: brick_tex.clone(),
                uv_transform: Affine2::from_scale(Vec2::new(3.0, 5.0)),
                ..default()
            })
        })
    });
    NsPanelAssets { mesh, mats }
}

pub(crate) fn spawn_ns_face(
    commands: &mut Commands,
    assets: &NsPanelAssets,
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
