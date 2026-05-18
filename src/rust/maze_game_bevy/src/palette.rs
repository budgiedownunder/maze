use bevy::prelude::*;

// Cross-module colours. Sub-module-local palettes (minimap, statusbar,
// title-shadow, win-gold, lose-red, weather effects) stay inside their
// respective modules.

pub(crate) const COLOR_GOLD: Color = Color::srgb(1.0, 0.75, 0.1);
pub(crate) const COLOR_OVERLAY_BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.75);

const COLOR_CLOCK_WARN: Color = Color::srgb(0.95, 0.25, 0.2);

pub(crate) const CLOCK_GOLD: Color = COLOR_GOLD;
pub(crate) const CLOCK_RED: Color = COLOR_CLOCK_WARN;

// ---------- Structural StandardMaterial base_color constants ----------
//
// Every emissive-only material in the world pipeline pairs an
// expressive `emissive` value with one of the two structural base
// colours below. Naming them documents the *role*, makes the pattern
// findable, and prevents the next person from "tuning" a value that
// would silently break the material.

/// Disables the PBR diffuse pathway on an emissive material. Output
/// reduces to the material's `emissive` field; corridor lighting no
/// longer multiplies into the colour. Used by every `world/walls`,
/// `world/floor`, `world/decorations`, and `world/objects` material.
pub(crate) const EMISSIVE_ONLY_BASE: Color = Color::BLACK;

/// Maximum-brightness output for `unlit: true` materials — the unlit
/// pipeline returns `base_color * base_color_texture` directly without
/// any lighting interaction, so WHITE means "show the texture (or solid
/// fill) at full intensity". Used by the sky dome and the 3D star
/// entities.
pub(crate) const UNLIT_FULL_BRIGHT: Color = Color::WHITE;
