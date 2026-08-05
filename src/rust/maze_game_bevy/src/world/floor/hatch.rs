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

/// Marker on the hatch's stone underside cap — the holed ring the level below sees
/// as the ceiling around the opening. Tagged so its placement (dropped to the gap
/// bottom on a lifted level) can be checked in tests.
#[derive(Component)]
pub(crate) struct HatchUnderside;

/// Shared meshes + materials for the round hatch, built once into [`FloorAssets`].
pub(crate) struct HatchAssets {
    /// The start tile with a round hole, normals **up** — reuses the green start
    /// material so the surface matches a plain start cell viewed from above.
    pub(crate) hole_mesh: Option<Handle<Mesh>>,
    /// The same hole, normals **down** with the cuboid bottom-face UV, for the
    /// underside the level below looks up at — so it tiles identically to the
    /// surrounding floor tiles' undersides instead of reading 180° rotated.
    pub(crate) underside_mesh: Option<Handle<Mesh>>,
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
    let underside_mesh = meshes.as_mut().map(|m| m.add(hole_underside_mesh()));
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
    // Underside cap material: the PLAIN floor-tile material (default cull). The
    // underside mesh faces DOWN, so its front renders from the level below and reads
    // as the same stone tiling as every surrounding floor cell's underside. (The old
    // cull-front approach reused the up-facing green-top mesh, which showed the
    // top-face UV from below — 180° off from the neighbours.)
    let hole_mat = tile::build_tile_material(materials, tile_tex);
    HatchAssets {
        hole_mesh,
        underside_mesh,
        lid_mesh,
        rim_mesh,
        wheel_mesh,
        spoke_mesh,
        metal_mat,
        lid_mat,
        hole_mat,
    }
}

/// The hatch's **top** surface — a flat ring filling the cell square except for the
/// central hole, normals **up**, with the cuboid **top**-face UV so it tiles exactly
/// like a plain start tile when the level above looks down on it. (The underside the
/// level below sees is a separate mesh, [`hole_underside_mesh`].)
fn square_hole_mesh() -> Mesh {
    const N: usize = 48;
    let h = CELL_SIZE / 2.0;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // Top-face UV (cuboid top: `u = (x + h)/size`, `v = (z + h)/size`).
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
        // Wound so the front face points UP (matching the +Y normals) for the level above.
        indices.extend_from_slice(&[a, d, b, a, c, d]);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// The hatch's **underside** — the same square-with-a-hole ring but normals **down**
/// and UVs matching a Bevy [`Cuboid`]'s **bottom** face (`u = (h - x)/size`,
/// `v = (h - z)/size`). The surrounding floor tiles are cuboids, so from the level
/// below they show their bottom face; this matches that exactly (same texels, same
/// orientation), so the hole reads as inset into one continuous ceiling rather than a
/// lighter, mis-tiled patch. Rendered with the plain (default-cull) floor-tile
/// material on its down-facing front.
fn hole_underside_mesh() -> Mesh {
    const N: usize = 48;
    let h = CELL_SIZE / 2.0;
    let uv = |p: Vec2| [(h - p.x) / CELL_SIZE, (h - p.y) / CELL_SIZE];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..=N {
        let t = i as f32 / N as f32 * TAU;
        let (s, c) = t.sin_cos();
        let m = c.abs().max(s.abs()).max(1e-3);
        let inner = Vec2::new(c, s) * HOLE_RADIUS;
        let outer = Vec2::new(c, s) * (h / m);
        positions.push([inner.x, 0.0, inner.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push(uv(inner));
        positions.push([outer.x, 0.0, outer.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push(uv(outer));
    }
    for i in 0..N as u32 {
        let (a, b) = (i * 2, i * 2 + 1); // inner / outer at i
        let (c, d) = ((i + 1) * 2, (i + 1) * 2 + 1); // inner / outer at i+1
        // Wound so the front face points DOWN (matching the -Y normals) for the level below.
        indices.extend_from_slice(&[a, b, d, a, d, c]);
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
    below_roofed: bool,
    // How far this level was lifted to clear its pools (0 when it carries none). The
    // neighbouring cells' undersides are sealed at the gap *bottom*, so the hatch's
    // stone underside cap drops by `gap` to sit at that same plane — else it floats a
    // `gap` above the surrounding ceiling on a lifted (pool) level.
    gap: f32,
) {
    let cx = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let cz = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let floor_y = placement.world_y(0.0);
    let h = &assets.hatch;

    // Holed floor (static) — the start tile with a round opening. The top reuses
    // the green start material so the surface matches a plain start cell viewed
    // from above; a separate down-facing underside mesh caps it so the level below
    // sees plain stone around the hole.
    let pos = Transform::from_xyz(cx, floor_y, cz);
    match (h.hole_mesh.clone(), assets.start_mat.clone()) {
        (Some(mesh), Some(green)) => {
            commands.spawn((FloorCell, placement.tag(), pos, Mesh3d(mesh), MeshMaterial3d(green)));
            // Stone underside cap only on an open-sky stack; a roofed level below
            // caps the opening with its own holed roof tile instead. The down-facing
            // `underside_mesh` (bottom-face UV) matches the surrounding floor
            // undersides, dropped to the gap bottom (`floor_y - gap`) so it's flush
            // with the sealed cells on a lifted (pool) level.
            if !below_roofed {
                if let (Some(under), Some(stone)) = (h.underside_mesh.clone(), h.hole_mat.clone()) {
                    let cap = Transform::from_xyz(cx, floor_y - gap, cz);
                    commands.spawn((HatchUnderside, placement.tag(), cap, Mesh3d(under), MeshMaterial3d(stone)));
                }
            }
        }
        _ => {
            commands.spawn((FloorCell, placement.tag(), pos));
        }
    }
    // Rim (static) — metal ring framing the opening.
    if let (Some(mesh), Some(mat)) = (h.rim_mesh.clone(), h.metal_mat.clone()) {
        commands.spawn((
            placement.tag(),
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
        (Some(mesh), Some(mat)) => commands.spawn((hatch, placement.tag(), xform, Mesh3d(mesh), MeshMaterial3d(mat))).id(),
        _ => commands.spawn((hatch, placement.tag(), xform)).id(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LayeredAlignment;

    /// A minimal `FloorAssets` whose holed-floor mesh + start/hole materials are set
    /// (the rest left `None`), so `spawn_hatch` reaches the cap-spawning branch.
    fn dummy_assets() -> FloorAssets {
        FloorAssets {
            floor_mesh: None,
            pool_edge_ns_mesh: None,
            pool_edge_ew_mesh: None,
            tile_mat: None,
            start_mat: Some(Handle::default()),
            finish_mat: None,
            lines: crate::world::floor::lines::LineAssets {
                line_ew: None,
                line_ns: None,
                line_mat: None,
            },
            hatch: HatchAssets {
                hole_mesh: Some(Handle::default()),
                underside_mesh: Some(Handle::default()),
                lid_mesh: None,
                rim_mesh: None,
                wheel_mesh: None,
                spoke_mesh: None,
                metal_mat: None,
                lid_mat: None,
                hole_mat: Some(Handle::default()),
            },
        }
    }

    #[test]
    fn the_underside_cap_drops_to_the_gap_bottom_on_a_lifted_level() {
        // A level lifted by `gap` for its pools: the stone underside cap must sit at
        // the gap bottom (`floor_y - gap`), flush with the surrounding sealed cells —
        // not floating a `gap` above them.
        let gap = 0.7_f32;
        let base_y = 3.7_f32; // a single upper level whose floor is lifted by the gap
        let placement = LevelPlacement::for_level(1, &[(1, 1), (1, 1)], LayeredAlignment::Edge, base_y, 0);
        let assets = dummy_assets();
        let mut app = App::new();
        app.add_systems(Update, move |mut commands: Commands| {
            spawn_hatch(&mut commands, &assets, 0, 0, placement, false, gap);
        });
        app.update();
        let mut q = app.world_mut().query_filtered::<&Transform, With<HatchUnderside>>();
        let cap = q.iter(app.world()).next().expect("a hatch underside cap spawned");
        assert!(
            (cap.translation.y - (base_y - gap)).abs() < 1e-6,
            "cap y {} should be the gap bottom {}",
            cap.translation.y,
            base_y - gap,
        );
    }

    #[test]
    fn the_underside_cap_is_skipped_when_the_level_below_is_roofed() {
        // On a roofed (dungeon/chamber) stack the level below caps the opening with
        // its own holed roof tile, so the hatch must NOT also spawn an underside —
        // otherwise the two overlap. `below_roofed = true` ⇒ no `HatchUnderside`.
        let placement = LevelPlacement::for_level(1, &[(1, 1), (1, 1)], LayeredAlignment::Edge, 3.0, 0);
        let assets = dummy_assets();
        let mut app = App::new();
        app.add_systems(Update, move |mut commands: Commands| {
            spawn_hatch(&mut commands, &assets, 0, 0, placement, true, 0.0);
        });
        app.update();
        assert_eq!(
            app.world_mut().query::<&HatchUnderside>().iter(app.world()).count(),
            0,
            "a roofed level below leaves the underside to its holed roof tile",
        );
    }

    #[test]
    fn the_underside_mesh_faces_down_with_the_cuboid_bottom_face_uv() {
        // The underside the level below sees must match the surrounding floor tiles'
        // undersides (Bevy Cuboid bottom faces): normals down, and UVs
        // `u = (h - x)/size, v = (h - z)/size` — NOT the up-facing top-face mapping
        // the green surface uses, which would read 180° rotated (lighter, mis-tiled).
        let mesh = hole_underside_mesh();
        let h = CELL_SIZE / 2.0;
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("expected Float32x3 normals");
        };
        assert!(normals.iter().all(|n| *n == [0.0, -1.0, 0.0]), "underside normals face down");
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("expected Float32x3 positions");
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("expected Float32x2 UVs");
        };
        for (p, uv) in positions.iter().zip(uvs) {
            let want = [(h - p[0]) / CELL_SIZE, (h - p[2]) / CELL_SIZE];
            assert!(
                (uv[0] - want[0]).abs() < 1e-4 && (uv[1] - want[1]).abs() < 1e-4,
                "underside UV {uv:?} should match the cuboid bottom face {want:?}"
            );
        }
    }
}
