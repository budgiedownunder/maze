pub(crate) mod night;

use bevy::prelude::*;

/// Spawns the current sky/atmosphere mode. Today always `night`; future
/// modes (day, dusk, storm, void) would dispatch on a
/// `GameConfig.sky_mode` field (not yet plumbed through the JSON host
/// path — add when the second mode lands).
pub(crate) fn spawn_sky(commands: &mut Commands) {
    night::spawn_night(commands);
}
