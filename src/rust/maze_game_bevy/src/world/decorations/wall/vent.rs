use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Dark stone frame width (texels) around the vent grate.
const FRAME: u32 = 3;
/// Vertical pitch between slats (texels).
const BAND_PITCH: u32 = 12;
/// Slat thickness in the pitch (texels). `BAND_PITCH - SLAT_THICKNESS`
/// is the gap between slats.
const SLAT_THICKNESS: u32 = 8;

/// Frame brightness (8-bit greyscale).
const FRAME_INTENSITY: u8 = 10;
/// Slat brightness.
const SLAT_INTENSITY: u8 = 180;
/// Inter-slat gap brightness.
const GAP_INTENSITY: u8 = 20;

/// Vent emissive RGB — cool grey with a subtle blue lean.
const VENT_EMISSIVE: LinearRgba = LinearRgba::new(0.30, 0.30, 0.32, 1.0);
/// UV scale (1.0 = one decoration covers the panel once).
const VENT_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Vent grate — horizontal slats with thin dark frame. 64×64 monochrome
/// grayscale; pixel intensity drives the emissive channel, which the vent
/// material then tints cool grey.
pub(crate) fn make_vent_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let frame = !(FRAME..W - FRAME).contains(&x) || !(FRAME..H - FRAME).contains(&y);
            let inside_slat = (y % BAND_PITCH) < SLAT_THICKNESS;
            let v: u8 = if frame {
                FRAME_INTENSITY
            } else if inside_slat {
                SLAT_INTENSITY
            } else {
                GAP_INTENSITY
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
            base_color: EMISSIVE_ONLY_BASE,
            emissive: VENT_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(VENT_UV),
            ..default()
        })
    })
}
