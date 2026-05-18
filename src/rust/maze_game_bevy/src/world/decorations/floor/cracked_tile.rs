use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_3, PI};

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Six radial cracks at evenly spaced angles around the centre.
const CRACK_ANGLES: [f32; 6] = [
    0.0,
    FRAC_PI_3,
    FRAC_PI_3 * 2.0,
    PI,
    PI + FRAC_PI_3,
    PI + FRAC_PI_3 * 2.0,
];

/// Half-width of a crack stroke in texels (`< 1.2` matches the original
/// hand-tuned value — anything wider reads as a groove not a crack).
const CRACK_HALF_WIDTH: f32 = 1.2;
/// Maximum distance from the centre at which a crack still renders
/// (texels). Cracks fade out past this.
const CRACK_MAX_DIST: f32 = 28.0;
/// Base intensity at the crack centre; dims with distance from centre.
const CRACK_BASE_INTENSITY: f32 = 180.0;
const CRACK_FADE_PER_TEXEL: f32 = 2.0;
const CRACK_INTENSITY_MIN: f32 = 60.0;
const CRACK_INTENSITY_MAX: f32 = 220.0;

/// Base intensity of the tile surface (centre of the noise range).
const TILE_BASE_INTENSITY: i32 = 140;
/// Per-pixel noise width: tile intensity is `TILE_BASE_INTENSITY ± TILE_NOISE_HALF_RANGE`.
const TILE_NOISE_HALF_RANGE: i32 = 12;

/// Cracked-tile emissive RGB — cool stone grey.
const CRACKED_TILE_EMISSIVE: LinearRgba = LinearRgba::new(0.50, 0.50, 0.48, 1.0);
const CRACKED_TILE_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Cracked tile — stone-grey base with thin radial cracks emanating from a
/// central impact point. Material tints cool grey.
pub(crate) fn make_cracked_tile_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            // Distance to nearest crack line (projection of (dx,dy) onto a unit
            // vector perpendicular to each crack direction).
            let mut nearest = f32::INFINITY;
            for a in CRACK_ANGLES {
                // Perpendicular distance to the crack line through the centre at angle `a`.
                let perp = (dx * -a.sin() + dy * a.cos()).abs();
                // Only count the half-ray pointing in direction `a`, not the other side.
                let along = dx * a.cos() + dy * a.sin();
                if along >= 0.0 {
                    nearest = nearest.min(perp);
                }
            }
            // Cracks are 1 px wide, brighter near the centre (less worn).
            let on_crack = nearest < CRACK_HALF_WIDTH && dist < CRACK_MAX_DIST;
            let v: u8 = if on_crack {
                (CRACK_BASE_INTENSITY - dist * CRACK_FADE_PER_TEXEL)
                    .clamp(CRACK_INTENSITY_MIN, CRACK_INTENSITY_MAX) as u8
            } else {
                // Base tile: gentle brightness variation around the base.
                let n = (((x.wrapping_mul(13)).wrapping_add(y.wrapping_mul(7))) % 25) as i32
                    - TILE_NOISE_HALF_RANGE;
                (TILE_BASE_INTENSITY + n).clamp(0, 255) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_cracked_tile_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_cracked_tile_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: CRACKED_TILE_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(CRACKED_TILE_UV),
            ..default()
        })
    })
}
