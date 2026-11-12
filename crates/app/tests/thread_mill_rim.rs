//! #1071: a thread mill on a model-sourced hole. The hole the model shows
//! (and the hole the op declares) is the thread's MINOR circle; the helix
//! runs the thread depth H = 0.5413 x pitch past it by design, out to the
//! major circle (`motion::machining2d::thread_mill`). The containment
//! verify read the declared rim at the minor circle, and the solid law the
//! model's minor wall, so every helix move of a matched thread mill was
//! convicted. These pin that a matched thread mill verifies clean against
//! a model, and that one cutting past its major circle - into a nearby
//! wall, or into a model hole smaller than the declared one - still
//! convicts.
//!
//! Gate map (#1071): covers app::operation::generate (the Thread verify),
//! app::operation::material_source (rim loops), sim::verify::section,
//! sim::verify::solid.
//! Lane shape (M4.4): runs in seconds.
//!
//! Provenance: our own fixtures over our own laws; the 0.5413 x pitch
//! radial depth is ISO 724's, as the generator cites it.

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{cube, Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_app::verify::Warning;
use camcraft_app::world::new_world;

/// The block's top, and the floor of its hole or cavity.
const TOP: f64 = 20.0;
const FLOOR: f64 = 10.0;
/// Sides of the round hole's polygon: 2.7 um of sagitta at r 5.
const SIDES: usize = 96;

fn flip(t: &Triangle) -> Triangle {
    Triangle::new(t.a, t.c, t.b)
}

/// A 40 x 40 x [0, 20] block with a 20 x 20 square cavity from the top,
/// blind at Z 10 (#1073's `coarse_rims` part): a thread there runs in air.
fn cavity_block() -> Mesh {
    const H: f64 = 10.0;
    let mut f: Vec<Triangle> = Vec::new();
    let outer = cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, TOP));
    f.extend(outer.facets[4..12].iter().cloned());
    let o = [(-20.0, -20.0), (20.0, -20.0), (20.0, 20.0), (-20.0, 20.0)];
    let i = [(-H, -H), (H, -H), (H, H), (-H, H)];
    let p = |(x, y): (f64, f64)| Point3::new(x, y, TOP);
    for k in 0..4 {
        let n = (k + 1) % 4;
        f.push(Triangle::new(p(o[k]), p(o[n]), p(i[n])));
        f.push(Triangle::new(p(o[k]), p(i[n]), p(i[k])));
    }
    let inner = cube(Point3::new(-H, -H, FLOOR), Point3::new(H, H, TOP));
    f.extend(inner.facets[4..12].iter().map(flip));
    f.extend(outer.facets[0..2].iter().cloned());
    f.extend(inner.facets[0..2].iter().map(flip));
    Mesh::from_facets(f)
}

/// The same block with a round blind hole of radius `r` at the origin,
/// from the top down to Z 10: what CAD draws of a tapped hole (the minor
/// cylinder). Closed and outward-facing.
fn drilled_block(r: f64) -> Mesh {
    let mut f: Vec<Triangle> = Vec::new();
    let outer = cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, TOP));
    f.extend(outer.facets[4..12].iter().cloned());
    f.extend(outer.facets[0..2].iter().cloned());
    let ang = |k: usize| std::f64::consts::TAU * (k % SIDES) as f64 / SIDES as f64 + std::f64::consts::FRAC_PI_4;
    let rim = |k: usize, z: f64| Point3::new(r * ang(k).cos(), r * ang(k).sin(), z);
    // The square boundary point on the same ray: the corners fall on
    // k = 0, 24, 48, 72 (the angles start at 45 degrees).
    let edge = |k: usize| {
        let (c, s) = (ang(k).cos(), ang(k).sin());
        let t = 20.0 / c.abs().max(s.abs());
        Point3::new(t * c, t * s, TOP)
    };
    let centre = Point3::new(0.0, 0.0, FLOOR);
    for k in 0..SIDES {
        let (p0, p1) = (rim(k, TOP), rim(k + 1, TOP));
        let (q0, q1) = (edge(k), edge(k + 1));
        f.push(Triangle::new(p0, q0, q1)); // top face, +z
        f.push(Triangle::new(p0, q1, p1));
        let (b0, b1) = (rim(k, FLOOR), rim(k + 1, FLOOR));
        f.push(Triangle::new(b0, p0, p1)); // hole wall, facing the axis
        f.push(Triangle::new(b0, p1, b1));
        f.push(Triangle::new(centre, b0, b1)); // floor, +z
    }
    Mesh::from_facets(f)
}

fn hole(x: f64, radius: f64) -> Hole {
    Hole { xy: Point2::new(x, 0.0), radius, entry_z: TOP, floor_z: 10.2, through: false, stepped: false }
}

fn thread(x: f64, radius: f64) -> OpParams {
    OpParams::Thread {
        clearance_z: Height::fixed(30.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: None,
        infeed: 0.1,
        feed: 400.0,
        plunge_feed: 150.0,
        climb: true,
        holes: vec![hole(x, radius)],
    }
}

/// A 6 mm Tap-form thread mill, pitch 1 (`mill2d_c`'s), long enough for
/// the hole.
fn thread_mill() -> Tool {
    Tool {
        kind: ToolKind::Tap,
        diameter: 6.0,
        corner_r: 0.0,
        tip_angle: 0.0,
        pitch: 1.0,
        nose_radius: 0.0,
        width: 0.0,
        flute_len: 25.0,
        shank_d: 6.0,
        shank_len: 40.0,
        holder_d: 44.0,
        holder_len: 35.0,
        material_group: "steel".into(),
        center_cutting: true,
        rpm: 6000.0,
        feed: 400.0,
        tool_material: None,
        flutes: None,
        orientation: None,
    }
}

/// The blocking warnings of `params` run over `part`, model-sourced.
fn verdict(part: Mesh, params: OpParams) -> Vec<Warning> {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", part));
    let loc = Location { pos: Point3::new(0.0, 0.0, -2.0), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 22.0 }, loc).expect("stock"));
    h.execute(&world, AddTool::new(&world, "T", thread_mill()).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "OP", "T", SrcRef::Entity("part".into()), params).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "OP").expect("generate"));
    let e = model::find_by_name(&world, "OP").expect("op");
    model::toolpath_of(&world, &e).expect("cached").warnings.into_iter().filter(Warning::blocks_post).collect()
}

fn gouged(w: &[Warning]) -> bool {
    w.iter().any(|w| matches!(w, Warning::Gouge { .. }))
}

#[test]
fn a_matched_thread_mill_in_air_verifies_clean() {
    // M10-style: a 5 mm minor radius in the cavity, nothing but the
    // declared rim to read against.
    assert_eq!(verdict(cavity_block(), thread(0.0, 5.0)), Vec::new());
}

#[test]
fn a_matched_thread_mill_in_a_drilled_hole_verifies_clean() {
    // The model's hole IS the minor cylinder the op declares.
    assert_eq!(verdict(drilled_block(5.0), thread(0.0, 5.0)), Vec::new());
}

#[test]
fn a_thread_whose_major_circle_reaches_a_wall_convicts() {
    // At X 4.7 the major circle (5.5413) reaches X 10.24: 0.24 mm into
    // the cavity's wall at X 10, while the minor circle (9.7) stays
    // short of it.
    let w = verdict(cavity_block(), thread(4.7, 5.0));
    assert!(gouged(&w), "{w:?}");
}

#[test]
fn a_thread_declared_wider_than_the_models_hole_convicts() {
    // The model drilled at r 4.5, the op declaring r 5: the helix cuts
    // 1.04 mm into the model's wall, not the thread depth past a wall it
    // was declared against.
    let w = verdict(drilled_block(4.5), thread(0.0, 5.0));
    assert!(gouged(&w), "{w:?}");
}

#[test]
fn a_thread_taken_below_the_models_hole_floor_convicts() {
    // The model's hole is blind at Z 10; the op declares its floor at
    // Z 9, so the helix's last turns cut a millimetre into the floor:
    // the grant covers the wall and the mouth, never a floor.
    let mut h = hole(0.0, 5.0);
    h.floor_z = 9.0;
    let OpParams::Thread { clearance_z, retract_z, top_z, bottom_z, infeed, feed, plunge_feed, climb, .. } = thread(0.0, 5.0) else {
        unreachable!("a thread op")
    };
    let p = OpParams::Thread { clearance_z, retract_z, top_z, bottom_z, infeed, feed, plunge_feed, climb, holes: vec![h] };
    let w = verdict(drilled_block(5.0), p);
    assert!(gouged(&w), "{w:?}");
}

#[test]
fn a_stock_sourced_thread_mill_verifies_clean_too() {
    // The same matched thread cut from the stock source: before #1071
    // it was blocked as well (52 gouges and 21 crossing pairs), which
    // `mill2d_c` never asserted.
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", cavity_block()));
    let loc = Location { pos: Point3::new(0.0, 0.0, -2.0), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 22.0 }, loc).expect("stock"));
    h.execute(&world, AddTool::new(&world, "T", thread_mill()).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "OP", "T", SrcRef::Stock, thread(0.0, 5.0)).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "OP").expect("generate"));
    let e = model::find_by_name(&world, "OP").expect("op");
    let w: Vec<Warning> =
        model::toolpath_of(&world, &e).expect("cached").warnings.into_iter().filter(Warning::blocks_post).collect();
    assert_eq!(w, Vec::new());
}
