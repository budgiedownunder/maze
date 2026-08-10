pub(crate) mod bag;
pub(crate) mod clock;
pub(crate) mod diagnostics;
pub(crate) mod hp;
pub(crate) mod level;
pub(crate) mod minimap;
pub(crate) mod score;
pub(crate) mod statusbar;
pub(crate) mod time_bonus;

/// Uniform downscale for the top-edge readouts (SCORE and the clock) on narrow
/// windows. The SCORE is anchored to the left and grows rightward while the
/// clock is centred, so on a thin (phone-portrait) width the two collide;
/// shrinking both toward their own anchors opens a gap without moving either to
/// a new row. Full size at `HUD_FULL_WIDTH` and above, clamped to
/// `HUD_MIN_SCALE` on very thin widths.
pub(crate) fn hud_scale(window_width: f32) -> f32 {
    const HUD_FULL_WIDTH: f32 = 620.0;
    const HUD_MIN_SCALE: f32 = 0.6;
    (window_width / HUD_FULL_WIDTH).clamp(HUD_MIN_SCALE, 1.0)
}

#[cfg(test)]
mod tests {
    use super::hud_scale;

    #[test]
    fn hud_scale_is_full_size_on_wide_windows() {
        assert_eq!(hud_scale(620.0), 1.0);
        assert_eq!(hud_scale(1280.0), 1.0);
    }

    #[test]
    fn hud_scale_shrinks_on_thin_windows_and_clamps() {
        // A phone-portrait width shrinks but stays above the floor.
        let phone = hud_scale(390.0);
        assert!(phone > 0.6 && phone < 1.0, "got {phone}");
        // Extremely thin widths clamp to the floor rather than vanishing.
        assert_eq!(hud_scale(100.0), 0.6);
    }
}
