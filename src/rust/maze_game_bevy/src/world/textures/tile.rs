use crate::images::make_image;
use bevy::prelude::*;

pub(crate) fn make_tile_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    const TILE: u32 = 30;
    const GROUT: u32 = 2;
    const UNIT: u32 = TILE + GROUT;

    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let xm = x % UNIT;
            let ym = y % UNIT;
            let is_grout = xm >= TILE || ym >= TILE;
            let idx = ((y * W + x) * 4) as usize;
            let v = if is_grout {
                35
            } else {
                let tx = x / UNIT;
                let ty = y / UNIT;
                let hash = tx
                    .wrapping_mul(17)
                    .wrapping_add(ty.wrapping_mul(31))
                    .wrapping_add(xm.wrapping_mul(3))
                    .wrapping_add(ym.wrapping_mul(5));
                (185u32 + hash % 55) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
