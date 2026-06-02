//! The sliding-leaf door rig.
//!
//! Used at corners, T-junctions, open areas, and dead-end stubs — anywhere the
//! opening isn't a clean corridor between two facing walls, where a swing would
//! sweep awkwardly through the open space beside it. The leaf retracts straight
//! down into the floor, which needs no side anchor. The orchestrator in [`super`]
//! decides when this rig applies; this module owns only how the leaf is posed as
//! it opens.

use crate::world::walls::PANEL_H;
use bevy::prelude::*;

/// How far the leaf retracts downward when fully open. `PANEL_H` drops the whole
/// leaf below the floor plane so the opening reads as clear.
pub(crate) const DISTANCE: f32 = PANEL_H;

/// The leaf transform at `fraction` (`0.0` closed … `1.0` open): the leaf holds
/// its yaw and translates straight down by up to [`DISTANCE`].
pub(crate) fn leaf_transform(base_translation: Vec3, closed_yaw: f32, fraction: f32) -> Transform {
    Transform::from_translation(base_translation - Vec3::Y * (fraction * DISTANCE))
        .with_rotation(Quat::from_rotation_y(closed_yaw))
}
