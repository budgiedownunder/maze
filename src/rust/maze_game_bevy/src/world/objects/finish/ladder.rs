//! The **ladder** transition rig drawn at an interim finish cell.
//!
//! Two vertical rails joined by evenly-spaced rungs, rising a full
//! [`LEVEL_HEIGHT`] from the finish cell's floor to the underside of the level
//! above — the structure the player climbs to advance. It is static geometry
//! (no per-frame animation); the **hatch** it climbs through, and the climb
//! itself, are wired up in the transition step. Built from the shared cuboid
//! primitive + inverted-hull outline so it reads as crisp timber against the
//! corridor behind it.
//!
//! Like the treasure / dead-end chests, the rig yaws so its climbing face turns
//! toward the cell's open neighbour — in a corridor it faces the player walking
//! up to it rather than presenting an edge-on rail.

use super::super::common::{
    build_emissive_material, spawn_with_outline, yaw_toward_open_neighbour, CommonObjectAssets,
};
use crate::world::{LevelPlacement, CELL_SIZE, LEVEL_HEIGHT};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Square cross-section of each vertical rail (units).
const RAIL_THICKNESS: f32 = 0.1;
/// Centre-to-centre distance between the two rails (units).
const RAIL_SPACING: f32 = 0.6;
/// Square cross-section of each rung (units).
const RUNG_THICKNESS: f32 = 0.08;
/// Vertical gap between successive rungs (units).
const RUNG_SPACING: f32 = 0.4;
/// Warm timber emissive RGB.
const LADDER_EMISSIVE: LinearRgba = LinearRgba::new(0.35, 0.22, 0.08, 1.0);

/// Marks every sub-mesh of a ladder rig, so the transition step (and headless
/// tests) can find one.
#[derive(Component)]
pub(crate) struct FinishLadder;

pub(crate) struct LadderAssets {
    pub(crate) wood_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_ladder_assets(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> LadderAssets {
    LadderAssets {
        wood_mat: build_emissive_material(materials, LADDER_EMISSIVE),
    }
}

/// Spawns the ladder centred in cell `(r, c)`, rising from the level's floor to
/// `LEVEL_HEIGHT` above it. The rig is yawed so its rung face turns toward the
/// cell's open neighbour: built in local space (rails offset along local X, rungs
/// spanning it, climbing face toward local +Z) then rotated by that yaw and
/// translated onto the cell — the same orientation convention the chests use.
pub(crate) fn spawn_ladder(
    commands: &mut Commands,
    common: &CommonObjectAssets,
    ladder: &LadderAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let base = Vec3::new(
        placement.world_x(c as f32 * CELL_SIZE + 1.0),
        placement.world_y(0.0),
        placement.world_z(r as f32 * CELL_SIZE + 1.0),
    );
    let yaw = Quat::from_rotation_y(yaw_toward_open_neighbour(grid, r, c));
    let half = RAIL_SPACING / 2.0;

    // Places one yawed sub-mesh at a local offset from the cell's floor centre.
    let mut spawn_part = |local: Vec3, scale: Vec3| {
        spawn_with_outline(
            commands,
            None,
            common.cuboid.clone(),
            ladder.wood_mat.clone(),
            common.outline_mat.clone(),
            Transform {
                translation: base + yaw * local,
                rotation: yaw,
                scale,
            },
            FinishLadder,
        );
    };

    // Two vertical rails, offset along local X, reaching the level above.
    for dx in [-half, half] {
        spawn_part(
            Vec3::new(dx, LEVEL_HEIGHT / 2.0, 0.0),
            Vec3::new(RAIL_THICKNESS, LEVEL_HEIGHT, RAIL_THICKNESS),
        );
    }

    // Rungs at a fixed vertical pitch, spanning the rails.
    let mut y = RUNG_SPACING;
    while y < LEVEL_HEIGHT {
        spawn_part(
            Vec3::new(0.0, y, 0.0),
            Vec3::new(RAIL_SPACING + RAIL_THICKNESS, RUNG_THICKNESS, RUNG_THICKNESS),
        );
        y += RUNG_SPACING;
    }
}
