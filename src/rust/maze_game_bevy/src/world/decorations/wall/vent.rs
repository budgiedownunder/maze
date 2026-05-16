use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Vent grate — five horizontal slats with thin dark frame. 64×64 monochrome
/// grayscale; pixel intensity drives the emissive channel, which the vent
/// material then tints cool grey.
pub(crate) fn make_vent_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let frame = !(3..W - 3).contains(&x) || !(3..H - 3).contains(&y);
            let band_pitch = 12u32;
            let slat_thickness = 8u32;
            let inside_slat = (y % band_pitch) < slat_thickness;
            let v: u8 = if frame {
                10
            } else if inside_slat {
                180
            } else {
                20
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_vent_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_vent_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.30, 0.30, 0.32, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
