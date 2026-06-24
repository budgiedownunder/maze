//! Non-occluding lava cell — a molten pool recessed in a basin below floor level.
//! The cell has no floor tile or grid lines; the recessed surface *is* the bottom
//! of the basin, so adjacent lava cells abut edge-to-edge into one continuous
//! flow. The rim wall up to floor level is drawn by [`super::rim`]. The surface
//! glows a hot orange and sits low, so — with the wall panels around it suppressed
//! (see [`super::solid::spawn_walls_for_cell`]) — the player sees across it.
//!
//! [`lava_animation_system`] bubbles the molten surface (a slightly more agitated
//! version of the water wave) and bobs a handful of small dark rocks up through it
//! and back under, so the pool reads as gently boiling. Both are phased by world
//! position so the motion flows across adjacent lava cells.

use super::rim::RECESS_DEPTH;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{lcg, LevelPlacement, CELL_SIZE};
use bevy::asset::RenderAssetUsages;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::mesh::PrimitiveTopology;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// Thin vertical extent of the surface sheet — small, so it reads as a flat
/// molten surface rather than a block. Matches the water surface.
const SURFACE_THICKNESS: f32 = 0.04;

/// Y of the surface — recessed [`RECESS_DEPTH`] below the surrounding floor tops
/// (≈ 0), matching the water surface so the two pool types sit at the same level.
/// The rim skirt ([`super::rim`]) fills the band up to the floor.
const SURFACE_Y: f32 = -RECESS_DEPTH;

/// Surface emissive — a deep, rich molten orange: a high red drive with just
/// enough green to pull it off pure red into orange (and no blue), brighter than
/// the water so the lava reads as an intense light source in the dim corridors. A
/// scrolling ripple texture modulates it into a flowing molten surface.
const LAVA_EMISSIVE: LinearRgba = LinearRgba::new(2.40, 0.45, 0.02, 1.0);

/// Ripple-texture repeats per cell (integer → seamless across cells), plus the
/// per-second UV scroll. Coarser, broader swells than the water (the
/// original pool-ripple look) reading as a slowly flowing molten crust.
const RIPPLE_UV: Vec2 = Vec2::new(2.0, 2.0);
const RIPPLE_SCROLL: Vec2 = Vec2::new(0.012, 0.008);
/// Plane-wave frequencies for the lava ripple texture — a few low-frequency
/// waves giving broad molten swells.
const RIPPLE_WAVES: &[(f32, f32)] = &[(2.0, 0.0), (0.0, 3.0), (2.0, 2.0)];
/// Contrast of the ripple texture around its mid grey.
const RIPPLE_AMP: f32 = 0.20;

/// Surface bubbling — a slightly larger, faster wave than the water so the molten
/// surface reads as agitated rather than calm. Amplitude stays small relative to
/// [`RECESS_DEPTH`] so the lava keeps to its basin.
const WAVE_AMP: f32 = 0.05;
const WAVE_K: f32 = 0.9;
const WAVE_SPEED: f32 = 1.1;

/// Number of dark rocks bobbing on each lava cell.
const ROCK_COUNT: usize = 3;
/// Rock geometry: a cube of half-size [`ROCK_HALF`] with each corner sliced back
/// by [`ROCK_CHAMFER`] (a truncated cube). The cuts add small corner + face facets
/// where the cube's faces met, so it reads as a chunky rock rather than a plain
/// cube or a smooth ball.
const ROCK_HALF: f32 = 0.085;
const ROCK_CHAMFER: f32 = 0.032;
/// Local `(x, z)` offsets of the rocks within the cell (relative to its centre),
/// kept inside ±0.55 so they don't poke past the cell edges. Distinct positions
/// also de-sync their bob (the world-position phase differs per rock).
const ROCK_OFFSETS: [(f32, f32); ROCK_COUNT] = [(-0.45, -0.30), (0.40, 0.45), (0.05, -0.52)];
/// Per-rock non-uniform scale so the three lumps read as distinct irregular
/// boulders rather than identical blocks. Paired by index with [`ROCK_OFFSETS`].
/// An overall size factor is baked in per rock so they range from full size down
/// to about half (1.0 / 0.75 / 0.55).
const ROCK_SCALES: [Vec3; ROCK_COUNT] = [
    Vec3::new(1.00, 0.78, 1.18),
    Vec3::new(0.92, 0.69, 0.62),
    Vec3::new(0.47, 0.62, 0.56),
];
/// Vertical travel of a rock as it rises above and sinks below the surface.
const ROCK_AMP: f32 = 0.10;
/// How far below the surface a rock's bob is centred, so it spends most of its
/// cycle submerged and only emerges a little (kept low so the rocks barely rise
/// out of the lava).
const ROCK_SINK: f32 = 0.07;
/// Temporal speed (radians/sec) of a rock's rise/fall.
const ROCK_SPEED: f32 = 1.6;
/// World-position phase frequency (radians/unit) so rocks across cells desync.
const ROCK_K: f32 = 1.3;
/// Slow tumble rate (radians/sec) for a little life.
const ROCK_SPIN: f32 = 0.23;
/// Rock emissive — near-black with a faint warmth, so cooled crust reads dark
/// against the bright molten surface.
const ROCK_EMISSIVE: LinearRgba = LinearRgba::new(0.06, 0.02, 0.0, 1.0);

/// Marker on a lava pool surface. Spawned per non-occluding lava `'W'` cell;
/// [`lava_animation_system`] queries it to bubble the surface. The stored
/// `base_y` is the pool's level floor Y (`base_level_y[level]`), so the animation
/// keeps the surface at its stacked — possibly pool-gap-lifted — height.
#[derive(Component)]
pub(crate) struct LavaSurface {
    base_y: f32,
}

/// Marker on a dark rock bobbing on a lava surface. [`lava_animation_system`]
/// raises and lowers it through the molten surface, phased by its world position.
/// The stored `base_y` keeps the bob centred on the pool's stacked floor Y.
#[derive(Component)]
pub(crate) struct LavaRock {
    base_y: f32,
}

pub(crate) struct LavaAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    rock_mesh: Option<Handle<Mesh>>,
    rock_material: Option<Handle<StandardMaterial>>,
}

/// Builds a flat-shaded truncated-cube rock mesh: a cube of half-size `h` with
/// every corner sliced back by `d`. Each corner becomes a small triangle and each
/// face a trimmed octagon, giving the blocky-but-faceted look of a rough rock.
/// Triangle winding is auto-oriented outward, so the face lists need not track it.
fn build_rock_mesh(h: f32, d: f32) -> Mesh {
    let signs = [-1.0f32, 1.0];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Emit one flat-shaded triangle, orienting it so its normal points away from
    // the origin (outward) regardless of the vertex order passed in.
    let mut push_tri = |a: Vec3, b: Vec3, c: Vec3| {
        let n0 = (b - a).cross(c - a).normalize_or_zero();
        let centroid = (a + b + c) / 3.0;
        let (v0, v1, v2, n) = if n0.dot(centroid) >= 0.0 {
            (a, b, c, n0)
        } else {
            (a, c, b, -n0)
        };
        for v in [v0, v1, v2] {
            positions.push(v.to_array());
            normals.push(n.to_array());
            uvs.push([0.0, 0.0]);
        }
    };

    // One small triangle per cut corner.
    for &sx in &signs {
        for &sy in &signs {
            for &sz in &signs {
                let vx = Vec3::new(sx * (h - d), sy * h, sz * h);
                let vy = Vec3::new(sx * h, sy * (h - d), sz * h);
                let vz = Vec3::new(sx * h, sy * h, sz * (h - d));
                push_tri(vx, vy, vz);
            }
        }
    }

    // A trimmed octagon per cube face — eight rim vertices fan-triangulated.
    let make = |axis: usize, s: f32, o1: f32, o2: f32| -> Vec3 {
        match axis {
            0 => Vec3::new(s * h, o1, o2),
            1 => Vec3::new(o1, s * h, o2),
            _ => Vec3::new(o1, o2, s * h),
        }
    };
    for axis in 0..3 {
        for &s in &signs {
            let mut pts: Vec<Vec3> = Vec::new();
            for &sa in &signs {
                for &sb in &signs {
                    pts.push(make(axis, s, sa * (h - d), sb * h));
                    pts.push(make(axis, s, sa * h, sb * (h - d)));
                }
            }
            // Order the rim CCW around the face centre, using the two in-plane axes.
            let plane = |v: Vec3| -> (f32, f32) {
                match axis {
                    0 => (v.y, v.z),
                    1 => (v.x, v.z),
                    _ => (v.x, v.y),
                }
            };
            pts.sort_by(|&p, &q| {
                let (pa, pb) = plane(p);
                let (qa, qb) = plane(q);
                pb.atan2(pa).total_cmp(&qb.atan2(qa))
            });
            for i in 1..pts.len() - 1 {
                push_tri(pts[0], pts[i], pts[i + 1]);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

pub(crate) fn build_lava_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> LavaAssets {
    // Full-cell slab (no border inset) so adjacent pools meet seamlessly.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, SURFACE_THICKNESS, CELL_SIZE)));
    let ripple = images
        .as_mut()
        .map(|imgs| super::ripple_texture(imgs, RIPPLE_WAVES, RIPPLE_AMP));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Lava is opaque — the molten surface hides what is beneath it (but
            // the player still sees *over* it because it is low and panel-free).
            // The ripple texture (scrolled by lava_animation_system) modulates the
            // emissive into a flowing molten crust.
            base_color: EMISSIVE_ONLY_BASE,
            emissive: LAVA_EMISSIVE,
            emissive_texture: ripple,
            uv_transform: Affine2::from_scale(RIPPLE_UV),
            ..default()
        })
    });
    // A truncated cube (stretched per-rock at spawn) — a cube with cut corners,
    // so it keeps the blocky rock read but gains small facets at the cuts.
    let rock_mesh = meshes
        .as_mut()
        .map(|m| m.add(build_rock_mesh(ROCK_HALF, ROCK_CHAMFER)));
    let rock_material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: ROCK_EMISSIVE,
            ..default()
        })
    });
    LavaAssets {
        mesh,
        material,
        rock_mesh,
        rock_material,
    }
}

/// Spawns the recessed lava pool surface filling cell `(r, c)` on run level
/// `level`, plus its bobbing rocks. The caller spawns the rim ([`super::rim`]);
/// the cell has no floor tile.
pub(crate) fn spawn_lava(
    commands: &mut Commands,
    assets: &LavaAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let surface_y = placement.world_y(SURFACE_Y);
    // The animations re-derive Y from the stored floor base; X/Z (offset above) are fixed.
    let base_y = placement.base_y();
    match (assets.mesh.clone(), assets.material.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                LavaSurface { base_y },
                Transform::from_xyz(x, surface_y, z),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
        _ => {
            commands.spawn((LavaSurface { base_y }, Transform::from_xyz(x, surface_y, z)));
        }
    }
    for (i, &(dx, dz)) in ROCK_OFFSETS.iter().enumerate() {
        let pos = Vec3::new(x + dx, surface_y - ROCK_SINK, z + dz);
        // Non-uniform scale stretches the sphere into an irregular boulder; the
        // animation system only rewrites the rock's Y and rotation, so this scale
        // persists.
        let transform = Transform::from_translation(pos).with_scale(ROCK_SCALES[i]);
        match (assets.rock_mesh.clone(), assets.rock_material.clone()) {
            (Some(mesh), Some(mat)) => {
                commands.spawn((LavaRock { base_y }, transform, Mesh3d(mesh), MeshMaterial3d(mat)));
            }
            _ => {
                commands.spawn((LavaRock { base_y }, transform));
            }
        };
    }
}

/// `Update` system: bubbles every lava surface (a more agitated [`super::pool_wave`]
/// than water) and bobs each [`LavaRock`] up through the surface and back under.
/// Both are phased by world `(x, z)` so the motion flows across adjacent lava
/// cells. The two `&mut Transform` queries are kept disjoint by marker.
pub(crate) fn lava_animation_system(
    time: Res<Time>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut surfaces: Query<(&mut Transform, &LavaSurface), Without<LavaRock>>,
    mut rocks: Query<(&mut Transform, &LavaRock), Without<LavaSurface>>,
    surface_mats: Query<&MeshMaterial3d<StandardMaterial>, With<LavaSurface>>,
) {
    let t = time.elapsed_secs();
    for (mut tr, surface) in surfaces.iter_mut() {
        let (dy, rot) = super::pool_wave(tr.translation.x, tr.translation.z, t, WAVE_AMP, WAVE_K, WAVE_SPEED);
        tr.translation.y = surface.base_y + SURFACE_Y + dy;
        tr.rotation = rot;
    }
    for (mut tr, rock) in rocks.iter_mut() {
        let phase = (tr.translation.x + tr.translation.z) * ROCK_K;
        tr.translation.y =
            rock.base_y + (SURFACE_Y - ROCK_SINK) + ROCK_AMP * (t * ROCK_SPEED + phase).sin();
        tr.rotation = Quat::from_rotation_y(t * ROCK_SPIN) * Quat::from_rotation_x(t * ROCK_SPIN * 0.6);
    }
    // Drift the shared ripple texture (one material across all lava tiles).
    if let (Some(materials), Some(handle)) = (materials.as_mut(), surface_mats.iter().next()) {
        if let Some(mat) = materials.get_mut(&handle.0) {
            mat.uv_transform.translation = RIPPLE_SCROLL * t;
        }
    }
}

// ---------- Steam dots ----------

/// Radius of a steam-dot mesh, before per-particle scaling. Minuscule — at peak
/// size a dot is barely a couple of pixels, so a cluster reads as a wisp of fine
/// vapour rather than floating balls.
const STEAM_RADIUS: f32 = 0.008;
/// Translucency of a steam dot — a little more opaque than before so the tiny
/// dots still register.
const STEAM_ALPHA: f32 = 0.50;
/// Warm-grey glow so the tiny dots stay visible in the dim corridors.
const STEAM_EMISSIVE: LinearRgba = LinearRgba::new(0.42, 0.38, 0.34, 1.0);
/// Seconds between steam bursts.
const STEAM_INTERVAL: f32 = 0.08;
/// Emit points per burst — one random spot over the lava each burst.
const STEAM_POINTS: usize = 1;
/// Dots emitted from each point in a burst. Many tiny dots from the *same* spot
/// read as one rising wisp of steam.
const STEAM_CLUSTER: usize = 6;
/// Horizontal spread of a cluster's dots around its emit point.
const STEAM_JITTER: f32 = 0.035;
/// How far a dot rises above the surface before it has fully dissipated. Capped
/// so the peak (`STEAM_RISE_MIN + STEAM_RISE_VAR` above the surface at
/// `-RECESS_DEPTH`) stays below floor level (`y = 0`): the steam fades out inside
/// the basin rather than rising out of the pit.
const STEAM_RISE_MIN: f32 = 0.12;
const STEAM_RISE_VAR: f32 = 0.13;
/// Lifetime (seconds) range of a dot — paired with the short rise so the dots
/// drift up slowly (the rise distance and lifetime were both cut from the earlier
/// taller version, keeping that slow drift speed).
const STEAM_LIFE_MIN: f32 = 1.0;
const STEAM_LIFE_VAR: f32 = 0.9;
// The steam must dissipate before it reaches floor level: the highest a dot can
// rise is `STEAM_RISE_MIN + STEAM_RISE_VAR` above the surface at `-RECESS_DEPTH`.
const _: () = assert!(STEAM_RISE_MIN + STEAM_RISE_VAR < RECESS_DEPTH);
/// Horizontal sway amplitude (units) as a wisp curls while rising.
const STEAM_SWAY: f32 = 0.13;

/// A rising steam dot emitted from a lava surface. Animated by
/// [`lava_steam_system`]: it rises a short distance, drifts, grows in then shrinks
/// to nothing (dissipating), and is despawned at the end of its life.
#[derive(Component)]
pub(crate) struct LavaSteam {
    age: f32,
    lifetime: f32,
    base: Vec3,
    rise: f32,
    sway_x: f32,
    sway_z: f32,
    sway_freq: f32,
}

/// Shared steam-wisp mesh + material, inserted as a resource by `spawn_world` so
/// [`lava_steam_system`] can emit wisps without rebuilding assets each spawn.
#[derive(Resource)]
pub(crate) struct LavaSteamAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_lava_steam_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> LavaSteamAssets {
    let mesh = meshes.as_mut().map(|m| m.add(Sphere::new(STEAM_RADIUS)));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::srgba(0.88, 0.84, 0.80, STEAM_ALPHA),
            emissive: STEAM_EMISSIVE,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    });
    LavaSteamAssets { mesh, material }
}

/// The scale envelope of a wisp over its normalised life `p` in `0..=1`: zero at
/// birth and death, peaking at mid-life, so the wisp grows in and then tapers to
/// nothing (dissipates).
fn steam_puff(p: f32) -> f32 {
    (p * PI).sin()
}

/// `Update` system: rises and dissipates existing steam wisps, and sparsely emits
/// new ones scattered over the lava surfaces. Spawn positions are sampled from the
/// live [`LavaSurface`] tiles, so steam only comes off actual lava.
pub(crate) fn lava_steam_system(
    mut commands: Commands,
    time: Res<Time>,
    steam_assets: Option<Res<LavaSteamAssets>>,
    lava: Query<(&Transform, &LavaSurface)>,
    mut wisps: Query<(Entity, &mut LavaSteam, &mut Transform), Without<LavaSurface>>,
    mut rng: Local<u64>,
    mut timer: Local<f32>,
) {
    let dt = time.delta_secs();

    // Advance existing wisps.
    for (entity, mut wisp, mut tr) in wisps.iter_mut() {
        wisp.age += dt;
        let p = wisp.age / wisp.lifetime;
        if p >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Ease-out rise (fast off the surface, slowing as it fades) + a curling
        // horizontal sway that grows with height.
        let rise = wisp.rise * (1.0 - (1.0 - p).powi(2));
        let wob = (p * wisp.sway_freq).sin() * p;
        tr.translation = wisp.base + Vec3::new(wisp.sway_x * wob, rise, wisp.sway_z * wob);
        tr.scale = Vec3::splat(steam_puff(p));
    }

    // Emit new wisps, sparsely, from random points over the lava.
    let Some(assets) = steam_assets else {
        return;
    };
    // The resting surface position of each lava cell (its level-correct Y, not the
    // bobbing Y), so a wisp sits on the pool at the right stacked height.
    let cells: Vec<Vec3> = lava
        .iter()
        .map(|(t, surface)| Vec3::new(t.translation.x, surface.base_y + SURFACE_Y, t.translation.z))
        .collect();
    if cells.is_empty() {
        return;
    }
    if *rng == 0 {
        *rng = time.elapsed_secs_f64().to_bits() | 1;
    }
    *timer += dt;
    while *timer >= STEAM_INTERVAL {
        *timer -= STEAM_INTERVAL;
        for _ in 0..STEAM_POINTS {
            let idx = (lcg(&mut rng) * cells.len() as f32) as usize % cells.len();
            let cell = cells[idx];
            // An emit point scattered within the cell (±0.6) at the resting
            // surface level (not the bobbing Y, so the wisp sits on the pool).
            let ox = (lcg(&mut rng) - 0.5) * 1.2;
            let oz = (lcg(&mut rng) - 0.5) * 1.2;
            let point = Vec3::new(cell.x + ox, cell.y, cell.z + oz);
            // A cluster of tiny dots from that one point.
            for _ in 0..STEAM_CLUSTER {
                let jx = (lcg(&mut rng) - 0.5) * 2.0 * STEAM_JITTER;
                let jz = (lcg(&mut rng) - 0.5) * 2.0 * STEAM_JITTER;
                let base = point + Vec3::new(jx, 0.0, jz);
                let dot = LavaSteam {
                    age: 0.0,
                    lifetime: STEAM_LIFE_MIN + lcg(&mut rng) * STEAM_LIFE_VAR,
                    base,
                    rise: STEAM_RISE_MIN + lcg(&mut rng) * STEAM_RISE_VAR,
                    sway_x: (lcg(&mut rng) - 0.5) * 2.0 * STEAM_SWAY,
                    sway_z: (lcg(&mut rng) - 0.5) * 2.0 * STEAM_SWAY,
                    sway_freq: 3.0 + lcg(&mut rng) * 3.0,
                };
                match (assets.mesh.clone(), assets.material.clone()) {
                    (Some(mesh), Some(mat)) => {
                        commands.spawn((
                            dot,
                            Transform::from_translation(base).with_scale(Vec3::ZERO),
                            Mesh3d(mesh),
                            MeshMaterial3d(mat),
                        ));
                    }
                    _ => {
                        commands.spawn((dot, Transform::from_translation(base).with_scale(Vec3::ZERO)));
                    }
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rock_mesh_is_a_closed_truncated_cube() {
        let mesh = build_rock_mesh(1.0, 0.3);
        let pos = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let norm = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float3()
            .unwrap();
        // 8 corner triangles + 6 octagon faces (6 fan triangles each) = 44 tris.
        assert_eq!(pos.len(), 44 * 3);
        assert_eq!(norm.len(), pos.len());
        // Flat-shaded normals are unit length and point outward (positive dot with
        // their own position — the solid is convex and centred on the origin).
        for (p, n) in pos.iter().zip(norm.iter()) {
            let nv = Vec3::from_array(*n);
            assert!((nv.length() - 1.0).abs() < 1e-3, "non-unit normal {nv:?}");
            assert!(nv.dot(Vec3::from_array(*p)) > 0.0, "normal not outward");
        }
    }

    #[test]
    fn steam_puff_grows_in_then_dissipates() {
        // Zero at birth and death (no wisp), peaking at mid-life.
        assert!(steam_puff(0.0).abs() < 1e-6);
        assert!(steam_puff(1.0).abs() < 1e-6);
        assert!((steam_puff(0.5) - 1.0).abs() < 1e-6);
        // Strictly positive and below the peak partway through.
        let mid = steam_puff(0.25);
        assert!(mid > 0.0 && mid < 1.0, "got {mid}");
    }
}
