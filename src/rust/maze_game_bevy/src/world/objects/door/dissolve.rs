//! The dissolve door rig — the leaf fades out in place instead of moving. To
//! avoid fading the *shared* wall material, each dissolve leaf renders with its
//! own cloned, alpha-blended copy; the fade drives both its alpha and emissive
//! toward zero over the open countdown. The orchestrator in [`super`] builds the
//! per-leaf material at spawn and calls [`apply`] each frame.

use bevy::prelude::*;

/// Clones the source material into an alpha-blended copy for a dissolve leaf,
/// returning `(clone handle, base emissive)`, or `None` when the material assets
/// aren't available (e.g. headless tests) or the source is missing. Used for the
/// panel and the keyhole pieces so they all fade together.
pub(crate) fn clone_blend(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    src: &Option<Handle<StandardMaterial>>,
) -> Option<(Handle<StandardMaterial>, LinearRgba)> {
    let mats = materials.as_mut()?;
    let src_handle = src.as_ref()?;
    let src_mat = mats.get(src_handle)?;
    let mut clone = src_mat.clone();
    clone.alpha_mode = AlphaMode::Blend;
    let base_emissive = clone.emissive;
    let handle = mats.add(clone);
    Some((handle, base_emissive))
}

/// Applies the dissolve fade to a leaf's material: emissive scales from its base
/// toward black and the base colour's alpha from opaque toward transparent as
/// `fraction` runs `0.0` → `1.0`.
pub(crate) fn apply(mat: &mut StandardMaterial, base_emissive: LinearRgba, fraction: f32) {
    let k = (1.0 - fraction).clamp(0.0, 1.0);
    mat.emissive = LinearRgba::new(
        base_emissive.red * k,
        base_emissive.green * k,
        base_emissive.blue * k,
        1.0,
    );
    mat.base_color = mat.base_color.with_alpha(k);
}
