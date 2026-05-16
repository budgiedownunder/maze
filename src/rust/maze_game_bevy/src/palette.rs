use bevy::prelude::*;

// Cross-module colours. Sub-module-local palettes (minimap, statusbar,
// title-shadow, win-gold, lose-red, weather effects) stay inside their
// respective modules.

pub(crate) const COLOR_GOLD: Color = Color::srgb(1.0, 0.75, 0.1);
pub(crate) const COLOR_OVERLAY_BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.75);

const COLOR_CLOCK_WARN: Color = Color::srgb(0.95, 0.25, 0.2);

pub(crate) const CLOCK_GOLD: Color = COLOR_GOLD;
pub(crate) const CLOCK_RED: Color = COLOR_CLOCK_WARN;
