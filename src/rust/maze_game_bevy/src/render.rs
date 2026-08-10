//! Render-target settings the host can override, for measuring where a frame's
//! time goes on a device.
//!
//! Neither of these is configured by the game normally: the drawing buffer is
//! whatever the platform's scale factor makes it (a phone's device pixel ratio,
//! typically 3), and multisampling is left at Bevy's own default. Both are
//! therefore unmeasured costs, and on mobile both are paid per pixel — the kind
//! of cost that barely moves when the scene's entity count changes.
//!
//! It also folds the native env overrides for the diagnostic switches into the
//! config once at startup, so the per-frame systems that read them never touch
//! the environment.
//!
//! Every override defaults to absent, so a normal run renders exactly as it did
//! before they existed.

use crate::state::{FinishType, GameConfig, LevelVisibility};
use bevy::prelude::*;

/// `Startup`: applies [`GameConfig::render_scale`] to the primary window.
///
/// The scale factor is how many physical pixels the renderer draws per logical
/// pixel, so halving it quarters the pixels shaded. The value is absolute rather
/// than a fraction of the platform's own: the host page knows the device pixel
/// ratio and does that arithmetic, which keeps the browser as the only thing
/// that has to know what the device reports.
pub(crate) fn apply_render_scale(config: Res<GameConfig>, mut windows: Query<&mut Window>) {
    let Some(scale) = config.render_scale else {
        return;
    };
    if scale <= 0.0 {
        return;
    }
    for mut window in &mut windows {
        window.resolution.set_scale_factor_override(Some(scale));
    }
}

/// Whether an env value asks for a diagnostic switch. `1` / `true` (any case);
/// anything else, including unset, leaves it off.
pub(crate) fn env_flag_from(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true"))
}

fn env_flag(name: &str) -> bool {
    if cfg!(test) {
        return false;
    }
    env_flag_from(std::env::var(name).ok().as_deref())
}

/// `Startup`: folds the native switches into the config, so the systems reading
/// them each frame see a plain `bool` and never look at the environment. The
/// browser host sets the same fields from its query string.
pub(crate) fn apply_env_overrides(mut config: ResMut<GameConfig>) {
    config.mobile_mode |= env_flag("MAZE_MOBILE");
    // First, so the individual switches below can only add to what it implies.
    resolve_mobile_mode(&mut config);
    config.freeze_wall_animation |= env_flag("MAZE_NO_WALL_ANIM");
    config.disable_object_glow |= env_flag("MAZE_NO_GLOW");
    if env_flag("MAZE_NO_LADDERS") {
        config.allow_ladders = false;
    }
    resolve_ladders(&mut config);
}

/// The largest footprint, in cells, that [`resolve_mobile_mode`] will draw at
/// one time.
///
/// One 40x40 level draws about 7,300 mesh entities and plays on an iPhone; two
/// of them, about 14,600, sees the run cut short within a few turns — the device
/// swamped, or the platform stepping in. Which of those was never isolated, and
/// the game's own heap is unremarkable when it happens, so this bounds the
/// rendering load rather than any allocation. The budget is the measured
/// survivable figure rather than a chosen one — at roughly 4.55 entities per
/// cell for brick walls, 1,600 cells is that 7,300.
///
/// Pool walls cost more per cell than brick — rims, edge seals and surfaces —
/// so a water or lava maze at the budget is dearer than the brick maze the
/// budget came from. That margin is not modelled; the budget is deliberately
/// set at the survivable figure rather than above it.
const MOBILE_DRAWN_CELL_BUDGET: u64 = 1600;

/// Whether a stack of this footprint can afford to be drawn two floors deep.
///
/// Judged on the base level, which a tapered stack's upper floors only ever
/// undercut, so this errs toward drawing less. A footprint of zero is a size
/// the config does not carry rather than an empty maze — the single-stored-maze
/// path — and such a game has no floor above to draw, so it costs nothing to
/// treat as affordable.
fn fits_drawn_budget(rows: u32, cols: u32) -> bool {
    let footprint = rows as u64 * cols as u64;
    footprint * 2 <= MOBILE_DRAWN_CELL_BUDGET
}

/// Applies what [`GameConfig::mobile_mode`] implies.
///
/// Each of these was measured on an iPhone rather than assumed:
///
/// - **The player's own floor is drawn and animated, and the one above it when
///   the maze is small enough to afford it.** A ten-level stack spent about
///   150 ms a frame on floors nobody could see, so the window is narrow. It
///   reaches upward rather than both ways because that is the direction of
///   travel: the floor below holds nothing the player still needs, while the
///   floor above is where the finish is. The floor above is drawn but *unlit*,
///   since the lights keep their own range. See
///   [`MOBILE_DRAWN_CELL_BUDGET`] for when it is given up.
/// - **Ladders follow the floor above.** A ladder is coherent exactly when the
///   floor it climbs into is drawn, so the mode leaves
///   [`GameConfig::allow_ladders`] alone while that floor is drawn and turns it
///   off when it is not. Turning it off independently still resolves interim
///   finishes to portals.
/// - **No key or treasure glow, and no light at the finish orb.** A shadowless
///   point light costs about 7 ms on that device, measured twice from different
///   sources, and a maze spawns one per key and per treasure. The meshes are
///   emissive, so every one of them still glows — what goes is the light they
///   cast on their surroundings.
///
/// Deliberately absent: the render scale and MSAA overrides, which measured
/// null, and freezing the pool animation, which measured null and leaves the
/// lava rocks submerged.
pub(crate) fn resolve_mobile_mode(config: &mut GameConfig) {
    if !config.mobile_mode {
        return;
    }
    let draw_floor_above = fits_drawn_budget(config.rows, config.cols);
    config.level_visibility = LevelVisibility {
        below: Some(0),
        above: Some(if draw_floor_above { 1 } else { 0 }),
    };
    if !draw_floor_above {
        // Nothing to climb into, so an interim finish becomes a portal.
        config.allow_ladders = false;
    }
    config.disable_object_glow = true;
    config.disable_orb_light = true;
}

/// Collapses [`GameConfig::allow_ladders`] into the finish type, once, before
/// anything reads it.
///
/// Every consumer of the ladder-or-portal choice — the rig spawned at an interim
/// finish, the climb-versus-step animation, the hatch given to the floor above,
/// and the hole cut in a roofed level's finish tile — resolves it from
/// `finish_type`. Rewriting that single value is therefore the whole mechanism:
/// there is no second condition that could fall out of step with it.
pub(crate) fn resolve_ladders(config: &mut GameConfig) {
    if !config.allow_ladders && config.finish_type != FinishType::Portal {
        config.finish_type = FinishType::Portal;
    }
}

/// The [`Msaa`] setting for a sample count, or `None` to leave Bevy's default
/// in place — which is what an absent or unusable value does, so a bad query
/// string degrades to a normal run rather than to no anti-aliasing.
///
/// Only `1` (off) and `4` are accepted. The browser is the platform this knob
/// exists to measure and WebGL2 supports no other sample count, so `2` and `8`
/// could only ever produce a comparison that silently measured nothing.
pub(crate) fn msaa_override(samples: Option<u32>) -> Option<Msaa> {
    match samples? {
        0 | 1 => Some(Msaa::Off),
        4 => Some(Msaa::Sample4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn scale_after(render_scale: Option<f32>) -> Option<f32> {
        let mut app = App::new();
        app.insert_resource(GameConfig { render_scale, ..default() });
        let window = app.world_mut().spawn(Window::default()).id();
        app.world_mut()
            .run_system_once(apply_render_scale)
            .expect("the render-scale system runs");
        app.world()
            .entity(window)
            .get::<Window>()
            .expect("the window survives")
            .resolution
            .scale_factor_override()
    }

    /// The default run must be untouched — an override left in place would
    /// silently change how every game renders.
    #[test]
    fn no_render_scale_leaves_the_window_alone() {
        assert_eq!(scale_after(None), None);
    }

    #[test]
    fn a_render_scale_overrides_the_windows_scale_factor() {
        assert_eq!(scale_after(Some(1.5)), Some(1.5));
    }

    /// A zero or negative scale would ask for a zero-pixel buffer.
    #[test]
    fn a_nonsensical_render_scale_is_ignored() {
        assert_eq!(scale_after(Some(0.0)), None);
        assert_eq!(scale_after(Some(-2.0)), None);
    }

    /// A maze within the drawn budget — see the large-maze case for the other
    /// side of it.
    #[test]
    fn mobile_mode_implies_what_the_measurements_justified() {
        let mut config = GameConfig { mobile_mode: true, rows: 15, cols: 15, ..GameConfig::default() };
        resolve_mobile_mode(&mut config);
        assert_eq!(
            config.level_visibility,
            LevelVisibility { below: Some(0), above: Some(1) },
            "the player's own floor and the one being climbed toward",
        );
        assert!(config.allow_ladders, "the floor a ladder climbs into is drawn");
        assert!(config.disable_object_glow);
        assert!(config.disable_orb_light);
    }

    /// A maze too large to draw two floors of gives up the floor above — and
    /// its ladders with it, since there is then nothing to climb into. A 40x40
    /// stack drawn two floors deep is what cut an iPhone's run short.
    #[test]
    fn a_large_maze_keeps_to_the_players_own_floor() {
        let mut config = GameConfig { mobile_mode: true, rows: 40, cols: 40, ..GameConfig::default() };
        resolve_mobile_mode(&mut config);
        assert_eq!(
            config.level_visibility,
            LevelVisibility { below: Some(0), above: Some(0) },
        );
        assert!(!config.allow_ladders, "nothing to climb into");
    }

    /// The maze the floor-above window was measured on stays as it was.
    #[test]
    fn a_small_maze_still_draws_the_floor_above() {
        let mut config = GameConfig { mobile_mode: true, rows: 15, cols: 15, ..GameConfig::default() };
        resolve_mobile_mode(&mut config);
        assert_eq!(
            config.level_visibility,
            LevelVisibility { below: Some(0), above: Some(1) },
        );
        assert!(config.allow_ladders);
    }

    /// The budget covers *both* floors, so the largest square maze that keeps
    /// the floor above is the one whose two levels together fit it.
    #[test]
    fn the_budget_counts_both_floors() {
        assert!(fits_drawn_budget(28, 28), "784 cells a floor, 1568 drawn");
        assert!(!fits_drawn_budget(29, 29), "841 a floor, 1682 drawn");
        // A size the config does not carry costs nothing to allow: that path is
        // a single stored maze, which has no floor above.
        assert!(fits_drawn_budget(0, 0));
    }

    /// A large maze reaches the finish type through the same one value, so an
    /// interim ladder becomes a portal without a second condition.
    #[test]
    fn a_large_maze_resolves_interim_finishes_to_portals() {
        let mut config = GameConfig {
            mobile_mode: true,
            rows: 40,
            cols: 40,
            finish_type: FinishType::Ladder,
            ..GameConfig::default()
        };
        resolve_mobile_mode(&mut config);
        resolve_ladders(&mut config);
        assert_eq!(config.finish_type, FinishType::Portal);
    }

    /// The lights keep their own, narrower range: the floor above is drawn so a
    /// ladder has somewhere to go, not so it can carry a floor's worth of point
    /// lights with it.
    #[test]
    fn mobile_mode_leaves_the_lights_narrower_than_the_scene() {
        let mut config = GameConfig { mobile_mode: true, ..GameConfig::default() };
        resolve_mobile_mode(&mut config);
        assert_eq!(config.light_visibility, LevelVisibility { below: Some(0), above: Some(0) });
    }

    /// The mode no longer forces portals, but the switch on top of it still
    /// does — a switch may add a restriction the mode did not impose.
    #[test]
    fn turning_ladders_off_over_the_mode_still_forces_portals() {
        let mut config = GameConfig {
            mobile_mode: true,
            finish_type: FinishType::Ladder,
            allow_ladders: false,
            ..GameConfig::default()
        };
        resolve_mobile_mode(&mut config);
        resolve_ladders(&mut config);
        assert_eq!(config.finish_type, FinishType::Portal);
    }

    /// The switches that measured null stay off: turning them on would cost
    /// picture quality — and, for the pool freeze, the lava rocks — for nothing.
    #[test]
    fn mobile_mode_leaves_the_switches_that_measured_null_alone() {
        let mut config = GameConfig { mobile_mode: true, ..GameConfig::default() };
        resolve_mobile_mode(&mut config);
        assert!(config.render_scale.is_none());
        assert!(config.msaa_samples.is_none());
        assert!(!config.freeze_wall_animation);
    }

    #[test]
    fn without_the_mode_nothing_is_implied() {
        let mut config = GameConfig::default();
        resolve_mobile_mode(&mut config);
        assert_eq!(config.level_visibility, LevelVisibility::default());
        assert!(config.allow_ladders);
        assert!(!config.disable_object_glow);
        assert!(!config.disable_orb_light);
    }

    /// The mode draws the floor a ladder climbs into, so it leaves a ladder
    /// finish standing where it once resolved it to a portal.
    #[test]
    fn mobile_mode_keeps_a_ladder_finish() {
        let mut config = GameConfig {
            mobile_mode: true,
            finish_type: FinishType::Ladder,
            ..GameConfig::default()
        };
        resolve_mobile_mode(&mut config);
        resolve_ladders(&mut config);
        assert_eq!(config.finish_type, FinishType::Ladder);
    }

    /// The switch collapses into the finish type, which is what every consumer
    /// reads — so there is nothing else to keep in step.
    #[test]
    fn disallowing_ladders_resolves_the_finish_type_to_a_portal() {
        for authored in [FinishType::Ladder, FinishType::Random] {
            let mut config =
                GameConfig { allow_ladders: false, finish_type: authored, ..GameConfig::default() };
            resolve_ladders(&mut config);
            assert_eq!(config.finish_type, FinishType::Portal, "from {authored:?}");
        }
    }

    #[test]
    fn allowing_ladders_leaves_the_authored_finish_type_alone() {
        for authored in [FinishType::Ladder, FinishType::Portal, FinishType::Random] {
            let mut config = GameConfig { finish_type: authored, ..GameConfig::default() };
            resolve_ladders(&mut config);
            assert_eq!(config.finish_type, authored, "the default allows ladders");
        }
    }

    #[test]
    fn a_portal_game_is_untouched_either_way() {
        let mut config =
            GameConfig { allow_ladders: false, finish_type: FinishType::Portal, ..GameConfig::default() };
        resolve_ladders(&mut config);
        assert_eq!(config.finish_type, FinishType::Portal);
    }

    #[test]
    fn msaa_maps_the_sample_counts_the_browser_supports() {
        assert!(matches!(msaa_override(Some(1)), Some(Msaa::Off)));
        assert!(matches!(msaa_override(Some(0)), Some(Msaa::Off)));
        assert!(matches!(msaa_override(Some(4)), Some(Msaa::Sample4)));
        assert!(msaa_override(None).is_none(), "absent → Bevy's default");
        assert!(msaa_override(Some(3)).is_none(), "unrecognised → Bevy's default");
        // 2 and 8 are valid Bevy settings but not on the web, so accepting them
        // would offer a comparison that cannot run on the platform under test.
        assert!(msaa_override(Some(2)).is_none());
        assert!(msaa_override(Some(8)).is_none());
    }
}
