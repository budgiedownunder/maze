pub(crate) mod orb;

use bevy::prelude::*;

pub(crate) struct FinishAssets {
    pub(crate) orb: orb::OrbAssets,
}

pub(crate) fn build_finish_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> FinishAssets {
    FinishAssets {
        orb: orb::build_orb_assets(meshes, materials),
    }
}

pub(crate) fn spawn_finish_for_cell(
    commands: &mut Commands,
    assets: &FinishAssets,
    cell: char,
    r: usize,
    c: usize,
) {
    // 'F'-cell predicate is enforced once here; the per-object spawn
    // helpers below run unconditionally and assume a finish cell.
    if cell != 'F' {
        return;
    }
    orb::spawn_orb(commands, &assets.orb, r, c);
}
