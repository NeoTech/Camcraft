//! A from-model stock of previously cut material can already sit below the
//! part in places. With the stock marked raw stock only and those deficits
//! acknowledged for the session (`SetStockSourceOnly::with_inherited_deficits`),
//! the post reports cells the program leaves at their incoming heights as a
//! located note instead of gouges. A program that cuts into such a cell, or
//! below the part anywhere else, still blocks. Opening the project clears the
//! acknowledgement. A program of touch probes only, which plans nothing in the
//! stock sim, is judged and noted the same way.
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
        picks: Default::default(),
        holes: vec![Hole { xy, radius, entry_z: MOUTH, floor_z: floor, through: false, stepped: false }],
    }
}

fn op(world: &World, h: &mut History, name: &str, params: OpParams) {
    h.execute(world, CreateOperation::new(world, name, TOOL, SrcRef::Entity("part".into()), params).expect("create op"));
    h.execute(world, GenerateOperation::new(world, name).expect("generate op"));
}

/// The stepped casting as the stock's source and the part, with `tool` in the
/// library. `acknowledge` marks the casting raw stock only with or without the
/// acknowledgement; ops are generated after the mark, as a user would.
fn setup(acknowledge: bool, tool_name: &str) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", stepped_casting()));
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == tool_name).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, tool_name, tool).expect("tool"));
    let mark = SetStockSourceOnly::with_inherited_deficits(&world, true, acknowledge).expect("mark");
    h.execute(&world, mark);
    (world, h)
}

/// The setup and a legal bore of the blind hole: nothing in the program
/// reaches the step.
fn job(acknowledge: bool, extra: Option<OpParams>) -> World {
    let (world, mut h) = setup(acknowledge, TOOL);
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

/// The setup and one top-face probe touch on the part away from the step and
/// the hole: a program of touch probes only, which removes nothing.
fn probe_job(acknowledge: bool) -> World {
    let (world, mut h) = setup(acknowledge, "PRB-0400");
    add_probe(&world, &mut h);
    world
}

/// Two setups, each with its own copy of the stepped casting marked raw stock
/// only, `acknowledge` naming which of the two carry the acknowledgement, and
/// the program's only op a touch probe in the second.
fn two_setup_probe_job(acknowledge: [bool; 2]) -> World {
    use camcraft_app::commands::{ActivateSetup, CreateSetup};
    let (world, mut h) = setup(false, "PRB-0400");
    h.execute(&world, CreateSetup::new(&world, Some("Setup1"), None).expect("setup 1"));
    h.execute(&world, CreateSetup::new(&world, Some("Setup2"), Some("Setup1")).expect("setup 2"));
    for (name, ack) in [("Setup1", acknowledge[0]), ("Setup2", acknowledge[1])] {
        h.execute(&world, ActivateSetup::new(&world, Some(name)).expect("activate"));
        h.execute(&world, SetStockSourceOnly::with_inherited_deficits(&world, true, ack).expect("mark"));
    }
    add_probe(&world, &mut h);
    world
}

/// One top-face probe touch, in the active setup.
fn add_probe(world: &World, h: &mut History) {
    add_probe_named(world, h, "Probe1");
}

fn add_probe_named(world: &World, h: &mut History, name: &str) {
    use camcraft_app::operation::HeightReference;
    use camcraft_app::probing::{ProbeDir, ProbeMode};
    let p = OpParams::Probe {
        clearance_z: Height::of(HeightReference::TopOfModel, 25.0),
        mode: ProbeMode::Face,
        dir: ProbeDir::ZNeg,
        dir_y: ProbeDir::YPos,
        target: [0.0, -15.0, MOUTH],
        radius: 10.0,
        approach: 5.0,
        overtravel: 1.0,
        backoff: 1.0,
        fast_feed: 300.0,
        slow_feed: 30.0,
        depth: 3.0,
        inset: 5.0,
        set_wcs: Some(0),
        tolerance: 0.5,
    };
    let src = SrcRef::Entity("part".into());
    h.execute(world, CreateOperation::new(world, name, "PRB-0400", src, p).expect("probe op"));
    h.execute(world, GenerateOperation::new(world, name).expect("generate probe"));
}

#[test]
fn an_idle_setups_acknowledgement_never_relieves_a_probe_program() {
    // The probe machines against the second setup's stock; the first
    // setup's acknowledgement says nothing about it.
    let world = two_setup_probe_job([true, false]);
    let report = posting::verification_report(&world);
    assert!(
        !report.not_judged.clone().unwrap_or_default().contains("incoming stock"),
        "the idle setup's acknowledgement relieved a probe in another: {report:?}"
    );
    assert!(report.problems.iter().any(|p| p.starts_with("gouge")), "the deficits no longer block: {report:?}");
    post(&world, "probe-idle-ack").expect_err("a probe program posted on another setup's acknowledgement");
}

#[test]
fn a_probe_program_uses_its_own_setups_acknowledgement() {
    // A probe-only program is judged on the uncut stock of the setup it runs
    // in, never on the first setup's; under that stock's acknowledgement every
    // deficit there is inherited.
    let world = two_setup_probe_job([false, true]);
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "the executing setup's acknowledgement did not relieve its probe: {:?}", report.problems);
    let note = report.not_judged.expect("the acknowledged deficits are said, never silent");
    assert!(note.starts_with("note: incoming stock 'casting' already has "), "{note}");
    assert!(note.contains("worst 5.00 mm at ("), "{note}");
    let (_, y, z) = located(&note);
    assert!((STEP_Y..=20.0).contains(&y) && (z - STEP_TOP).abs() < 1e-6, "the worst cell is not on the step: {note}");
    post(&world, "probe-own-ack").expect("a probe program refused under its own setup's acknowledgement");
}

/// Two setups made fresh, setup one holding `one` (`None`: no stock at all)
/// and setup two `two`; a stock from the casting is marked raw stock only,
/// acknowledged per `acknowledge`. The program's only op is a touch probe in
/// setup two, so setup one is idle.
fn probe_in_setup_two(one: Option<StockShape>, two: StockShape, acknowledge: [bool; 2]) -> World {
    use camcraft_app::commands::{ActivateSetup, CreateSetup};
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("casting", stepped_casting()));
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let tool = catalog().into_iter().find(|(n, _)| *n == "PRB-0400").map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, "PRB-0400", tool).expect("tool"));
    for (name, stock, ack) in [("Setup1", one, acknowledge[0]), ("Setup2", Some(two), acknowledge[1])] {
        h.execute(&world, CreateSetup::new(&world, Some(name), None).expect("setup"));
        h.execute(&world, ActivateSetup::new(&world, Some(name)).expect("activate"));
        let Some(shape) = stock else { continue };
        let cast = matches!(shape, StockShape::FromModel { .. });
        h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
        if cast {
            h.execute(&world, SetStockSourceOnly::with_inherited_deficits(&world, true, ack).expect("mark"));
        }
    }
    add_probe(&world, &mut h);
    world
}

fn casting() -> StockShape {
    StockShape::FromModel { source: "casting".into() }
}

/// A box over the whole part: no deficit anywhere.
fn clean_box() -> StockShape {
    StockShape::Box { dx: 60.0, dy: 60.0, dz: 25.0 }
}

#[test]
fn a_probe_program_on_clean_stock_posts_beside_an_idle_deficient_setup() {
    // The probe runs on setup two's clean box; setup one's deficient casting
    // is never machined, so it neither blocks nor earns a note.
    let world = probe_in_setup_two(Some(casting()), clean_box(), [false, false]);
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "an idle setup's stock blocked a probe on clean stock: {:?}", report.problems);
    assert!(
        !report.not_judged.clone().unwrap_or_default().contains("incoming stock"),
        "a clean stock earned a deficit note: {report:?}"
    );
    post(&world, "probe-clean-own").expect("a probe program on clean stock was refused");
}

#[test]
fn an_idle_setup_with_no_stock_leaves_the_probe_judged_on_its_own() {
    // Setup one has no stock at all; the probe's setup two holds the
    // acknowledged casting, so the post carries the located note.
    let world = probe_in_setup_two(None, casting(), [false, true]);
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "the acknowledged casting still blocks: {:?}", report.problems);
    let note = report.not_judged.expect("the acknowledged deficits are said, never silent");
    assert!(note.starts_with("note: incoming stock 'casting' already has "), "{note}");
    assert!(note.contains("worst 5.00 mm at ("), "{note}");
    let (_, y, z) = located(&note);
    assert!((STEP_Y..=20.0).contains(&y) && (z - STEP_TOP).abs() < 1e-6, "the worst cell is not on the step: {note}");
    let posted = post(&world, "probe-idle-none").expect("an acknowledged probe program was refused");
    assert_eq!(posted.not_judged.as_deref(), Some(note.as_str()), "the post reports the same note");
}

#[test]
fn an_idle_clean_setup_never_hides_the_probes_own_deficits() {
    // Without the acknowledgement, setup two's deficient casting blocks as a
    // gouge however clean the idle setup one's box is.
    let world = probe_in_setup_two(Some(clean_box()), casting(), [false, false]);
    let why = post(&world, "probe-idle-box").expect_err("a probe on deficient stock posted beside a clean idle setup");
    assert!(why.contains("gouge"), "refused, but not as a gouge: {why}");
}

#[test]
fn a_probe_only_program_on_the_step_blocks_without_the_acknowledgement() {
    let world = probe_job(false);
    let why = post(&world, "probe-plain").expect_err("stock under the part posted without the acknowledgement");
    assert!(why.contains("gouge"), "refused, but not as a gouge: {why}");
}

#[test]
fn acknowledged_a_probe_only_program_posts_with_the_located_note() {
    // A probe plans nothing in the stock sim (#108), so the program's setup
    // and lattice come from the walk's start, not from a planned op.
    let world = probe_job(true);
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "the untouched step still blocks a probe-only program: {:?}", report.problems);
    let note = report.not_judged.expect("the acknowledged deficits are said, never silent");
    assert!(note.starts_with("note: incoming stock 'casting' already has "), "{note}");
    assert!(note.contains("worst 5.00 mm at ("), "{note}");
    let (_, y, z) = located(&note);
    assert!((STEP_Y..=20.0).contains(&y) && (z - STEP_TOP).abs() < 1e-6, "the worst cell is not on the step: {note}");
    post(&world, "probe-acknowledged").expect("an acknowledged probe-only program was refused");
}

/// The stepped casting and the part on the plasma table, acknowledged, and a
/// torch profile of the part: the torch cuts through along a kerf the column
/// sim has no atom for, so it plans nothing there.
fn torch_job() -> World {
    torch_job_in(false, true)
}

/// `torch_job`, with `two_setups` a second setup holding its own copy of the
/// casting and a torch profile in each, and `acknowledge` the acknowledgement.
fn torch_job_in(two_setups: bool, acknowledge: bool) -> World {
    use camcraft_app::commands::{ActivateSetup, CreateSetup};
    use camcraft_app::operation::{PlasmaComp, PlasmaOp};
    const TORCH: &str = "PT-KERF-150";
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, camcraft_app::commands::LoadMachine::builtin(&world, "Tormach 1300PL").expect("plasma table"));
    let plasma = camcraft_app::schema::PostRef::Named("Tormach PathPilot Plasma".into());
    h.execute(&world, camcraft_app::commands::SelectPost::new(&world, plasma).expect("plasma post"));
    h.execute(&world, ImportModel::from_mesh("casting", stepped_casting()));
    h.execute(&world, ImportModel::from_mesh("part", part()));
    let shape = StockShape::FromModel { source: "casting".into() };
    h.execute(&world, SetStock::new(&world, shape, Location::default()).expect("stock"));
    h.execute(&world, AddTool::new(&world, TORCH, camcraft_app::tools::torch_tool(1.5)).expect("torch"));
    h.execute(&world, SetStockSourceOnly::with_inherited_deficits(&world, true, acknowledge).expect("mark"));
    let p = OpParams::PlasmaProfile(PlasmaOp {
        clearance_z: Height::fixed(40.0),
        top_z: Height::fixed(MOUTH),
        bottom_z: Height::fixed(0.0),
        kerf: None,
        comp: PlasmaComp::Computer,
        reversed: false,
        lead_in: 4.0,
        lead_out: 3.0,
        small_hole_dia: 32.0,
        small_hole_feed: 0.65,
        overburn: 1.0,
        feed: Some(2500.0),
        pierce_height: None,
        cut_height: None,
        pierce_delay: None,
        picks: Default::default(),
    });
    let setups: &[&str] = if two_setups { &["Setup1", "Setup2"] } else { &[""] };
    if two_setups {
        h.execute(&world, CreateSetup::new(&world, Some("Setup1"), None).expect("setup 1"));
        h.execute(&world, CreateSetup::new(&world, Some("Setup2"), Some("Setup1")).expect("setup 2"));
        // The plasma post cannot stop for a re-clamp: run straight on.
        let between = camcraft_app::schema::BetweenSetups::Continue;
        let options = camcraft_app::schema::PostOptions { between_setups: between, ..posting::post_options(&world) };
        h.execute(&world, camcraft_app::commands::SetPostOptions::new(&world, options).expect("between setups"));
    }
    for (k, setup) in setups.iter().enumerate() {
        if two_setups {
            h.execute(&world, ActivateSetup::new(&world, Some(*setup)).expect("activate"));
            h.execute(&world, SetStockSourceOnly::with_inherited_deficits(&world, true, acknowledge).expect("mark"));
        }
        let name = format!("Plasma{}", k + 1);
        let src = SrcRef::Entity("part".into());
        h.execute(&world, CreateOperation::new(&world, &name, TORCH, src, p.clone()).expect("torch op"));
        h.execute(&world, GenerateOperation::new(&world, &name).expect("generate torch"));
    }
    world
}

#[test]
fn acknowledged_a_torch_program_keeps_the_full_gate() {
    // The receipt's "unchanged" reads only the planned cuts; a torch's cut
    // is not among them, so its program never gets the note.
    let world = torch_job();
    let report = posting::verification_report(&world);
    assert!(
        !report.not_judged.clone().unwrap_or_default().contains("incoming stock"),
        "a torch program was told its deficits are unchanged: {report:?}"
    );
    assert!(report.problems.iter().any(|p| p.starts_with("gouge")), "the inherited deficits no longer block: {report:?}");
}

/// Post to a fresh path; the refusal, and that no NC file was written.
fn refused_with_no_file(world: &World, tag: &str) -> String {
    let out = std::env::temp_dir().join(format!("camcraft-incoming-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let why = posting::post_setup(world, &out).expect_err("the program posted");
    assert!(!out.exists(), "a refused post wrote {}", out.display());
    why
}

#[test]
fn a_probe_program_across_setups_the_sim_cannot_cross_refuses() {
    // Probes in both setups, each on its own unacknowledged copy of the
    // deficient casting. The walk cannot clip to the second setup's model
    // stock, and nothing else judges the stock under probes.
    use camcraft_app::commands::ActivateSetup;
    let world = two_setup_probe_job([false, false]);
    let mut h = History::default();
    h.execute(&world, ActivateSetup::new(&world, Some("Setup1")).expect("activate"));
    add_probe_named(&world, &mut h, "Probe2");
    let why = refused_with_no_file(&world, "probe-both-setups");
    assert!(why.contains(posting::NOT_JUDGED) && why.contains("do not verify the stock under probes or torches"), "refused, but not for the stock: {why}");
}

#[test]
fn a_torch_program_across_setups_the_sim_cannot_cross_refuses() {
    for acknowledge in [false, true] {
        let world = torch_job_in(true, acknowledge);
        let why = refused_with_no_file(&world, &format!("torch-both-setups-{acknowledge}"));
        assert!(
            why.contains(posting::NOT_JUDGED) && why.contains("do not verify the stock under probes or torches"),
            "refused, but not for the stock (acknowledged {acknowledge}): {why}"
        );
    }
}

#[test]
fn acknowledged_an_undercut_program_keeps_the_full_gate() {
    // A T-slot pass in the casting beside the part, away from the step. An
    // undercut lowers no column top in the column sim (#1444), so "unchanged"
    // cannot be read from the tops: the deficits block as gouges, no note.
    use camcraft_app::operation::SlotUndercutOp;
    use camcraft_app::tools::ToolKind;
    let (world, mut h) = setup(true, TOOL);
    op(&world, &mut h, "Op1", bore(Point2::new(0.0, 0.0), 8.0, FLOOR));
    let tslot = camcraft_app::undercut::cutter(ToolKind::TSlot, 8.0, 3.0, 4.0, 20.0, 0.0);
    h.execute(&world, AddTool::new(&world, "TS-08", tslot).expect("t-slot cutter"));
    let params = OpParams::SlotUndercut(SlotUndercutOp {
        a: Point2::new(25.0, -25.0),
        b: Point2::new(25.0, -5.0),
        bulge: 0.0,
        width: 8.0,
        run_in: 12.0,
        neck_slot: 5.0,
        clearance_z: Height::fixed(40.0),
        retract_z: Some(Height::fixed(30.0)),
        top_z: Height::fixed(MOUTH),
        bottom_z: Height::fixed(6.0),
        stepover: 3.0,
        feed: 120.0,
        plunge_feed: 60.0,
        climb: true,
        picks: Default::default(),
    });
    let src = SrcRef::Entity("part".into());
    h.execute(&world, CreateOperation::new(&world, "Undercut1", "TS-08", src, params).expect("create undercut"));
    h.execute(&world, GenerateOperation::new(&world, "Undercut1").expect("generate undercut"));
    let report = posting::verification_report(&world);
    assert!(
        !report.not_judged.clone().unwrap_or_default().contains("incoming stock"),
        "an undercut program was told its deficits are unchanged: {report:?}"
    );
    assert!(report.problems.iter().any(|p| p.starts_with("gouge")), "the inherited deficits no longer block: {report:?}");
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
