//! Built ceiling for [`crate::state::SkyType::Chamber`], drawn in the cell's
//! own wall material so a brick maze gets a brick ceiling, a timber maze a
//! timber one, and so on — the maze reads as a finished, roofed-over building
//! rather than a hewn cave.

use crate::state::GameConfig;
use crate::world::walls::{wall_kind_for_cell, WallAssets};
use bevy::prelude::*;

/// The material a cell's ceiling tile should use — the same wall-material kind
/// ([`wall_kind_for_cell`]) as the surrounding walls, so the ceiling reads as
/// the same stone / wood / etc. as the walls it caps.
pub(crate) fn material_for_cell(
    wall_assets: &WallAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    config: &GameConfig,
) -> Option<Handle<StandardMaterial>> {
    let rows = grid.len();
    let cols = grid[r].len();
    let kind = wall_kind_for_cell(r, c, rows, cols, config);
    wall_assets.ns.material_mats[kind].clone()
}
