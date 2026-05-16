use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Window — stone frame with a bright sky-glow centre, divided by a simple
/// cross mullion. Slightly darker at the bottom suggests a horizon line.
/// Material tints sky blue.
pub(crate) fn make_window_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let outer = !(4..W - 4).contains(&x) || !(4..H - 4).contains(&y);
            let inner_frame = !(8..W - 8).contains(&x) || !(8..H - 8).contains(&y);
            let mullion_v = (x as i32 - W as i32 / 2).abs() < 2;
            let mullion_h = (y as i32 - H as i32 / 2).abs() < 2;
            let v: u8 = if outer {
                20
            } else if inner_frame {
                80
            } else if mullion_v || mullion_h {
                60
            } else {
                // Sky gradient: brighter near top, slightly darker below midline.
                let t = (y as f32 - 8.0) / (H as f32 - 16.0);
                (215.0 - t * 50.0) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_window_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_window_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.45, 0.65, 1.00, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
