//! #804: the index-swing clause of the program gate. Before a 4th axis
//! that turns the work indexes, the tool sits at the safe Z; its tip must
//! clear the radius the stock, parts and turning fixtures sweep about the
//! A centerline by `MARGIN_MM`, or the post is refused naming the op, the
//! A, the swept radius and the clearance. Machines with no work-turning A
//! are untouched.
//! Provenance: our own tests over our own commands and machine files; the
//! microARC 4 / 770MX numbers from Tormach's published spec sheets.
//!
//! Gate map (M4.4): covers app::posting (index_swing, verification_gate),
//! post::machine (nose_to_axis, turns_part).

use camcraft_app::commands::{
    AddTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SelectPost, SetStock,
    TagFixture,
};
use camcraft_app::geometry::{Point3, cube};
use camcraft_app::geo2d::Point2;
use camcraft_app::machine::{Machine, parse_mch};
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{Height, OpParams, SrcRef, Tilt};
use camcraft_app::posting::{MACHINES, index_swing::MARGIN_MM, verification_gate};
use camcraft_app::schema::{Location, PostRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use flecs_ecs::prelude::World;

const IN: f64 = 25.4;
const R: f64 = 0.375 * IN;
const STICK: f64 = 1.5 * IN;
const ARC4: &str = "Tormach 770MX + microARC 4";
/// The microARC file's statement: 13.75 in nose to table less 3.15 in
/// table to centerline.
const NOSE: f64 = 10.6 * IN;

fn shipped(name: &str) -> Machine {
    let text = MACHINES.iter().find(|(n, _)| *n == name).map(|(_, t)| *t).expect("shipped");
    parse_mch(text).expect("parses")
}

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    r.unwrap_or_else(|e| panic!("{what}: {e:?}"))
}

/// The kit's 3/4" bar (its bounding box as stock, 1.5" out of the chuck)
/// on `machine`, one 1/4" end mill `len` mm long from nose to tip, and
/// one shallow flat at `a` (None: upright).
fn doc(machine: Machine, len: f64, a: Option<f64>) -> World {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, ok(LoadMachine::new(&world, "test".into(), Some(machine)), "machine"));
    h.execute(&world, ok(SelectPost::new(&world, PostRef::Named("Tormach PathPilot".into())), "post"));
    let stock = SetStock::new(
        &world,
        StockShape::Box { dx: STICK, dy: 2.0 * R, dz: 2.0 * R },
        Location { pos: Point3::new(-STICK / 2.0, 0.0, -R), ..Location::default() },
    );
    h.execute(&world, ok(stock, "stock"));
    let mut t = catalog().into_iter().find(|(n, _)| *n == "EM-0600-4FL").map(|(_, t)| t).expect("catalog");
    t.diameter = 0.25 * IN;
    t.shank_d = t.diameter;
    t.rpm = 6000.0;
    t.feed = 15.0 * IN;
    t.holder_len = len - t.flute_len - t.shank_len;
    assert!(t.holder_len > 0.0, "the holder takes up the rest of the length");
    h.execute(&world, ok(AddTool::new(&world, "EM 1/4", t), "tool"));
    let tilt = a.map(|a_deg| Tilt { a_deg, b_deg: 0.0, origin: [0.0; 3] });
    let flat = OpParams::Slot {
        a: Point2::new(-0.75 * IN, 0.0),
        b: Point2::new(-0.25 * IN, 0.0),
        width: 0.25 * IN,
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(R + 0.5 * IN),
        retract_z: Some(Height::fixed(R + 0.1 * IN)),
        top_z: Height::fixed(R),
        bottom_z: Height::fixed(R - 0.015 * IN),
        stepdown: 0.015 * IN,
        stepover: 0.1 * IN,
        feed: 15.0 * IN,
        plunge_feed: 4.0 * IN,
        climb: true,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        links: camcraft_app::links::Links::PLANAR,
        tilt,
    };
    h.execute(&world, ok(CreateOperation::new(&world, "FLAT", "EM 1/4", SrcRef::Stock, flat), "op"));
    h.execute(&world, ok(GenerateOperation::new(&world, "FLAT"), "generate"));
    world
}

fn swing_lines(world: &World) -> Vec<String> {
    verification_gate(world)
        .into_iter()
        .filter(|p| p.contains("index turns the work") || p.contains("nose_to_axis"))
        .collect()
}

/// The bar's box corner is the farthest swept point: R * sqrt(2).
fn bar_swept() -> f64 {
    R * std::f64::consts::SQRT_2
}

#[test]
fn the_shipped_microarc_states_its_nose_height_and_a_work_turning_a() {
    let m = shipped(ARC4);
    let a = m.axes.iter().find(|a| a.name == "A").expect("an A");
    assert!(a.turns_part, "the microARC sits on the bed: its A turns the work");
    let nose = a.nose_to_axis.expect("stated");
    assert!((nose - NOSE).abs() < 1e-9, "{nose}");
    // A head tilt carries the spindle: its A turns the tool, not the work.
    let tilt = shipped("Tilt-head 5-axis");
    assert!(tilt.axes.iter().filter(|a| a.rotary).all(|a| !a.turns_part && a.nose_to_axis.is_none()));
}

#[test]
fn a_short_tool_over_the_kit_bar_clears_and_posts() {
    let world = doc(shipped(ARC4), 3.0 * IN, Some(90.0));
    assert!(swing_lines(&world).is_empty(), "{:?}", verification_gate(&world));
    let out = std::env::temp_dir().join(format!("camcraft_swing_ok_{}.nc", std::process::id()));
    let posted = camcraft_app::posting::post_setup(&world, &out);
    let _ = std::fs::remove_file(&out);
    let nc = posted.unwrap_or_else(|e| panic!("the short tool must post: {e}")).program;
    assert!(nc.contains("G0 A90"), "{nc}");
}

#[test]
fn a_tool_too_long_to_clear_the_bar_is_refused_by_name() {
    // The tip sits 2 mm short of the margin over the bar's swept corner.
    let len = NOSE - bar_swept() - MARGIN_MM + 2.0;
    let world = doc(shipped(ARC4), len, Some(90.0));
    let lines = swing_lines(&world);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let l = &lines[0];
    assert!(l.starts_with("FLAT: the A90.000 index turns the work"), "{l}");
    assert!(l.contains(&format!("sweep {:.3} mm", bar_swept())), "{l}");
    assert!(l.contains("a clearance of 3.000 mm against the 5.0 mm needed"), "{l}");
    let out = std::env::temp_dir().join(format!("camcraft_swing_long_{}.nc", std::process::id()));
    let err = camcraft_app::posting::post_setup(&world, &out).expect_err("a tool that hits must not post");
    assert!(err.contains("FLAT: the A90.000 index"), "{err}");
    assert!(!out.exists(), "a refused post writes nothing");
}

#[test]
fn a_fat_chuck_turning_with_the_bar_is_refused_by_name() {
    let world = doc(shipped(ARC4), 3.0 * IN, Some(90.0));
    let mut h = History::default();
    // A 400 mm square "chuck" behind the bar: it turns with the work and
    // its corners sweep 283 mm, past the 3" tool's 193 mm tip.
    let chuck = cube(Point3::new(-STICK - 40.0, -200.0, -200.0), Point3::new(-STICK - 5.0, 200.0, 200.0));
    h.execute(&world, ImportModel::from_mesh("chuck", chuck));
    h.execute(&world, ok(TagFixture::new(&world, "chuck", None), "tag"));
    let lines = swing_lines(&world);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(&format!("sweep {:.3} mm", 200.0 * std::f64::consts::SQRT_2)), "{}", lines[0]);
}

#[test]
fn a_work_turning_a_with_no_nose_height_is_refused_by_name() {
    let mut m = shipped(ARC4);
    for a in &mut m.axes {
        a.nose_to_axis = None;
    }
    let world = doc(m, 3.0 * IN, Some(-90.0));
    let lines = swing_lines(&world);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].starts_with("FLAT: Tormach 770MX + microARC 4 turns the work on A but states no nose_to_axis"), "{}", lines[0]);
    assert!(lines[0].contains("A-90.000 index"), "{}", lines[0]);
}

#[test]
fn a_machine_with_no_a_is_unaffected() {
    // The same too-long tool, upright on the plain 770MX: no A, no clause.
    let len = NOSE - bar_swept() - MARGIN_MM + 2.0;
    let world = doc(shipped("Tormach 770MX"), len, None);
    assert!(swing_lines(&world).is_empty(), "{:?}", verification_gate(&world));
}
