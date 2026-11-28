//! #3334: the Plasma Profile takes a start pick and an exit pick. The
//! lead-in of the picked contour lands on the projection of the start
//! pick on the profile, so the pierce stands in the scrap beside it. The
//! cut runs all the way round, then on to the projection of the exit
//! pick, and the lead-out and the torch off are there. A pick that a law
//! cannot hold is refused by name, never moved. The picks ride
//! `EditOperationParams` (undo, redo). A plasma cut without a pick is
//! byte-identical (`golden_cl::plasma_plate_with_holes_matches_golden`).
//!
//! Gate map (#3334): covers app::operation (plasma_op, pick_kinds,
//! types::plasma, types::sane), app::commands (EditOperationParams,
//! GenerateOperation), camcraft_motion::plasma (picks, emit).
//! Provenance: our own tests over our own fixtures; the plasma rules
//! they pin are cited in `crates/motion/src/plasma/mod.rs`.

use flecs_ecs::prelude::World;

use camcraft_app::cl::{ContourRole, Toolpath};
use camcraft_app::commands::{AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, OpType, PickCapability, PlasmaComp, PlasmaOp, SrcRef};
use camcraft_app::path_picks::{PathPick, PathPicks, PickRole};
use camcraft_app::tools::torch_tool;
use camcraft_app::world::new_world;

use crate::common::{gen_to_cltext, write_golden};

const TORCH: &str = "PT-KERF-150";
/// Half the 1.5 mm kerf: the torch centre stands this far off the part.
const R: f64 = 0.75;

/// A 40 x 40 x 5 mm plate with a 10 x 10 mm through hole, centred on the
/// origin.
fn holed_plate() -> Mesh {
    let ring = |h: f64, z: f64| [Point3::new(-h, -h, z), Point3::new(h, -h, z), Point3::new(h, h, z), Point3::new(-h, h, z)];
    let quad = |a, b, c, d| [Triangle::new(a, b, c), Triangle::new(a, c, d)];
    let mut facets = Vec::new();
    for (z, up) in [(5.0, true), (0.0, false)] {
        let (outer, inner) = (ring(20.0, z), ring(5.0, z));
        for i in 0..4 {
            let j = (i + 1) % 4;
            facets.extend(if up { quad(outer[i], outer[j], inner[j], inner[i]) } else { quad(outer[i], inner[i], inner[j], outer[j]) });
        }
    }
    for (radius, outward) in [(20.0, true), (5.0, false)] {
        let (lo, hi) = (ring(radius, 0.0), ring(radius, 5.0));
        for i in 0..4 {
            let j = (i + 1) % 4;
            facets.extend(if outward { quad(lo[i], lo[j], hi[j], hi[i]) } else { quad(lo[i], hi[i], hi[j], lo[j]) });
        }
    }
    Mesh::from_facets(facets)
}

fn picks(start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> PathPicks {
    let one = |p: Option<(f64, f64)>| p.map(|(x, y)| PathPick::xy(Point2::new(x, y))).into_iter().collect();
    PathPicks { start: one(start), exit: one(exit) }
}

fn plasma_op(picks: PathPicks) -> PlasmaOp {
    PlasmaOp {
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(5.0),
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
        picks,
    }
}

/// A plasma document: the 1300PL mounted, the torch, the holed plate.
fn plasma_doc() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, LoadMachine::builtin(&world, "Tormach 1300PL").expect("mounts"));
    h.execute(&world, AddTool::new(&world, TORCH, torch_tool(1.5)).expect("torch"));
    h.execute(&world, ImportModel::from_mesh("plate", holed_plate()));
    (world, h)
}

fn create(world: &World, h: &mut History, op: PlasmaOp) {
    let cmd = CreateOperation::new(world, "Plasma1", TORCH, SrcRef::Entity("plate".into()), OpParams::PlasmaProfile(op));
    h.execute(world, cmd.expect("the operation is taken"));
}

fn generate(world: &World, h: &mut History) -> Result<Toolpath, String> {
    let cmd = GenerateOperation::new(world, "Plasma1").map_err(|e| format!("{e:?}"))?;
    h.execute(world, cmd);
    Ok(path(world))
}

fn path(world: &World) -> Toolpath {
    let e = model::find_by_name(world, "Plasma1").expect("op");
    model::toolpath_of(world, &e).expect("generated").path
}

/// The path of the holed plate cut with `op`, or the refusal.
fn cut(op: PlasmaOp) -> Result<Toolpath, String> {
    let (world, mut h) = plasma_doc();
    let cmd = CreateOperation::new(&world, "Plasma1", TORCH, SrcRef::Entity("plate".into()), OpParams::PlasmaProfile(op));
    h.execute(&world, cmd.map_err(|e| format!("{e:?}"))?);
    generate(&world, &mut h)
}

fn near(p: Point3, x: f64, y: f64) -> bool {
    (p.x - x).abs() < 1e-6 && (p.y - y).abs() < 1e-6
}

/// The lit moves (first, last) of the contour of `role`.
fn span(t: &Toolpath, role: ContourRole) -> (usize, usize) {
    let rec = t.plasma.as_ref().expect("the plasma record");
    let s = rec.torches.iter().find(|s| s.role == role).expect("the contour");
    (s.on, s.off)
}

#[test]
fn the_plasma_profile_takes_a_start_pick_and_an_exit_pick() {
    let cap = OpType::PlasmaProfile.info().pick();
    assert_eq!(cap, PickCapability::Ring, "a point on a closed contour");
    assert!(cap.takes(PickRole::Start) && cap.takes(PickRole::Exit));
}

/// The outline is cut clockwise, so the torch runs down the right edge
/// (x = 20, the torch centre at x = 20.75).
#[test]
fn the_pierce_is_at_the_start_pick_and_the_torch_goes_off_at_the_exit_pick() {
    let auto = cut(plasma_op(PathPicks::default())).expect("no pick");
    let both = cut(plasma_op(picks(Some((20.0, -8.0)), Some((20.0, -14.0))))).expect("both picks");
    let (on, off) = span(&both, ContourRole::Outer);
    // The lead-in lands on the projection of the start pick, and the
    // pierce (where the lead-in starts) stands in the scrap beside it.
    assert!(near(both.moves[on].to, 20.0 + R, -8.0), "the lead-in lands at the start pick: {:?}", both.moves[on].to);
    let pierce = both.moves[on].from;
    assert!(pierce.x > 20.0 + R + 1.0, "the pierce is in the scrap outside the outline: {pierce:?}");
    assert!(near(both.moves[on - 1].to, pierce.x, pierce.y), "the torch descends on the pierce");
    // The cut passes the start two times: after the lead-in and after the full lap.
    let laps = both.moves[on..=off].iter().filter(|m| near(m.to, 20.0 + R, -8.0)).count();
    assert_eq!(laps, 2, "the outline is cut all the way round before the cut runs on");
    // The lead-out leaves from the projection of the exit pick, and the torch goes off at its end.
    assert!(near(both.moves[off].from, 20.0 + R, -14.0), "the lead-out starts at the exit pick: {:?}", both.moves[off].from);
    assert!(both.moves[off].to.x > 20.0 + R + 1.0, "the torch goes off in the scrap: {:?}", both.moves[off].to);
    assert!(both.notes.iter().any(|n| n.contains("start point")), "{:?}", both.notes);
    assert!(both.notes.iter().any(|n| n.contains("exit point")), "{:?}", both.notes);
    // The automatic seam is elsewhere, and the hole (no pick is near it) is not changed.
    let (a_on, a_off) = span(&auto, ContourRole::Outer);
    assert!(!near(auto.moves[a_on].to, 20.0 + R, -8.0), "the pick moved the seam");
    let hole = |t: &Toolpath| {
        let (on, off) = span(t, ContourRole::Inner);
        t.moves[on - 1..=off].iter().map(|m| (m.from, m.to, m.feed)).collect::<Vec<_>>()
    };
    assert_eq!(hole(&auto), hole(&both), "the hole keeps its automatic cut");
    assert!(a_off - a_on < off - on, "the picked exit adds the run past the start");
    write_golden("plasma_start_exit_picks.cl", &gen_to_cltext(&both));
}

/// With the small-hole rule off, the 10 x 10 hole is cut counter-clockwise
/// with arc leads. A lead-in to its corner comes out of the part.
#[test]
fn a_pick_whose_pierce_falls_on_the_part_is_refused_by_name() {
    let regular = |p: PathPicks| PlasmaOp { small_hole_dia: 0.0, ..plasma_op(p) };
    assert!(cut(regular(PathPicks::default())).is_ok(), "the hole is cut without a pick");
    // A start pick in the middle of the hole's right wall is held.
    let mid = cut(regular(picks(Some((5.0, 0.0)), None))).expect("a pick with room");
    let (on, _) = span(&mid, ContourRole::Inner);
    assert!(near(mid.moves[on].to, 5.0 - R, 0.0), "{:?}", mid.moves[on].to);
    assert!(mid.moves[on].from.x.abs() < 5.0 - R && mid.moves[on].from.y.abs() < 5.0 - R, "the pierce is in the slug");
    // A start pick at the hole's corner: the lead-in would start in the part.
    let why = cut(regular(picks(Some((5.0, 4.9)), None))).expect_err("refused");
    assert!(why.contains("start point (5.000, 4.900)"), "{why}");
    assert!(why.contains("the pierce would fall on the part"), "{why}");
    // An exit pick there: the lead-out would end in the part.
    let why = cut(regular(picks(None, Some((5.0, 4.9))))).expect_err("refused");
    assert!(why.contains("exit point (5.000, 4.900)"), "{why}");
    assert!(why.contains("lead-out") && why.contains("part"), "{why}");
}

#[test]
fn a_pick_that_is_far_from_each_profile_is_refused_by_name() {
    for (p, what) in [(picks(Some((60.0, 0.0)), None), "start point"), (picks(None, Some((60.0, 0.0))), "exit point")] {
        let why = cut(plasma_op(p)).expect_err("refused");
        assert!(why.contains(&format!("{what} (60.000, 0.000)")), "{why}");
        assert!(why.contains("from the nearest closed profile"), "{why}");
    }
}

/// The small-hole rule keeps its law: the pierce stays at the centre and
/// the hole takes no lead-out. The overburn is cut before the exit.
#[test]
fn a_small_hole_keeps_its_rule_under_the_picks() {
    let start = cut(plasma_op(picks(Some((5.0, 2.0)), None))).expect("a start pick on the small hole");
    let (on, off) = span(&start, ContourRole::Inner);
    assert!(near(start.moves[on].from, 0.0, 0.0), "the pierce stays at the centre: {:?}", start.moves[on].from);
    assert!(near(start.moves[on].to, 5.0 - R, 2.0), "the lead-in lands at the start pick: {:?}", start.moves[on].to);
    // The 1 mm overburn runs past the start, up the right wall.
    assert!(near(start.moves[off].to, 5.0 - R, 3.0), "{:?}", start.moves[off].to);
    // An exit pick past the overburn: the torch goes off there, with no lead-out.
    let exit = cut(plasma_op(picks(Some((5.0, 2.0)), Some((5.0, 4.0))))).expect("an exit past the overburn");
    let (_, off) = span(&exit, ContourRole::Inner);
    assert!(near(exit.moves[off].to, 5.0 - R, 4.0), "{:?}", exit.moves[off].to);
    // An exit pick inside the overburn is refused by name.
    let why = cut(plasma_op(picks(Some((5.0, 2.0)), Some((5.0, 2.5))))).expect_err("refused");
    assert!(why.contains("exit point (5.000, 2.500)") && why.contains("overburn"), "{why}");
}

#[test]
fn an_exit_pick_under_control_compensation_is_refused_at_the_door() {
    let (world, _) = plasma_doc();
    let op = PlasmaOp { comp: PlasmaComp::Control, ..plasma_op(picks(None, Some((20.0, -14.0)))) };
    let err = CreateOperation::new(&world, "Plasma1", TORCH, SrcRef::Entity("plate".into()), OpParams::PlasmaProfile(op))
        .err()
        .expect("refused at the door");
    let why = format!("{err:?}");
    assert!(why.contains("Plasma Profile") && why.contains("exit point") && why.contains("G41/G42"), "{why}");
    // The start pick is held under control compensation: the span chains onto the cut.
    let op = PlasmaOp { comp: PlasmaComp::Control, ..plasma_op(picks(Some((20.0, -8.0)), None)) };
    let t = cut(op).expect("a start pick under G41/G42");
    let (on, _) = span(&t, ContourRole::Outer);
    assert!(near(t.moves[on].to, 20.0 + R, -8.0), "{:?}", t.moves[on].to);
    assert!(t.comps.iter().any(|c| c.first == on && c.is_sane(&t.moves)), "the G41/G42 span starts at the picked lead-in");
}

#[test]
fn undo_and_redo_restore_the_picks_and_the_path() {
    let (world, mut h) = plasma_doc();
    let p0 = plasma_op(PathPicks::default());
    let p1 = plasma_op(picks(Some((20.0, -8.0)), Some((20.0, -14.0))));
    create(&world, &mut h, p0.clone());
    let auto = generate(&world, &mut h).expect("no pick");
    h.execute(&world, EditOperationParams::new(&world, "Plasma1", OpParams::PlasmaProfile(p1.clone())).expect("the picks are taken"));
    let picked = generate(&world, &mut h).expect("both picks");
    assert_ne!(auto.moves, picked.moves, "the picks change the path");
    let params = |w: &World| model::find_by_name(w, "Plasma1").and_then(|e| model::operation_of(w, &e)).map(|o| o.params);
    // Undo the generation and the edit: the automatic path and no pick.
    assert!(h.undo(&world) && h.undo(&world));
    assert_eq!(params(&world), Some(OpParams::PlasmaProfile(p0)));
    assert_eq!(path(&world).moves, auto.moves, "undo restores the previous path");
    // Redo the two: the picks and the picked path.
    assert!(h.redo(&world) && h.redo(&world));
    assert_eq!(params(&world), Some(OpParams::PlasmaProfile(p1)));
    assert_eq!(path(&world).moves, picked.moves, "redo restores the picked path");
}
