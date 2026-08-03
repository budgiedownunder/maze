//! Shared building blocks for the world's decorative / collectible objects.
//!
//! The dead-end landmarks (`brazier` / `urn` / `pillar` / `chest`) and the key
//! holder both build from the same primitives, materials, and the inverted-hull
//! outline trick. Those rigs and helpers live here so each consumer links to
//! one implementation rather than duplicating it:
//!
//! - [`CommonObjectAssets`] — every prop rig, baked at asset-build time.
//! - [`bake`] — how a rig is combined into one mesh per material and spawned.
//! - [`build_emissive_material`] / [`build_outline_material`] — material helpers.
//! - [`yaw_toward_open_neighbour`] — orients a prop's front face at a cell's
//!   single open neighbour.
//!
//! The prop modules each expose a `TOP_Y` (the rig's apex height) so a caller
//! can float something — the key — a fixed clearance above it.

pub(crate) mod bake;
pub(crate) mod brazier;
pub(crate) mod chest;
pub(crate) mod pillar;
pub(crate) mod urn;

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::GridFacing;
use crate::world::initial_facing;
use bake::{BakedRig, UnitMeshes};
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use chest::ChestLid;
use std::f32::consts::{FRAC_PI_2, PI};

/// Default outline colour (pure black) for the shared outline material.
const OUTLINE_BASE_COLOR: Color = Color::BLACK;

pub(crate) struct CommonObjectAssets {
    /// The shared black inverted-hull outline material. Baked into the rigs
    /// below, and still handed out for the outline meshes the treasure loot and
    /// the finish ladder bake for themselves.
    pub(crate) outline_mat: Option<Handle<StandardMaterial>>,
    pub(crate) brazier: BakedRig,
    pub(crate) urn: BakedRig,
    /// Baked at full height; the key holder's half-height pedestal is the same
    /// rig under a vertical scale (see [`pillar::spawn_pillar`]).
    pub(crate) pillar: BakedRig,
    pub(crate) chest_closed: BakedRig,
    pub(crate) chest_open: BakedRig,
}

pub(crate) fn build_common_object_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> CommonObjectAssets {
    let prims = UnitMeshes::new();
    let outline_mat = build_outline_material(materials, OUTLINE_BASE_COLOR);
    let pillar_outline_mat = build_outline_material(materials, pillar::OUTLINE_BASE_COLOR);
    // The chest's materials are built once and baked into both lid variants, so
    // an open and a closed chest still share one set of materials between them.
    let chest_mats = chest::build_chest_materials(materials, &outline_mat);
    CommonObjectAssets {
        brazier: brazier::build_brazier_rig(&prims, meshes, materials, &outline_mat),
        urn: urn::build_urn_rig(&prims, meshes, materials),
        pillar: pillar::build_pillar_rig(&prims, meshes, materials, &pillar_outline_mat),
        chest_closed: chest::build_chest_rig(&prims, meshes, &chest_mats, ChestLid::Closed),
        chest_open: chest::build_chest_rig(&prims, meshes, &chest_mats, ChestLid::Open),
        outline_mat,
    }
}

pub(crate) fn build_outline_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    base_color: Color,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color,
            unlit: true,
            // Render only the back faces of the enlarged outline shell. The
            // visible portion is the thin rim that pokes past the body's
            // silhouette — classic inverted-hull outline.
            cull_mode: Some(Face::Front),
            ..default()
        })
    })
}

/// Builds a tinted emissive-only material in a single line — the common case
/// for every prop's matte, self-lit surfaces.
pub(crate) fn build_emissive_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive,
            ..default()
        })
    })
}

/// Rotation around Y that orients a prop's local `+Z` (its front / lock face)
/// toward the cell's single open neighbour, so the player walking up to it sees
/// the front rather than a blank back face.
///
/// Coordinate mapping: in the maze grid, row+1 is world +Z (south) and col+1 is
/// world +X (east). Bevy's `Quat::from_rotation_y(θ)` rotates +Z toward +X for
/// positive θ, so a yaw of `π/2` rotates the front face from default south (+Z)
/// to east (+X), and so on around the compass.
pub(crate) fn yaw_toward_open_neighbour(grid: &[Vec<char>], r: usize, c: usize) -> f32 {
    // `initial_facing` cycles S→E→N→W and returns the first open neighbour. For
    // a dead-end (exactly one open neighbour) the result is unique; for a key in
    // a junction it picks a deterministic open side to face.
    match initial_facing(grid, r, c) {
        GridFacing::South => 0.0,
        GridFacing::East => FRAC_PI_2,
        GridFacing::North => PI,
        GridFacing::West => -FRAC_PI_2,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// Runs `spawn` against a bare world and reports how many entities it cost.
    ///
    /// Each prop asserts its own figure: a rig baked into one mesh per material
    /// spawns one entity per material, and a regression back to one entity per
    /// sub-mesh shows up as a count in the dozens.
    pub(crate) fn entities_spawned(spawn: impl FnOnce(&mut Commands)) -> usize {
        let mut world = World::new();
        {
            let mut commands = world.commands();
            spawn(&mut commands);
        }
        world.flush();
        // Counted by `Transform` (every rig entity is placed) rather than
        // `World::entities`, which reports allocated ids and rounds up.
        let mut placed = world.query::<&Transform>();
        placed.iter(&world).count()
    }
}
