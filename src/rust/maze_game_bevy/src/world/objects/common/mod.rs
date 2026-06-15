//! Shared building blocks for the world's decorative / collectible objects.
//!
//! The dead-end landmarks (`brazier` / `urn` / `pillar` / `chest`) and the key
//! holder both build from the same primitives, materials, and the inverted-hull
//! outline trick. Those rigs and helpers live here so each consumer links to
//! one implementation rather than duplicating it:
//!
//! - [`CommonObjectAssets`] — the shared primitive meshes + every prop material.
//! - [`spawn_with_outline`] — spawns a body sub-mesh paired with a slightly
//!   larger black inverted-hull outline sibling. An optional `parent` lets a
//!   caller (the key holder) nest the pair under an animated group; a `None`
//!   parent spawns them free in the world (the dead-end props).
//! - [`build_emissive_material`] / [`build_outline_material`] — material helpers.
//! - [`yaw_toward_open_neighbour`] — orients a prop's front face at a cell's
//!   single open neighbour.
//!
//! The prop modules each expose a `TOP_Y` (the rig's apex height) so a caller
//! can float something — the key — a fixed clearance above it.

pub(crate) mod brazier;
pub(crate) mod chest;
pub(crate) mod pillar;
pub(crate) mod urn;

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::GridFacing;
use crate::world::initial_facing;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::f32::consts::{FRAC_PI_2, PI};

/// Uniform scale-up factor applied to each sibling outline mesh. The outline
/// shell reuses the body's mesh handle scaled by this factor with
/// `cull_mode: Some(Face::Front)`, so only a thin dark rim pokes out around the
/// body's silhouette — the classic inverted-hull outline trick.
pub(crate) const OUTLINE_SCALE: f32 = 1.06;

/// Default outline colour (pure black) for the shared outline material.
const OUTLINE_BASE_COLOR: Color = Color::BLACK;

pub(crate) struct CommonObjectAssets {
    pub(crate) cylinder: Option<Handle<Mesh>>,
    pub(crate) cuboid: Option<Handle<Mesh>>,
    pub(crate) cone: Option<Handle<Mesh>>,
    pub(crate) stone_mat: Option<Handle<StandardMaterial>>,
    pub(crate) glow_mat: Option<Handle<StandardMaterial>>,
    pub(crate) halo_mat: Option<Handle<StandardMaterial>>,
    pub(crate) urn_mat: Option<Handle<StandardMaterial>>,
    pub(crate) dark_terracotta_mat: Option<Handle<StandardMaterial>>,
    pub(crate) pillar_mat: Option<Handle<StandardMaterial>>,
    pub(crate) groove_mat: Option<Handle<StandardMaterial>>,
    pub(crate) chest_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lid_mat: Option<Handle<StandardMaterial>>,
    pub(crate) hinge_mat: Option<Handle<StandardMaterial>>,
    pub(crate) leather_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lock_mat: Option<Handle<StandardMaterial>>,
    pub(crate) outline_mat: Option<Handle<StandardMaterial>>,
    pub(crate) pillar_outline_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_common_object_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> CommonObjectAssets {
    // Shared unit primitives: every prop sub-mesh transforms one of these via
    // `Transform::with_scale` instead of materialising its own mesh asset,
    // keeping the mesh count flat.
    let cylinder = meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0)));
    let cuboid = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    let cone = meshes.as_mut().map(|m| m.add(Cone::new(0.5, 1.0)));
    CommonObjectAssets {
        cylinder,
        cuboid,
        cone,
        stone_mat: brazier::build_stone_material(materials),
        glow_mat: brazier::build_glow_material(materials),
        halo_mat: brazier::build_halo_material(materials),
        urn_mat: urn::build_urn_material(materials),
        dark_terracotta_mat: urn::build_dark_terracotta_material(materials),
        pillar_mat: pillar::build_pillar_material(materials),
        groove_mat: pillar::build_groove_material(materials),
        chest_mat: chest::build_chest_material(materials),
        lid_mat: chest::build_lid_material(materials),
        hinge_mat: chest::build_hinge_material(materials),
        leather_mat: chest::build_leather_material(materials),
        lock_mat: chest::build_lock_material(materials),
        outline_mat: build_outline_material(materials, OUTLINE_BASE_COLOR),
        pillar_outline_mat: build_outline_material(materials, pillar::OUTLINE_BASE_COLOR),
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

/// Spawns one body sub-mesh paired with a slightly larger black inverted-hull
/// outline sibling so the part reads as distinct from its neighbours rather than
/// merging into one shape.
///
/// When `parent` is `Some`, both the body and the outline are added as children
/// of that entity (so they inherit an animated group's transform — the key
/// holder's bobbing key). When `None`, they spawn free in the world (the
/// dead-end props and key-holder bases, which stand still at a cell).
///
/// `extras` is attached to the body entity only — e.g. a [`brazier::BrazierBowl`]
/// marker, or `()` for an unmarked piece.
pub(crate) fn spawn_with_outline<B: Bundle>(
    commands: &mut Commands,
    parent: Option<Entity>,
    mesh: Option<Handle<Mesh>>,
    body_mat: Option<Handle<StandardMaterial>>,
    outline_mat: Option<Handle<StandardMaterial>>,
    body_xform: Transform,
    extras: B,
) {
    let outline_xform = Transform {
        translation: body_xform.translation,
        rotation: body_xform.rotation,
        scale: body_xform.scale * OUTLINE_SCALE,
    };
    let body = match (mesh.clone(), body_mat) {
        (Some(m), Some(mt)) => commands
            .spawn((Mesh3d(m), MeshMaterial3d(mt), body_xform, extras))
            .id(),
        // No render assets (headless tests): spawn the transform + extras so
        // marker-counting still works without a mesh.
        _ => commands.spawn((body_xform, extras)).id(),
    };
    if let Some(p) = parent {
        commands.entity(p).add_child(body);
    }
    if let (Some(m), Some(mt)) = (mesh, outline_mat) {
        let edge = commands
            .spawn((Mesh3d(m), MeshMaterial3d(mt), outline_xform))
            .id();
        if let Some(p) = parent {
            commands.entity(p).add_child(edge);
        }
    }
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
