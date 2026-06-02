//! The swinging-leaf door rig.
//!
//! Used for a straight corridor (a `'D'` cell with two open edges on opposing
//! sides), where the leaf is anchored between two facing walls and so swings
//! flush on a vertical hinge — the familiar door swing. The orchestrator in
//! [`super`] decides when this rig applies; this module owns only how the leaf
//! is posed as it opens.

use bevy::prelude::*;
use std::f32::consts::PI;

/// How far the leaf rotates around its hinge when fully open (radians). Slightly
/// past 90° so the open leaf tucks fully out of the opening.
pub(crate) const OPEN_ANGLE: f32 = PI * 100.0 / 180.0;

/// The leaf transform at `fraction` (`0.0` closed … `1.0` open): the leaf holds
/// its hinge position and rotates around it.
pub(crate) fn leaf_transform(base_translation: Vec3, closed_yaw: f32, fraction: f32) -> Transform {
    Transform::from_translation(base_translation)
        .with_rotation(Quat::from_rotation_y(closed_yaw + fraction * OPEN_ANGLE))
}
