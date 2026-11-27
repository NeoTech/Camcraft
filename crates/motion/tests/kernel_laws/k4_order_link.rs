//! #4177 (K4 of the path kernel): the kernel's ring guide, order stage and
//! link stage round the disc of #3934 and the dog bone of #3741, Inside and
//! Outside, read with the same law measures as the old planners (#4154).
//! The kernel plan of each case (guide, order, link, and the entry and
//! exit stage) is made into a toolpath by the real converter of motion
//! (`camcraft_motion::kernel_emit`, #4269), which writes one link note for
//! each lift reason (the note text of `links.rs`).
//! Pins: no crossing feed, each lift named, and the lift count.

use super::common::law_measure::{Readings, measure};
use super::fixture::{R, STEP, TOP, Z, parts, planned_deg, spec_round, stock};
use camcraft_motion::cl::Toolpath;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::kernel_emit::{EmitFeeds, EmitHeights, Emitted, emit_plan};
use camcraft_pathkernel::entry::{ArcChoice, EntryChoice, EntryKind, EntryPlanner, EntryStage, Heights};
use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, RingPlanner, RingRef, RingsChoice};
use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally};
use camcraft_pathkernel::order::{Direction, OrderChoice, OrderPlanner, OrderStage, StartSide};
use camcraft_pathkernel::plan::{Cut, Plan, Step};
use camcraft_pathkernel::{Bounds, MaterialSide, Selection, ToolShape, WorkPlane, region};

/// The clearance height of the converted path (setup Z).
pub const CLEARANCE: f64 = 10.0;

/// The kernel plan of the material round `part` from `side` at the one
/// level Z, with its link tally: the stages in the order of the law kit
/// (`crates/pathkernel/tests/laws/kit.rs`), a helix asked for each entry.
pub fn kernel_plan(part: &Region, side: StartSide) -> (Plan, LinkTally) {
    let sel = Selection::face(part.loops()[0].points().to_vec(), vec![]).unwrap();
    let bounds = Bounds::new(Some(stock()), Region::empty(), TOP, Z).unwrap();
    let tool = ToolShape::disc(R, 0.0, true, 30.0).unwrap();
    let r = region(&sel, MaterialSide::Outside, &bounds, WorkPlane::setup_xy(Z).unwrap(), tool, 0.0).unwrap();
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Part, 0.0, STEP, None, None).unwrap());
    let set = RingPlanner.guide(&r, &g, 1e-3).unwrap().out().unwrap().clone();
    let oc = OrderChoice { start: side, direction: Direction::Climb };
    let ordered = OrderStage.order(set, &r, &oc, None).unwrap().out().unwrap().clone();
    let cuts: Vec<Cut> = ordered.curves().iter().map(|c| Cut::new(c.clone(), None).unwrap()).collect();
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
    (Plan::new(steps).unwrap(), tally)
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
    let (plan, tally) = kernel_plan(part, side);
    let out = convert(&plan);
    let cuts = plan.steps().iter().filter(|s| matches!(s, Step::Cut(_))).count();
    assert_eq!(out.cut_moves.len(), cuts, "the converter emits each cut");
    assert_eq!(out.lifts, tally.lifts() as usize, "the converter emits each lift");
    (out.path, tally)
}

/// The lift count of the kernel for each part and side: (part, side, lifts).
/// The old planners on the same fixtures (#4154 tables): 2D Pocket 18 on
/// the disc and 26 on the dog bone, none named; 2D Offset 0. The lifts that
/// stay are named in `docs/decisions/4177-kernel-order-link.md`: Outside
/// starts at the stock edge, and the link from one piece there to the next
/// runs over the uncut strip along the edge (#4234: that strip is not open
/// ground until a cut clears it). A link may go out through the edge band
/// into the air (#4241), but the way back in to the next piece's start runs
/// over stock that no cut has cleared: the counts do not change.
const LIFTS: [(&str, &str, usize); 4] = [
    ("disc", "Inside", 0),
    ("disc", "Outside", 13),
    ("dog bone", "Inside", 0),
    ("dog bone", "Outside", 15),
];

#[test]
fn kernel_order_and_link_round_the_disc_and_the_dog_bone() {
    let mut got = Vec::new();
    for (pname, part) in parts() {
        for (sname, side) in [("Inside", StartSide::Inside), ("Outside", StartSide::Outside)] {
            let (path, tally) = kernel_path(&part, side);
            let r: Readings = measure(&path, &spec_round(&part, planned_deg(STEP, R))).unwrap();
            eprintln!("{pname} {sname}: {}", r.row());
            eprintln!("  lift reasons {:?} at {:?}", tally.by_reason(), r.lifts.places);
            assert_eq!(r.crossings.count, 0, "{pname} {sname}: no feed crosses another: {:?}", r.crossings.places);
            assert_eq!(r.named_lifts, r.lifts.count, "{pname} {sname}: each lift has a named reason");
            assert_eq!(r.lifts.count, tally.lifts() as usize, "{pname} {sname}: the tally counts each lift");
            got.push((pname, sname, r.lifts.count));
        }
    }
    assert_eq!(got, LIFTS.to_vec(), "the lift counts of the kernel");
}
