//! The **portal** transition rig drawn at an interim finish cell.
//!
//! An upright, semi-transparent luminescent cylinder the player steps into to
//! advance. Its aura is a set of thin concentric light rings that continuously
//! travel **down** the cylinder's length ([`portal_system`]), giving the column
//! a sense of flowing energy, capped by a fixed **silver** ring at the top and
//! bottom that frames the column. Unlike the ladder it needs nothing in the roof
//! above.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{LevelPlacement, CELL_SIZE, LevelTag};
use crate::world::visibility::LevelWindow;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Cylinder radius (units) — comfortably inside a cell so walls never clip it.
const PORTAL_RADIUS: f32 = 0.55;
/// Cylinder height (units) — a standing-doorway column, shorter than a full
/// level so it reads as an object rather than a pillar to the ceiling.
const PORTAL_HEIGHT: f32 = 2.4;
/// Clearance of the cylinder's base above the floor (units).
const PORTAL_BASE_Y: f32 = 0.05;
/// Luminescent body tint (cool cyan-violet), emissive.
const PORTAL_EMISSIVE: LinearRgba = LinearRgba::new(0.15, 0.65, 1.1, 1.0);
/// Body opacity — low, so the corridor reads through the column.
const PORTAL_ALPHA: f32 = 0.32;

/// Number of light rings travelling down the column at once.
const RING_COUNT: usize = 8;
/// Major radius of each ring (units) — hugs the cylinder surface.
const RING_RADIUS: f32 = PORTAL_RADIUS + 0.02;
/// Tube thickness of each travelling ring (units) — thin filaments of light.
const RING_THICKNESS: f32 = 0.0125;
/// Bright ring emissive RGB.
const RING_EMISSIVE: LinearRgba = LinearRgba::new(0.6, 1.4, 2.0, 1.0);
/// Downward travel rate of the rings, in full-length sweeps per second.
const RING_TRAVEL_RATE: f32 = 0.0875;

/// Tube thickness of the fixed top / bottom cap rings (units) — chunkier than the
/// travelling filaments so they read as solid silver frames.
const CAP_THICKNESS: f32 = 0.05;
/// Silver emissive RGB for the fixed cap rings — neutral and cool, distinct from
/// the cyan travelling rings.
const CAP_EMISSIVE: LinearRgba = LinearRgba::new(0.8, 0.82, 0.88, 1.0);

/// Marks the translucent portal body, so the transition step (and headless
/// tests) can find a portal rig.
#[derive(Component)]
pub(crate) struct FinishPortal;

/// One travelling light ring. `base_y` is the absolute world Y of the column's
/// base and `height` its length, so [`portal_system`] can map `phase`
/// (0 = top → 1 = bottom) to an absolute Y without re-reading the level offset.
#[derive(Component)]
pub(crate) struct PortalRing {
    base_y: f32,
    height: f32,
    phase: f32,
}

/// A fixed silver cap ring framing the top / bottom of the column. Carries no
/// phase — [`portal_system`] ignores it, so it stays put while the filaments
/// flow past.
#[derive(Component)]
pub(crate) struct PortalCap;

pub(crate) struct PortalAssets {
    pub(crate) body_mesh: Option<Handle<Mesh>>,
    pub(crate) body_mat: Option<Handle<StandardMaterial>>,
    pub(crate) ring_mesh: Option<Handle<Mesh>>,
    pub(crate) ring_mat: Option<Handle<StandardMaterial>>,
    pub(crate) cap_mesh: Option<Handle<Mesh>>,
    pub(crate) cap_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_portal_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> PortalAssets {
    let body_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cylinder::new(PORTAL_RADIUS, PORTAL_HEIGHT)));
    let ring_mesh = meshes
        .as_mut()
        .map(|m| m.add(Torus::new(RING_RADIUS - RING_THICKNESS, RING_RADIUS + RING_THICKNESS)));
    let body_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE.with_alpha(PORTAL_ALPHA),
            emissive: PORTAL_EMISSIVE,
            alpha_mode: AlphaMode::Blend,
            // Visible from both sides so the inside of the column glows too.
            double_sided: true,
            cull_mode: None,
            ..default()
        })
    });
    let ring_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: RING_EMISSIVE,
            ..default()
        })
    });
    let cap_mesh = meshes
        .as_mut()
        .map(|m| m.add(Torus::new(RING_RADIUS - CAP_THICKNESS, RING_RADIUS + CAP_THICKNESS)));
    let cap_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: CAP_EMISSIVE,
            ..default()
        })
    });
    PortalAssets {
        body_mesh,
        body_mat,
        ring_mesh,
        ring_mat,
        cap_mesh,
        cap_mat,
    }
}

/// Spawns the portal column centred in cell `(r, c)` plus its travelling rings.
pub(crate) fn spawn_portal(
    commands: &mut Commands,
    assets: &PortalAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let cx = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let cz = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let base_y = placement.world_y(PORTAL_BASE_Y);
    let centre_y = base_y + PORTAL_HEIGHT / 2.0;

    match (assets.body_mesh.clone(), assets.body_mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FinishPortal,
                placement.tag(),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(cx, centre_y, cz),
            ));
        }
        _ => {
            commands.spawn((FinishPortal, placement.tag(), Transform::from_xyz(cx, centre_y, cz)));
        }
    }

    // Fixed silver caps framing the top and bottom of the column.
    for cap_y in [base_y, base_y + PORTAL_HEIGHT] {
        match (assets.cap_mesh.clone(), assets.cap_mat.clone()) {
            (Some(mesh), Some(mat)) => {
                commands.spawn((
                    PortalCap,
                    placement.tag(),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(cx, cap_y, cz),
                ));
            }
            _ => {
                commands.spawn((PortalCap, placement.tag(), Transform::from_xyz(cx, cap_y, cz)));
            }
        }
    }

    // Rings evenly spaced down the length; each carries its own phase so they
    // stay equally spaced as they all travel downward.
    for i in 0..RING_COUNT {
        let phase = i as f32 / RING_COUNT as f32;
        let y = base_y + (1.0 - phase) * PORTAL_HEIGHT;
        let ring = PortalRing {
            base_y,
            height: PORTAL_HEIGHT,
            phase,
        };
        match (assets.ring_mesh.clone(), assets.ring_mat.clone()) {
            (Some(mesh), Some(mat)) => {
                commands.spawn((
                    ring,
                    placement.tag(),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(cx, y, cz),
                ));
            }
            _ => {
                commands.spawn((ring, placement.tag(), Transform::from_xyz(cx, y, cz)));
            }
        }
    }
}

/// Slides each portal ring down its column, wrapping back to the top — the
/// flowing-energy aura. Runs only while playing.
pub(crate) fn portal_system(
    time: Res<Time>,
    window: Res<LevelWindow>,
    mut rings: Query<(&mut Transform, &mut PortalRing, &LevelTag)>,
) {
    for (mut t, mut ring, tag) in &mut rings {
        // Off-window floors are neither drawn nor animated.
        if !window.contains(tag.0) {
            continue;
        }
        ring.phase = (ring.phase + time.delta_secs() * RING_TRAVEL_RATE).fract();
        t.translation.y = ring.base_y + (1.0 - ring.phase) * ring.height;
    }
}
