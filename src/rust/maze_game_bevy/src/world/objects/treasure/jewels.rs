//! Jewels treasure loot — a mound of multi-coloured faceted gems inside the
//! open chest (ruby / emerald / sapphire / amethyst, one per gem colour group),
//! with sparkles cycling through the same palette.

use super::{gem_material, spawn_gem_loot, spawn_sparkles, sparkle_material, LootContext};
use bevy::prelude::*;

pub(crate) struct JewelsAssets {
    gem_mats: Vec<Option<Handle<StandardMaterial>>>,
    sparkle_mats: Vec<Option<Handle<StandardMaterial>>>,
}

pub(crate) fn build_jewels_assets(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> JewelsAssets {
    JewelsAssets {
        gem_mats: vec![
            gem_material(materials, Color::srgb(0.90, 0.10, 0.20), LinearRgba::new(0.50, 0.05, 0.10, 1.0)),
            gem_material(materials, Color::srgb(0.10, 0.80, 0.30), LinearRgba::new(0.05, 0.40, 0.15, 1.0)),
            gem_material(materials, Color::srgb(0.15, 0.35, 0.95), LinearRgba::new(0.06, 0.15, 0.50, 1.0)),
            gem_material(materials, Color::srgb(0.70, 0.30, 0.95), LinearRgba::new(0.35, 0.12, 0.50, 1.0)),
        ],
        sparkle_mats: vec![
            sparkle_material(materials, Color::srgb(0.75, 0.23, 0.30)),
            sparkle_material(materials, Color::srgb(0.23, 0.75, 0.41)),
            sparkle_material(materials, Color::srgb(0.34, 0.45, 0.75)),
            sparkle_material(materials, Color::srgb(0.68, 0.38, 0.75)),
        ],
    }
}

pub(crate) fn spawn_jewels(
    commands: &mut Commands,
    root: Entity,
    assets: &JewelsAssets,
    ctx: &LootContext,
) {
    spawn_gem_loot(commands, root, ctx, &assets.gem_mats);
    spawn_sparkles(commands, root, ctx.sparkle_mesh, &assets.sparkle_mats);
}
