use super::{build_emissive_material, spawn_with_outline, CommonObjectAssets};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

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

// Lid: a regular `Cylinder` mesh rotated 90° around Z so the cylinder's
// axis runs along the chest's width (X). Bottom half sits inside the
// body; top half pokes above as the rounded lid silhouette.
const LID_Y: f32 = 0.35;
/// Scale of the unit cylinder (radius 0.5, height 1.0) before rotation.
/// x and z scale to 0.60 → radius 0.30 in YZ plane after rotation;
/// y scale to 0.80 → cylinder length 0.80 (matches body width along X
/// after the 90°-around-Z rotation).
const LID_SCALE: Vec3 = Vec3::new(0.60, 0.80, 0.60);

// Horizontal hinge band wrapping all four side faces. Slightly wider
// and deeper than the body so it pokes past each face by ~0.01.
const HINGE_Y: f32 = 0.20;
const HINGE_SCALE: Vec3 = Vec3::new(0.82, 0.05, 0.62);

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
/// Strap bottom Y — just below the floor so the strap appears to wrap
/// under the chest (the floor visually conceals the negative-Y portion).
const STRAP_BOTTOM_Y: f32 = -0.025;
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

/// Applies a yaw rotation around the cell centre `(x, 0, z)` to a
/// local-frame transform. Only the X/Z components rotate; Y stays put,
/// so vertical stacking inside the chest is preserved. Used by every
/// `spawn_with_outline` call inside `spawn_chest` so the whole chest
/// can pivot to face its open neighbour.
fn apply_yaw(x: f32, z: f32, yaw: f32, local: Transform) -> Transform {
    let yaw_rot = Quat::from_rotation_y(yaw);
    let centre = Vec3::new(x, 0.0, z);
    let rotated = yaw_rot * (local.translation - centre);
    Transform {
        translation: centre + rotated,
        rotation: yaw_rot * local.rotation,
        scale: local.scale,
    }
}

pub(crate) fn spawn_chest(
    commands: &mut Commands,
    assets: &CommonObjectAssets,
    x: f32,
    z: f32,
    yaw: f32,
) {
    let outline = || assets.outline_mat.clone();
    let cuboid = || assets.cuboid.clone();
    let xform = |local: Transform| apply_yaw(x, z, yaw, local);
    let at = |lx: f32, ly: f32, lz: f32| Vec3::new(x + lx, ly, z + lz);

    // Body cuboid.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.chest_mat.clone(),
        outline(),
        xform(Transform::from_translation(at(0.0, BODY_Y, 0.0)).with_scale(BODY_SCALE)),
        (),
    );

    // Lid: cylinder rotated 90° around Z so its axis runs along the
    // chest's local X. Bottom half is buried inside the body.
    spawn_with_outline(
        commands,
        None,
        assets.cylinder.clone(),
        assets.lid_mat.clone(),
        outline(),
        xform(
            Transform::from_translation(at(0.0, LID_Y, 0.0))
                .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                .with_scale(LID_SCALE),
        ),
        (),
    );

    // Horizontal hinge band wrapping mid-height across all 4 side faces.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        assets.hinge_mat.clone(),
        outline(),
        xform(Transform::from_translation(at(0.0, HINGE_Y, 0.0)).with_scale(HINGE_SCALE)),
        (),
    );

    // Four vertical straps — one on each side face, extending from
    // below-floor up to the lid apex so they meet the lid bindings flush.
    let leather = assets.leather_mat.clone();
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        xform(
            Transform::from_translation(at(0.0, STRAP_Y, STRAP_FRONT_BACK_Z))
                .with_scale(STRAP_FRONT_BACK_SCALE),
        ),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        xform(
            Transform::from_translation(at(0.0, STRAP_Y, -STRAP_FRONT_BACK_Z))
                .with_scale(STRAP_FRONT_BACK_SCALE),
        ),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        xform(
            Transform::from_translation(at(-STRAP_LEFT_RIGHT_X, STRAP_Y, 0.0))
                .with_scale(STRAP_LEFT_RIGHT_SCALE),
        ),
        (),
    );
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        xform(
            Transform::from_translation(at(STRAP_LEFT_RIGHT_X, STRAP_Y, 0.0))
                .with_scale(STRAP_LEFT_RIGHT_SCALE),
        ),
        (),
    );

    // Two perpendicular lid bindings forming a `+` cross over the lid
    // apex. Each binding spans from one strap's outer face to the
    // opposite strap's outer face so each loop reads as continuous.
    spawn_with_outline(
        commands,
        None,
        cuboid(),
        leather.clone(),
        outline(),
        xform(
            Transform::from_translation(at(0.0, LID_BINDING_Y, 0.0))
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
        xform(
            Transform::from_translation(at(0.0, LID_BINDING_Y, 0.0))
                .with_scale(LID_BINDING_LEFT_RIGHT_SCALE),
        ),
        (),
    );

    // Keyhole on the (now-rotated) front face: cone (wide base at Y=0.20,
    // narrow tip at Y=0.30 — triangle widens DOWNWARD when viewed from
    // the front) plus a small circle covering the cone's tip. Both
    // pieces are extra-flat in Z and sit essentially flush with the
    // strap face so the dark material reads as a CUTOUT painted on the
    // leather, not a 3D nub mounted on top.
    let lock = assets.lock_mat.clone();
    let lock_cone_z = STRAP_FRONT_BACK_Z + STRAP_THICKNESS * 0.5 + LOCK_FRONT_OFFSET;
    spawn_with_outline(
        commands,
        None,
        assets.cone.clone(),
        lock.clone(),
        outline(),
        xform(
            Transform::from_translation(at(0.0, LOCK_CONE_Y, lock_cone_z))
                .with_scale(LOCK_CONE_SCALE),
        ),
        (),
    );
    // Lock circle: cylinder rotated 90° around X so the disc faces the
    // viewer of the lock face.
    spawn_with_outline(
        commands,
        None,
        assets.cylinder.clone(),
        lock,
        outline(),
        xform(
            Transform::from_translation(at(0.0, LOCK_CIRCLE_Y, lock_cone_z + LOCK_CIRCLE_OVERLAP))
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(LOCK_CIRCLE_SCALE),
        ),
        (),
    );
}
