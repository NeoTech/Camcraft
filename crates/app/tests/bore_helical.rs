//! #2830: the Bore operation's Helical strategy through the operation
//! layer. The issue's counterbore (Ø11 x 7 deep, a Ø6 flat end mill,
//! 1 mm per turn) generates as one helix and a floor circle with no
//! axial plunge into the solid block, also with a cutter that is not
//! centre-cutting. The strategy survives the project file, an older file
//! (no key) reads as the ring ladder, a ring ladder writes no new key,
//! and the stamp moves when the strategy changes.
//!
//! Provenance: our own tests over the operation layer; helical
//! interpolation of a counterbore with an end mill is standard practice
//! in public machining handbooks. No vendor material consulted.
//!
//! Gate map (#2830): covers app::operation (generate, the Bore arm,
//! params digest), app::project (params DTO), motion::machining2d
//! (bore, bore_helix), motion::plunge_law.

use flecs_ecs::prelude::World;

use camcraft_app::cl::{MoveKind, Toolpath};
use camcraft_app::commands::{AddTool, History, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::machining2d::BoreStrategy;
use camcraft_app::operation::{self, Height, OpParams, SrcRef};
use camcraft_app::project;
use camcraft_app::schema::{Location, Operation};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{Tool, catalog};
use camcraft_app::world::new_world;
use camcraft_motion::plunge_law::{Solid, first_solid_plunge};

const TOP: f64 = 20.0;
const FLOOR: f64 = 13.0;

/// A solid 60 x 60 x 20 block and a Ø6 end mill twice: "CC" as
/// catalogued, "NC" the same body not centre-cutting.
fn world() -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock"));
    let t = catalog().into_iter().find(|(n, _)| *n == "EM-0600-4FL").map(|(_, t)| t).expect("catalog entry");
    h.execute(&world, AddTool::new(&world, "CC", t.clone()).expect("tool"));
    h.execute(&world, AddTool::new(&world, "NC", Tool { center_cutting: false, ..t }).expect("tool"));
    world
}

fn bore(strategy: BoreStrategy) -> OpParams {
    OpParams::Bore {
        break_through: None,
        strategy,
        pass_start: camcraft_app::machining2d::PassStart::Auto,
        links: camcraft_app::links::Links::PLANAR,
        clearance_z: Height::fixed(30.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: None,
        stock_to_leave: 0.0,
        stepdown: 1.0,
        feed: 600.0,
        plunge_feed: 200.0,
        climb: true,
        tolerance: 0.01,
        holes: vec![Hole { xy: Point2::new(15.0, 15.0), radius: 5.5, entry_z: TOP, floor_z: FLOOR, through: false, stepped: false }],
    }
}

fn op(tool: &str, params: OpParams) -> Operation {
    Operation {
        tool: tool.into(),
        src: SrcRef::Stock,
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

fn generate(world: &World, tool: &str, params: OpParams) -> Result<Toolpath, String> {
    operation::generate(world, &op(tool, params), "B").map(|(p, _)| p).map_err(|e| e.to_string())
}

const SOLID: Solid = Solid { top: TOP, footprint: None };
const HELICAL: BoreStrategy = BoreStrategy::Helical { spring_pass: false };

#[test]
fn the_issue_counterbore_is_one_helix_and_never_plunges_into_the_block() {
    let world = world();
    for tool in ["CC", "NC"] {
        let path = match generate(&world, tool, bore(HELICAL)) {
            Ok(p) => p,
            Err(e) => panic!("{tool}: the helical counterbore generates: {e}"),
        };
        assert!(path.all_finite());
        let warns = operation::generate(&world, &op(tool, bore(HELICAL)), "B").map(|(_, w)| w).unwrap_or_default();
        assert!(warns.is_empty(), "{tool}: the verifier passes the helix: {warns:?}");
        assert_eq!(first_solid_plunge(&path, 3.0, &SOLID), None, "{tool}: an axial plunge into the block");
        assert!(path.moves.iter().all(|m| m.kind != MoveKind::Plunge), "{tool}: the helix posts no plunge");
        let arcs = path.moves.iter().filter(|m| m.arc.is_some() && m.to.z < m.from.z).count();
        // 9 mm (7 deep and the 2 mm plunge height) at 1 mm per turn, in
        // half turns.
        assert_eq!(arcs, 18, "{tool}: nine turns of helix in half-turn arcs");
        let low = path.moves.iter().filter(|m| m.kind != MoveKind::Rapid).map(|m| m.to.z).fold(f64::INFINITY, f64::min);
        assert!((low - FLOOR).abs() < 1e-6, "{tool}: the helix bottoms at the floor: {low}");
    }
    // The ring ladder with the centre-cutting tool still dives on the axis.
    let rings = generate(&world, "CC", bore(BoreStrategy::Rings)).expect("rings");
    assert!(first_solid_plunge(&rings, 3.0, &SOLID).is_some(), "the ladder's centre dive");
}

#[test]
fn a_hole_too_wide_for_one_helix_is_refused_through_the_operation() {
    let world = world();
    let mut p = bore(HELICAL);
    if let OpParams::Bore { holes, .. } = &mut p {
        holes[0].radius = 7.0;
    }
    let msg = generate(&world, "CC", p).err().unwrap_or_default();
    assert!(msg.contains("too wide for one helix") && msg.contains("12.000 mm"), "{msg}");
}

#[test]
fn the_strategy_rides_the_project_file_and_an_older_file_reads_rings() {
    let spring = BoreStrategy::Helical { spring_pass: true };
    let json = project::params_to_json(&bore(spring));
    assert_eq!(json["bore"]["helical"], true, "{json}");
    assert_eq!(json["bore"]["spring_pass"], true, "{json}");
    assert_eq!(project::params_from_json(&json), Ok(bore(spring)));

    // A ring ladder writes no new key: an older reader opens it as before.
    let rings = project::params_to_json(&bore(BoreStrategy::Rings));
    assert!(rings["bore"].get("helical").is_none() && rings["bore"].get("spring_pass").is_none(), "{rings}");
    // A file written before #2830 (no key) reads as the ring ladder.
    assert_eq!(project::params_from_json(&rings), Ok(bore(BoreStrategy::Rings)));
}

#[test]
fn the_stamp_moves_when_the_strategy_changes() {
    let world = world();
    let stamp = |s: BoreStrategy| operation::op_stamp(&world, &op("CC", bore(s)), "B");
    let (rings, helix, spring) = (stamp(BoreStrategy::Rings), stamp(HELICAL), stamp(BoreStrategy::Helical { spring_pass: true }));
    assert!(rings.is_some());
    assert_ne!(rings, helix, "rings to helix marks the op Stale");
    assert_ne!(helix, spring, "the spring pass marks the op Stale");
}
