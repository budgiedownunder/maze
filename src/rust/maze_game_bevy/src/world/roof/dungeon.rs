//! Hewn dark-rock cave ceiling for [`crate::state::SkyType::Dungeon`].
//!
//! A greyscale rock texture (see [`crate::world::textures::rock`]) tinted by a
//! dim emissive, shared by every ceiling tile, so the player reads as sealed
//! inside natural stone.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::textures::rock::make_rock_texture;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Dark-rock emissive tint, multiplied by the greyscale rock texture
/// (`emissive_texture`). Dimmer than the head-on wall panels (~0.4) — the
/// ceiling is overhead and shadowed — but far brighter than the near-black
/// dome behind it, so it reads as a solid surface rather than open sky.
/// Slightly warm. Emissive-only (paired with [`EMISSIVE_ONLY_BASE`]) like
/// every other world material, so corridor lighting does not multiply into it.
const ROCK_EMISSIVE: LinearRgba = LinearRgba::rgb(0.30, 0.27, 0.24);

/// Texture tiles per cell. One rock face per cell keeps the blotches large and
/// rocky rather than busy.
const ROCK_UV_SCALE: f32 = 1.0;

/// Builds the single dark-rock material shared by every dungeon ceiling tile.
pub(crate) fn build_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let rock_tex = images.as_mut().map(|imgs| make_rock_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: ROCK_EMISSIVE,
            emissive_texture: rock_tex,
            uv_transform: Affine2::from_scale(Vec2::splat(ROCK_UV_SCALE)),
            ..default()
        })
    })
}
