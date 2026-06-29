//! Silver treasure loot — a mound of silver coins inside the open chest, with a
//! cool silvery sparkle.

use super::{metal_material, spawn_coin_loot, spawn_sparkles, sparkle_material, LootContext};
use bevy::prelude::*;

pub(crate) struct SilverAssets {
    coin_mat: Option<Handle<StandardMaterial>>,
    sparkle_mats: Vec<Option<Handle<StandardMaterial>>>,
}

pub(crate) fn build_silver_assets(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> SilverAssets {
    SilverAssets {
        // A muted grey-white (dimmer + greyer than the near-white it was) so the
        // silver loot doesn't read like the bright cyan-white diamonds at range.
        coin_mat: metal_material(
            materials,
            Color::srgb(0.64, 0.66, 0.70),
            LinearRgba::new(0.30, 0.32, 0.36, 1.0),
        ),
        sparkle_mats: vec![sparkle_material(materials, Color::srgb(0.60, 0.64, 0.75))],
    }
}

pub(crate) fn spawn_silver(
    commands: &mut Commands,
    root: Entity,
    assets: &SilverAssets,
    ctx: &LootContext,
) {
    spawn_coin_loot(commands, root, ctx, &assets.coin_mat);
    spawn_sparkles(commands, root, ctx.sparkle_mesh, &assets.sparkle_mats, ctx.ray_count);
}
