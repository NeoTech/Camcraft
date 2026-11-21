//! #2898: a Circular with several radial passes (R10 with a Ø6 cutter
//! and a 2 mm stepover: four rings) linked each ring's floor to the next
//! ring with one diagonal rapid through the core it had still to cut.
//! Generate convicted the rapids (`rapid move N travels through stock`)
//! and the post was held, so no program passed. These pin the issue's
//! job through generate, verify and post: no rapid is convicted and the
//! program posts, for the plunge and the ramp entry. The other half of
//! "Done" - a spliced unsafe link is still convicted - is pinned in
//! `crates/motion/tests/circular_ring_links.rs`. The helix entry still
//! meets the crossing law between rings (its own follow-up issue).
//!
//! Provenance: our own fixture (a 40 x 40 x 20 block with a R10.2
//! through hole, the issue's tool, steps and heights), specced from the
//! Prime Directive; no vendor material.
//!
//! Gate map: covers app::operation::generate (Circular arm, verify
//! wiring), app::posting (the gate).

use flecs_ecs::prelude::World;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machining3d::Entry3D;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::world::new_world;

const TOOL: &str = "EM-0600-4FL";
const N: usize = 128;
/// The modeled hole's radius: the issue's envelope.
const HOLE: f64 = 10.2;
const TOP: f64 = 20.0;

/// A 40 x 40 x [`TOP`] block about the origin with one round through
/// hole of radius [`HOLE`] (128 chords), outward-wound.
fn holed_block() -> Mesh {
    let ang = |i: usize| std::f64::consts::TAU * (i % N) as f64 / N as f64;
    let rim = |i: usize, z: f64| Point3::new(HOLE * ang(i).cos(), HOLE * ang(i).sin(), z);
    let outer = |i: usize, z: f64| {
        let (x, y) = (ang(i).cos(), ang(i).sin());
        let s = x.abs().max(y.abs());
        Point3::new(x * 20.0 / s, y * 20.0 / s, z)
    };
    let mut f = Vec::new();
    let mut quad = |a: Point3, b: Point3, c: Point3, d: Point3| {
        f.push(Triangle::new(a, b, c));
        f.push(Triangle::new(a, c, d));
    };
    for i in 0..N {
        let j = i + 1;
        quad(rim(j, TOP), rim(i, TOP), outer(i, TOP), outer(j, TOP));
        quad(rim(i, TOP), rim(j, TOP), rim(j, 0.0), rim(i, 0.0));
        quad(outer(j, TOP), outer(i, TOP), outer(i, 0.0), outer(j, 0.0));
        quad(rim(i, 0.0), rim(j, 0.0), outer(j, 0.0), outer(i, 0.0));
    }
    Mesh::from_facets(f)
}

/// The block as the part, a full 40 x 40 x 20 box as the stock: the
/// hole's core is uncut stock when the Circular starts.
fn job() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let mut tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    // The issue's tool: 25 mm flutes, so the 8 mm shank stays over the
    // 20 mm cut (the catalog's 15 mm flutes are a real shank hit here).
    tool.flute_len = 25.0;
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", holed_block()));
    let stock = StockShape::Box { dx: 40.0, dy: 40.0, dz: TOP };
    h.execute(&world, SetStock::new(&world, stock, Location::default()).expect("stock"));
    (world, h)
}

/// The issue's Circular: R10, stepover 2, stepdown 2, Z20 to Z0,
/// clearance Z30, nothing left on the wall.
fn circular(entry: Entry3D) -> OpParams {
    OpParams::Circular {
        links: camcraft_app::links::Links::PLANAR,
        center: Point2::new(0.0, 0.0),
        radius: 10.0,
        vault: false,
        stepover: 2.0,
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(30.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: Height::fixed(0.0),
        stepdown: 2.0,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        entry,
        helix_radius: 1.0,
        tilt: None,
    }
}

fn generated(world: &World, h: &mut History, entry: Entry3D) -> Vec<Warning> {
    h.execute(world, CreateOperation::new(world, "C1", TOOL, SrcRef::Entity("part".into()), circular(entry)).expect("create"));
    h.execute(world, GenerateOperation::new(world, "C1").expect("generate"));
    let e = model::find_by_name(world, "C1").expect("op");
    let tp = model::toolpath_of(world, &e).expect("cached path");
    assert!(!tp.path.moves.is_empty(), "the rings were generated");
    tp.warnings.clone()
}

#[test]
fn a_multi_ring_circular_links_its_rings_over_the_stock_and_posts() {
    for entry in [Entry3D::Plunge, Entry3D::Ramp] {
        let (world, mut h) = job();
        let warns = generated(&world, &mut h, entry);
        assert!(warns.is_empty(), "{entry:?}: verification must find nothing: {warns:?}");
        let posted = posting::post_setup_unwritten(&world);
        assert!(posted.is_ok(), "{entry:?}: the program must post: {:?}", posted.err());
    }
}
