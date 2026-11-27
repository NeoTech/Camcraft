//! #4177 (K4 of the path kernel): the kernel's ring guide, order stage and
//! link stage round the disc of #3934 and the dog bone of #3741, Inside and
//! Outside, read with the same law measures as the old planners (#4154).
//! The test turns the kernel plan into a toolpath only as far as the
//! measures need: a rapid and a plunge for each entry, a feed along each
//! cut and each stay-down link, a rapid up for each lift, and one link
//! note for each lift reason (the note text of `links.rs`).
//! Pins: no crossing feed, each lift named, and the lift count. The test
//! prints the places of the full-width feeds (load is K3's, #4254 reports
//! them).

use super::common::law_measure::{Readings, measure};
use super::fixture::{R, STEP, TOP, Z, parts, planned_deg, spec_round, stock};
use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::geometry::Point3;
use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, RingPlanner, RingRef, RingsChoice};
use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally};
use camcraft_pathkernel::order::{Direction, OrderChoice, OrderPlanner, OrderStage, StartSide};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{Bounds, MaterialSide, Selection, ToolShape, WorkPlane, region};

/// The retract plane of the converted path.
const RETRACT: f64 = 2.0;

struct Pen {
    path: Toolpath,
    at: Point3,
}

impl Pen {
    fn go(&mut self, kind: MoveKind, x: f64, y: f64, z: f64) {
        let to = Point3::new(x, y, z);
        let feed = (kind != MoveKind::Rapid).then_some(1000.0);
        self.path.moves.push(Move::new(kind, self.at, to, feed));
        self.at = to;
    }
    fn feed(&mut self, q: Point2) {
        if (q.x - self.at.x).hypot(q.y - self.at.y) > 1e-9 {
            self.go(MoveKind::Feed, q.x, q.y, Z);
        }
    }
    fn enter(&mut self, q: Point2) {
        self.go(MoveKind::Rapid, self.at.x, self.at.y, RETRACT);
        self.go(MoveKind::Rapid, q.x, q.y, RETRACT);
        self.go(MoveKind::Plunge, q.x, q.y, Z);
    }
}

/// The kernel plan of the material round `part` from `side`, as a
/// toolpath, with its link tally.
fn kernel_path(part: &Region, side: StartSide) -> (Toolpath, LinkTally) {
    let sel = Selection::face(part.loops()[0].points().to_vec(), vec![]).unwrap();
    let bounds = Bounds::new(Some(stock()), Region::empty(), TOP, Z).unwrap();
    let tool = ToolShape::disc(R, 0.0, true, 30.0).unwrap();
    let r = region(&sel, MaterialSide::Outside, &bounds, WorkPlane::setup_xy(0.0).unwrap(), tool, 0.0).unwrap();
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Part, 0.0, STEP, None, None).unwrap());
    let set = RingPlanner.guide(&r, &g, 1e-3).unwrap().out().unwrap().clone();
    let oc = OrderChoice { start: side, direction: Direction::Climb };
    let ordered = OrderStage.order(set, &r, &oc, None).unwrap().out().unwrap().clone();
    let cuts: Vec<Cut> = ordered.curves().iter().map(|c| Cut::new(c.clone(), None).unwrap()).collect();
    let mut pen = Pen { path: Toolpath::default(), at: Point3::new(0.0, 0.0, 10.0) };
    let mut tally = LinkTally::default();
    let mut paths: Vec<Vec<Point2>> = Vec::new();
    for (i, cut) in cuts.iter().enumerate() {
        let pts: Vec<Point2> = cut.curve().pts().iter().map(|q| q.at()).collect();
        let first = pts[0];
        match i.checked_sub(1) {
            None => pen.enter(first),
            Some(j) => match LinkStage.link(&cuts[j], cut, &r, Level::new(&cuts[..i], &cuts[i..]).with_links(&paths), LinkPolicy::StayWhereProved, &mut tally) {
                Hop::Stay(path) => {
                    for q in path.points() {
                        pen.feed(q);
                    }
                    pen.feed(first);
                    paths.push(path.points());
                }
                Hop::Lift(_) => pen.enter(first),
            },
        }
        for q in &pts[1..] {
            pen.feed(*q);
        }
        if cut.curve().closed() {
            pen.feed(first);
        }
    }
    pen.go(MoveKind::Rapid, pen.at.x, pen.at.y, RETRACT);
    for (why, n) in tally.by_reason() {
        pen.path.notes.push(format!("link: the tool lifted to the retract plane {n} time(s) instead of the keep-down link - {why}"));
    }
    (pen.path, tally)
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

#[test]
fn kernel_order_and_link_round_the_disc_and_the_dog_bone() {
    let mut got = Vec::new();
    for (pname, part) in parts() {
        for (sname, side) in [("Inside", StartSide::Inside), ("Outside", StartSide::Outside)] {
            let (path, tally) = kernel_path(&part, side);
            let r: Readings = measure(&path, &spec_round(&part, planned_deg(STEP, R))).unwrap();
            eprintln!("{pname} {sname}: {}", r.row());
            eprintln!("  lift reasons {:?} at {:?}", tally.by_reason(), r.lifts.places);
            eprintln!("  full-width feeds at {:?}", r.full_width.places);
            assert_eq!(r.crossings.count, 0, "{pname} {sname}: no feed crosses another: {:?}", r.crossings.places);
            assert_eq!(r.named_lifts, r.lifts.count, "{pname} {sname}: each lift has a named reason");
            assert_eq!(r.lifts.count, tally.lifts() as usize, "{pname} {sname}: the tally counts each lift");
            got.push((pname, sname, r.lifts.count));
        }
    }
    assert_eq!(got, LIFTS.to_vec(), "the lift counts of the kernel");
}
