//! The ladder **hatch** in a level's start-cell floor.
//!
//! When the level directly below has a **ladder** finish, this level's start cell
//! is the opening the climb emerges through — so instead of a solid start tile it
//! carries a **round** opening, framed by a metal rim, with a hinged metal **lid**
//! (a submersible-style hatch) that stands **open** while you climb and swings
//! **closed**, sealing the cell, the moment you arrive on this level. A portal
//! finish needs no hatch; the bottom level never has one (nothing climbs to it).

use super::{tile, FloorAssets, FloorCell};
use crate::state::MultiLevelRun;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::TAU;

// ---------- Tuning constants ----------

/// Radius of the round opening cut in the start cell's floor — about the ladder's
/// width (~0.7), a touch bigger so the climb passes cleanly through.
const HOLE_RADIUS: f32 = 0.42;
/// Lid disc radius — overlaps the hole so it seats on the rim when shut.
const LID_RADIUS: f32 = 0.52;
/// Lid disc thickness (units).
const LID_THICKNESS: f32 = 0.07;
/// Tube radius of the rim torus framing the hole (units).
const RIM_TUBE: f32 = 0.05;
/// Height the lid + rim sit above the floor so the lid seats on the rim.
const LID_LIFT: f32 = 0.05;
/// Angle (radians) the lid stands open at — a touch past vertical, like a hatch
/// flung back on its hinge.
const OPEN_ANGLE: f32 = 100.0 * std::f32::consts::PI / 180.0;
/// Seconds the lid takes to swing from open to sealed once the player arrives.
const CLOSE_DURATION: f32 = 0.6;

/// Crossed-wheel handle on the lid (the "twist to seal" wheel): a metal ring with
/// two perpendicular spokes across it, sitting on the lid's top face.
const WHEEL_RADIUS: f32 = 0.15;
const WHEEL_TUBE: f32 = 0.022;
const SPOKE_LEN: f32 = 0.32;
const SPOKE_THICK: f32 = 0.03;

/// Brushed-metal rim / wheel — lit (metallic) with a dim emissive floor so it
/// stays legible in dark corridors.
const METAL_BASE: Color = Color::srgb(0.55, 0.57, 0.60);
const METAL_EMISSIVE: LinearRgba = LinearRgba::new(0.06, 0.06, 0.07, 1.0);
/// Dark-grey lid disc (per the submersible look) — same metallic finish, darker.
const LID_BASE: Color = Color::srgb(0.20, 0.21, 0.23);

/// Shared meshes + materials for the round hatch, built once into [`FloorAssets`].
pub(crate) struct HatchAssets {
    /// The start tile with a round hole — a flat ring from the cell square in to
    /// the hole circle. Its top reuses the green start material (so the surface
    /// matches a plain start cell); `hole_mat` below caps the underside in stone.
    pub(crate) hole_mesh: Option<Handle<Mesh>>,
    /// Flat disc lid.
    pub(crate) lid_mesh: Option<Handle<Mesh>>,
    /// Rim torus framing the hole.
    pub(crate) rim_mesh: Option<Handle<Mesh>>,
    /// Crossed-wheel handle: a ring + a unit cuboid spoke (scaled per spawn).
    pub(crate) wheel_mesh: Option<Handle<Mesh>>,
    pub(crate) spoke_mesh: Option<Handle<Mesh>>,
    pub(crate) metal_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lid_mat: Option<Handle<StandardMaterial>>,
    pub(crate) hole_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_hatch_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tile_tex: &Option<Handle<Image>>,
) -> HatchAssets {
    let hole_mesh = meshes.as_mut().map(|m| m.add(square_hole_mesh()));
    let lid_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(LID_RADIUS, LID_THICKNESS)));
    let rim_mesh = meshes
        .as_mut()
        .map(|m| m.add(Torus::new(HOLE_RADIUS - RIM_TUBE, HOLE_RADIUS + RIM_TUBE)));
    let wheel_mesh = meshes
        .as_mut()
        .map(|m| m.add(Torus::new(WHEEL_RADIUS - WHEEL_TUBE, WHEEL_RADIUS + WHEEL_TUBE)));
    let spoke_mesh = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    let metal = |base: Color| StandardMaterial {
        base_color: base,
        metallic: 0.9,
        perceptual_roughness: 0.35,
        emissive: METAL_EMISSIVE,
        ..default()
    };
    let metal_mat = materials.as_mut().map(|m| m.add(metal(METAL_BASE)));
    let lid_mat = materials.as_mut().map(|m| m.add(metal(LID_BASE)));
    // Underside cap for the holed floor: the shared floor-tile material on the
    // ring's back faces only, so from the level below the hole reads as inset into
    // the same ceiling surface as every other cell (today the stone floor tile;
    // it would follow a wooden / dungeon roof material the same way).
    let hole_mat = tile::build_underside_material(materials, tile_tex);
    HatchAssets {
        hole_mesh,
        lid_mesh,
        rim_mesh,
        wheel_mesh,
        spoke_mesh,
        metal_mat,
        lid_mat,
        hole_mat,
    }
}

/// A flat mesh filling the cell square except for a central circular hole — a ring
/// of quads from the hole circle out to the square boundary, normals up, drawn
/// double-sided so the underside reads as stone (not a glowing gap) from below.
fn square_hole_mesh() -> Mesh {
    const N: usize = 48;
    let h = CELL_SIZE / 2.0;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // UVs map cell-local X/Z to [0,1] across the cell so the shared tile texture
    // (scaled by the start material's uv_transform) tiles exactly like a plain
    // start tile.
    let uv = |p: Vec2| [(p.x + h) / CELL_SIZE, (p.y + h) / CELL_SIZE];
    for i in 0..=N {
        let t = i as f32 / N as f32 * TAU;
        let (s, c) = t.sin_cos();
        let m = c.abs().max(s.abs()).max(1e-3);
        let inner = Vec2::new(c, s) * HOLE_RADIUS; // on the hole circle
        let outer = Vec2::new(c, s) * (h / m); // on the cell square boundary
        positions.push([inner.x, 0.0, inner.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push(uv(inner));
        positions.push([outer.x, 0.0, outer.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push(uv(outer));
    }
    for i in 0..N as u32 {
        let (a, b) = (i * 2, i * 2 + 1); // inner / outer at i
        let (c, d) = ((i + 1) * 2, (i + 1) * 2 + 1); // inner / outer at i+1
        // Wound so the front face points UP (matching the +Y normals), so the
        // green top renders for the level above and the stone cull-front underside
        // renders for the level below.
        indices.extend_from_slice(&[a, d, b, a, c, d]);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// A start-cell hatch lid. `level` is the level it caps; the close watcher flips
/// `closing` when the player climbs up onto that level, and the animation system
/// drives `anim` (0 = fully open → 1 = sealed). The cell anchor is stored so the
/// lid transform can be rebuilt each frame from the hinge angle.
#[derive(Component)]
pub(crate) struct LevelHatch {
    pub(crate) level: usize,
    pub(crate) closing: bool,
    pub(crate) anim: f32,
    cx: f32,
    cz: f32,
    floor_y: f32,
}

/// The lid disc's world transform for a hinge angle (`0` = flat/sealed,
/// `OPEN_ANGLE` = stood open). Hinged on the disc's west (`-X`) edge so the east
/// edge swings up.
fn lid_transform(cx: f32, cz: f32, floor_y: f32, angle: f32) -> Transform {
    let y = floor_y + LID_LIFT;
    let pivot = Vec3::new(cx - LID_RADIUS, y, cz);
    let flat_centre = Vec3::new(cx, y, cz);
    let rot = Quat::from_rotation_z(angle);
    Transform {
        translation: pivot + rot * (flat_centre - pivot),
        rotation: rot,
        scale: Vec3::ONE,
    }
}

/// Spawns the round hatch for cell `(r, c)`: a holed floor + a metal rim (both
/// static) and the hinged metal lid, stood open. The holed floor carries
/// [`FloorCell`] (it replaces the tile); the lid carries [`LevelHatch`].
pub(crate) fn spawn_hatch(
    commands: &mut Commands,
    assets: &FloorAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let cx = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let cz = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let floor_y = placement.world_y(0.0);
    let h = &assets.hatch;

    // Holed floor (static) — the start tile with a round opening. The top reuses
    // the green start material so the surface matches a plain start cell viewed
    // from above; a stone underside (cull-front) caps it so the level below sees
    // plain stone around the hole.
    let pos = Transform::from_xyz(cx, floor_y, cz);
    match (h.hole_mesh.clone(), assets.start_mat.clone()) {
        (Some(mesh), Some(green)) => {
            commands.spawn((FloorCell, pos, Mesh3d(mesh.clone()), MeshMaterial3d(green)));
            if let Some(stone) = h.hole_mat.clone() {
                commands.spawn((pos, Mesh3d(mesh), MeshMaterial3d(stone)));
            }
        }
        _ => {
            commands.spawn((FloorCell, pos));
        }
    }
    // Rim (static) — metal ring framing the opening.
    if let (Some(mesh), Some(mat)) = (h.rim_mesh.clone(), h.metal_mat.clone()) {
        commands.spawn((
            Transform::from_xyz(cx, floor_y + LID_LIFT, cz),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
        ));
    }
    // Lid (animated) — dark-grey metal disc, hinged on its west edge, stood open.
    let hatch = LevelHatch {
        level: placement.level,
        closing: false,
        anim: 0.0,
        cx,
        cz,
        floor_y,
    };
    let xform = lid_transform(cx, cz, floor_y, OPEN_ANGLE);
    let lid = match (h.lid_mesh.clone(), h.lid_mat.clone()) {
        (Some(mesh), Some(mat)) => commands.spawn((hatch, xform, Mesh3d(mesh), MeshMaterial3d(mat))).id(),
        _ => commands.spawn((hatch, xform)).id(),
    };
    // Crossed-wheel handle on the lid top — a ring + two perpendicular spokes,
    // spawned as children so they swing (and would twist) with the lid.
    if let (Some(wheel), Some(spoke), Some(mat)) =
        (h.wheel_mesh.clone(), h.spoke_mesh.clone(), h.metal_mat.clone())
    {
        let top = LID_THICKNESS / 2.0 + WHEEL_TUBE;
        commands.entity(lid).with_children(|p| {
            p.spawn((Mesh3d(wheel), MeshMaterial3d(mat.clone()), Transform::from_xyz(0.0, top, 0.0)));
            p.spawn((
                Mesh3d(spoke.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.0, top, 0.0).with_scale(Vec3::new(SPOKE_LEN, SPOKE_THICK, SPOKE_THICK)),
            ));
            p.spawn((
                Mesh3d(spoke),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.0, top, 0.0).with_scale(Vec3::new(SPOKE_THICK, SPOKE_THICK, SPOKE_LEN)),
            ));
        });
    }
}

/// Flips a level's hatch to `closing` the moment the player climbs up onto that
/// level (the run's `current_level` reaches it). Single-level games never enter
/// here (no hatch entities).
pub(crate) fn hatch_close_watcher(
    run: Res<MultiLevelRun>,
    mut last_level: Local<usize>,
    mut hatches: Query<&mut LevelHatch>,
) {
    if run.current_level == *last_level {
        return;
    }
    *last_level = run.current_level;
    for mut hatch in &mut hatches {
        if hatch.level == run.current_level {
            hatch.closing = true;
        }
    }
}

/// Swings each closing hatch lid from open to sealed over [`CLOSE_DURATION`].
pub(crate) fn hatch_animation_system(
    time: Res<Time>,
    mut hatches: Query<(&mut LevelHatch, &mut Transform)>,
) {
    for (mut hatch, mut transform) in &mut hatches {
        if !hatch.closing || hatch.anim >= 1.0 {
            continue;
        }
        hatch.anim = (hatch.anim + time.delta_secs() / CLOSE_DURATION).min(1.0);
        let angle = OPEN_ANGLE * (1.0 - smoothstep(hatch.anim));
        *transform = lid_transform(hatch.cx, hatch.cz, hatch.floor_y, angle);
    }
}

fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}
