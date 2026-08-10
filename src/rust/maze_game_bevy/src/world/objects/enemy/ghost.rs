//! Ghost enemy rig — a translucent floating figure with a rounded head
//! and a flowing sheet-like body. The body hovers above the floor; its
//! hem ripples to suggest a flowing cloth. Two arch-shaped black eyes
//! sit on the face, each with a central black eyeball; there's no mouth.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{icosphere, CELL_SIZE, LevelPlacement};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use std::f32::consts::{FRAC_PI_2, TAU};

// ---------- Tuning constants ----------

/// Hemisphere head radius. The hemisphere's flat side sits at the
/// rig-root Y plane (Y = 0); the dome bulges up to Y = HEAD_RADIUS.
const HEAD_RADIUS: f32 = 0.22;
/// Latitude / longitude subdivisions for the hemisphere head mesh —
/// enough that the dome reads smooth at typical viewing distances.
const HEAD_LATITUDES: u32 = 14;
const HEAD_LONGITUDES: u32 = 24;

/// Truncated-cone body. The top radius matches `HEAD_RADIUS` so the cone
/// joins the hemisphere flush at Y = 0; the bottom flares wider for a
/// sheet-like silhouette.
const BODY_TOP_RADIUS: f32 = HEAD_RADIUS;
const BODY_BOTTOM_RADIUS: f32 = 0.32;
const BODY_HEIGHT: f32 = 0.50;
/// `ConicalFrustum` is centred at its midpoint, so positioning the body
/// at Y = -BODY_HEIGHT/2 puts its top face at Y = 0 (the join plane)
/// and its bottom face at Y = -BODY_HEIGHT.
const BODY_CENTRE_Y: f32 = -BODY_HEIGHT / 2.0;

/// Resting Y position of the rig — keeps the entire ghost floating well
/// above the floor so the rippling hem reads clearly.
pub(crate) const ENEMY_BASE_Y: f32 = 0.95;
/// Idle bob frequency (radians/sec) for the entire rig.
pub(crate) const BOB_RATE: f32 = 1.6;
/// Idle bob amplitude (units of vertical travel from the resting Y).
pub(crate) const BOB_AMPLITUDE: f32 = 0.06;

/// Number of small spheres around the bottom hem — more spheres make a
/// smoother wave.
const HEM_SPHERE_COUNT: usize = 14;
/// Hem sphere radius — small enough that adjacent spheres kiss when
/// arranged in a ring at the body's bottom radius.
const HEM_SPHERE_RADIUS: f32 = 0.07;
/// Y position of the hem ring centre — flush with the bottom of the
/// truncated-cone body so the spheres look like the cone's flared
/// bottom edge.
const HEM_OFFSET_Y: f32 = -BODY_HEIGHT;
/// Vertical wave amplitude for the hem ripple — how far each hem sphere
/// rises and falls from the ring's centre line.
const HEM_WAVE_AMPLITUDE: f32 = 0.05;
/// Wave temporal rate in radians per second.
const HEM_WAVE_RATE: f32 = 3.0;
/// Number of full wave crests around the ring — higher means more
/// undulations visible at once.
const HEM_WAVE_CRESTS: f32 = 3.0;

// Eye geometry — each eye is an arch outline (row of small black
// spheres along a parabolic curve) with a single larger black eyeball
// sphere sitting inside the arch.
const EYE_ARC_SPHERES: usize = 7;
const EYE_ARC_SPHERE_RADIUS: f32 = 0.018;
/// Half-width of each arch eye (units).
const EYE_ARC_HALF_W: f32 = 0.06;
/// Vertical span of the arch from baseline to apex (units).
const EYE_ARC_HEIGHT: f32 = 0.04;
/// Horizontal offset of each eye centre from the head's midline.
const EYE_OFFSET_X: f32 = 0.10;
/// Vertical position of the eye baseline (the row's lowest sphere)
/// relative to the rig root. The hemisphere head spans Y ∈ [0, R];
/// the baseline sits roughly a third of the way up so the arch apex
/// lands near the hemisphere's mid-height.
const EYE_OFFSET_Y: f32 = HEAD_RADIUS * 0.30;
/// Forward offset of the eye spheres — slightly inside the front face of
/// the hemisphere head at the eye's Y so they sit visibly proud.
const EYE_OFFSET_Z: f32 = HEAD_RADIUS - 0.04;
/// Central eyeball radius — larger than the arch-outline spheres so it
/// reads as a pupil rather than another arch dot.
const EYEBALL_RADIUS: f32 = 0.025;
/// Vertical position of the central eyeball — midway between the arch
/// baseline and the apex, so the eyeball sits inside the arch.
const EYEBALL_OFFSET_Y: f32 = EYE_OFFSET_Y + EYE_ARC_HEIGHT * 0.5;

// Material constants ----------

/// Ghost body emissive — slightly cool off-white so the ghost reads
/// distinctly against the warmer corridor walls. Paired with a low alpha
/// on the base colour for the translucent look.
const BODY_EMISSIVE: LinearRgba = LinearRgba::new(0.85, 0.90, 1.00, 1.0);
/// Translucent alpha — high enough that the silhouette reads from across
/// a corridor, low enough that the player can see what's behind a hem
/// sphere through it.
const BODY_ALPHA: f32 = 0.55;
/// Eye emissive — pure black, paired with the BLACK base colour so each
/// arch sphere reads as a flat dark dot regardless of corridor lighting.
const EYE_EMISSIVE: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 1.0);
/// Eyeball emissive — a saturated red that self-illuminates so the
/// pupils glow against the dark arch outline regardless of corridor
/// lighting.
const EYEBALL_EMISSIVE: LinearRgba = LinearRgba::new(2.5, 0.05, 0.05, 1.0);

/// Zero-cost tag identifying a ghost-variant enemy at the root marker
/// level. Spawned unconditionally so headless tests (no `Assets<Mesh>`
/// plugin) can distinguish ghost from goblin without depending on child
/// entities, which only spawn when the rig assets are available.
#[derive(Component)]
pub(crate) struct GhostTag;

#[derive(Component)]
pub(crate) struct GhostHemSphere {
    /// Index 0..HEM_SPHERE_COUNT — drives the per-sphere phase offset in
    /// the wave animation so adjacent spheres rise and fall at slightly
    /// different times, producing a flowing-sheet ripple.
    pub(crate) index: usize,
}

pub(crate) struct GhostAssets {
    head_mesh: Option<Handle<Mesh>>,
    body_mesh: Option<Handle<Mesh>>,
    hem_sphere_mesh: Option<Handle<Mesh>>,
    eye_sphere_mesh: Option<Handle<Mesh>>,
    eyeball_mesh: Option<Handle<Mesh>>,
    translucent_mat: Option<Handle<StandardMaterial>>,
    eye_mat: Option<Handle<StandardMaterial>>,
    eyeball_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_ghost_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> GhostAssets {
    let head_mesh = meshes
        .as_mut()
        .map(|m| m.add(build_hemisphere_mesh(HEAD_RADIUS, HEAD_LATITUDES, HEAD_LONGITUDES)));
    let body_mesh = meshes.as_mut().map(|m| {
        m.add(ConicalFrustum {
            radius_top: BODY_TOP_RADIUS,
            radius_bottom: BODY_BOTTOM_RADIUS,
            height: BODY_HEIGHT,
        })
    });
    let hem_sphere_mesh = meshes
        .as_mut()
        .map(|m| m.add(icosphere(HEM_SPHERE_RADIUS, 2)));
    let eye_sphere_mesh = meshes
        .as_mut()
        .map(|m| m.add(icosphere(EYE_ARC_SPHERE_RADIUS, 1)));
    let eyeball_mesh = meshes
        .as_mut()
        .map(|m| m.add(icosphere(EYEBALL_RADIUS, 1)));
    let translucent_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Translucent: the base colour carries the alpha (alpha blend
            // is driven by `base_color.alpha`), while the emissive carries
            // the colour so corridor lighting doesn't multiply into it.
            base_color: Color::srgba(1.0, 1.0, 1.0, BODY_ALPHA),
            emissive: BODY_EMISSIVE,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    });
    let eye_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: EYE_EMISSIVE,
            ..default()
        })
    });
    let eyeball_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: EYEBALL_EMISSIVE,
            ..default()
        })
    });
    GhostAssets {
        head_mesh,
        body_mesh,
        hem_sphere_mesh,
        eye_sphere_mesh,
        eyeball_mesh,
        translucent_mat,
        eye_mat,
        eyeball_mat,
    }
}

/// Spawns the Ghost entity hierarchy for the `'E'` cell at `(r, c)` with
/// the given enemy `id`. The marker is the root transform; head, body,
/// hem spheres, and eye-arch spheres are all children inheriting it.
pub(crate) fn spawn_ghost(
    commands: &mut Commands,
    assets: &GhostAssets,
    r: usize,
    c: usize,
    id: u32,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let root = commands
        .spawn((
            placement.tag(),
            super::EnemyMarker {
                id,
                spawn_cell: (r, c),
                placement,
            },
            GhostTag,
            Transform::from_xyz(x, placement.world_y(ENEMY_BASE_Y), z),
            Visibility::default(),
        ))
        .id();
    let (
        Some(head_mesh),
        Some(body_mesh),
        Some(hem_mesh),
        Some(eye_mesh),
        Some(eyeball_mesh),
        Some(trans_mat),
        Some(eye_mat),
        Some(eyeball_mat),
    ) = (
        assets.head_mesh.clone(),
        assets.body_mesh.clone(),
        assets.hem_sphere_mesh.clone(),
        assets.eye_sphere_mesh.clone(),
        assets.eyeball_mesh.clone(),
        assets.translucent_mat.clone(),
        assets.eye_mat.clone(),
        assets.eyeball_mat.clone(),
    )
    else {
        return;
    };
    commands.entity(root).with_children(|parent| {
        // Hemisphere head — flat side at Y = 0, dome bulging up to
        // Y = HEAD_RADIUS. Sits flush atop the cone body which has its
        // top face at Y = 0.
        parent.spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(trans_mat.clone()),
            Transform::default(),
        ));
        // Truncated-cone body — top radius matches the head so the join
        // is flush; bottom flares wider for a sheet-like silhouette.
        parent.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(trans_mat.clone()),
            Transform::from_xyz(0.0, BODY_CENTRE_Y, 0.0),
        ));
        // Hem ring — small translucent spheres at the cone's bottom
        // edge, arranged in a circle of `BODY_BOTTOM_RADIUS`. Each
        // carries a `GhostHemSphere` with its index so
        // `ghost_hem_wave_system` can phase-offset its vertical bob.
        for index in 0..HEM_SPHERE_COUNT {
            let theta = TAU * (index as f32) / (HEM_SPHERE_COUNT as f32);
            let hx = BODY_BOTTOM_RADIUS * theta.cos();
            let hz = BODY_BOTTOM_RADIUS * theta.sin();
            parent.spawn((
                GhostHemSphere { index },
                Mesh3d(hem_mesh.clone()),
                MeshMaterial3d(trans_mat.clone()),
                Transform::from_xyz(hx, HEM_OFFSET_Y, hz),
            ));
        }
        // Two eyes on the front face of the head. Each eye is a black
        // arch outline (row of small spheres traced along a parabolic
        // curve — outer spheres sit at the baseline, the centre sphere
        // at the apex) plus a single larger glowing-red eyeball sphere
        // centred inside the arch.
        for side in [-1.0_f32, 1.0_f32] {
            let centre_x = side * EYE_OFFSET_X;
            for i in 0..EYE_ARC_SPHERES {
                let t = if EYE_ARC_SPHERES == 1 {
                    0.0
                } else {
                    -1.0 + 2.0 * (i as f32) / (EYE_ARC_SPHERES as f32 - 1.0)
                };
                // Parabolic arch: outer ends at the baseline (t² = 1),
                // centre at the apex (t² = 0).
                let x = centre_x + EYE_ARC_HALF_W * t;
                let y = EYE_OFFSET_Y + EYE_ARC_HEIGHT * (1.0 - t * t);
                parent.spawn((
                    Mesh3d(eye_mesh.clone()),
                    MeshMaterial3d(eye_mat.clone()),
                    Transform::from_xyz(x, y, EYE_OFFSET_Z),
                ));
            }
            // Central eyeball — single glowing-red sphere inside the
            // arch.
            parent.spawn((
                Mesh3d(eyeball_mesh.clone()),
                MeshMaterial3d(eyeball_mat.clone()),
                Transform::from_xyz(centre_x, EYEBALL_OFFSET_Y, EYE_OFFSET_Z),
            ));
        }
    });
}

/// `Update` system that ripples the hem spheres of every ghost in a
/// continuous wave around the body's perimeter. Each sphere's vertical
/// offset is `amplitude * sin(time * rate + index * 2π * crests / count)`
/// — phase-shifting by index produces a travelling wave around the ring,
/// while time multiplied by the rate keeps the wave flowing.
pub(crate) fn ghost_hem_wave_system(
    time: Res<Time>,
    mut hems: Query<(&GhostHemSphere, &mut Transform)>,
) {
    let t = time.elapsed_secs() * HEM_WAVE_RATE;
    for (hem, mut transform) in hems.iter_mut() {
        let phase = (hem.index as f32) * TAU * HEM_WAVE_CRESTS / (HEM_SPHERE_COUNT as f32);
        let offset = HEM_WAVE_AMPLITUDE * (t + phase).sin();
        transform.translation.y = HEM_OFFSET_Y + offset;
    }
}

/// Builds an open-bottom hemisphere mesh: a dome whose flat side sits at
/// `y = 0` and whose apex reaches `y = radius`. No bottom disc — the
/// truncated-cone body's top cap closes the silhouette at the join
/// plane, so a separate disc here would just stack a redundant face
/// behind the cone cap.
///
/// `latitudes` is the number of rings from apex to equator; `longitudes`
/// is the segments per ring. Both should be ≥ 1.
fn build_hemisphere_mesh(radius: f32, latitudes: u32, longitudes: u32) -> Mesh {
    let lat = latitudes.max(1);
    let lon = longitudes.max(3);
    let ring_size = lon + 1;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(((lat + 1) * ring_size) as usize);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(((lat + 1) * ring_size) as usize);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(((lat + 1) * ring_size) as usize);
    let mut indices: Vec<u32> = Vec::with_capacity((lat * lon * 6) as usize);

    for i in 0..=lat {
        // `phi` walks from 0 at the north pole to π/2 at the equator,
        // so the dome lives entirely in `y >= 0`.
        let phi = (i as f32 / lat as f32) * FRAC_PI_2;
        let y = radius * phi.cos();
        let ring_r = radius * phi.sin();
        for j in 0..=lon {
            let theta = (j as f32 / lon as f32) * TAU;
            let x = ring_r * theta.cos();
            let z = ring_r * theta.sin();
            positions.push([x, y, z]);
            // For a sphere centred at origin, the surface normal at any
            // point is simply the unit vector pointing from origin to
            // that point.
            let inv_r = if radius > 0.0 { 1.0 / radius } else { 0.0 };
            normals.push([x * inv_r, y * inv_r, z * inv_r]);
            uvs.push([j as f32 / lon as f32, i as f32 / lat as f32]);
        }
    }

    for i in 0..lat {
        for j in 0..lon {
            let a = i * ring_size + j;
            let b = (i + 1) * ring_size + j;
            let c = (i + 1) * ring_size + j + 1;
            let d = i * ring_size + j + 1;
            // Two triangles per quad, wound so the outward face is the
            // top half of the dome (the rendered surface for a ghost
            // approaching the camera).
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

