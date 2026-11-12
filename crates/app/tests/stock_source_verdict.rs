//! #1094: a casting imported to be copied into a "from model" stock and
//! left in the tree is stock, not the part. The program verdict used to
//! read every non-fixture model as the finished part, so every cut into
//! the casting - every cut - was a gouge and the post was blocked. These
//! tests pin the rule (`stock_source::stock_only_model`): the casting is
//! left out only while it is the source of every program setup's stock,
//! no operation names it, and an operation names a separate part; the
//! separate part stays judged (a real gouge into it still blocks), and
//! every other shape of document keeps today's verdict.
//!
//! Provenance: our own fixtures (`ladder_entry_height.rs`'s blind hole
//! under a casting), specced from the Prime Directive; no vendor material.
//!
//! Gate map (#1094): covers app::stock_source, app::stock_sim (the part
//! set), app::setup_align (parts_facets_in), app::posting (the gate).
//! Lane shape: all tests here run in seconds.

use flecs_ecs::prelude::World;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle, cube};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_source::stock_only_model;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

/// The part's top and the hole's mouth.
const MOUTH: f64 = 20.0;
/// The modelled hole's floor.
const FLOOR: f64 = 10.0;
const TOOL: &str = "EM-0800-4FL";

fn quad(a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>) {
    f.push(Triangle::new(a, b, c));
    f.push(Triangle::new(a, c, d));
}

fn flip(t: &Triangle) -> Triangle {
    Triangle::new(t.a, t.c, t.b)
}

/// A 60 x 60 casting, Z -10..20 - the stock's source.
fn casting() -> Mesh {
    cube(Point3::new(-30.0, -30.0, -10.0), Point3::new(30.0, 30.0, MOUTH))
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

/// A milled bore of a 16 mm hole at the origin, from the mouth to `floor`.
fn bore(floor: f64) -> OpParams {
    OpParams::Bore {
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
        holes: vec![Hole { xy: Point2::new(0.0, 0.0), radius: 8.0, entry_z: MOUTH, floor_z: floor, through: false, stepped: false }],
    }
}

/// The casting left in the tree as the stock's source; with `with_part`
/// a separate part, and the Bore (to `floor`) reads it, else the stock.
fn job(with_part: bool, floor: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", casting()));
    let src = if with_part {
        h.execute(&world, ImportModel::from_mesh("part", part()));
        SrcRef::Entity("part".into())
    } else {
        SrcRef::Stock
    };
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "Op1", TOOL, src, bore(floor)).expect("create op"));
    h.execute(&world, GenerateOperation::new(&world, "Op1").expect("generate op"));
    (world, h)
}

fn post(world: &World, tag: &str) -> Result<(), String> {
    let out = std::env::temp_dir().join(format!("camcraft-1094-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let posted = posting::post_setup(world, &out).map(|_| ());
    let _ = std::fs::remove_file(&out);
    posted
}

fn gouge_line(world: &World) -> Option<String> {
    posting::verification_gate(world).into_iter().find(|l| l.contains("gouge"))
}

fn casting_entity(world: &World) -> flecs_ecs::prelude::Entity {
    model::find_by_name(world, "casting").expect("casting")
}

#[test]
fn a_casting_left_in_the_tree_as_stock_source_is_not_judged_and_the_program_posts() {
    let (world, _) = job(true, FLOOR);
    assert_eq!(stock_only_model(&world), Some(casting_entity(&world)), "the casting is only the stock");
    assert_eq!(gouge_line(&world), None, "the cut into the casting read as a gouge");
    let posted = post(&world, "legal");
    assert!(posted.is_ok(), "a legal program over a separate part was refused: {posted:?}");
}

#[test]
fn the_separate_part_stays_judged_a_cut_below_its_floor_still_blocks() {
    // The same job, the Bore 5 mm below the modelled hole's floor: the
    // part is still the part, so the post is refused.
    let (world, _) = job(true, FLOOR - 5.0);
    assert_eq!(stock_only_model(&world), Some(casting_entity(&world)));
    let posted = post(&world, "deep");
    let why = posted.expect_err("a bore 5 mm into the part's floor posted");
    assert!(why.contains("gouge"), "refused, but not as a gouge: {why}");
}

#[test]
fn without_a_separate_part_the_casting_keeps_todays_verdict() {
    // Every op on the stock and no other model: nothing else to judge, so
    // the casting stays the part (the stock made from the part itself).
    let (world, _) = job(false, FLOOR);
    assert_eq!(stock_only_model(&world), None);
    assert!(gouge_line(&world).is_some(), "the casting left the verdict with no part to replace it");
}

#[test]
fn picking_the_casting_for_an_op_takes_it_back_into_the_verdict_and_undo_lets_it_go() {
    let (world, mut h) = job(true, FLOOR);
    // A second op (never generated: the rule reads sources, not paths)
    // names the casting - it is a part again.
    let op2 = CreateOperation::new(&world, "Op2", TOOL, SrcRef::Entity("casting".into()), bore(FLOOR)).expect("op2");
    h.execute(&world, op2);
    assert_eq!(stock_only_model(&world), None, "a casting an op machines must stay judged");
    h.undo(&world);
    assert_eq!(stock_only_model(&world), Some(casting_entity(&world)), "undo of the op lets it go again");
    h.redo(&world);
    assert_eq!(stock_only_model(&world), None, "redo takes it back");
}

#[test]
fn undo_of_set_stock_takes_the_casting_back_into_the_verdict() {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", casting()));
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    let shape = StockShape::Box { dx: 60.0, dy: 60.0, dz: 30.0 };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("box stock"));
    h.execute(&world, CreateOperation::new(&world, "Op1", TOOL, SrcRef::Entity("part".into()), bore(FLOOR)).expect("op"));
    assert_eq!(stock_only_model(&world), None, "a box stock has no source model");
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("model stock"));
    assert_eq!(stock_only_model(&world), Some(casting_entity(&world)));
    h.undo(&world);
    assert_eq!(stock_only_model(&world), None, "undo of SetStock: the casting is a model again");
    h.redo(&world);
    assert_eq!(stock_only_model(&world), Some(casting_entity(&world)), "redo");
}

#[test]
fn a_part_that_is_both_the_stock_source_and_the_ops_source_stays_judged() {
    // `edge_break_gate.rs`'s brick: the stock IS the part, the op on it.
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let shape = StockShape::FromModel { source: "part".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "Op1", TOOL, SrcRef::Entity("part".into()), bore(FLOOR)).expect("op"));
    assert_eq!(stock_only_model(&world), None);
}
