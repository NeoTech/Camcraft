//! #2443 (review): Auto chooses the drill cycle by the depth in material -
//! from the higher of the hole's mouth and the material over it (the part,
//! the raw stock, the stock the program ahead leaves), never above the R
//! plane. The air between the stock and the R plane is not depth; uncut
//! stock over a mouth is.
//!
//! The case is the issue's: a 10 mm plate in a 17 mm stock, a 3.5 mm hole
//! under the plate top, a 6 mm drill, the R plane 2 mm over the stock.
//!
//! Gate map (#2443): covers camcraft_app::operation (generate, drill_prior),
//! camcraft_motion::drilling (cycle), camcraft_app::setup_sheet.
//!
//! Provenance: our own fixtures (common::solid_part); the drilling depth
//! rules of machining handbooks (peck beyond about one diameter). No vendor
//! material.
//!
//! A submodule of the `drill_prior` suite (#3187: tests join a directory
//! suite): it reads the stock that the program ahead leaves.

use crate::common::solid_part::{circle, plate, rect};
use flecs_ecs::prelude::World;

use camcraft_app::cl::CycleKind;
use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::drilling::{BreakThroughFrom, CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::Mesh;
use camcraft_app::model;
use camcraft_app::operation::{Height, HeightReference, OpParams, OpStatus, SrcRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;

/// A 100 x 60 plate, z 0..10, with a 6 mm hole at the origin, in a
/// 100 x 60 x 17 stock: 7 mm of uncut stock over the plate top.
fn world() -> (World, History) {
    let mut f = Vec::new();
    plate(&rect(-50.0, -30.0, 50.0, 30.0), Some((&circle((0.0, 0.0), 3.0, 48), 0.0)), 0.0, 10.0, &mut f);
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("plate", Mesh::from_facets(f)));
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 100.0, dy: 60.0, dz: 17.0 }, Default::default()).expect("stock"),
    );
    for name in ["DR-0600", "EM-1000-4FL"] {
        let t = catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog");
        h.execute(&world, AddTool::new(&world, name, t).expect("tool"));
    }
    (world, h)
}

/// The issue's hole: 3.5 mm deep under the plate top.
fn drill() -> OpParams {
    OpParams::Drill {
        clearance_z: Height::of(HeightReference::TopOfStock, 15.0),
        retract_z: None,
        top_z: Height::of(HeightReference::TopOfStock, 2.0),
        bottom_z: None,
        cycle: CycleChoice::Auto,
        peck: 0.0,
        dwell: 0.0,
        feed: 300.0,
        plunge_feed: 100.0,
        holes: vec![Hole { xy: Point2::new(0.0, 0.0), radius: 3.0, entry_z: 10.0, floor_z: 6.5, through: false, stepped: false }],
        cross: Vec::new(),
        break_through: None,
        break_through_from: BreakThroughFrom::FullDiameter,
        picks: Default::default(),
    }
}

fn face_to_the_plate_top(world: &World, h: &mut History) {
    let face = OpParams::Face {
        holes: camcraft_app::machining2d::FaceHoles::CutAir,
        tilt: None,
        blend_turns: false,
        blend_radius: None,
        stepover: 6.0,
        clearance_z: Height::of(HeightReference::TopOfStock, 15.0),
        cut_z: Height::of(HeightReference::TopOfModel, 0.0),
        top_z: Some(Height::of(HeightReference::TopOfStock, 0.0)),
        stepdown: Some(4.0),
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        picks: Default::default(),
    };
    h.execute(world, CreateOperation::new(world, "F1", "EM-1000-4FL", SrcRef::Stock, face).expect("create face"));
    h.execute(world, GenerateOperation::new(world, "F1").expect("generate face"));
}

fn drilled_kind(world: &World, h: &mut History) -> CycleKind {
    h.execute(world, CreateOperation::new(world, "D1", "DR-0600", SrcRef::Entity("plate".into()), drill()).expect("create"));
    h.execute(world, GenerateOperation::new(world, "D1").expect("generate"));
    let e = model::find_by_name(world, "D1").expect("op");
    assert_eq!(camcraft_app::operation::status_of(world, &e), OpStatus::Ok);
    let tp = model::toolpath_of(world, &e).expect("cached path");
    assert_eq!(tp.path.cycles.len(), 1, "one hole");
    tp.path.cycles[0].kind
}

/// Nothing ahead cuts the 7 mm of stock over the plate: the drill goes
/// through 10.5 mm of material, more than one diameter, so Auto pecks.
/// (The mouth alone would read 3.5 mm and drill in one stroke.)
#[test]
fn uncut_stock_over_the_mouth_is_depth() {
    let (world, mut h) = world();
    assert_eq!(drilled_kind(&world, &mut h), CycleKind::Peck);
}

/// A Face ahead cuts the stock down to the plate top: only the 3.5 mm
/// hole is material, so Auto drills it in one stroke (G81) - the issue's
/// case, where the R plane over the stock is air.
#[test]
fn stock_the_program_ahead_cut_away_is_not_depth() {
    let (world, mut h) = world();
    face_to_the_plate_top(&world, &mut h);
    assert_eq!(drilled_kind(&world, &mut h), CycleKind::Drill);
    // The setup sheet names the cycle the generator chose.
    let sheet = camcraft_app::setup_sheet::render(&world, None).expect("setup sheet");
    assert!(sheet.lines().any(|l| l.contains("hole  1") && l.contains("drill")), "{sheet}");
}
