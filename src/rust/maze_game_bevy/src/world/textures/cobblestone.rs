use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 128;
const H: u32 = 128;

/// Seed-point grid resolution: 5×5 = 25 stones spread across the
/// texture. Each grid cell contributes one seed; the per-cell jitter
/// below pushes them off-centre so the resulting Voronoi tessellation
/// reads as irregular polygonal stones rather than a hexagonal honeycomb.
const SEED_GRID: u32 = 5;
const CELL: u32 = W / SEED_GRID;

/// Per-seed jitter (texels). The jitter range is wide enough that
/// adjacent stones vary noticeably in size and shape without ever
/// crossing into a neighbour's cell entirely.
const SEED_JITTER: i32 = 8;

/// Boundary width (texels). A pixel counts as on-boundary when the
/// distance to the second-closest seed is within this many texels of
/// the distance to the closest seed. Small values give a hairline
/// "stones cemented together" look; larger values give visible grout.
const BOUNDARY_WIDTH: f32 = 0.7;

/// Per-pixel noise added to each stone's tone (signed, in texel-byte
/// units). `±NOISE_HALF_RANGE` adds natural mottle within a stone
/// without disturbing the per-stone tone palette.
const NOISE_HALF_RANGE: i32 = 10;

/// Stone tone palette — light grey variants clustered tight around
/// neutral, with the tiniest cool/warm shifts so adjacent stones read
/// as different stones rather than the same fill colour. Deliberately
/// no green / moss / brown tones (the previous palette's saturated
/// variety produced a "1970s wallpaper" look that the cemented-stones
/// aesthetic wants to avoid).
const STONE_TONES: [(u32, u32, u32); 5] = [
    (210, 210, 210), // light neutral
    (195, 197, 200), // light cool
    (200, 198, 195), // light warm
    (185, 185, 185), // medium neutral
    (220, 218, 215), // very light, slight warm
];

/// Grout chromaticity — a darker neutral grey that reads as cement
/// between the stones rather than dirt under them.
const GROUT_TONE: (u32, u32, u32) = (110, 110, 110);

pub(crate) fn make_cobblestone_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];

    // Precompute jittered seed points + each seed's tone index. Done
    // once outside the pixel loop so the per-pixel inner loop just
    // walks the precomputed array.
    let seeds: Vec<(i32, i32, usize)> = (0..SEED_GRID)
        .flat_map(|gy| {
            (0..SEED_GRID).map(move |gx| {
                let h_x = gx.wrapping_mul(73).wrapping_add(gy.wrapping_mul(131));
                let h_y = gx.wrapping_mul(53).wrapping_add(gy.wrapping_mul(101));
                let h_tone = gx.wrapping_mul(19).wrapping_add(gy.wrapping_mul(43));
                let jitter_x = (h_x % (SEED_JITTER as u32 * 2 + 1)) as i32 - SEED_JITTER;
                let jitter_y = (h_y % (SEED_JITTER as u32 * 2 + 1)) as i32 - SEED_JITTER;
                let sx = (gx * CELL + CELL / 2) as i32 + jitter_x;
                let sy = (gy * CELL + CELL / 2) as i32 + jitter_y;
                let tone_idx = (h_tone as usize) % STONE_TONES.len();
                (sx, sy, tone_idx)
            })
        })
        .collect();

    let w_i = W as i32;
    let h_i = H as i32;

    for y in 0..H {
        for x in 0..W {
            // Find nearest + second-nearest seed across the seamless
            // (toroidal) texture. Considering the 9 wrap copies of each
            // seed makes the resulting pattern tile cleanly when the
            // wall material samples it.
            let mut closest = f32::INFINITY;
            let mut second = f32::INFINITY;
            let mut closest_idx = 0usize;
            for (i, &(sx, sy, _)) in seeds.iter().enumerate() {
                for wx in [-w_i, 0, w_i] {
                    for wy in [-h_i, 0, h_i] {
                        let dx = (x as i32 - sx - wx) as f32;
                        let dy = (y as i32 - sy - wy) as f32;
                        let d = (dx * dx + dy * dy).sqrt();
                        if d < closest {
                            second = closest;
                            closest = d;
                            closest_idx = i;
                        } else if d < second {
                            second = d;
                        }
                    }
                }
            }

            let idx = ((y * W + x) * 4) as usize;
            let on_boundary = (second - closest) < BOUNDARY_WIDTH;
            let (r, g, b) = if on_boundary {
                GROUT_TONE
            } else {
                let tone = STONE_TONES[seeds[closest_idx].2];
                // Per-pixel noise via a small hash. Same shift to R, G
                // and B so the noise reads as a brightness wobble, not a
                // colour wobble.
                let noise_hash = x.wrapping_mul(7).wrapping_add(y.wrapping_mul(11));
                let noise = (noise_hash as i32 % (NOISE_HALF_RANGE * 2 + 1)) - NOISE_HALF_RANGE;
                let r = (tone.0 as i32 + noise).clamp(0, 255) as u32;
                let g = (tone.1 as i32 + noise).clamp(0, 255) as u32;
                let b = (tone.2 as i32 + noise).clamp(0, 255) as u32;
                (r, g, b)
            };
            pixels[idx] = r as u8;
            pixels[idx + 1] = g as u8;
            pixels[idx + 2] = b as u8;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
