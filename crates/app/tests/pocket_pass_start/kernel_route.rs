//! #3752: the 2D Pocket through the real generate path with the planner
//! engine given (`GenerateOperation::new_on`, `operation::generate_on`;
//! #4055: a test passes the engine, it never sets the process setting).
//! Under "new" the path kernel plans the pocket: it generates, the
//! verifier finds nothing that blocks the post, and it posts on a shipped
//! machine post. Under the default engine the op is the old planner's,
//! unchanged. The options that the first recipe does not plan are refused
//! by name, and each other phase-1 kind is still refused under "new".

use super::{EM8, PassStart, PocketStrategy, R8, fig8_world, horizontal_face, model, sel};
use crate::motion_common::law_measure::{EntryKind as MeasuredEntry, Readings, Spec, measure};
use camcraft_app::geo2d::{Loop, Point2, Region};
use crate::area_support::*;
use crate::pocket_support::{block_with_pocket, rounded_rect};
use camcraft_app::commands::{CreateOperation, GenerateOperation, ImportModel};
use camcraft_app::machining2d::{KERNEL_POCKET_NOTE, RestParams, RestSource};
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{HeightReference, KERNEL_KINDS, OpType, PHASE1_KINDS, check_engine, generate_on};
use camcraft_app::pathkernel::PlannerEngine;
use camcraft_app::pathkernel::entry::NO_SAFE_ENTRY;
use camcraft_app::posting;

/// A 60 x 40 x 20 block with a 20 x 20 pocket 10 mm deep, EM-0800-4FL.
fn block_world() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", block_with_pocket(30.0, 20.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0)));
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 40.0, dz: 20.0 }, Default::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == EM8).map(|(_, t)| t).expect("catalog tool");
    h.execute(&world, AddTool::new(&world, EM8, tool).expect("tool"));
    (world, h)
}

/// A Waterline pocket on the floor of the block pocket: two levels of 5 mm,
/// the entry `entry`.
fn block_op(world: &World, h: &mut History, name: &str, entry: Entry3D, strategy: PocketStrategy) {
    let src = sel("part", horizontal_face(world, "part", 10.0));
    let mut p = pocket(20.0, 10.0);
    if let OpParams::Pocket { clearance_z, top_z, bottom_z, stepdown, stepover, strategy: s, entry: en, helix_radius, .. } = &mut p {
        *clearance_z = Height { reference: HeightReference::TopOfStock, offset: 5.0 };
        *top_z = Height { reference: HeightReference::TopOfStock, offset: 0.0 };
        *bottom_z = Height { reference: HeightReference::BottomOfSelection, offset: 0.0 };
        *stepdown = 5.0;
        *stepover = 3.0;
        *s = strategy;
        *en = entry;
        *helix_radius = 2.0;
    }
    h.execute(world, CreateOperation::new(world, name, EM8, src, p).expect("create"));
}

fn op_of(world: &World, name: &str) -> Operation {
    let e = model::find_by_name(world, name).expect("op");
    model::operation_of(world, &e).expect("operation")
}

#[test]
fn kernel_route_pocket_generates_verifies_and_posts() {
    let (world, mut h) = block_world();
    block_op(&world, &mut h, "K1", Entry3D::Helix, PocketStrategy::Waterline);
    let (path, warns) = generate_on(&world, &op_of(&world, "K1"), "K1", PlannerEngine::New).expect("the kernel plans the pocket");
    eprintln!("kernel: {} moves, notes {:?}, warnings {warns:?}", path.moves.len(), path.notes);
    assert!(path.notes.iter().any(|n| n == KERNEL_POCKET_NOTE), "the status line names the kernel");
    let blocking: Vec<_> = warns.iter().filter(|w| w.blocks_post()).collect();
    assert!(blocking.is_empty(), "the verifier blocks the kernel pocket: {blocking:?}");
    h.execute(&world, GenerateOperation::new_on(&world, "K1", PlannerEngine::New).expect("generate on the kernel"));
    let dir = std::env::temp_dir().join(format!("camcraft-3752-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let cps = posting::builtin_post_text("Fanuc").expect("built-in post");
    let posted = posting::post_script(&world, &dir.join("prog.nc"), cps).map(|p| p.program);
    let _ = std::fs::remove_dir_all(&dir);
    let nc = posted.expect("the kernel pocket posts");
    assert!(nc.contains("G1") || nc.contains("G01"), "the program cuts: {nc}");
}

#[test]
fn kernel_route_default_engine_keeps_the_old_pocket() {
    let (world, mut h) = block_world();
    block_op(&world, &mut h, "K1", Entry3D::Helix, PocketStrategy::Waterline);
    let op = op_of(&world, "K1");
    let (old, _) = generate_on(&world, &op, "K1", PlannerEngine::Old).expect("the old planner");
    let (default, _) = generate_on(&world, &op, "K1", PlannerEngine::default()).expect("the default");
    h.execute(&world, GenerateOperation::new(&world, "K1").expect("generate"));
    let cached = model::toolpath_of(&world, &model::find_by_name(&world, "K1").expect("op")).expect("cached").path.clone();
    assert!(!old.notes.iter().any(|n| n == KERNEL_POCKET_NOTE), "the default is the old planner");
    assert_eq!(old.moves, default.moves, "the default engine is the old planner");
    assert_eq!(old.moves, cached.moves, "the command reads the default engine");
}

#[test]
fn kernel_route_refuses_what_the_recipe_does_not_plan() {
    let (world, mut h) = block_world();
    block_op(&world, &mut h, "Plunge", Entry3D::Plunge, PocketStrategy::Waterline);
    let e = generate_on(&world, &op_of(&world, "Plunge"), "Plunge", PlannerEngine::New).expect_err("refused by name");
    assert!(e.to_string().contains(NO_SAFE_ENTRY), "a closed pocket with no hole and no helix or ramp: {e}");
    // A document written past the door of the command (Adaptive is refused at creation).
    let mut op = op_of(&world, "Plunge");
    if let OpParams::Pocket { strategy, entry, .. } = &mut op.params {
        *strategy = PocketStrategy::Adaptive;
        *entry = Entry3D::Helix;
    }
    let e = generate_on(&world, &op, "Plunge", PlannerEngine::New).expect_err("refused by name");
    assert!(e.to_string().contains("Adaptive"), "{e}");
    block_op(&world, &mut h, "Rest", Entry3D::Helix, PocketStrategy::Waterline);
    let mut op = op_of(&world, "Rest");
    if let OpParams::Pocket { rest, .. } = &mut op.params {
        *rest = Some(RestParams {
            source: RestSource::EnteredTool,
            previous_tool_diameter: 12.0,
            previous_corner_radius: 0.0,
            previous_stock_to_leave: 0.0,
            allowance: 0.0,
        });
    }
    let e = generate_on(&world, &op, "Rest", PlannerEngine::New).expect_err("refused by name");
    assert!(e.to_string().contains("rest machining") && e.to_string().contains("#3753"), "{e}");
    for kind in PHASE1_KINDS.into_iter().filter(|k| !KERNEL_KINDS.contains(k)) {
        let e = check_engine(kind, PlannerEngine::New).expect_err("still refused under new");
        assert!(e.to_string().contains("does not plan"), "{kind}: {e}");
    }
    assert_eq!(check_engine(OpType::Pocket, PlannerEngine::New), Ok(()), "the 2D Pocket passes under new");
}

/// The law readings (#4154) of the fig8 pocket, Waterline with a helix,
/// on the old planner and on the kernel: (old, new).
fn fig8_readings(start: PassStart) -> (Readings, Readings) {
    let (world, mut h, name, stock) = fig8_world();
    let zmin = stock.0.z;
    let src = sel(&name, horizontal_face(&world, &name, zmin));
    let mut p = pocket(zmin + 20.0, zmin);
    if let OpParams::Pocket { clearance_z, top_z, bottom_z, stepdown, stepover, strategy, pass_start, entry, helix_radius, .. } = &mut p {
        *clearance_z = Height { reference: HeightReference::TopOfStock, offset: 10.0 };
        *top_z = Height { reference: HeightReference::TopOfStock, offset: 0.0 };
        *bottom_z = Height { reference: HeightReference::BottomOfSelection, offset: 1.5 };
        *stepdown = 10.0;
        *stepover = 3.0;
        *strategy = PocketStrategy::Waterline;
        *pass_start = start;
        *entry = Entry3D::Helix;
        *helix_radius = 2.0;
    }
    let area = AreaControls { containment: Some(Containment::Outside), ..Default::default() };
    h.execute(&world, CreateOperation::new(&world, "P1", EM8, src, p.clone()).expect("create"));
    h.execute(&world, EditOperationParams::with_area(&world, "P1", p, area).expect("area"));
    let op = op_of(&world, "P1");
    let (old, _) = generate_on(&world, &op, "P1", PlannerEngine::Old).expect("the old planner plans fig8");
    let (new, warns) = generate_on(&world, &op, "P1", PlannerEngine::New).expect("the kernel plans fig8");
    eprintln!("fig8 {start:?} kernel: {} moves, verifier {warns:?}", new.moves.len());
    assert!(warns.iter().all(|w| !w.blocks_post()), "the verifier blocks the fig8 pocket: {warns:?}");
    let (lo, hi) = stock;
    let pts = vec![Point2::new(lo.x, lo.y), Point2::new(hi.x, lo.y), Point2::new(hi.x, hi.y), Point2::new(lo.x, hi.y)];
    let st = Region::new(vec![Loop::new(pts).expect("loop")]).expect("stock");
    let mut spec = Spec::new(R8, st, hi.z, (1.0 - 3.0 / R8).acos().to_degrees());
    spec.asked_entry = Some(MeasuredEntry::Helix);
    let read = |t: &Toolpath| measure(t, &spec).expect("measured");
    let (ro, rn) = (read(&old), read(&new));
    eprintln!("fig8 {start:?} old: {} | breaks {}", ro.row(), ro.breaks(false));
    eprintln!("fig8 {start:?} new: {} | breaks {}", rn.row(), rn.breaks(false));
    (ro, rn)
}

/// #3752 pin on fig8: each lift of the kernel's pocket has a named reason,
/// and the kernel lifts less than the old planner.
#[test]
#[ignore = "heavy lane: app heavy: pocket_pass_start (#3752: six fig8 generations run past the 60 s debug budget)"]
fn fig8_kernel_pocket_lifts_are_named() {
    for start in PassStart::ALL {
        let (old, new) = fig8_readings(start);
        assert_eq!(new.named_lifts, new.lifts.count, "fig8 {start:?}: each lift has a named reason");
        assert!(new.lifts.count <= old.lifts.count, "fig8 {start:?}: the kernel lifts no more than the old planner");
    }
}

/// Light twin of `fig8_kernel_pocket_lifts_are_named`: the Auto start only.
#[test]
#[ignore = "light lane: the Auto start of the full test only (#3752); the full test is fig8_kernel_pocket_lifts_are_named"]
fn fig8_kernel_pocket_lifts_are_named_light() {
    let (old, new) = fig8_readings(PassStart::Auto);
    assert_eq!(new.named_lifts, new.lifts.count, "fig8 Auto: each lift has a named reason");
    assert!(new.lifts.count <= old.lifts.count, "fig8 Auto: the kernel lifts no more than the old planner");
}
