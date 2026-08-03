pub(crate) mod ladder;
pub(crate) mod orb;
pub(crate) mod portal;

use super::common::CommonObjectAssets;
use crate::state::FinishType;
use crate::world::LevelPlacement;
use bevy::prelude::*;

pub(crate) struct FinishAssets {
    pub(crate) orb: orb::OrbAssets,
    pub(crate) ladder: ladder::LadderAssets,
    pub(crate) portal: portal::PortalAssets,
}

pub(crate) fn build_finish_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    common: &CommonObjectAssets,
) -> FinishAssets {
    FinishAssets {
        orb: orb::build_orb_assets(meshes, materials),
        ladder: ladder::build_ladder_assets(meshes, materials, common),
        portal: portal::build_portal_assets(meshes, materials),
    }
}

/// Spawns whatever marks the finish cell `(r, c)`:
/// - the run's **final** level keeps the bobbing gold orb (also the single-level
///   case — a one-level game is its own final level);
/// - an **interim** level draws the transition rig the player uses to ascend,
///   chosen by `finish_type` (`Random` resolves per cell off `seed`).
///
/// A **ladder** needs the next level's start cell directly above it to climb to
/// (`ladder_allowed`); where there's nothing above to land on, it falls back to a
/// **portal**, which can be entered anywhere. So a portal can always stand in for
/// a ladder, but not vice versa — the choice depends on what sits above this
/// finish (decided by the caller from the level layout).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_finish_for_cell(
    commands: &mut Commands,
    assets: &FinishAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    placement: LevelPlacement,
    finish_type: FinishType,
    seed: u64,
    is_final: bool,
    ladder_allowed: bool,
) {
    // 'F'-cell predicate is enforced once here; the per-object spawn
    // helpers below run unconditionally and assume a finish cell.
    if cell != 'F' {
        return;
    }
    if is_final {
        orb::spawn_orb(commands, &assets.orb, r, c, placement);
        return;
    }
    let rig = finish_type.concrete_for_cell(r, c, seed);
    // A ladder with nowhere above to climb to becomes a portal.
    let rig = if rig == FinishType::Ladder && !ladder_allowed {
        FinishType::Portal
    } else {
        rig
    };
    match rig {
        FinishType::Portal => portal::spawn_portal(commands, &assets.portal, r, c, placement),
        // `Ladder`; `Random` is already resolved, and a no-ladder cell became a
        // portal above.
        _ => ladder::spawn_ladder(commands, &assets.ladder, grid, r, c, placement),
    }
}
