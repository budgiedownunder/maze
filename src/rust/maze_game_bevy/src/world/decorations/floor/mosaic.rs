use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Mosaic — 4-fold symmetric pattern of concentric squares with darker
/// grout lines between bands. Material tints warm terracotta so it reads
/// as a decorative tile mosaic at a junction.
pub(crate) fn make_mosaic_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let cx = (W as f32 - 1.0) / 2.0;
    let cy = (H as f32 - 1.0) / 2.0;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    // Concentric square bands: pixel intensity steps every 6 cells outward
    // from the centre, with dark grout between bands.
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let d = (x as f32 - cx).abs().max((y as f32 - cy).abs());
            let band = (d / 6.0) as u32;
            let in_band = (d as u32) % 6;
            let v: u8 = if in_band == 0 || in_band == 5 {
                // grout
                40
            } else {
                // Bands alternate between two brightness steps so adjacent
                // bands read distinctly under the warm tint.
                if band.is_multiple_of(2) {
                    200
                } else {
                    140
                }
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
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.70, 0.40, 0.25, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
