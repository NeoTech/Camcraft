//! #4177 (K4 of the path kernel): the kernel's ring guide, order stage and
//! link stage round the disc of #3934 and the dog bone of #3741, Inside and
//! Outside, read with the same law measures as the old planners (#4154).
//! The kernel plan of each case (guide, order, link, and the entry and
//! exit stage) is made into a toolpath by the real converter of motion
//! (`camcraft_motion::kernel_emit`, #4269), which writes one link note for
//! each lift reason (the note text of `links.rs`).
//! Pins: the crossing feeds as measured (`CROSSINGS`), each lift named,
//! and the lift count. The test prints the places of the full-width feeds
//! (load is K3's, #4254 reports them).

use super::common::law_measure::{Readings, measure};
use super::fixture::{R, STEP, TOP, Z, parts, planned_deg, spec_round, stock};
use camcraft_motion::cl::{MoveKind, PathBuilder, Toolpath};
use camcraft_motion::geometry::Point3;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::kernel_emit::{EmitFeeds, EmitHeights, Emitted, emit_plan};
use camcraft_pathkernel::entry::{ArcChoice, EntryChoice, EntryKind, EntryPlanner, EntryStage, Heights};
use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, RingPlanner, RingRef, RingsChoice};
use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally};
use camcraft_pathkernel::load::{Controlled, LoadControl, LoadPlanner, StockRead};
use camcraft_pathkernel::order::{Direction, OrderChoice, OrderPlanner, OrderStage, StartSide};
use camcraft_pathkernel::plan::{Cut, Plan, Step};
use camcraft_pathkernel::{Bounds, MaterialSide, Selection, ToolShape, Verdict, WorkPlane, region};

/// The clearance height of the converted path (setup Z).
pub const CLEARANCE: f64 = 10.0;

/// The load stage of a kernel run (#4026): the planner, the choice and
/// the stock reader; `None` runs no load stage.
pub type Load<'a> = Option<(&'a dyn LoadPlanner, LoadControl, &'a dyn StockRead)>;

/// The kernel plan of the material round `part` from `side` at the one
/// level Z, with its link tally: the stages in the order of the law kit
/// (`crates/pathkernel/tests/laws/kit.rs`), a helix asked for each entry.
pub fn kernel_plan(part: &Region, side: StartSide) -> (Plan, LinkTally) {
    let (plan, tally, _) = kernel_plan_with(part, side, None);
    (plan, tally)
}

/// [`kernel_plan`] with the load stage `load` between order and link; the
/// load verdict goes with it.
pub fn kernel_plan_with(part: &Region, side: StartSide, load: Load) -> (Plan, LinkTally, Option<Verdict<Controlled>>) {
    let sel = Selection::face(part.loops()[0].points().to_vec(), vec![]).unwrap();
    let bounds = Bounds::new(Some(stock()), Region::empty(), TOP, Z).unwrap();
    let tool = ToolShape::disc(R, 0.0, true, 30.0).unwrap();
    let r = region(&sel, MaterialSide::Outside, &bounds, WorkPlane::setup_xy(Z).unwrap(), tool, 0.0).unwrap();
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Part, 0.0, STEP, None, None).unwrap());
    let set = RingPlanner.guide(&r, &g, 1e-3).unwrap().out().unwrap().clone();
    let oc = OrderChoice { start: side, direction: Direction::Climb };
    let ordered = OrderStage.order(set, &r, &oc, None).unwrap().out().unwrap().clone();
    let controlled = load.map(|(stage, lc, reader)| stage.control(ordered.clone(), &r, reader, &lc, R).unwrap());
    let curves = controlled.as_ref().and_then(Verdict::out).map_or(ordered.curves(), |c| c.set().curves());
    let cuts: Vec<Cut> = curves.iter().map(|c| Cut::new(c.clone(), None).unwrap()).collect();
    // The heights of the entry stage are along the plane normal above the level.
    let stage = EntryStage::new(Heights::new(TOP - Z, 1.0, CLEARANCE - Z).unwrap(), ArcChoice::new(R, 90.0).unwrap(), 0.01).unwrap();
    let choice = EntryChoice::new(EntryKind::Helix, 1.0, vec![]).unwrap();
    let cleared = Region::empty();
    let mut tally = LinkTally::default();
    let mut paths: Vec<Vec<Point2>> = Vec::new();
    let mut steps = Vec::new();
    for (i, cut) in cuts.iter().enumerate() {
        let lifted = match i.checked_sub(1) {
            None => true,
            Some(j) => {
                let level = Level::new(&cuts[..i], &cuts[i..]).with_links(&paths);
                let hop = LinkStage.link(&cuts[j], cut, &r, level, LinkPolicy::StayWhereProved, &mut tally);
                if let Hop::Stay(path) = &hop {
                    paths.push(path.points());
                }
                let lift = matches!(hop, Hop::Lift(_));
                steps.push(Step::Link(hop));
                lift
            }
        };
        if lifted {
            let e = stage.entry(cut, &r, &cleared, &choice).unwrap();
            steps.push(Step::Entry(e.out().unwrap().clone()));
        }
        steps.push(Step::Cut(cut.clone()));
    }
    if let Some(last) = cuts.last() {
        steps.push(Step::Exit(stage.exit(last, &r, &cleared).unwrap()));
    }
    (Plan::new(steps).unwrap(), tally, controlled)
}

/// The plan made into a toolpath by the converter.
pub fn convert(plan: &Plan) -> Emitted {
    let h = EmitHeights::new(CLEARANCE, TOP).unwrap();
    let f = EmitFeeds::new(1000.0, 300.0).unwrap();
    emit_plan(plan, WorkPlane::setup_xy(Z).unwrap(), h, f).unwrap()
}

/// The kernel plan of the material round `part` from `side`, as a
/// toolpath, with its link tally.
fn kernel_path(part: &Region, side: StartSide) -> (Toolpath, LinkTally) {
    let (path, tally, _) = kernel_path_with(part, side, None);
    (path, tally)
}

/// [`kernel_path`] with the load stage `load` between order and link; the
/// load verdict goes with it, and each of its notes is a path note (the
/// converter writes only the lift notes: the caller writes the notes of
/// the stage verdicts).
pub fn kernel_path_with(part: &Region, side: StartSide, load: Load) -> (Toolpath, LinkTally, Option<Verdict<Controlled>>) {
    let (plan, tally, controlled) = kernel_plan_with(part, side, load);
    let mut out = convert(&plan);
    let cuts = plan.steps().iter().filter(|s| matches!(s, Step::Cut(_))).count();
    assert_eq!(out.cut_moves.len(), cuts, "the converter emits each cut");
    assert_eq!(out.lifts, tally.lifts() as usize, "the converter emits each lift");
    let count = |f: fn(&Step) -> bool| plan.steps().iter().filter(|s| f(s)).count();
    assert_eq!(out.entries, count(|s| matches!(s, Step::Entry(_))), "the converter emits each entry");
    assert_eq!(out.stays, count(|s| matches!(s, Step::Link(Hop::Stay(_)))), "the converter emits each stay-down link");
    assert_eq!(out.exits, count(|s| matches!(s, Step::Exit(_))), "the converter emits each exit");
    if let Some(Verdict::Done { notes, .. }) = &controlled {
        out.path.notes.extend(notes.iter().map(|n| n.text().to_string()));
    }
    (out.path, tally, controlled)
}

/// The retract plane of the cuts-only path.
const RETRACT: f64 = 2.0;

/// The cuts of the kernel plan with the load stage `load`, without the
/// moves of the entry and exit stage: a rapid and a plunge at each entry
/// to the start of its cut, a feed along each cut and each stay-down link,
/// a rapid up at the end, and one link note for each lift reason. The load
/// readings of `k3_load` (#4026) measure the cuts that the load stage
/// gives, so the leads of the entry stage do not go into them.
pub fn cuts_path_with(part: &Region, side: StartSide, load: Load) -> (Toolpath, LinkTally, Option<Verdict<Controlled>>) {
    let (plan, tally, controlled) = kernel_plan_with(part, side, load);
    let mut b = PathBuilder::new(Point3::new(0.0, 0.0, 10.0));
    // A feed shorter than 1e-9 mm in XY is skipped, as the K4 test pen did.
    let feed = |b: &mut PathBuilder, q: Point2| {
        if (q.x - b.at().x).hypot(q.y - b.at().y) > 1e-9 {
            b.push(MoveKind::Feed, Point3::new(q.x, q.y, Z), Some(1000.0));
        }
    };
    let mut entered = false;
    for step in plan.steps() {
        match step {
            Step::Entry(_) => entered = true,
            Step::Cut(c) => {
                let pts: Vec<Point2> = c.curve().pts().iter().map(|q| q.at()).collect();
                if std::mem::take(&mut entered) {
                    let at = b.at();
                    b.push(MoveKind::Rapid, Point3::new(at.x, at.y, RETRACT), None);
                    b.push(MoveKind::Rapid, Point3::new(pts[0].x, pts[0].y, RETRACT), None);
                    b.push(MoveKind::Plunge, Point3::new(pts[0].x, pts[0].y, Z), Some(1000.0));
                }
                for q in &pts[1..] {
                    feed(&mut b, *q);
                }
                if c.curve().closed() {
                    feed(&mut b, pts[0]);
                }
            }
            Step::Link(Hop::Stay(path)) => path.points().into_iter().for_each(|q| feed(&mut b, q)),
            _ => {}
        }
    }
    let at = b.at();
    b.push(MoveKind::Rapid, Point3::new(at.x, at.y, RETRACT), None);
    let mut path = b.finish();
    for (why, n) in tally.by_reason() {
        path.notes.push(format!("link: the tool lifted to the retract plane {n} time(s) instead of the keep-down link - {why}"));
    }
    if let Some(Verdict::Done { notes, .. }) = &controlled {
        path.notes.extend(notes.iter().map(|n| n.text().to_string()));
    }
    (path, tally, controlled)
}

/// The lift count of the kernel for each part and side: (part, side, lifts).
/// The old planners on the same fixtures (#4154 tables): 2D Pocket 18 on
/// the disc and 26 on the dog bone, none named; 2D Offset 0. Outside starts
/// at the stock edge. A link leaves a piece there through the edge band
/// into the air (#4241) and comes back in to the start of the next piece
/// with an approach from the air whose disc cuts only the swath of that
/// piece (#4254, `docs/decisions/4254-kernel-air-approach.md`): no lift
/// stays.
const LIFTS: [(&str, &str, usize); 4] = [
    ("disc", "Inside", 0),
    ("disc", "Outside", 0),
    ("dog bone", "Inside", 0),
    ("dog bone", "Outside", 0),
];

/// The feed crossings of the converted plan for each part and side:
/// (part, side, crossings). Measured on the merge of #4254 (the approach
/// from the air) and #4269 (the real converter, which emits the arc lead
/// of each entry). Each Outside crossing is in the air, one tool radius
/// outside the stock: a link of #4254 runs along the air line (y -29) to
/// the door of its approach and crosses the arc lead of the first entry
/// (moves 2 to 11). The link stage proves a link against the cuts, the
/// links and the planned cuts, not against the leads of the entry stage
/// (follow-up issue named in `docs/decisions/4269-kernel-plan-converter.md`).
/// The kernel's own test pen has no leads and reads 0 on each case.
const CROSSINGS: [(&str, &str, usize); 4] = [
    ("disc", "Inside", 0),
    ("disc", "Outside", 3),
    ("dog bone", "Inside", 0),
    ("dog bone", "Outside", 1),
];

#[test]
fn kernel_order_and_link_round_the_disc_and_the_dog_bone() {
    let (mut got, mut crossed) = (Vec::new(), Vec::new());
    for (pname, part) in parts() {
        for (sname, side) in [("Inside", StartSide::Inside), ("Outside", StartSide::Outside)] {
            let (path, tally) = kernel_path(&part, side);
            let r: Readings = measure(&path, &spec_round(&part, planned_deg(STEP, R))).unwrap();
            eprintln!("{pname} {sname}: {}", r.row());
            eprintln!("  lift reasons {:?} at {:?}", tally.by_reason(), r.lifts.places);
            eprintln!("  full-width feeds at {:?}", r.full_width.places);
            eprintln!("  crossings {} at {:?}", r.crossings.count, r.crossings.places);
            assert_eq!(r.named_lifts, r.lifts.count, "{pname} {sname}: each lift has a named reason");
            assert_eq!(r.lifts.count, tally.lifts() as usize, "{pname} {sname}: the tally counts each lift");
            got.push((pname, sname, r.lifts.count));
            crossed.push((pname, sname, r.crossings.count));
        }
    }
    assert_eq!(got, LIFTS.to_vec(), "the lift counts of the kernel");
    assert_eq!(crossed, CROSSINGS.to_vec(), "the feed crossings of the converted plan");
}
