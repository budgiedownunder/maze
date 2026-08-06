//! Drawing and animating only the floors near the player.
//!
//! Every level of a stack is spawned up front and stays spawned, so a ten-level
//! run pays for ten floors on every frame however few the player can see. Two
//! separate costs come with that, and hiding geometry only removes the first:
//!
//! - **Display** — extract, cull, batch and draw. `Visibility::Hidden` on a
//!   level's tagged roots removes it, and their children follow through
//!   inherited visibility.
//! - **Processing** — the `Update` animation systems, which iterate their
//!   geometry by marker regardless of whether it is visible. The pool wave alone
//!   rewrites a `Transform` for every water or lava surface and rock on every
//!   floor, and each write marks the entity changed, so propagation and the
//!   per-instance upload run again for geometry nobody is looking at.
//!
//! [`LevelWindow`] is the shared answer to "does this level count right now",
//! consulted by both halves.

use crate::state::{GameConfig, GameState, LevelVisibility, MultiLevelRun};
use crate::world::LevelTag;
use bevy::prelude::*;

/// How much of a stack is drawn and animated, as an inclusive level range.
/// `None` means all of it — the default, and what the game did before this
/// existed.
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub(crate) struct LevelWindow {
    bounds: Option<(usize, usize)>,
}

impl LevelWindow {
    /// Whether `level` is currently drawn and animated.
    pub(crate) fn contains(&self, level: usize) -> bool {
        match self.bounds {
            None => true,
            Some((low, high)) => level >= low && level <= high,
        }
    }
}

/// Native override of the window, as `<below>,<above>` — `MAZE_FLOORS=0,0`
/// draws and animates the player's own floor and nothing else.
///
/// The host page sets [`GameConfig::level_visibility`] from `?floors=`, but a
/// native `cargo run` has no host to do that, so the effect of a window — and
/// how a windowed stack *looks* — could otherwise only be judged on the web,
/// which is the hardest place to inspect it. Mirrors the `MAZE_DEBUG_MEM`
/// convention.
const FLOORS_ENV: &str = "MAZE_FLOORS";

/// Parses a `<below>,<above>` pair. Anything else — a missing side, a negative,
/// a non-number, or unset — leaves the setting alone, so a stray value cannot
/// silently change what a run draws.
pub(crate) fn level_visibility_from(value: Option<&str>) -> Option<LevelVisibility> {
    let (below, above) = value?.split_once(',')?;
    Some(LevelVisibility {
        below: Some(below.trim().parse().ok()?),
        above: Some(above.trim().parse().ok()?),
    })
}

/// Reads [`FLOORS_ENV`]. Forced off under `cfg(test)` so a developer with the
/// variable still set in their shell cannot change what the headless tests
/// spawn — the same trap `MAZE_DEMO` handling already guards against.
pub(crate) fn level_visibility_env() -> Option<LevelVisibility> {
    if cfg!(test) {
        return None;
    }
    level_visibility_from(std::env::var(FLOORS_ENV).ok().as_deref())
}

/// The window around `current` for a run configured with `below` / `above`
/// levels of margin, where `None` on either side means unbounded.
///
/// `reveal` is a level that must be inside the window whatever the margins say —
/// the destination of a transition in flight. Without it the player climbs a
/// ladder into a hidden floor and watches it appear on arrival.
pub(crate) fn window_bounds(
    below: Option<u32>,
    above: Option<u32>,
    current: usize,
    reveal: Option<usize>,
) -> Option<(usize, usize)> {
    let (Some(below), Some(above)) = (below, above) else {
        return None;
    };
    let low = current.saturating_sub(below as usize);
    let high = current.saturating_add(above as usize);
    Some(match reveal {
        Some(r) => (low.min(r), high.max(r)),
        None => (low, high),
    })
}

/// `Update`: keeps [`LevelWindow`] in step with the player's level, and hides or
/// shows a level's geometry when it changes.
///
/// The visibility pass runs only when the window moves — on a level change or at
/// the ends of a transition — not every frame.
pub(crate) fn apply_level_window(
    config: Res<GameConfig>,
    state: Res<GameState>,
    run: Option<Res<MultiLevelRun>>,
    mut window: ResMut<LevelWindow>,
    mut tagged: Query<(&LevelTag, &mut Visibility)>,
) {
    let current = run.as_ref().map_or(0, |r| r.current_level);
    // A transition climbs toward the level above, so that is what to reveal.
    let reveal = state.transition.as_ref().map(|_| current + 1);
    // The env override stands in for the host's `?floors=` on a native run.
    let visibility = level_visibility_env().unwrap_or(config.level_visibility);
    let bounds = window_bounds(visibility.below, visibility.above, current, reveal);
    if window.bounds == bounds {
        return;
    }
    window.bounds = bounds;
    for (tag, mut visibility) in &mut tagged {
        let wanted = if window.contains(tag.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_env_override_accepts_a_pair_and_nothing_else() {
        assert_eq!(
            level_visibility_from(Some("0,0")),
            Some(LevelVisibility { below: Some(0), above: Some(0) }),
        );
        assert_eq!(
            level_visibility_from(Some(" 1 , 2 ")),
            Some(LevelVisibility { below: Some(1), above: Some(2) }),
        );
        // Unset, half a pair, or anything unparseable leaves the run alone
        // rather than guessing at a window.
        assert!(level_visibility_from(None).is_none());
        assert!(level_visibility_from(Some("")).is_none());
        assert!(level_visibility_from(Some("1")).is_none());
        assert!(level_visibility_from(Some("1,")).is_none());
        assert!(level_visibility_from(Some("-1,2")).is_none());
        assert!(level_visibility_from(Some("all")).is_none());
    }

    #[test]
    fn an_unset_margin_leaves_every_level_drawn() {
        assert_eq!(window_bounds(None, None, 3, None), None);
        assert_eq!(window_bounds(Some(1), None, 3, None), None);
        assert_eq!(window_bounds(None, Some(1), 3, None), None);
        assert!(LevelWindow::default().contains(9));
    }

    #[test]
    fn a_zero_margin_keeps_only_the_players_own_level() {
        let window = LevelWindow { bounds: window_bounds(Some(0), Some(0), 4, None) };
        assert!(window.contains(4));
        assert!(!window.contains(3));
        assert!(!window.contains(5));
    }

    #[test]
    fn margins_widen_the_window_in_each_direction() {
        assert_eq!(window_bounds(Some(1), Some(2), 4, None), Some((3, 6)));
    }

    /// The bottom of a stack has nothing below it to subtract.
    #[test]
    fn the_window_does_not_run_off_the_bottom() {
        assert_eq!(window_bounds(Some(2), Some(0), 1, None), Some((0, 1)));
    }

    /// Mid-climb the destination is outside a zero-margin window, and hiding it
    /// would have the player rise into blackness.
    #[test]
    fn a_transition_reveals_the_level_being_climbed_to() {
        assert_eq!(window_bounds(Some(0), Some(0), 2, Some(3)), Some((2, 3)));
    }
}
