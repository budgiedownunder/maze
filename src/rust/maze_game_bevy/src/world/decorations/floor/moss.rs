use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Moss patch — dark base with irregular soft-edged bright patches placed
/// pseudo-randomly across the texture. Material tints muted green so the
/// patches read as moss growing on the floor at a junction.
pub(crate) fn make_moss_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    // A handful of soft circular blobs at fixed pseudo-random centres.
    // Hard-coded so the texture is fully deterministic and small to compile.
    let blobs: [(f32, f32, f32); 8] = [
        (12.0, 14.0, 7.0),
        (40.0, 18.0, 9.0),
        (28.0, 32.0, 6.0),
        (52.0, 38.0, 8.0),
        (18.0, 46.0, 7.0),
        (44.0, 54.0, 9.0),
        (8.0, 30.0, 5.0),
        (34.0, 8.0, 5.0),
    ];
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let mut peak: f32 = 0.0;
            for (bx, by, br) in blobs {
                let dx = x as f32 - bx;
                let dy = y as f32 - by;
                let d = (dx * dx + dy * dy).sqrt();
                if d < br {
                    // Soft-edged falloff: brightest at the centre, fading out.
                    let t = 1.0 - d / br;
                    peak = peak.max(t * 200.0);
                }
            }
            let v: u8 = (30.0 + peak).clamp(0.0, 255.0) as u8;
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_moss_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_moss_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.30, 0.65, 0.25, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
