use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Texture resolution (square; consumed via UV-wrap on wall panels).
const W: u32 = 64;
const H: u32 = 64;
/// Brick face dimensions in texels.
const BRICK_W: u32 = 30;
const BRICK_H: u32 = 14;
/// Mortar joint width between bricks (texels).
const MORTAR: u32 = 2;
/// Row / column pitch derived from brick + mortar.
const ROW_H: u32 = BRICK_H + MORTAR;
const COL_W: u32 = BRICK_W + MORTAR;

/// Brightness (8-bit greyscale) for mortar pixels.
const MORTAR_INTENSITY: u8 = 35;
/// Base brightness for brick interior pixels; per-brick hash adds up to
/// [`BRICK_NOISE_RANGE`] for subtle per-brick variation.
const BRICK_BASE_INTENSITY: u32 = 200;
const BRICK_NOISE_RANGE: u32 = 45;

pub(crate) fn make_brick_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let row = y / ROW_H;
            let y_in_row = y % ROW_H;
            let is_mortar = if y_in_row >= BRICK_H {
                true
            } else {
                let offset = if row.is_multiple_of(2) { 0 } else { COL_W / 2 };
                (x + offset) % COL_W >= BRICK_W
            };
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_mortar {
                MORTAR_INTENSITY
            } else {
                let bx = (x + if row.is_multiple_of(2) { 0 } else { COL_W / 2 }) % COL_W;
                let by = y % ROW_H;
                let hash = bx
                    .wrapping_mul(7)
                    .wrapping_add(by.wrapping_mul(13))
                    .wrapping_add(row.wrapping_mul(31));
                (BRICK_BASE_INTENSITY + hash % BRICK_NOISE_RANGE) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
