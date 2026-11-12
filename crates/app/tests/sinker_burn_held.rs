//! #541 review (PR #1131): the Sinker Burn through the stock the program
//! walk holds (`Held::simulate_box`) and through the post-time family
//! door (`posting_family::family_at_post`). A carry of the stock onto a
//! sinker is refused by name before any burn meets a carried grid (the
//! carried column arm is pinned on the holder, `stock_sim_held_tests`);
//! on the field engine, and in the field view, a burn is refused by
//! name, never cut as a disc nor skipped; at post a Sinker Burn is
//! admitted on its sinker, a milling op remounted onto the sinker is
//! refused by the family door's text, and a burn remounted onto a mill
//! never posts.
//!
//! Gate map (#541 review): covers app::stock_sim_held (Held::simulate_box),
//! app::stock_sim_carry (carry_turn, the walk's electrode arm),
//! app::stock_view (display_inputs), app::posting (op_streams),
//! app::posting_family.
//! Provenance: our own SYNTHETIC fixtures (`edm/sinker_fixture.rs`); no
//! vendor material consulted. Lane shape (M4.4): all tests run in seconds.

#[path = "edm/sinker_fixture.rs"]
mod sinker_fixture;

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, LoadMachine, SetSetupMachine,
    SetSetupRelative, SetStock,
};
use camcraft_app::machine::parse_mch;
use camcraft_app::operation::{Height, OpParams, SinkerDatum, SrcRef};
use camcraft_app::posting::op_streams;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_sim::{
    incoming_body, simulate_document, with_carry_engine, Backend, CARRY_MACHINE_REFUSAL, ELECTRODE_FIELD_REFUSAL,
    FIELD_BAND,
};
use camcraft_app::stock_view::field_view_with;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

use sinker_fixture::{burn, sinker_doc, sinker_machine, ELECTRODE, MILL_TOOL};

fn add_burn(world: &World, h: &mut History, name: &str, depth: f64) {
    let op = CreateOperation::new(world, name, ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, depth, 14.0));
    h.execute(world, op.expect("a sinker burn on a sinker"));
    h.execute(world, GenerateOperation::new(world, name).expect("generates"));
}

/// The sinker document with Setup1 materialized and a 3 mm Burn0 in it.
fn one_setup() -> (World, History) {
    let (world, mut h) = sinker_doc(6.0);
    h.execute(&world, CreateSetup::new(&world, None, None).expect("materialize Setup1"));
    add_burn(&world, &mut h, "Burn0", 3.0);
    (world, h)
}

const FIELD: Backend = Backend::Field { band: FIELD_BAND };

#[test]
fn the_field_engine_refuses_a_burn_by_name() {
    // One setup: the field arm holds carried stock from the start.
    let (world, mut h) = sinker_doc(6.0);
    add_burn(&world, &mut h, "Burn1", 6.0);
    simulate_document(&world).expect("the column engine judges it");
    let err = with_carry_engine(FIELD, || simulate_document(&world)).expect_err("never a silent no-op on the field");
    assert_eq!(err, format!("Burn1: {ELECTRODE_FIELD_REFUSAL}"));
}

#[test]
fn the_field_view_refuses_a_burn_by_name() {
    let (world, mut h) = sinker_doc(6.0);
    add_burn(&world, &mut h, "Burn1", 6.0);
    let Err(err) = field_view_with(&world, 1.0e6) else { panic!("the field view drew a sinker burn") };
    assert_eq!(err, format!("Burn1: {ELECTRODE_FIELD_REFUSAL}"));
}

#[test]
fn a_carry_onto_the_sinker_is_refused_by_name_before_any_burn() {
    // Burn1 would start from the stock Setup1 left, carried by the link:
    // the carry onto a machine that is not a mill refuses by name, so no
    // burn ever meets a carried grid in the walk.
    let (world, mut h) = one_setup();
    h.execute(&world, CreateSetup::new(&world, None, Some("Setup1")).expect("Setup2"));
    let link = SetSetupRelative::link(&world, "Setup2", "Setup1", [0.0, 0.0, 0.0]).expect("link");
    h.execute(&world, link);
    let op = CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, 6.0, 14.0));
    h.execute(&world, op.expect("create"));
    let err = GenerateOperation::new(&world, "Burn1").err().expect("refused at the carry").to_string();
    assert!(err.contains("from setup 'Setup1' into setup 'Setup2'") && err.contains(CARRY_MACHINE_REFUSAL), "{err}");
    assert!(incoming_body(&world, "Setup2").is_err(), "no carried stock is read either");
}

#[test]
fn the_post_admits_a_burn_on_its_sinker() {
    let (world, _h) = one_setup();
    let streams = op_streams(&world).expect("a sinker burn posts on its sinker");
    assert_eq!(streams.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["Burn0"]);
}

#[test]
fn the_post_refuses_a_milling_op_remounted_onto_the_sinker_by_name() {
    // Face1 made and generated on the document's default (a mill), then
    // the sinker mounted under it: its inputs did not move, so the family
    // door at post is what refuses it.
    let world = new_world();
    let mut h = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    h.execute(&world, stock.expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == MILL_TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, MILL_TOOL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "Face1", MILL_TOOL, SrcRef::Stock, face()).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("generates"));
    op_streams(&world).expect("a face posts on a mill");

    let m = parse_mch(&sinker_machine()).expect("sinker parses");
    h.execute(&world, LoadMachine::new(&world, "sinker.mch".into(), Some(m)).expect("mount"));
    let err = op_streams(&world).expect_err("a face never posts on a sinker");
    assert_eq!(
        err,
        "Face1: a Face operation cannot run on its setup's sinker EDM machine - mount a machine that runs it, or suppress the operation"
    );
    // Undo puts the mill back: the post admits it again.
    assert!(h.undo(&world));
    op_streams(&world).expect("admitted again after undo");
}

#[test]
fn a_burn_remounted_onto_a_mill_never_posts() {
    let (world, mut h) = one_setup();
    h.execute(&world, SetSetupMachine::builtin(&world, "Setup1", "Tormach 770M").expect("remount Setup1 on a mill"));
    // The condition's gap is gone with the sinker, so the burn reads
    // Stale and is refused by name before the family door; regenerating
    // it on the mill is refused too.
    let err = op_streams(&world).expect_err("a sinker burn never posts on a mill");
    assert!(err.starts_with("Burn0: "), "{err}");
    assert!(GenerateOperation::new(&world, "Burn0").is_err(), "a burn never regenerates on a mill");
    assert!(h.undo(&world));
    op_streams(&world).expect("admitted again after undo");
}

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        stepover: 3.0,
        clearance_z: Height::fixed(30.0),
        cut_z: Height::fixed(19.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    }
}
