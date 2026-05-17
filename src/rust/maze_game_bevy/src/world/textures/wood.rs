use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Vertical plank width (texels).
const PLANK_W: u32 = 16;
/// Dark inter-plank seam width (texels).
const SEAM: u32 = 2;
/// Grain stripe period within a plank — every Nth column gets a
/// darker grain line.
const GRAIN_PERIOD: u32 = 4;

const SEAM_INTENSITY: u8 = 30;
/// Base brightness on a grain column (darker than the plank base).
const GRAIN_INTENSITY: u32 = 95;
/// Base brightness on a non-grain plank column.
const PLANK_INTENSITY: u32 = 155;
/// Subtle per-row noise added to the base for variation.
const PLANK_NOISE_RANGE: u32 = 25;

pub(crate) fn make_wood_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let plank = x / PLANK_W;
            let x_in_plank = x % PLANK_W;
            let is_seam = x_in_plank >= PLANK_W - SEAM;
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_seam {
                SEAM_INTENSITY
            } else {
                let on_grain = (x_in_plank + plank * 5).is_multiple_of(GRAIN_PERIOD);
                let base = if on_grain { GRAIN_INTENSITY } else { PLANK_INTENSITY };
                let hash = y.wrapping_mul(7).wrapping_add(plank.wrapping_mul(23));
                (base + hash % PLANK_NOISE_RANGE) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
