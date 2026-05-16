use crate::images::make_image;
use bevy::prelude::*;

pub(crate) fn make_wood_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    // Vertical wood-grain planks. PLANK_W picks the plank width;
    // SEAM is the dark inter-plank seam in pixels.
    const PLANK_W: u32 = 16;
    const SEAM: u32 = 2;
    // Grain stripe period within a plank — every GRAIN_PERIOD columns one
    // darker grain line runs the full height.
    const GRAIN_PERIOD: u32 = 4;

    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let plank = x / PLANK_W;
            let x_in_plank = x % PLANK_W;
            let is_seam = x_in_plank >= PLANK_W - SEAM;
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_seam {
                30
            } else {
                let on_grain = (x_in_plank + plank * 5).is_multiple_of(GRAIN_PERIOD);
                let base = if on_grain { 95u32 } else { 155u32 };
                // Subtle per-row noise so the planks aren't perfectly flat.
                let hash = y.wrapping_mul(7).wrapping_add(plank.wrapping_mul(23));
                (base + hash % 25) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
