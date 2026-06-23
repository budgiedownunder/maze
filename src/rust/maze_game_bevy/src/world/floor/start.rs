use super::FloorAssets;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::LevelPlacement;
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Start-cell emissive RGB — saturated green, the universal "start"
/// colour cue, dim enough not to wash out the brick-tile texture beneath.
const START_EMISSIVE: LinearRgba = LinearRgba::new(0.0, 0.6, 0.0, 1.0);
/// UV repeat across the cell, matched to the regular floor tile so the
/// start cell reads as a coloured variant of the same surface.
const START_UV_SCALE: Vec2 = Vec2::new(2.0, 2.0);

#[derive(Component)]
pub(crate) struct StartCell;

pub(crate) fn build_start_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tile_tex: &Option<Handle<Image>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: START_EMISSIVE,
            emissive_texture: tile_tex.clone(),
            uv_transform: Affine2::from_scale(START_UV_SCALE),
            ..default()
        })
    })
}

pub(crate) fn spawn_start(
    commands: &mut Commands,
    assets: &FloorAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    // Coloured green top over a plain-stone underside cap, so the start cell shows
    // from above but reads as ordinary floor from the level below.
    super::spawn_capped_tile(commands, assets, assets.start_mat.clone(), StartCell, r, c, placement);
}
