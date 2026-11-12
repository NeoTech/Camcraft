//! #1091: the rapid-in-stock law (#96, `RapidInStock`) caps the stock
//! column at the stock's highest point. On a stepped "from model" stock -
//! a casting solid to Z 20 over the hole's mouth with a boss to Z 30
//! elsewhere - a Bore's entry dive to Z 22 was convicted as if the boss
//! stood everywhere, and the post was blocked for a legal program. These
//! tests pin the law reading the stock's top under the dive
//! (`verify::acquit_by_stock_top`): the dive over the low part passes, and
//! the same dive with the boss under it - or the boss reaching into the
//! tool's footprint by less than its radius - is still convicted and
//! still blocks the post. Both a part-sourced and a stock-sourced Bore.
//!
//! Provenance: our own fixtures (`ladder_entry_height.rs`'s blind hole
//! under a stepped casting), specced from the Prime Directive and the
//! shop rule "never rapid into uncut stock"; no vendor material.
//!
//! Gate map (#1091): covers app::operation (generate's verify wiring,
//! the Bore arm, stock_top acquittal), sim::verify (rapids, stock_top),
//! app::posting (the gate).
//! Lane shape: all tests here run in seconds.

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{AddTool, CreateOperation, DeleteEntity, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle, cube};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::world::new_world;

/// The part's top, the hole's mouth and the casting's low top: Z 20.
const MOUTH: f64 = 20.0;
/// The boss's top.
const BOSS: f64 = 30.0;
const FLOOR: f64 = 10.0;
const TOOL: &str = "EM-0800-4FL";
/// The 8 mm end mill's radius.
const R: f64 = 4.0;

fn quad(a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>) {
    f.push(Triangle::new(a, b, c));
    f.push(Triangle::new(a, c, d));
}

/// A closed, outward-wound casting: a 60 x 60 slab Z -10..20 with a boss
/// over `[x0, x1] x [y0, y1]` up to Z 30 (inside the slab's outline).
fn casting(x0: f64, x1: f64, y0: f64, y1: f64) -> Mesh {
    let slab = cube(Point3::new(-30.0, -30.0, -10.0), Point3::new(30.0, 30.0, MOUTH));
    let boss = cube(Point3::new(x0, y0, MOUTH), Point3::new(x1, y1, BOSS));
    let mut f: Vec<Triangle> = slab.facets[0..2].to_vec(); // bottom
    f.extend(slab.facets[4..12].iter().cloned()); // sides
    let o = [(-30.0, -30.0), (30.0, -30.0), (30.0, 30.0), (-30.0, 30.0)];
    let i = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
    let p = |(x, y): (f64, f64)| Point3::new(x, y, MOUTH);
    for k in 0..4 {
        let n = (k + 1) % 4;
        quad(p(o[k]), p(o[n]), p(i[n]), p(i[k]), &mut f); // the low top, ringed round the boss
    }
    f.extend(boss.facets[2..12].iter().cloned()); // the boss's top and sides
    Mesh::from_facets(f)
}

fn flip(t: &Triangle) -> Triangle {
    Triangle::new(t.a, t.c, t.b)
}

/// `ladder_entry_height.rs`'s part: a 40 x 40 x [0, 20] block with a
/// 20 x 20 blind hole from its top to Z 10.
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

/// A milled bore of a 16 mm hole at the origin, mouth Z 20.
fn bore() -> OpParams {
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
        holes: vec![Hole { xy: Point2::new(0.0, 0.0), radius: 8.0, entry_z: MOUTH, floor_z: FLOOR, through: false, stepped: false }],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Src {
    /// The Bore reads the part; the casting is only the stock.
    Part,
    /// The Bore reads the stock itself (no part model at all).
    Stock,
}

/// The Bore from `src` under the casting with its boss over `boss`: the
/// first vertical rapid down (index, XY, end Z), the warnings, the post.
fn run(src: Src, boss: [f64; 4]) -> ((usize, Point2, f64), Vec<Warning>, Result<(), String>) {
    let world = new_world();
    let mut h = History::default();
    let [x0, x1, y0, y1] = boss;
    h.execute(&world, ImportModel::from_mesh("casting", casting(x0, x1, y0, y1)));
    let source = match src {
        Src::Part => {
            h.execute(&world, ImportModel::from_mesh("part", part()));
            SrcRef::Entity("part".into())
        }
        Src::Stock => SrcRef::Stock,
    };
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    // Stock only: the stock keeps its copy of the casting, and the casting
    // leaves the tree so the program verdict does not read it as a part.
    h.execute(&world, DeleteEntity::new(&world, "casting").expect("delete casting"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "Op1", TOOL, source, bore()).expect("create op"));
    h.execute(&world, GenerateOperation::new(&world, "Op1").expect("generate op"));
    let e = model::find_by_name(&world, "Op1").expect("op");
    let cache = model::toolpath_of(&world, &e).unwrap_or_else(|| panic!("{src:?}: no toolpath"));
    let moves = &cache.path.moves;
    let entry = moves
        .iter()
        .position(|m| m.kind == MoveKind::Rapid && m.from.x == m.to.x && m.from.y == m.to.y && m.to.z < m.from.z)
        .and_then(|i| moves.get(i).map(|m| (i, Point2::new(m.to.x, m.to.y), m.to.z)))
        .unwrap_or_else(|| panic!("{src:?}: no entry rapid"));
    // One file per run: the tests run in parallel and repeat a boss.
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("camcraft-1091-{src:?}-{n}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let posted = posting::post_setup(&world, &out).map(|_| ());
    let _ = std::fs::remove_file(&out);
    (entry, cache.warnings.clone(), posted)
}

/// The boss well clear of the hole: x 18..28 (the hole's rim is at 8).
const FAR: [f64; 4] = [18.0, 28.0, -5.0, 5.0];

fn convicted_and_blocked(src: Src, boss: [f64; 4], what: &str) {
    let ((at, _, z), warns, posted) = run(src, boss);
    assert!(z < BOSS, "{src:?} {what}: the fixture must dive under the boss's top, not to Z {z}");
    assert!(warns.contains(&Warning::RapidInStock { at }), "{src:?} {what}: the dive went unnamed: {warns:?}");
    match posted {
        Ok(()) => panic!("{src:?} {what}: a program diving into the boss posted"),
        Err(e) => assert!(
            e.contains("post blocked") && e.contains(&format!("rapid move {at} ")),
            "{src:?} {what}: refused, but not by the rapid law: {e}"
        ),
    }
}

#[test]
fn a_dive_over_the_low_part_of_a_stepped_stock_passes_and_posts() {
    // Solid to Z 20 over the mouth, the boss to Z 30 well away: the
    // entry rapid to Z 22 ends 2 mm over the stock really under it.
    for src in [Src::Part, Src::Stock] {
        let ((_, _, z), warns, posted) = run(src, FAR);
        assert!(z > MOUTH && z < BOSS, "{src:?}: the fixture must dive between the two tops, not to Z {z}");
        assert!(!warns.iter().any(|w| matches!(w, Warning::RapidInStock { .. })), "{src:?}: {warns:?}");
        assert!(posted.is_ok(), "{src:?}: {posted:?}");
    }
}

#[test]
fn the_same_dive_with_the_boss_under_it_is_convicted() {
    // The boss stands over the whole hole.
    for src in [Src::Part, Src::Stock] {
        convicted_and_blocked(src, [-12.0, 12.0, -12.0, 12.0], "boss over the hole");
    }
}

#[test]
fn a_boss_reaching_into_the_tool_footprint_is_convicted() {
    // The boss's edge stands 1.5 mm beside the dive's centre line, under
    // the tool's rim (radius 4) but not under its centre.
    for src in [Src::Part, Src::Stock] {
        let ((_, c, _), _, _) = run(src, FAR);
        let edge = c.x + R - 2.5;
        convicted_and_blocked(src, [edge, edge + 10.0, c.y - 5.0, c.y + 5.0], "boss under the rim");
    }
}
