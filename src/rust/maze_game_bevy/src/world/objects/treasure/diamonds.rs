//! Diamonds treasure loot — a mound of clear faceted gems inside the open
//! chest, with a bright cyan-white sparkle.

use super::{gem_material, spawn_gem_loot, spawn_sparkles, sparkle_material, LootContext};
use bevy::prelude::*;

pub(crate) struct DiamondsAssets {
    gem_mat: Option<Handle<StandardMaterial>>,
    sparkle_mats: Vec<Option<Handle<StandardMaterial>>>,
}

pub(crate) fn build_diamonds_assets(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> DiamondsAssets {
    DiamondsAssets {
        gem_mat: gem_material(
            materials,
            Color::srgb(0.75, 0.95, 1.0),
            LinearRgba::new(0.40, 0.90, 1.10, 1.0),
        ),
        sparkle_mats: vec![sparkle_material(materials, Color::srgb(0.45, 0.71, 0.75))],
    }
}

pub(crate) fn spawn_diamonds(
    commands: &mut Commands,
    root: Entity,
    assets: &DiamondsAssets,
    ctx: &LootContext,
) {
    // One material reused across every gem colour group.
    spawn_gem_loot(commands, root, ctx, std::slice::from_ref(&assets.gem_mat));
    spawn_sparkles(commands, root, ctx.sparkle_mesh, &assets.sparkle_mats, ctx.ray_count);
}
