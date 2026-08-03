//! Developer diagnostics readout, shown below the minimap's dimensions strip
//! when `GameConfig::debug_memory` is set (the host page's `/game/?mem=1`).
//!
//! Reports visible meshes against total, memory, and the resident asset counts,
//! so render load can be told apart from heap growth while playing.

use crate::hud::minimap::{
    minimap_dimensions_y, MINIMAP_DIM_STRIP_H, MINIMAP_EDGE_MARGIN, MINIMAP_PANEL_BG,
    MINIMAP_PANEL_TEXT,
};
use crate::state::GameConfig;
use bevy::prelude::*;
use bevy::sprite::Anchor;

const DIAG_FONT: f32 = 14.0;
const DIAG_GAP: f32 = 2.0;
/// One metric per row, each `<label> <value>` — short enough to fit the
/// minimap's width without overflowing the screen edge.
const DIAG_ROWS: f32 = 7.0;
const DIAG_LINE_H: f32 = 16.0;
const DIAG_PAD_Y: f32 = 8.0;
const DIAG_STRIP_H: f32 = DIAG_ROWS * DIAG_LINE_H + DIAG_PAD_Y;
/// The background must cover the rows it sits behind — enforced here rather than
/// in a test, since both sides are constants and the compiler can settle it.
const _: () = assert!(DIAG_STRIP_H >= DIAG_ROWS * DIAG_LINE_H);
/// Inset of the text from the strip's left edge, so the rows align down the
/// left of the minimap column rather than centring under it.
const DIAG_TEXT_PAD_X: f32 = 6.0;

/// How often the readout recomputes and rewrites its text. Not every frame:
/// re-laying out a text node that often costs enough to move the numbers being
/// measured.
const DIAG_UPDATE_SECS: f32 = 0.25;

/// Smoothing factor for the frame-rate estimate — a plain exponential moving
/// average, so a single slow frame doesn't make the figure jump. Avoids pulling
/// in `FrameTimeDiagnosticsPlugin` for one debug line.
const FPS_SMOOTHING: f32 = 0.1;

/// Environment variable that turns the readout on for a native run. The browser
/// host sets `GameConfig::debug_memory` from `/game/?mem=1`, but a native
/// `cargo run` has no host to do that — without this the readout could only ever
/// be seen on the web, which is exactly where it is hardest to inspect.
/// Mirrors the `MAZE_DEMO` convention.
const DEBUG_MEMORY_ENV: &str = "MAZE_DEBUG_MEM";

/// Whether an env value asks for the readout. `1` / `true` (any case) turn it
/// on; anything else — including unset — leaves it off.
pub(crate) fn debug_memory_from(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true"))
}

/// Reads [`DEBUG_MEMORY_ENV`]. Forced off under `cfg(test)` so a developer with
/// the variable still set in their shell cannot change what the headless tests
/// spawn — the same trap `MAZE_DEMO` handling already guards against.
pub(crate) fn debug_memory_env() -> bool {
    if cfg!(test) {
        return false;
    }
    debug_memory_from(std::env::var(DEBUG_MEMORY_ENV).ok().as_deref())
}

#[derive(Component)]
pub(crate) struct DiagnosticsReadout;

/// The text node specifically — the background strip shares
/// [`DiagnosticsReadout`], but only the text is offset to the panel's left edge,
/// so the resize systems have to move the two differently.
#[derive(Component)]
pub(crate) struct DiagnosticsText;

/// Frame-rate estimate and update accumulator for the readout.
#[derive(Resource, Default)]
pub(crate) struct DiagnosticsState {
    fps: f32,
    since_update: f32,
}

/// Y of the diagnostics strip — directly under the minimap's dimensions footer.
pub(crate) fn diagnostics_y(center_y: f32, map_size: f32) -> f32 {
    minimap_dimensions_y(center_y, map_size) - MINIMAP_DIM_STRIP_H / 2.0 - DIAG_GAP
        - DIAG_STRIP_H / 2.0
}

/// Renders a byte count as whole megabytes, or `n/a` where the platform cannot
/// report one. Native builds have no cheap equivalent of the WASM linear-memory
/// figure, and Bevy's `sysinfo_plugin` would add binary weight for it.
pub(crate) fn format_memory(bytes: Option<usize>) -> String {
    match bytes {
        Some(b) => format!("{} MB", b / (1024 * 1024)),
        None => "n/a".to_string(),
    }
}

/// The readout body — one metric per row, `<label> <value>`. `mes` / `mat` /
/// `img` are the resident mesh, material and image **asset** counts (shared
/// assets, not instances).
#[allow(clippy::too_many_arguments)]
pub(crate) fn diagnostics_label(
    visible: usize,
    total: usize,
    fps: f32,
    memory_bytes: Option<usize>,
    live_bytes: usize,
    meshes: usize,
    materials: usize,
    images: usize,
) -> String {
    format!(
        "vis {visible}/{total}\nfps {fps:.0}\nmem {}\nlive {}\nmes {meshes}\nmat {materials}\nimg {images}",
        format_memory(memory_bytes),
        format_memory(Some(live_bytes)),
    )
}

/// The WASM linear-memory size — the allocated heap, which is what runs out.
#[cfg(target_arch = "wasm32")]
fn linear_memory_bytes() -> Option<usize> {
    use wasm_bindgen::JsCast;
    let memory = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .ok()?;
    let buffer = memory.buffer().dyn_into::<js_sys::ArrayBuffer>().ok()?;
    Some(buffer.byte_length() as usize)
}

#[cfg(not(target_arch = "wasm32"))]
fn linear_memory_bytes() -> Option<usize> {
    None
}

/// Spawns the readout on entering the title screen, so the countdown shows the
/// pre-world figures — the world is not built until `AppState::Playing`. Not
/// tagged `TitleEntity`, so `teardown_title` leaves it and the same entities
/// carry into play from a single spawn site.
///
/// Also where `MAZE_DEBUG_MEM` is folded in, since it must be applied before
/// anything reads the flag.
///
/// Without the flag nothing is spawned and no system below does any work.
pub(crate) fn setup_diagnostics(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    window: Query<&Window>,
) {
    config.debug_memory |= debug_memory_env();
    if !config.debug_memory {
        return;
    }
    let config = &*config;
    commands.insert_resource(DiagnosticsState::default());

    let cell_px = config.minimap_cell_px as f32;
    let map_size = (config.minimap_radius as i32 * 2 + 1) as f32 * cell_px;
    let (center_x, center_y) = window
        .single()
        .map(|w| panel_centre(w.width(), w.height(), map_size))
        .unwrap_or((0.0, 0.0));
    let y = diagnostics_y(center_y, map_size);

    commands.spawn((
        DiagnosticsReadout,
        Sprite {
            color: MINIMAP_PANEL_BG,
            custom_size: Some(Vec2::new(map_size + 4.0, DIAG_STRIP_H)),
            ..default()
        },
        Transform::from_xyz(center_x, y, 8.8),
    ));
    // Left-anchored at the panel's left edge so every row starts on the same
    // column, rather than centring and overflowing the screen edge.
    commands.spawn((
        DiagnosticsReadout,
        DiagnosticsText,
        Text2d::new(""),
        TextFont { font_size: DIAG_FONT, ..default() },
        TextColor(MINIMAP_PANEL_TEXT),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(text_x(center_x, map_size), y, 9.0),
    ));
}

/// X of the text block's left edge — the panel's left edge plus a small inset.
fn text_x(center_x: f32, map_size: f32) -> f32 {
    center_x - (map_size + 4.0) / 2.0 + DIAG_TEXT_PAD_X
}

/// Top-right corner placement shared by the minimap column.
fn panel_centre(window_w: f32, window_h: f32, map_size: f32) -> (f32, f32) {
    (
        window_w / 2.0 - MINIMAP_EDGE_MARGIN - map_size / 2.0,
        window_h / 2.0 - MINIMAP_EDGE_MARGIN - map_size / 2.0,
    )
}

/// Recomputes the readout on the [`DIAG_UPDATE_SECS`] cadence. The visible count
/// is over `Mesh3d` entities only: the minimap runs its own camera, so counting
/// every entity would fold HUD sprites into a figure that is supposed to track
/// 3D draw cost.
#[allow(clippy::too_many_arguments)]
pub(crate) fn diagnostics_update_system(
    time: Res<Time>,
    state: Option<ResMut<DiagnosticsState>>,
    meshes: Option<Res<Assets<Mesh>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    images: Option<Res<Assets<Image>>>,
    visibility: Query<&ViewVisibility, With<Mesh3d>>,
    mut text: Query<&mut Text2d, With<DiagnosticsReadout>>,
) {
    let Some(mut state) = state else { return };

    let dt = time.delta_secs();
    if dt > 0.0 {
        let instant = 1.0 / dt;
        state.fps = if state.fps == 0.0 {
            instant
        } else {
            state.fps + (instant - state.fps) * FPS_SMOOTHING
        };
    }

    state.since_update += dt;
    if state.since_update < DIAG_UPDATE_SECS {
        return;
    }
    state.since_update = 0.0;

    let total = visibility.iter().count();
    let visible = visibility.iter().filter(|v| v.get()).count();
    let label = diagnostics_label(
        visible,
        total,
        state.fps,
        linear_memory_bytes(),
        crate::live_bytes(),
        meshes.map_or(0, |m| m.len()),
        materials.map_or(0, |m| m.len()),
        images.map_or(0, |i| i.len()),
    );
    for mut t in &mut text {
        if t.0 != label {
            t.0 = label.clone();
        }
    }
}

/// Keeps the readout under the minimap column when the window resizes, matching
/// `minimap_dimensions_resize_system`.
pub(crate) fn diagnostics_resize_system(
    window: Query<&Window>,
    config: Res<GameConfig>,
    mut last_size: Local<(f32, f32)>,
    mut readout: Query<(&mut Transform, Option<&DiagnosticsText>), With<DiagnosticsReadout>>,
) {
    let Ok(win) = window.single() else { return };
    let (w, h) = (win.width(), win.height());
    if (w - last_size.0).abs() < 0.5 && (h - last_size.1).abs() < 0.5 {
        return;
    }
    *last_size = (w, h);

    let map_size = (config.minimap_radius as i32 * 2 + 1) as f32 * config.minimap_cell_px as f32;
    let (center_x, center_y) = panel_centre(w, h, map_size);
    let y = diagnostics_y(center_y, map_size);
    for (mut t, is_text) in &mut readout {
        t.translation.x = if is_text.is_some() { text_x(center_x, map_size) } else { center_x };
        t.translation.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_is_reported_in_whole_megabytes() {
        assert_eq!(format_memory(Some(214 * 1024 * 1024)), "214 MB");
        // Sub-megabyte rounds down rather than showing noise.
        assert_eq!(format_memory(Some(1024)), "0 MB");
    }

    #[test]
    fn memory_reads_as_unavailable_off_the_web() {
        // Native builds report no linear-memory figure — see `format_memory`.
        assert_eq!(format_memory(None), "n/a");
    }

    #[test]
    fn label_leads_with_the_visible_against_total_pair() {
        // The pair that tests the culling hypothesis has to be first and whole:
        // "visible" alone says nothing without the total to read it against.
        let label =
            diagnostics_label(1234, 5678, 58.4, Some(214 * 1024 * 1024), 96 * 1024 * 1024, 42, 31, 18);
        let rows: Vec<&str> = label.lines().collect();
        assert_eq!(rows[0], "vis 1234/5678", "got {label}");
        assert_eq!(rows[1], "fps 58", "got {label}");
        assert_eq!(rows[2], "mem 214 MB", "got {label}");
        // `live` sits beside `mem` deliberately: the pair reads as "holding this
        // much of a ceiling that big", and only `live` can fall.
        assert_eq!(rows[3], "live 96 MB", "got {label}");
        assert_eq!(rows[4], "mes 42", "got {label}");
        assert_eq!(rows[5], "mat 31", "got {label}");
        assert_eq!(rows[6], "img 18", "got {label}");
    }

    #[test]
    fn every_metric_gets_its_own_row() {
        // One metric per row is what keeps each row inside the minimap's width.
        // A combined row ran off the screen edge and truncated its tail.
        let label = diagnostics_label(1, 2, 60.0, None, 1024, 1, 1, 1);
        assert_eq!(label.lines().count(), DIAG_ROWS as usize, "got {label}");
        assert_eq!(label.lines().count(), 7, "got {label}");
    }

    #[test]
    fn the_text_starts_at_the_panels_left_edge() {
        // Left-aligned rather than centred: a centred block pushed the longest
        // row past the screen edge, which is what truncated it.
        let center_x = 500.0;
        let map_size = 110.0;
        let left_edge = center_x - (map_size + 4.0) / 2.0;
        let x = text_x(center_x, map_size);
        assert!(x >= left_edge, "text must not start left of the panel");
        assert!(x < center_x, "text must be left-aligned, not centred");
    }

    #[test]
    fn the_env_flag_accepts_the_documented_values_only() {
        assert!(debug_memory_from(Some("1")));
        assert!(debug_memory_from(Some("true")));
        assert!(debug_memory_from(Some("TRUE")));
        // Unset, empty, or anything else leaves the readout off — a stray value
        // should not silently change what a run renders.
        assert!(!debug_memory_from(None));
        assert!(!debug_memory_from(Some("")));
        assert!(!debug_memory_from(Some("0")));
        assert!(!debug_memory_from(Some("yes")));
    }

    #[test]
    fn the_env_flag_is_ignored_under_test() {
        // Guards the trap MAZE_DEMO already hit: a variable left set in a
        // developer's shell must not change what the headless tests spawn.
        assert!(!debug_memory_env());
    }

    #[test]
    fn the_strip_sits_below_the_minimap_dimensions_footer() {
        // Screen space grows upward, so "below" means a smaller y than the
        // footer it follows.
        let center_y = 200.0;
        let map_size = 110.0;
        assert!(diagnostics_y(center_y, map_size) < minimap_dimensions_y(center_y, map_size));
    }
}
