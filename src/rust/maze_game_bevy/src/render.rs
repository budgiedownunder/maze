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

use crate::state::{FinishType, GameConfig};
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
    config.freeze_wall_animation |= env_flag("MAZE_NO_WALL_ANIM");
    config.disable_object_glow |= env_flag("MAZE_NO_GLOW");
    if env_flag("MAZE_NO_LADDERS") {
        config.allow_ladders = false;
    }
    resolve_ladders(&mut config);
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
