use crate::images::make_image;
use bevy::prelude::*;

pub(crate) fn make_cobblestone_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    // Disc-shaped bright cobbles at hash-driven centres on a dark grout
    // background. A 4x4 grid of cells, one cobble per cell, with jittered
    // centres and radii so the cobbles read as irregular rather than tiled.
    const CELLS: u32 = 4;
    const CELL: u32 = W / CELLS;
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let cx = x / CELL;
            let cy = y / CELL;
            // Deterministic jitter in the cobble's centre and radius based on
            // its cell index.
            let h = cx
                .wrapping_mul(73)
                .wrapping_add(cy.wrapping_mul(131))
                .wrapping_add(17);
            let jitter_x = (h % 5) as i32 - 2;
            let jitter_y = ((h / 5) % 5) as i32 - 2;
            let radius = 5 + ((h / 25) % 3) as i32;
            let centre_x = (cx * CELL + CELL / 2) as i32 + jitter_x;
            let centre_y = (cy * CELL + CELL / 2) as i32 + jitter_y;
            let dx = x as i32 - centre_x;
            let dy = y as i32 - centre_y;
            let dist_sq = dx * dx + dy * dy;
            let idx = ((y * W + x) * 4) as usize;
            let v = if dist_sq <= radius * radius {
                // Brighter centre, slightly darker rim — fake-shaded disc.
                let normalised = dist_sq as u32 * 60 / (radius * radius) as u32;
                (190u32 - normalised.min(60)) as u8
            } else {
                35
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
