use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Width of each concentric band step (texels).
const BAND_WIDTH: u32 = 6;
/// Grout brightness between bands (positions 0 and BAND_WIDTH-1 of each
/// band cycle).
const GROUT_INTENSITY: u8 = 40;
/// Brightness of even-indexed bands.
const BAND_BRIGHT_INTENSITY: u8 = 200;
/// Brightness of odd-indexed bands (contrast against the bright bands).
const BAND_DIM_INTENSITY: u8 = 140;

/// Mosaic emissive RGB — warm terracotta.
const MOSAIC_EMISSIVE: LinearRgba = LinearRgba::new(0.70, 0.40, 0.25, 1.0);
const MOSAIC_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Mosaic — 4-fold symmetric pattern of concentric squares with darker
/// grout lines between bands. Material tints warm terracotta so it reads
/// as a decorative tile mosaic at a junction.
pub(crate) fn make_mosaic_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let cx = (W as f32 - 1.0) / 2.0;
    let cy = (H as f32 - 1.0) / 2.0;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    // Concentric square bands: pixel intensity steps every BAND_WIDTH texels
    // outward from the centre, with dark grout between bands.
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let d = (x as f32 - cx).abs().max((y as f32 - cy).abs());
            let band = (d / BAND_WIDTH as f32) as u32;
            let in_band = (d as u32) % BAND_WIDTH;
            let v: u8 = if in_band == 0 || in_band == BAND_WIDTH - 1 {
                // grout
                GROUT_INTENSITY
            } else if band.is_multiple_of(2) {
                BAND_BRIGHT_INTENSITY
            } else {
                BAND_DIM_INTENSITY
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_mosaic_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_mosaic_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: MOSAIC_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(MOSAIC_UV),
            ..default()
        })
    })
}
