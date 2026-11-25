//! A from-model stock of previously cut material can already sit below the
//! part in places. With the stock marked raw stock only and those deficits
//! acknowledged for the session (`SetStockSourceOnly::with_inherited_deficits`),
//! the post reports cells the program leaves at their incoming heights as a
//! located note instead of gouges. A program that cuts into such a cell, or
//! below the part anywhere else, still blocks. Opening the project clears the
//! acknowledgement.
//!
//! Provenance: our own fixtures (`stock_source_verdict.rs`'s casting and blind
//! hole part, with a 5 mm step lowered into the casting); no vendor material.
//!
//! Gate map: covers app::stock_sim (incoming), app::stock_source
//! (incoming_stock_source), app::commands (stock_source_only),
//! app::posting_sim (the clause), app::project (the mark on load).
//! Lane shape: all tests here run in seconds.

use flecs_ecs::prelude::*;

use camcraft_app::accessory::mesh::prism;
use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock, SetStockSourceOnly};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle, cube};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

/// The part's top and the hole's mouth.
const MOUTH: f64 = 20.0;
/// The modelled hole's floor.
const FLOOR: f64 = 10.0;
/// The casting's top over the step (y >= STEP_Y): 5 mm under the part's top.
const STEP_TOP: f64 = 15.0;
const STEP_Y: f64 = 12.0;
const TOOL: &str = "EM-0800-4FL";

fn quad(a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>) {
    f.push(Triangle::new(a, b, c));
    f.push(Triangle::new(a, c, d));
}

fn flip(t: &Triangle) -> Triangle {
    Triangle::new(t.a, t.c, t.b)
}

/// A 60 x 60 casting, Z -10..20, cut down to Z 15 for y >= 12: stock that an
/// earlier setup already took below the part along one side.
fn stepped_casting() -> Mesh {
    let profile = [(-30.0, -10.0), (30.0, -10.0), (30.0, STEP_TOP), (STEP_Y, STEP_TOP), (STEP_Y, MOUTH), (-30.0, MOUTH)];
    Mesh::from_facets(prism(&profile, -30.0, 30.0).expect("casting prism"))
}

/// A 40 x 40 x [0, 20] block with a 20 x 20 blind hole from its top to Z 10.
fn part() -> Mesh {
    let mut f = Vec::new();
    let outer = cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, MOUTH));
    f.extend(outer.facets[4..12].iter().cloned());
    let o = [(-20.0, -20.0), (20.0, -20.0), (20.0, 20.0), (-20.0, 20.0)];
    let i = [(-10.0, -10.0), (10.0, -10.0), (10.0, 10.0), (-10.0, 10.0)];
    let p = |(x, y): (f64, f64)| Point3::new(x, y, MOUTH);
    for k in 0..4 {
        let n = (k + 1) % 4;
        quad(p(o[k]), p(o[n]), p(i[n]), p(i[k]), &mut f);
    }
    let inner = cube(Point3::new(-10.0, -10.0, FLOOR), Point3::new(10.0, 10.0, MOUTH));
    f.extend(inner.facets[4..12].iter().map(flip));
    f.extend(outer.facets[0..2].iter().cloned());
    f.extend(inner.facets[0..2].iter().map(flip));
    Mesh::from_facets(f)
}

/// A milled bore of one hole, from the part's top to `floor`.
fn bore(xy: Point2, radius: f64, floor: f64) -> OpParams {
    OpParams::Bore {
        pass_start: camcraft_app::machining2d::PassStart::Auto,
        break_through: None,
        links: camcraft_app::links::Links::PLANAR,
        clearance_z: Height::fixed(40.0),
        retract_z: None,
        top_z: Height::fixed(MOUTH),
        bottom_z: None,
        stock_to_leave: 0.0,
        stepdown: 5.0,
        feed: 500.0,
        plunge_feed: 150.0,
        climb: true,
        tolerance: 0.01,
        holes: vec![Hole { xy, radius, entry_z: MOUTH, floor_z: floor, through: false, stepped: false }],
    }
}

fn op(world: &World, h: &mut History, name: &str, params: OpParams) {
    h.execute(world, CreateOperation::new(world, name, TOOL, SrcRef::Entity("part".into()), params).expect("create op"));
    h.execute(world, GenerateOperation::new(world, name).expect("generate op"));
}

/// The stepped casting as the stock's source, the part, and a legal bore of
/// the blind hole: nothing in the program reaches the step. `acknowledge`
/// marks the casting raw stock only with or without the acknowledgement;
/// the ops are generated after the mark, as a user would.
fn job(acknowledge: bool, extra: Option<OpParams>) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", stepped_casting()));
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    let mark = SetStockSourceOnly::with_inherited_deficits(&world, true, acknowledge).expect("mark");
    h.execute(&world, mark);
    op(&world, &mut h, "Op1", bore(Point2::new(0.0, 0.0), 8.0, FLOOR));
    if let Some(params) = extra {
        op(&world, &mut h, "Op2", params);
    }
    world
}

fn post(world: &World, tag: &str) -> Result<posting::Posted, String> {
    let out = std::env::temp_dir().join(format!("camcraft-incoming-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let posted = posting::post_setup(world, &out);
    let _ = std::fs::remove_file(&out);
    posted
}

/// The (x, y, z) a note or gouge line names after "at (".
fn located(line: &str) -> (f64, f64, f64) {
    let start = line.find(" at (").expect("a location") + 5;
    let end = start + line[start..].find(')').expect("closing parenthesis");
    let mut parts = line[start..end].split(", ");
    let mut next = || parts.next().expect("three values").trim_start_matches("z ").parse::<f64>().expect("a number");
    (next(), next(), next())
}

#[test]
fn without_the_acknowledgement_the_incoming_step_blocks_as_a_gouge() {
    let world = job(false, None);
    let why = post(&world, "plain").expect_err("stock under the part posted without the acknowledgement");
    assert!(why.contains("gouge"), "refused, but not as a gouge: {why}");
}

#[test]
fn acknowledged_the_unchanged_step_posts_with_a_located_note() {
    let world = job(true, None);
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "the untouched step still blocks: {:?}", report.problems);
    let note = report.not_judged.expect("the acknowledged deficits are said, never silent");
    assert!(note.starts_with("note: incoming stock 'casting' already has "), "{note}");
    assert!(note.contains("worst 5.00 mm at ("), "{note}");
    let (x, y, z) = located(&note);
    assert!((-20.0..=20.0).contains(&x) && (STEP_Y..=20.0).contains(&y), "the worst cell is off the step: {note}");
    assert!((z - STEP_TOP).abs() < 1e-6, "the worst cell is not at the step's height: {note}");
    let posted = post(&world, "acknowledged").expect("a program that leaves the step alone was refused");
    assert_eq!(posted.not_judged.as_deref(), Some(note.as_str()), "the post reports the same note");
}

#[test]
fn acknowledged_a_cut_into_the_step_still_blocks() {
    // A 9 mm bore in the step, to Z 12: 3 mm deeper than the incoming
    // stock there and 8 mm under the part's top.
    let world = job(true, Some(bore(Point2::new(0.0, 16.0), 4.5, 12.0)));
    let why = post(&world, "deepened").expect_err("a bore into the acknowledged step posted");
    assert!(why.contains("gouge"), "refused, but not as a gouge: {why}");
    let (x, y, z) = located(&why);
    let off_centre = x.hypot(y - 16.0);
    assert!(off_centre <= 4.5 + 0.5 && (z - 12.0).abs() < 1e-6, "the gouge is not the bore's cut: {why}");
}

#[test]
fn the_acknowledgement_needs_the_mark() {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", stepped_casting()));
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    let why = SetStockSourceOnly::with_inherited_deficits(&world, false, true).err().map(|e| e.to_string());
    assert!(why.is_some_and(|w| w.contains("requires marking")), "acknowledged without the mark");
}

#[test]
fn opening_the_project_clears_the_acknowledgement() {
    let world = job(true, None);
    assert!(posting::verification_report(&world).problems.is_empty());
    let path = std::env::temp_dir().join(format!("camcraft-incoming-reopen-{}.camproj", std::process::id()));
    camcraft_app::project::save(&world, &path).expect("save");
    let loaded = new_world();
    camcraft_app::project::load(&loaded, &path).expect("load");
    let _ = std::fs::remove_file(&path);
    assert_eq!(camcraft_app::stock_source::raw_stock_only_of(&loaded, None), Some(true));
    let report = posting::verification_report(&loaded);
    assert!(
        report.problems.iter().any(|p| p.starts_with("gouge")),
        "the acknowledgement survived opening the project: {report:?}"
    );
    assert!(!report.not_judged.unwrap_or_default().contains("incoming stock"), "the note survived opening the project");
}
