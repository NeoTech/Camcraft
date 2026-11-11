//! #701: WCS-from-features roles - every pick carries a chosen role
//! (Origin, X, Y or Z) over flat faces, straight edges, vertices and
//! curved walls. Pins: the 3-2-1 role table round-trips through
//! `align_roles`, the third axis is derived right-handed, a cylinder
//! wall fits its own axis (upright and tilted), the origin anchor and
//! projection rules, and every named refusal. The result rides the
//! existing `Alignment` and `SetWcs`; nothing here writes the world.
//! Provenance: our own fixtures over the picked-datum alignment law.
//!
//! Gate map (#701): covers app::pick::roles (align_roles, Role).
//! Lane shape (M4.4): all tests here run in seconds.

use camcraft_app::commands::{History, ImportModel, MoveModel};
use camcraft_app::geometry::{Point3, cube, cylinder, feature, quat_from_euler, quat_rotate_point};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel};
use camcraft_app::pick::{Role, align_roles};
use camcraft_app::schema::Location;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

/// A 20 x 20 x 10 block at the origin, optionally turned `yaw_deg` about Z.
fn block(yaw_deg: f64) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("block", cube(Point3::new(0.0, 0.0, 0.0), Point3::new(20.0, 20.0, 10.0))));
    if yaw_deg != 0.0 {
        let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: quat_from_euler(0.0, 0.0, yaw_deg.to_radians()) };
        h.execute(&world, MoveModel::new(&world, "block", loc).expect("turn the block"));
    }
    world
}

/// A 20-dia, 10-high, 48-segment cylinder at the origin, optionally
/// pitched `tilt_deg` about X: a bore turned off the table. Its wall
/// welds to one patch; fitting an axis through that wall is new maths.
fn bore(tilt_deg: f64) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("bore", cylinder(20.0, 10.0, 48)));
    if tilt_deg != 0.0 {
        let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: quat_from_euler(tilt_deg.to_radians(), 0.0, 0.0) };
        h.execute(&world, MoveModel::new(&world, "bore", loc).expect("tilt the bore"));
    }
    world
}

fn features(world: &World, name: &str) -> std::sync::Arc<feature::Features> {
    let e = model::find_by_name(world, name).expect(name);
    model::features_of(world, &e).expect("features")
}

/// Picks are by the model's LOCAL geometry (ids do not move with it).
fn face(world: &World, entity: &str, p: [f64; 3]) -> GeoSel {
    let f = features(world, entity);
    let id = f
        .faces
        .iter()
        .find(|f| (f.normal[0] * p[0] + f.normal[1] * p[1] + f.normal[2] * p[2] - f.offset).abs() < 1e-9)
        .map(|f| f.id)
        .expect("the model has that face");
    GeoSel { entity: entity.into(), feature: FeatureSel::Face(id) }
}

fn edge(world: &World, entity: &str, a: [f64; 3], b: [f64; 3]) -> GeoSel {
    let f = features(world, entity);
    let id = feature::chains(&f)
        .into_iter()
        .find(|c| {
            let p = feature::chain_points(&f, c);
            p.len() == 2 && ((p[0] == a && p[1] == b) || (p[0] == b && p[1] == a))
        })
        .map(|c| c.id)
        .expect("the model has that edge");
    GeoSel { entity: entity.into(), feature: FeatureSel::Chain(id) }
}

fn corner(world: &World, entity: &str, at: [f64; 3]) -> GeoSel {
    let f = features(world, entity);
    let id = feature::corner_verts(&f)
        .into_iter()
        .find(|&vi| f.verts[vi as usize] == at)
        .and_then(|vi| feature::vertex_id(&f, vi))
        .expect("the model has that corner");
    GeoSel { entity: entity.into(), feature: FeatureSel::Vertex(id) }
}

/// The 20 x 20 x 10 block tipped `tilt_deg` about X: a part off the
/// table, so an edge-picked axis no longer sits on a world axis.
fn tilted_block(tilt_deg: f64) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("block", cube(Point3::new(0.0, 0.0, 0.0), Point3::new(20.0, 20.0, 10.0))));
    if tilt_deg != 0.0 {
        let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: quat_from_euler(tilt_deg.to_radians(), 0.0, 0.0) };
        h.execute(&world, MoveModel::new(&world, "block", loc).expect("tip the block"));
    }
    world
}

/// The cylinder's wall: the multi-strip weld whose strips' own normals
/// all stand out of the bore axis (caps stay single-strip planar faces).
fn wall_patch(world: &World, name: &str) -> GeoSel {
    let f = features(world, name);
    let p = feature::patches(&f)
        .into_iter()
        .max_by_key(|p| p.faces.len())
        .expect("the wall welds to a patch");
    assert_eq!(p.faces.len(), 48, "one strip per segment: {:?}", p.faces.len());
    GeoSel { entity: name.into(), feature: FeatureSel::Patch(p.id) }
}

fn near(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
}

fn close(a: [f64; 3], b: [f64; 3], tol: f64) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < tol)
}

#[test]
fn z_face_x_edge_and_origin_corner_reproduce_the_3_2_1_frame() {
    // The role table with the old fixed roles must agree with align_321
    // to the last digit, on a square and on a turned block, at either
    // end of the edge (the origin picks side flips X, #187).
    let w = block(0.0);
    let top = face(&w, "block", [10.0, 10.0, 10.0]);
    let front = edge(&w, "block", [0.0, 0.0, 10.0], [20.0, 0.0, 10.0]);
    let c0 = corner(&w, "block", [0.0, 0.0, 10.0]);
    let c1 = corner(&w, "block", [20.0, 0.0, 10.0]);
    let a = align_roles(&w, &[(top.clone(), Role::Z), (front.clone(), Role::X), (c0.clone(), Role::Origin)])
        .expect("face, edge, corner");
    assert!(near(a.pos, [0.0, 0.0, 10.0]), "origin on the corner: {:?}", a.pos);
    assert!(near(a.x, [1.0, 0.0, 0.0]), "X along the edge: {:?}", a.x);
    assert!(near(a.y, [0.0, 1.0, 0.0]), "right-handed: {:?}", a.y);
    let flipped = align_roles(&w, &[(top.clone(), Role::Z), (front, Role::X), (c1, Role::Origin)])
        .expect("the far corner");
    assert!(near(flipped.x, [-1.0, 0.0, 0.0]), "X runs from the origin along the edge: {:?}", flipped.x);
    let t = block(30.0);
    let top = face(&t, "block", [10.0, 10.0, 10.0]);
    let front = edge(&t, "block", [0.0, 0.0, 10.0], [20.0, 0.0, 10.0]);
    let c = corner(&t, "block", [0.0, 0.0, 10.0]);
    let r = align_roles(&t, &[(top.clone(), Role::Z), (front.clone(), Role::X), (c.clone(), Role::Origin)])
        .expect("aligned");
    let (s, cc) = 30f64.to_radians().sin_cos();
    assert!(close(r.x, [cc, s, 0.0], 1e-9), "X follows the turned edge: {:?}", r.x);
    assert!(near(r.z, [0.0, 0.0, 1.0]), "{:?}", r.z);
    assert!((r.yaw_deg - 30.0).abs() < 1e-9, "{r:?}");
    let legacy = camcraft_app::pick::align_321(&t, &[top, front, c]).expect("3-2-1 agrees");
    assert!(close(r.x, legacy.x, 1e-9) && close(r.y, legacy.y, 1e-9) && close(r.z, legacy.z, 1e-9) && close(r.pos, legacy.pos, 1e-9), "role table vs 3-2-1:\n{r:?}\n{legacy:?}");
}

#[test]
fn the_third_axis_role_is_derived_right_handed_never_read() {
    // All three axes picked: Z and X build the frame (Z exact, X
    // projected), Y = Z x X no matter what the Y pick stood for - the
    // same rule #187 used. The third pick is overridden wholesale:
    // even one standing parallel to Z rides along unread.
    let w = block(0.0);
    let top = face(&w, "block", [10.0, 10.0, 10.0]);
    let front = edge(&w, "block", [0.0, 0.0, 10.0], [20.0, 0.0, 10.0]);
    let left = edge(&w, "block", [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]); // stands where Y never can
    let two = align_roles(&w, &[(top.clone(), Role::Z), (front.clone(), Role::X)]).expect("two axes");
    let three = align_roles(&w, &[(top, Role::Z), (front, Role::X), (left, Role::Y)]).expect("three axes");
    assert!(near(two.x, three.x) && near(two.y, three.y) && near(two.z, three.z), "Y is derived, not read:\n{two:?}\n{three:?}");
    assert!(near(three.y, [0.0, 1.0, 0.0]));
}

#[test]
fn a_bore_wall_picked_as_z_stands_upright_with_a_pinned_sign() {
    // The fitted axis of an upright wall is vertical; the sign is
    // pinned: the fit returns the +Z-pointing root, whatever the walk
    // order, so two callers see the same frame. The bore has no
    // straight edge of its own, so a block beside it supplies the X.
    let w = bore(0.0);
    let mut h = History::default();
    h.execute(&w, ImportModel::from_mesh("block", cube(Point3::new(30.0, 0.0, 0.0), Point3::new(50.0, 20.0, 10.0))));
    let wall = wall_patch(&w, "bore");
    let front = edge(&w, "block", [30.0, 0.0, 10.0], [50.0, 0.0, 10.0]);
    let one = align_roles(&w, &[(front.clone(), Role::X)]);
    assert!(one.is_err(), "one axis alone must refuse");
    let a = align_roles(&w, &[(wall, Role::Z), (front, Role::X)]).expect("wall sets Z, edge sets X");
    assert!(near(a.z, [0.0, 0.0, 1.0]), "wall axis, +Z root: {:?}", a.z);
    // No origin pick: the origin is the second axis pick's anchor -
    // whichever end the walk started at, with X running away from it -
    // and a wall datum is a line, not a plane, so nothing is projected.
    let starts_a = near(a.pos, [30.0, 0.0, 10.0]);
    let starts_b = near(a.pos, [50.0, 0.0, 10.0]);
    assert!(starts_a || starts_b, "origin at an edge end: {:?}", a.pos);
    let away = if starts_a { [1.0, 0.0, 0.0] } else { [-1.0, 0.0, 0.0] };
    assert!(near(a.x, away), "X runs away from the anchor: {:?}", a.x);
}

#[test]
fn a_tilted_bore_wall_fits_its_real_axis_and_centre() {
    // A bore pitched 20 degrees about X: the old code could only read a
    // level ring of a vertical wall; this fits the axis through the
    // wall facets themselves, tilted and centred.
    let w = bore(20.0);
    let mut h = History::default();
    h.execute(&w, ImportModel::from_mesh("block", cube(Point3::new(30.0, 0.0, 0.0), Point3::new(50.0, 20.0, 10.0))));
    let wall = wall_patch(&w, "bore");
    let top = face(&w, "block", [40.0, 10.0, 10.0]);
    let a = align_roles(&w, &[(wall.clone(), Role::Z), (top.clone(), Role::X)]).expect("tilted wall sets Z, block face sets X");
    let q = quat_from_euler(20f64.to_radians(), 0.0, 0.0);
    let want = quat_rotate_point(q, Point3::new(0.0, 0.0, 1.0));
    let mut want = [want.x, want.y, want.z];
    if want[2] < 0.0 {
        want = [-want[0], -want[1], -want[2]]; // the same +Z root rule
    }
    assert!(close(a.z, want, 1e-9), "fitted axis follows the tilt:\n{:?}\nwant {want:?}", a.z);
    assert!(a.x.iter().zip(a.z).map(|(u, v)| u * v).sum::<f64>().abs() < 1e-9, "X squared to the fitted axis: {:?}", a.x);
    assert!(a.y.iter().zip(a.x).map(|(u, v)| u * v).sum::<f64>().abs() < 1e-9, "right-handed: {:?}", a.y);
    // The same wall also picked as Origin sits on the fitted axis at
    // its middle: the exact axis datum is that wall (a line, no plane),
    // so nothing is projected - a bore's centre stays in its bore.
    let b = align_roles(&w, &[(wall.clone(), Role::Z), (top, Role::X), (wall, Role::Origin)])
        .expect("wall sets Z and centres the origin");
    let centre = quat_rotate_point(q, Point3::new(0.0, 0.0, 5.0));
    assert!(close(b.pos, [centre.x, centre.y, centre.z], 1e-6), "bore centre: {:?} want {centre:?}", b.pos);
    assert!(close(b.z, want, 1e-9), "Z still the fitted wall axis: {:?}", b.z);
}

#[test]
fn the_origin_projects_onto_a_z_face_and_anchors_on_its_centroid() {
    // #187's projection rule carries: the origin lands on the datum
    // plane the exact axis read from a face. With no origin pick at all
    // and faces on both axes, the X face's centroid anchors it.
    let w = block(0.0);
    let top = face(&w, "block", [10.0, 10.0, 10.0]);
    let front = face(&w, "block", [10.0, 0.0, 5.0]);
    let c = corner(&w, "block", [0.0, 20.0, 0.0]); // far below the top plane
    let a = align_roles(&w, &[(top.clone(), Role::Z), (front.clone(), Role::X), (c, Role::Origin)])
        .expect("two faces and a corner");
    assert!(near(a.z, [0.0, 0.0, 1.0]), "{:?}", a.z);
    assert!(close(a.pos, [0.0, 20.0, 10.0], 1e-9), "corner projected up onto the top plane: {:?}", a.pos);
    let no_origin = align_roles(&w, &[(top, Role::Z), (front, Role::X)]).expect("two faces");
    assert!(close(no_origin.pos, [10.0, 0.0, 10.0], 1e-9), "front face centroid, on the top plane: {:?}", no_origin.pos);
    assert!(near(no_origin.x, [0.0, -1.0, 0.0]), "X out of the front face: {:?}", no_origin.x);
    assert!(near(no_origin.y, [1.0, 0.0, 0.0]), "{:?}", no_origin.y);
}

#[test]
fn an_edge_as_the_first_axis_reads_unit_on_a_part_tipped_off_the_table() {
    // A 10 mm edge walk carries a length; the frame axis must not ride
    // on it. Reading the raw walk as the first axis left `Alignment.x/y/z`
    // scaled, and `quat_from_axes` (Shepperd on a scaled matrix) then
    // returned a silently wrong rotation: this very fixture read the
    // 25 degree tip as a 45.56 roll. Pins: unit, pairwise-square axes,
    // the tip back out of the euler angles, and the origin at the corner
    // that sided the edge.
    let w = tilted_block(25.0);
    let upright = edge(&w, "block", [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
    let side = face(&w, "block", [20.0, 10.0, 5.0]);
    let c = corner(&w, "block", [0.0, 0.0, 0.0]);
    let a = align_roles(&w, &[(upright, Role::Z), (side, Role::X), (c, Role::Origin)])
        .expect("edge sets Z, face sets X, corner is the origin");
    let len = |v: [f64; 3]| v.iter().map(|x| x * x).sum::<f64>().sqrt();
    let dot = |u: [f64; 3], v: [f64; 3]| u.iter().zip(v).map(|(a, b)| a * b).sum::<f64>();
    for axis in [a.x, a.y, a.z] {
        assert!((len(axis) - 1.0).abs() < 1e-9, "unit axes: {axis:?}");
    }
    assert!(dot(a.x, a.y).abs() < 1e-9 && dot(a.x, a.z).abs() < 1e-9 && dot(a.y, a.z).abs() < 1e-9, "square axes:\n{:?}\n{:?}\n{:?}", a.x, a.y, a.z);
    let q = quat_from_euler(25f64.to_radians(), 0.0, 0.0);
    let want = |v: Point3| {
        let r = quat_rotate_point(q, v);
        [r.x, r.y, r.z]
    };
    assert!(close(a.x, want(Point3::new(1.0, 0.0, 0.0)), 1e-9), "X out of the side face: {:?}", a.x);
    assert!(close(a.z, want(Point3::new(0.0, 0.0, 1.0)), 1e-9), "Z along the tipped edge: {:?}", a.z);
    assert!(close(a.y, want(Point3::new(0.0, 1.0, 0.0)), 1e-9), "right-handed: {:?}", a.y);
    assert!((a.roll_deg - 25.0).abs() < 1e-9 && a.pitch_deg.abs() < 1e-9 && a.yaw_deg.abs() < 1e-9, "the tip reads true: {:?}", (a.roll_deg, a.pitch_deg, a.yaw_deg));
    assert!(close(a.pos, [0.0, 0.0, 0.0], 1e-9), "origin at the corner that sided the edge: {:?}", a.pos);
}

#[test]
fn every_degenerate_role_set_is_refused_by_name() {
    let w = block(0.0);
    let top = face(&w, "block", [10.0, 10.0, 10.0]);
    let side = face(&w, "block", [10.0, 0.0, 5.0]);
    let front = edge(&w, "block", [0.0, 0.0, 10.0], [20.0, 0.0, 10.0]);
    let upright = edge(&w, "block", [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
    let c = corner(&w, "block", [0.0, 0.0, 10.0]);
    let says = |picks: &[(GeoSel, Role)], want: &str| {
        let e = align_roles(&w, picks).expect_err("must refuse");
        assert!(e.contains(want), "{want:?} not in {e:?}");
    };
    says(&[], "at least two picks");
    says(&[(top.clone(), Role::Z)], "at least two picks");
    says(&[(c.clone(), Role::Origin)], "at least two picks");
    says(&[(top.clone(), Role::Z), (side.clone(), Role::Z)], "only one pick may set Z");
    says(&[(top.clone(), Role::Z), (c.clone(), Role::Origin), (corner(&w, "block", [20.0, 0.0, 10.0]), Role::Origin)], "only one pick may set Origin");
    says(&[(top.clone(), Role::Z), (upright.clone(), Role::X)], "too near parallel");
    says(&[(c.clone(), Role::X), (front.clone(), Role::Z)], "sets no axis");
    says(&[(front.clone(), Role::Origin), (top.clone(), Role::Z), (edge(&w, "block", [0.0, 0.0, 0.0], [20.0, 0.0, 0.0]), Role::X)], "sets an axis");
    let loop_pick = GeoSel { entity: "block".into(), feature: FeatureSel::Chain(0xdead_beef) };
    says(&[(loop_pick, Role::X), (top.clone(), Role::Z)], "no longer in the model");
    let ghost = GeoSel { entity: "block".into(), feature: FeatureSel::Vertex(0xdead_beef) };
    says(&[(top.clone(), Role::Z), (front.clone(), Role::X), (ghost, Role::Origin)], "no longer in the model");
}

