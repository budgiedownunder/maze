use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Texture resolution (doubled from the original 64×64 monochrome
/// version so per-plank tone variation has room to breathe).
const W: u32 = 128;
const H: u32 = 128;
/// Vertical plank width (texels). At W=128 / PLANK_W=32 the texture
/// holds 4 planks across — matches the look of the original 64×64
/// version once tiled.
const PLANK_W: u32 = 32;
/// Dark inter-plank seam width (texels).
const SEAM: u32 = 4;
/// Grain stripe period within a plank — every Nth column gets a
/// darker grain line.
const GRAIN_PERIOD: u32 = 8;

/// Structural greyscale intensities (8-bit). These drive the *brightness*
/// channel; the per-plank tone palette below provides the chromaticity.
const SEAM_INTENSITY: u8 = 30;
/// Brightness on a grain column (darker than the plank base).
const GRAIN_INTENSITY: u32 = 110;
/// Brightness on a non-grain plank column.
const PLANK_INTENSITY: u32 = 200;
/// Subtle per-row noise added to the structural brightness.
const PLANK_NOISE_RANGE: u32 = 30;

/// Per-plank tone palette — RGB peak chromaticity at full brightness.
/// Each plank picks one entry by hash, so adjacent planks read as
/// different cuts of timber (honey, oak, walnut, dark walnut).
const WOOD_TONES: [(u32, u32, u32); 4] = [
    (220, 165, 105), // light honey
    (190, 135, 80),  // medium oak
    (160, 105, 60),  // walnut
    (130, 85, 50),   // dark walnut
];

pub(crate) fn make_wood_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let plank = x / PLANK_W;
            let x_in_plank = x % PLANK_W;
            let is_seam = x_in_plank >= PLANK_W - SEAM;
            let idx = ((y * W + x) * 4) as usize;
            // Structural brightness (0-255) — seams are darkest, grain
            // lines are darker than the plank body, then per-row noise
            // adds character without disturbing the plank silhouette.
            let v: u32 = if is_seam {
                SEAM_INTENSITY as u32
            } else {
                let on_grain = (x_in_plank + plank * 5).is_multiple_of(GRAIN_PERIOD);
                let base = if on_grain { GRAIN_INTENSITY } else { PLANK_INTENSITY };
                let noise = y.wrapping_mul(7).wrapping_add(plank.wrapping_mul(23));
                base + noise % PLANK_NOISE_RANGE
            };
            // Per-plank tone — hash decorrelated from the noise hash so
            // tone and structural shading don't covary.
            let tone_hash = plank.wrapping_mul(31).wrapping_add(11);
            let tone = WOOD_TONES[(tone_hash as usize) % WOOD_TONES.len()];
            let brightness = v as f32 / 255.0;
            pixels[idx] = (tone.0 as f32 * brightness).round() as u8;
            pixels[idx + 1] = (tone.1 as f32 * brightness).round() as u8;
            pixels[idx + 2] = (tone.2 as f32 * brightness).round() as u8;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
