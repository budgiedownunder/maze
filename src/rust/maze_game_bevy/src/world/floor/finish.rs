use super::FloorAssets;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::LevelPlacement;
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
    // Coloured top over a plain-stone underside cap, so the finish cell shows from
    // above but reads as ordinary floor from the level below.
    super::spawn_capped_tile(commands, assets, assets.finish_mat.clone(), FinishCell, r, c, placement);
}
