use super::{build_emissive_material, spawn_with_outline, CommonObjectAssets};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

// ---------- Tuning constants ----------

/// Chest body emissive RGB — dark wood brown.
const CHEST_EMISSIVE: LinearRgba = LinearRgba::new(0.40, 0.25, 0.10, 1.0);
/// Lid emissive RGB — slightly darker than the body so the rounded lid
/// reads as a distinct piece sitting on top of the body.
const LID_EMISSIVE: LinearRgba = LinearRgba::new(0.30, 0.18, 0.07, 1.0);
/// Horizontal hinge band emissive RGB — dark metallic grey.
const HINGE_EMISSIVE: LinearRgba = LinearRgba::new(0.18, 0.18, 0.20, 1.0);
/// Leather strap / lid-top binding emissive RGB — dark leather brown.
const LEATHER_EMISSIVE: LinearRgba = LinearRgba::new(0.25, 0.14, 0.06, 1.0);
/// Lock cone + circle emissive RGB — pure black so the keyhole reads
/// as the dark interior of a cutout rather than a near-black metal
/// piece. Combined with [`EMISSIVE_ONLY_BASE`] (BLACK) this gives a
/// material that renders pure black regardless of corridor lighting.
const LOCK_EMISSIVE: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 1.0);

// All positions below are expressed in the chest's LOCAL frame (origin
// at the cell floor centre, +Z = lock face). `spawn_chest` rotates them
// around Y by `yaw` so the lock face points at the dead-end's single
// open neighbour.

// Body: wide low cuboid. Sitting on the floor (bottom at Y=0).
const BODY_Y: f32 = 0.175;
const BODY_SCALE: Vec3 = Vec3::new(0.80, 0.35, 0.60);

/// Front (positive Z, lock face) plane Z. Half of body depth so the
/// strap inner face sits flush with the body silhouette.
const BODY_FACE_Z: f32 = 0.30;
/// Half body width (X). Used to place left/right strap centres on the
/// side faces.
const BODY_HALF_W: f32 = 0.40;

/// Wall thickness for the hollow trunk. The body is built as a floor slab + four
/// wall panels of this thickness rather than a solid block, so an **open** chest
/// reveals its interior: the loot fills a real cavity and each panel's
/// inverted-hull outline draws the inner edges (the vertical back-corner edges
/// and the bottom floor-to-wall edges). A closed chest is unchanged from outside
/// — the cavity is hidden under the lid.
const WALL_T: f32 = 0.06;

/// Thickness of the black border bars drawn along an open chest's interior
/// edges (the vertical corner edges + the top and bottom perimeter) so the
/// wall-panel edges read distinctly. The inverted-hull outline only rims the
/// outer silhouette, not these concave interior edges, so they're drawn
/// explicitly.
const EDGE_T: f32 = 0.012;

// Lid: a half-cylinder ("half-pipe") mesh — axis along the chest's width (X),
// flat diametral face down, rounded dome up. The flat face rests at the trunk
// top `Y` (so a closed chest reads as a rounded dome over the opening); an open
// chest swings it up on its rear flat edge. A whole cylinder would show its
// buried lower half once swung open, which looks wrong.
const LID_Y: f32 = 0.35;
/// Scale of the unit half-cylinder (radius 0.5, length 1.0 along X, flat at
/// local `y = 0`): x → length 0.80; y → dome height 0.30; z → half-width 0.30.
const LID_SCALE: Vec3 = Vec3::new(0.80, 0.60, 0.60);

// Horizontal hinge band wrapping all four side faces — four thin strips hugging
// the outer faces (a solid slab would fill the hollow interior). Each strip is
// slightly longer than its face so the band wraps the corners.
const HINGE_Y: f32 = 0.20;
const HINGE_BAND_H: f32 = 0.05;
const HINGE_STRIP_T: f32 = 0.04;
const HINGE_FRONT_BACK_LEN: f32 = 0.82;
const HINGE_LEFT_RIGHT_LEN: f32 = 0.62;

// Vertical leather straps. Each strap forms half of a continuous loop
// that wraps the chest: front+back straps plus the front-to-back lid
// binding make one loop; left+right straps plus the left-to-right lid
// binding make the perpendicular loop. The straps now sit FULLY OUTSIDE
// the body face (inner face at `BODY_FACE_Z`, outer face at
// `BODY_FACE_Z + STRAP_THICKNESS`) and extend UP to the lid apex so
// they meet the corresponding lid binding flush.
const STRAP_THICKNESS: f32 = 0.04;
const STRAP_WIDTH: f32 = 0.10;
/// Lid apex Y (`LID_Y + lid radius after rotation = 0.35 + 0.30`). The
/// strap top sits exactly here so the lid binding (whose bottom face
/// also sits at this Y) shares a clean edge with each strap top.
const LID_APEX_Y: f32 = 0.65;
/// Strap bottom Y — sits at the floor, not below it. (It used to dip to a small
/// negative Y so the strap "wrapped under" the chest, relying on the floor to hide
/// it — but in a multi-level stack the level below sees that negative-Y portion,
/// and its inverted-hull outline, poke through the floor. Keeping it at the floor
/// keeps the chest from showing its underside from below.)
const STRAP_BOTTOM_Y: f32 = 0.0;
const STRAP_HEIGHT: f32 = LID_APEX_Y - STRAP_BOTTOM_Y;
const STRAP_Y: f32 = (LID_APEX_Y + STRAP_BOTTOM_Y) * 0.5;
/// Front/back strap scale: width across face (X), height (Y),
/// thickness perpendicular to face (Z).
const STRAP_FRONT_BACK_SCALE: Vec3 = Vec3::new(STRAP_WIDTH, STRAP_HEIGHT, STRAP_THICKNESS);
/// Left/right strap scale — same shape rotated 90° (thickness is now
/// along X, width across face is along Z).
const STRAP_LEFT_RIGHT_SCALE: Vec3 = Vec3::new(STRAP_THICKNESS, STRAP_HEIGHT, STRAP_WIDTH);
/// Distance from chest centre to a front/back strap centre. Body
/// half-depth + half the strap thickness so the strap's inner face is
/// flush with the body face.
const STRAP_FRONT_BACK_Z: f32 = BODY_FACE_Z + STRAP_THICKNESS * 0.5;
/// Distance from chest centre to a left/right strap centre, same logic
/// on the X axis.
const STRAP_LEFT_RIGHT_X: f32 = BODY_HALF_W + STRAP_THICKNESS * 0.5;

// Lid-top bindings. Two perpendicular planks, one running front-to-back
// over the lid (connecting the front + back vertical straps), one
// running left-to-right (connecting the left + right straps). They form
// a `+` cross at the chest's top, completing each leather loop visibly.
const LID_BINDING_THICKNESS: f32 = 0.04;
/// Lid binding centre Y — half the thickness above the lid apex so the
/// binding's bottom face rests on the lid curve.
const LID_BINDING_Y: f32 = LID_APEX_Y + LID_BINDING_THICKNESS * 0.5;
/// Front-to-back binding: span Z from `-STRAP_FRONT_BACK_Z` to
/// `+STRAP_FRONT_BACK_Z` so each end meets the outer face of its
/// matching vertical strap.
const LID_BINDING_FRONT_BACK_SCALE: Vec3 = Vec3::new(
    STRAP_WIDTH,
    LID_BINDING_THICKNESS,
    STRAP_FRONT_BACK_Z * 2.0,
);
/// Left-to-right binding: span X from `-STRAP_LEFT_RIGHT_X` to
/// `+STRAP_LEFT_RIGHT_X`.
const LID_BINDING_LEFT_RIGHT_SCALE: Vec3 = Vec3::new(
    STRAP_LEFT_RIGHT_X * 2.0,
    LID_BINDING_THICKNESS,
    STRAP_WIDTH,
);

/// Apex height of the chest — the top of the lid bindings.
pub(crate) const TOP_Y: f32 = LID_BINDING_Y + LID_BINDING_THICKNESS * 0.5;

// Lock — cone (apex up, base down → triangle widens DOWNWARD from the
// front) plus a small circle hiding the cone's narrow tip. Both pieces
// are deliberately VERY THIN in Z and sit flush with the strap face so
// the keyhole reads as a flat dark CUTOUT painted on the leather (i.e.
// the lock area looks like missing material exposing darkness inside),
// rather than as a 3D feature mounted on top of the chest.
/// Lock cone vertical centre. Base at Y=0.20, tip at Y=0.30; cone height
/// is 0.10, so centre is at 0.25.
const LOCK_CONE_Y: f32 = 0.25;
/// Lock cone scale `(X width, Y height, Z thickness)`. X+Y match a
/// 0.10 × 0.10 silhouette as before, but Z is flattened to 0.01 so the
/// cone reads as a flat triangle on the strap face, not a 3D nub.
const LOCK_CONE_SCALE: Vec3 = Vec3::new(0.10, 0.10, 0.01);
/// Lock circle vertical centre — at the cone's tip Y so the circle
/// covers the cone's narrow end, forming the classic keyhole silhouette.
const LOCK_CIRCLE_Y: f32 = 0.30;
/// Lock circle scale (unit cylinder, rotated 90° around X so its axis
/// runs along world Z). X+Z stay at 0.08 (disc radius 0.04 after the
/// rotation); the original Y component becomes the post-rotation Z
/// thickness — flattened to 0.005 so the disc reads as a flat circle.
const LOCK_CIRCLE_SCALE: Vec3 = Vec3::new(0.08, 0.005, 0.08);
/// +Z offset of the cone CENTRE past the strap's outer face. Just large
/// enough to clear the cone's own half-thickness so the cone's back
/// face sits a hair past the strap — avoids z-fighting at the surface
/// while keeping the cone visually flush.
const LOCK_FRONT_OFFSET: f32 = 0.006;
/// +Z offset of the circle CENTRE past the cone centre. Enough to put
/// the circle's back face clear of the cone's front face so the two
/// pieces don't z-fight where the keyhole shape transitions.
const LOCK_CIRCLE_OVERLAP: f32 = 0.012;

pub(crate) fn build_chest_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, CHEST_EMISSIVE)
}

pub(crate) fn build_lid_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, LID_EMISSIVE)
}

pub(crate) fn build_hinge_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, HINGE_EMISSIVE)
}

pub(crate) fn build_leather_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, LEATHER_EMISSIVE)
}

pub(crate) fn build_lock_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    build_emissive_material(materials, LOCK_EMISSIVE)
}

/// Whether a chest renders sealed (`Closed`) or with its lid swung open
/// (`Open`). Key-holder and dead-end chests are `Closed`; a treasure chest is
/// `Open` so its piled contents are visible. `TOP_Y` (the apex a key floats
/// above) only applies to a `Closed` chest.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChestLid {
    Closed,
    Open,
}

/// Angle (radians) the lid swings about its rear flat edge when `Open`: a
/// quarter turn up so it stands vertical (its flat inner face toward the front),
/// like a chest lid hinged at the back. Negative lifts the front edge up.
const LID_OPEN_ANGLE: f32 = -FRAC_PI_2;
/// Rear-top hinge line for the lid (local frame): along the chest's X axis at
/// the lid's rear edge, at lid height.
const LID_HINGE_Z: f32 = -0.30;
const LID_HINGE_Y: f32 = LID_Y;

// Open-lid straps stop at the body rim — the closed lid (which the closed
// straps reach up to meet) is swung away, so a full-height strap would hang in
// the air above the trunk.
const OPEN_STRAP_TOP_Y: f32 = 0.34;
const OPEN_STRAP_HEIGHT: f32 = OPEN_STRAP_TOP_Y - STRAP_BOTTOM_Y;
const OPEN_STRAP_Y: f32 = (OPEN_STRAP_TOP_Y + STRAP_BOTTOM_Y) * 0.5;
const OPEN_STRAP_FRONT_BACK_SCALE: Vec3 = Vec3::new(STRAP_WIDTH, OPEN_STRAP_HEIGHT, STRAP_THICKNESS);
const OPEN_STRAP_LEFT_RIGHT_SCALE: Vec3 = Vec3::new(STRAP_THICKNESS, OPEN_STRAP_HEIGHT, STRAP_WIDTH);

/// Swings a local-frame lid transform open about the rear-top hinge by
/// [`LID_OPEN_ANGLE`]. Only the lid and the lid bindings ride it, so they lift
/// away as one while the trunk stays put.
fn hinge_open(local: Transform) -> Transform {
    let pivot = Vec3::new(0.0, LID_HINGE_Y, LID_HINGE_Z);
    let rot = Quat::from_rotation_x(LID_OPEN_ANGLE);
    Transform {
        translation: pivot + rot * (local.translation - pivot),
        rotation: rot * local.rotation,
        scale: local.scale,
    }
}

/// Bakes a chest sub-mesh's LOCAL transform (origin at the cell floor centre,
/// `+Z` = front) into a world transform at the cell `(x, z)` rotated by `yaw`.
/// Every chest is free-standing: the dead-end / key-holder chests are
/// landmarks, and the treasure chest stays behind (open, emptied) once its
/// contents are collected — only the loot (a separate entity) is whisked away.
fn world_xform(x: f32, z: f32, yaw: f32, base_y: f32, local: Transform) -> Transform {
    let yaw_rot = Quat::from_rotation_y(yaw);
    Transform {
        translation: Vec3::new(x, base_y, z) + yaw_rot * local.translation,
        rotation: yaw_rot * local.rotation,
        scale: local.scale,
    }
}

/// Spawns a free-standing chest rig at `(x, z)` on run `level`, rotated by `yaw`.
/// `lid` selects sealed (dead-end / key-holder bases) vs open (treasure chests —
/// the loot is spawned separately so the chest persists, empty, after collection).
pub(crate) fn spawn_chest(
    commands: &mut Commands,
    assets: &CommonObjectAssets,
    x: f32,
    z: f32,
    yaw: f32,
    lid: ChestLid,
    base_y: f32,
) {
    let outline = || assets.outline_mat.clone();
    let cuboid = || assets.cuboid.clone();
    // Body / hinge / straps / lock stay anchored to the trunk; the lid + lid
    // bindings additionally swing open when `lid == Open`.
    let place = |local: Transform| world_xform(x, z, yaw, base_y, local);
    let lid_place = |local: Transform| {
        let l = if lid == ChestLid::Open { hinge_open(local) } else { local };
        world_xform(x, z, yaw, base_y, l)
    };

    // Hollow trunk: a floor slab + four wall panels instead of a solid block, so
    // an open chest reveals its interior; each panel's inverted-hull outline
    // draws the inner edges. (A closed chest hides the cavity under the lid.)
    let (bw, bh, bd) = (BODY_SCALE.x, BODY_SCALE.y, BODY_SCALE.z);
    // Floor slab.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, WALL_T * 0.5, 0.0)).with_scale(Vec3::new(bw, WALL_T, bd))),
        (),
    );
    // Back / front walls (full width, thin in Z; outer face flush at ±BODY_FACE_Z).
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, BODY_Y, -(BODY_FACE_Z - WALL_T * 0.5))).with_scale(Vec3::new(bw, bh, WALL_T))),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, BODY_Y, BODY_FACE_Z - WALL_T * 0.5)).with_scale(Vec3::new(bw, bh, WALL_T))),
        (),
    );
    // Left / right walls (full depth, thin in X).
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(-(BODY_HALF_W - WALL_T * 0.5), BODY_Y, 0.0)).with_scale(Vec3::new(WALL_T, bh, bd))),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(BODY_HALF_W - WALL_T * 0.5, BODY_Y, 0.0)).with_scale(Vec3::new(WALL_T, bh, bd))),
        (),
    );

    // Black border bars along the open chest's interior edges — the four
    // vertical corner edges and the top + bottom perimeter — so the wall-panel
    // edges read distinctly. Open chests only; a closed chest hides the cavity.
    if lid == ChestLid::Open {
        if let (Some(mesh), Some(mat)) = (assets.cuboid.clone(), assets.lock_mat.clone()) {
            let ix = BODY_HALF_W - WALL_T;
            let iz = BODY_FACE_Z - WALL_T;
            let yb = WALL_T;
            let yt = BODY_Y + bh * 0.5;
            let mid_y = (yb + yt) * 0.5;
            let vbar = Vec3::new(EDGE_T, yt - yb, EDGE_T);
            let xbar = Vec3::new(2.0 * ix, EDGE_T, EDGE_T);
            let zbar = Vec3::new(EDGE_T, EDGE_T, 2.0 * iz);
            let edges: [(Vec3, Vec3); 12] = [
                // Four vertical corner edges.
                (Vec3::new(ix, mid_y, iz), vbar),
                (Vec3::new(-ix, mid_y, iz), vbar),
                (Vec3::new(ix, mid_y, -iz), vbar),
                (Vec3::new(-ix, mid_y, -iz), vbar),
                // Top rim perimeter.
                (Vec3::new(0.0, yt, iz), xbar),
                (Vec3::new(0.0, yt, -iz), xbar),
                (Vec3::new(ix, yt, 0.0), zbar),
                (Vec3::new(-ix, yt, 0.0), zbar),
                // Bottom floor-to-wall perimeter.
                (Vec3::new(0.0, yb, iz), xbar),
                (Vec3::new(0.0, yb, -iz), xbar),
                (Vec3::new(ix, yb, 0.0), zbar),
                (Vec3::new(-ix, yb, 0.0), zbar),
            ];
            for (translation, scale) in edges {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    place(Transform::from_translation(translation).with_scale(scale)),
                ));
            }

            // Lid flat-face perimeter edges. These ride the lid (via `lid_place`,
            // which hinges them open with it) so the raised lid shows its inner
            // structure too. The flat face spans the full trunk top.
            let lx = BODY_HALF_W;
            let lz = BODY_FACE_Z;
            let lid_edges: [(Vec3, Vec3); 4] = [
                (Vec3::new(0.0, LID_Y, lz), Vec3::new(2.0 * lx, EDGE_T, EDGE_T)),
                (Vec3::new(0.0, LID_Y, -lz), Vec3::new(2.0 * lx, EDGE_T, EDGE_T)),
                (Vec3::new(lx, LID_Y, 0.0), Vec3::new(EDGE_T, EDGE_T, 2.0 * lz)),
                (Vec3::new(-lx, LID_Y, 0.0), Vec3::new(EDGE_T, EDGE_T, 2.0 * lz)),
            ];
            for (translation, scale) in lid_edges {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    lid_place(Transform::from_translation(translation).with_scale(scale)),
                ));
            }
        }
    }

    // Lid: a half-cylinder with its flat face resting at the trunk top. Closed,
    // it reads as a rounded dome over the opening; open, it swings up on its rear
    // flat edge to stand vertical.
    spawn_with_outline(
        commands,
        None,
        assets.half_cylinder.clone(),
        assets.lid_mat.clone(),
        outline(),
        lid_place(Transform::from_translation(Vec3::new(0.0, LID_Y, 0.0)).with_scale(LID_SCALE)),
        (),
    );

    // Horizontal hinge band across mid-height — four strips hugging the outer
    // faces (front / back thin in Z, left / right thin in X) so the band wraps
    // the trunk without filling its hollow interior.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.hinge_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, HINGE_Y, BODY_FACE_Z)).with_scale(Vec3::new(HINGE_FRONT_BACK_LEN, HINGE_BAND_H, HINGE_STRIP_T))),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.hinge_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, HINGE_Y, -BODY_FACE_Z)).with_scale(Vec3::new(HINGE_FRONT_BACK_LEN, HINGE_BAND_H, HINGE_STRIP_T))),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.hinge_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(BODY_HALF_W, HINGE_Y, 0.0)).with_scale(Vec3::new(HINGE_STRIP_T, HINGE_BAND_H, HINGE_LEFT_RIGHT_LEN))),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.hinge_mat.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(-BODY_HALF_W, HINGE_Y, 0.0)).with_scale(Vec3::new(HINGE_STRIP_T, HINGE_BAND_H, HINGE_LEFT_RIGHT_LEN))),
        (),
    );

    // Four vertical straps. Closed straps reach the lid apex to meet the lid
    // bindings; open straps stop at the body rim.
    let (strap_y, fb_scale, lr_scale) = match lid {
        ChestLid::Closed => (STRAP_Y, STRAP_FRONT_BACK_SCALE, STRAP_LEFT_RIGHT_SCALE),
        ChestLid::Open => (OPEN_STRAP_Y, OPEN_STRAP_FRONT_BACK_SCALE, OPEN_STRAP_LEFT_RIGHT_SCALE),
    };
    let leather = assets.leather_mat.clone();
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, strap_y, STRAP_FRONT_BACK_Z)).with_scale(fb_scale)),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(0.0, strap_y, -STRAP_FRONT_BACK_Z)).with_scale(fb_scale)),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(-STRAP_LEFT_RIGHT_X, strap_y, 0.0)).with_scale(lr_scale)),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        place(Transform::from_translation(Vec3::new(STRAP_LEFT_RIGHT_X, strap_y, 0.0)).with_scale(lr_scale)),
        (),
    );

    // Two perpendicular lid bindings forming a `+` cross over the lid. They ride
    // the lid, so they swing away with it when the chest is open.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        lid_place(
            Transform::from_translation(Vec3::new(0.0, LID_BINDING_Y, 0.0))
                .with_scale(LID_BINDING_FRONT_BACK_SCALE),
        ),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather,
        outline(),
        lid_place(
            Transform::from_translation(Vec3::new(0.0, LID_BINDING_Y, 0.0))
                .with_scale(LID_BINDING_LEFT_RIGHT_SCALE),
        ),
        (),
    );

    // Keyhole — only on a sealed chest (an open treasure chest isn't locked).
    // Cone (wide base low, narrow tip high — triangle widening DOWNWARD from the
    // front) plus a small circle covering the cone's tip, both extra-flat in Z
    // so the dark material reads as a CUTOUT painted on the leather.
    if lid == ChestLid::Closed {
        let lock = assets.lock_mat.clone();
        let lock_cone_z = STRAP_FRONT_BACK_Z + STRAP_THICKNESS * 0.5 + LOCK_FRONT_OFFSET;
        spawn_with_outline(
            commands,
            None,
            assets.cone.clone(),
            lock.clone(),
            outline(),
            place(Transform::from_translation(Vec3::new(0.0, LOCK_CONE_Y, lock_cone_z)).with_scale(LOCK_CONE_SCALE)),
            (),
        );
        spawn_with_outline(
            commands,
            None,
            assets.cylinder.clone(),
            lock,
            outline(),
            place(
                Transform::from_translation(Vec3::new(0.0, LOCK_CIRCLE_Y, lock_cone_z + LOCK_CIRCLE_OVERLAP))
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(LOCK_CIRCLE_SCALE),
            ),
            (),
        );
    }
}

/// Builds the half-cylinder ("half-pipe") lid mesh: axis along local `X`
/// (length 1, centred), a semicircular cross-section of radius `0.5` in the YZ
/// plane with the flat diametral face at `y = 0` and the dome bulging up to
/// `y = 0.5`. Built once into [`CommonObjectAssets`] and scaled per chest. A
/// closed chest rests the flat face on the trunk top as a rounded dome; an open
/// chest hinges it up on its rear flat edge.
pub(crate) fn half_cylinder_mesh() -> Mesh {
    const N: usize = 20;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Arc point i as (z, y) on the upper semicircle of radius 0.5.
    let arc = |i: usize| -> (f32, f32) {
        let phi = i as f32 / N as f32 * PI;
        (0.5 * phi.cos(), 0.5 * phi.sin())
    };

    // Curved dome surface (outward radial normals).
    for i in 0..N {
        let (z0, y0) = arc(i);
        let (z1, y1) = arc(i + 1);
        let n0 = [0.0, 2.0 * y0, 2.0 * z0];
        let n1 = [0.0, 2.0 * y1, 2.0 * z1];
        let base = positions.len() as u32;
        positions.push([-0.5, y0, z0]);
        normals.push(n0);
        uvs.push([0.0, 0.0]);
        positions.push([0.5, y0, z0]);
        normals.push(n0);
        uvs.push([0.0, 0.0]);
        positions.push([0.5, y1, z1]);
        normals.push(n1);
        uvs.push([0.0, 0.0]);
        positions.push([-0.5, y1, z1]);
        normals.push(n1);
        uvs.push([0.0, 0.0]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // Flat diametral face (y = 0), normal pointing down.
    {
        let base = positions.len() as u32;
        for &(x, z) in &[(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            positions.push([x, 0.0, z]);
            normals.push([0.0, -1.0, 0.0]);
            uvs.push([0.0, 0.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    // Semicircular end caps at x = ±0.5.
    for (x, sign) in [(0.5_f32, 1.0_f32), (-0.5_f32, -1.0_f32)] {
        let center = positions.len() as u32;
        positions.push([x, 0.0, 0.0]);
        normals.push([sign, 0.0, 0.0]);
        uvs.push([0.0, 0.0]);
        let first = positions.len() as u32;
        for i in 0..=N {
            let (z, y) = arc(i);
            positions.push([x, y, z]);
            normals.push([sign, 0.0, 0.0]);
            uvs.push([0.0, 0.0]);
        }
        for i in 0..N as u32 {
            let (a, b) = (first + i, first + i + 1);
            if sign > 0.0 {
                indices.extend_from_slice(&[center, b, a]);
            } else {
                indices.extend_from_slice(&[center, a, b]);
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
