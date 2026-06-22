//! Gold treasure loot — a mound of gold coins inside the open chest, with a
//! warm golden sparkle.

use super::{metal_material, spawn_coin_loot, spawn_sparkles, sparkle_material, LootContext};
use bevy::prelude::*;

pub(crate) struct GoldAssets {
    coin_mat: Option<Handle<StandardMaterial>>,
    sparkle_mats: Vec<Option<Handle<StandardMaterial>>>,
}

pub(crate) fn build_gold_assets(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> GoldAssets {
    GoldAssets {
        coin_mat: metal_material(
            materials,
            Color::srgb(1.0, 0.82, 0.30),
            LinearRgba::new(0.95, 0.66, 0.18, 1.0),
        ),
        sparkle_mats: vec![sparkle_material(materials, Color::srgb(0.75, 0.62, 0.30))],
    }
}

pub(crate) fn spawn_gold(
    commands: &mut Commands,
    root: Entity,
    assets: &GoldAssets,
    ctx: &LootContext,
) {
    spawn_coin_loot(commands, root, ctx, &assets.coin_mat);
    spawn_sparkles(commands, root, ctx.sparkle_mesh, &assets.sparkle_mats, ctx.ray_count);
}
