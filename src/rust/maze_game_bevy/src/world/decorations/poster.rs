use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Faded poster — bright at the top fading to mid at the bottom, with a dark
/// frame and a few horizontal "tears" for character. Material tints warm
/// orange.
pub(crate) fn make_poster_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let frame = !(4..W - 4).contains(&x) || !(4..H - 4).contains(&y);
            let in_tear = matches!(y, 18 | 19 | 36 | 37 | 48);
            let v: u8 = if frame {
                25
            } else if in_tear {
                90
            } else {
                let t = y as f32 / H as f32;
                (200.0 - t * 90.0) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_poster_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_poster_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.65, 0.40, 0.18, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
