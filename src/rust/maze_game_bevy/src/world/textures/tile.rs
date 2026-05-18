use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Tile face width (texels).
const TILE: u32 = 30;
/// Grout gap width between tiles (texels).
const GROUT: u32 = 2;
const UNIT: u32 = TILE + GROUT;

const GROUT_INTENSITY: u8 = 35;
/// Base brightness for tile interior; per-tile hash adds up to
/// [`TILE_NOISE_RANGE`] for subtle per-tile variation.
const TILE_BASE_INTENSITY: u32 = 185;
const TILE_NOISE_RANGE: u32 = 55;

pub(crate) fn make_tile_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let xm = x % UNIT;
            let ym = y % UNIT;
            let is_grout = xm >= TILE || ym >= TILE;
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_grout {
                GROUT_INTENSITY
            } else {
                let tx = x / UNIT;
                let ty = y / UNIT;
                let hash = tx
                    .wrapping_mul(17)
                    .wrapping_add(ty.wrapping_mul(31))
                    .wrapping_add(xm.wrapping_mul(3))
                    .wrapping_add(ym.wrapping_mul(5));
                (TILE_BASE_INTENSITY + hash % TILE_NOISE_RANGE) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
