//! Deburr's start and exit points (#3329, the Deburr slice of #1448):
//! the chain nearest the start pick is cut first from its point nearest
//! it, the chain nearest the exit pick last to its point nearest it, and
//! the cut geometry stays the automatic ride's. Every impossible pick is
//! refused by name; each chain end that a pick moves has its plunge or
//! retract column checked exactly. A module of the `deburr` suite.

use super::*;
use camcraft_motion::cl::Toolpath;
use camcraft_motion::finishing3d::{deburr_seamed, ChainSeam};
use camcraft_motion::path_picks::PathPick;

/// The chamfer mill of these tests: 4 mm across, so a pick reaches 6 mm
/// and a moved column clears 2 mm.
const D: f64 = 4.0;

fn pick(x: f64, y: f64, z: Option<f64>) -> PathPick {
    PathPick { xy: Point2::new(x, y), z }
}

fn seam(start: Option<PathPick>, exit: Option<PathPick>) -> ChainSeam {
    ChainSeam { start, exit, tool_diameter: D }
}

fn ride(f: &[Triangle], p: &DeburrParams, s: ChainSeam) -> Result<Toolpath, GenError> {
    deburr_seamed(f, p, &DeburrEdges::default(), &s)
}

/// The box 0..10 x 0..6 x 0..5 in a window that keeps its top ring only:
/// one closed chain at z 5, its tip at z 3.
fn top_ring() -> (Vec<Triangle>, DeburrParams) {
    (box_mesh(0.0, 0.0, 10.0, 6.0, 0.0, 5.0), params(4.0, 0.0))
}

/// The curtain's top rim: one open chain at z 0, its tip at z -2.
fn rim() -> (Vec<Triangle>, DeburrParams) {
    (curtain(0.0, 10.0, -10.0, 0.0), params(-2.0, -4.0))
}

/// A shelf 4 mm square around (x, y) at height z: an up-facing sheet
/// over a chain end, out of the ride's window (its rim tips stand above
/// Window Top Z).
fn slab(x: f64, y: f64, z: f64) -> Vec<Triangle> {
    let (x0, y0, x1, y1) = (x - 2.0, y - 2.0, x + 2.0, y + 2.0);
    vec![
        Triangle::new(p3(x0, y0, z), p3(x1, y0, z), p3(x1, y1, z)),
        Triangle::new(p3(x0, y0, z), p3(x1, y1, z), p3(x0, y1, z)),
    ]
}

fn feeds(t: &Toolpath) -> Vec<(Point3, Point3)> {
    t.moves.iter().filter(|m| m.kind == MoveKind::Feed).map(|m| (m.from, m.to)).collect()
}

fn first_plunge(t: &Toolpath) -> Point3 {
    t.moves.iter().find(|m| m.kind == MoveKind::Plunge).map(|m| m.to).expect("a plunge")
}

fn last_feed(t: &Toolpath) -> Point3 {
    t.moves.iter().rev().find(|m| m.kind == MoveKind::Feed).map(|m| m.to).expect("a feed")
}

fn near(a: Point3, b: Point3) -> bool {
    (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6 && (a.z - b.z).abs() < 1e-6
}

fn off(c: Point3, segs: &[(Point3, Point3)]) -> f64 {
    segs.iter()
        .map(|&(a, b)| {
            let d = p3(b.x - a.x, b.y - a.y, b.z - a.z);
            let l2 = d.x * d.x + d.y * d.y + d.z * d.z;
            let t = if l2 > 0.0 { (((c.x - a.x) * d.x + (c.y - a.y) * d.y + (c.z - a.z) * d.z) / l2).clamp(0.0, 1.0) } else { 0.0 };
            ((c.x - a.x - d.x * t).powi(2) + (c.y - a.y - d.y * t).powi(2) + (c.z - a.z - d.z * t).powi(2)).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

/// Every point of every cut of `got`, sampled each 0.25 mm along its
/// length, lies on a cut of `auto`, and the other way round: the pick
/// changes the order, never what is cut.
fn same_cut(auto: &Toolpath, got: &Toolpath) {
    for (from, to, what) in [(got, auto, "a picked cut leaves the automatic cut"), (auto, got, "an automatic cut is missing")] {
        let segs = feeds(to);
        for (a, b) in feeds(from) {
            let len = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt();
            let n = ((len / 0.25).ceil() as usize).max(1);
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let c = p3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t);
                assert!(off(c, &segs) < 1e-6, "{what} at {c:?} ({} mm)", off(c, &segs));
            }
        }
    }
}

fn length(t: &Toolpath) -> f64 {
    feeds(t).iter().map(|(a, b)| ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt()).sum()
}

#[test]
fn no_pick_is_the_automatic_ride_byte_for_byte() {
    for (f, p) in [top_ring(), rim()] {
        let auto = deburr(&f, &p).expect("auto");
        assert_eq!(ride(&f, &p, seam(None, None)).expect("seamed"), auto);
    }
}

#[test]
fn a_start_pick_seats_the_closed_chain_at_its_nearest_point() {
    let (f, p) = top_ring();
    let auto = deburr(&f, &p).expect("auto");
    let t = ride(&f, &p, seam(Some(pick(10.5, 3.0, None)), None)).expect("the started ring");
    assert!(near(first_plunge(&t), p3(10.0, 3.0, 3.0)), "the cut starts at the pick: {:?}", first_plunge(&t));
    assert!(near(last_feed(&t), p3(10.0, 3.0, 3.0)), "the closed lap ends where it started: {:?}", last_feed(&t));
    same_cut(&auto, &t);
    assert!((length(&t) - length(&auto)).abs() < 1e-6, "no cut is added: {} vs {}", length(&t), length(&auto));
}

#[test]
fn an_exit_pick_ends_the_closed_chain_at_its_nearest_point_and_rises_straight() {
    let (f, p) = top_ring();
    let auto = deburr(&f, &p).expect("auto");
    let t = ride(&f, &p, seam(None, Some(pick(5.0, -0.5, Some(5.0))))).expect("the exited ring");
    let end = t.moves.iter().rposition(|m| m.kind == MoveKind::Feed).expect("a feed");
    assert!(near(t.moves[end].to, p3(5.0, 0.0, 3.0)), "the cut ends at the pick: {:?}", t.moves[end].to);
    let rise = &t.moves[end + 1];
    assert!(rise.from.x == rise.to.x && rise.from.y == rise.to.y && rise.to.z > rise.from.z, "the tool leaves straight up: {rise:?}");
    same_cut(&auto, &t);
}

#[test]
fn start_and_exit_on_one_closed_chain_lap_then_run_on_to_the_exit() {
    let (f, p) = top_ring();
    let auto = deburr(&f, &p).expect("auto");
    let t = ride(&f, &p, seam(Some(pick(10.5, 3.0, None)), Some(pick(5.0, -0.5, None)))).expect("the ring");
    assert!(near(first_plunge(&t), p3(10.0, 3.0, 3.0)), "{:?}", first_plunge(&t));
    assert!(near(last_feed(&t), p3(5.0, 0.0, 3.0)), "{:?}", last_feed(&t));
    same_cut(&auto, &t);
    let lap = length(&auto);
    assert!(length(&t) > lap + 1e-6 && length(&t) < 2.0 * lap, "one lap and a run-on: {} of {lap}", length(&t));
}

#[test]
fn picks_on_an_open_chain_choose_its_direction() {
    let (f, p) = rim();
    let auto = deburr(&f, &p).expect("auto");
    let (a, b) = (first_plunge(&auto), last_feed(&auto));
    let reversed = |t: &Toolpath| {
        assert!(near(first_plunge(t), b) && near(last_feed(t), a), "the chain runs from {:?} to {:?}", first_plunge(t), last_feed(t));
        same_cut(&auto, t);
    };
    // A start at the far end, with the pick's Z.
    reversed(&ride(&f, &p, seam(Some(pick(b.x, b.y + 0.3, Some(0.0))), None)).expect("started"));
    // An exit at the automatic first end.
    reversed(&ride(&f, &p, seam(None, Some(pick(a.x, a.y, None)))).expect("exited"));
    reversed(&ride(&f, &p, seam(Some(pick(b.x, b.y, None)), Some(pick(a.x, a.y, None)))).expect("both"));
    // Picks that keep the automatic direction keep the path.
    assert_eq!(ride(&f, &p, seam(Some(pick(a.x, a.y, None)), Some(pick(b.x, b.y, None)))).expect("kept"), auto);
}

#[test]
fn the_start_chain_is_cut_first_and_the_exit_chain_last() {
    // The curtain's rim and a box's top ring 5 mm behind it.
    let mut f = curtain(0.0, 10.0, -10.0, 0.0);
    f.extend(box_mesh(0.0, 5.0, 10.0, 11.0, -10.0, 0.0));
    let p = params(-2.0, -4.0);
    let auto = deburr(&f, &p).expect("auto");
    let on_rim = |q: Point3| q.y.abs() < 1e-9;
    let rim_first = on_rim(first_plunge(&auto));
    // Start on the chain the automatic order cuts last.
    let s = if rim_first { pick(5.0, 11.5, None) } else { pick(5.0, -0.5, None) };
    let t = ride(&f, &p, seam(Some(s), None)).expect("started");
    assert_eq!(on_rim(first_plunge(&t)), !rim_first, "the picked chain is cut first: {:?}", first_plunge(&t));
    same_cut(&auto, &t);
    // Exit on the chain the automatic order cuts first.
    let e = if rim_first { pick(5.0, -0.5, None) } else { pick(5.0, 11.5, None) };
    let t = ride(&f, &p, seam(None, Some(e))).expect("exited");
    assert_eq!(on_rim(last_feed(&t)), rim_first, "the picked chain is cut last: {:?}", last_feed(&t));
    same_cut(&auto, &t);
}

#[test]
fn an_impossible_pick_is_refused_by_name() {
    let (f, p) = top_ring();
    let e = err(ride(&f, &p, seam(Some(pick(40.0, 3.0, None)), None)));
    assert!(e.contains("start point (40.000, 3.000) is") && e.contains("farther than the 6.000 mm"), "{e}");
    let e = err(ride(&f, &p, seam(None, Some(pick(5.0, 3.0, Some(30.0))))));
    assert!(e.contains("exit point (5.000, 3.000, 30.000) is") && e.contains("nearest edge chain"), "{e}");
    // Both picks at one end of an open chain: it cannot end where it starts.
    let (f, p) = rim();
    let auto = deburr(&f, &p).expect("auto");
    let a = first_plunge(&auto);
    let e = err(ride(&f, &p, seam(Some(pick(a.x, a.y, None)), Some(pick(a.x, a.y + 0.5, None)))));
    assert!(e.contains("exit point") && e.contains("ends at its other end"), "{e}");
}

#[test]
fn same_chain_picks_with_another_chain_must_keep_the_exit_or_refuse_it() {
    // Two boxes: two closed top rings at z 5, their tips at z 3.
    let mut f = box_mesh(0.0, 0.0, 10.0, 6.0, 0.0, 5.0);
    f.extend(box_mesh(20.0, 0.0, 30.0, 6.0, 0.0, 5.0));
    // The width 2 * sqrt(2) and the tip window Z 0..4 of `params`.
    let p = params(4.0, 0.0);
    match ride(&f, &p, seam(Some(pick(10.5, 3.0, None)), Some(pick(5.0, -0.5, None)))) {
        Ok(t) => assert!(near(last_feed(&t), p3(5.0, 0.0, 3.0)), "exit silently moved from (5, 0, 3) to {:?}", last_feed(&t)),
        Err(e) => {
            let e = e.to_string();
            assert!(e.contains("exit point (5.000, -0.500)") && e.contains("one edge chain"), "{e}");
        }
    }
    // Picks on two different chains still ride: start on one, exit on the other.
    let t = ride(&f, &p, seam(Some(pick(10.5, 3.0, None)), Some(pick(25.0, -0.5, None)))).expect("two chains");
    assert!(near(first_plunge(&t), p3(10.0, 3.0, 3.0)), "{:?}", first_plunge(&t));
    assert!(near(last_feed(&t), p3(25.0, 0.0, 3.0)), "{:?}", last_feed(&t));
}

#[test]
fn a_moved_plunge_or_retract_column_under_an_overhang_is_refused() {
    // A slab over the ring's x = 10 edge, above the window: the
    // automatic ride starts elsewhere and is not judged here.
    let (mut f, p) = top_ring();
    f.extend(slab(10.0, 3.0, 12.0));
    ride(&f, &p, seam(None, None)).expect("the automatic ride does not move a column");
    let e = err(ride(&f, &p, seam(Some(pick(10.5, 3.0, None)), None)));
    assert!(e.contains("start point (10.500, 3.000)") && e.contains("plunge column") && e.contains("tool radius 2.000"), "{e}");
    let e = err(ride(&f, &p, seam(None, Some(pick(10.5, 3.0, None)))));
    assert!(e.contains("exit point (10.500, 3.000)") && e.contains("column"), "{e}");
    // A pick clear of the slab rides.
    ride(&f, &p, seam(Some(pick(3.0, -0.5, None)), None)).expect("a clear column rides");
}

#[test]
fn a_reversed_chain_checks_both_moved_ends() {
    // The slab stands over the rim's automatic LAST end. An exit at the
    // first end reverses the chain, so the plunge moves under the slab:
    // the exit pick answers for it.
    let (f0, p) = rim();
    let auto = deburr(&f0, &p).expect("auto");
    let (a, b) = (first_plunge(&auto), last_feed(&auto));
    let mut f = f0.clone();
    f.extend(slab(b.x, b.y, 6.0));
    ride(&f, &p, seam(None, None)).expect("the automatic ride does not move a column");
    let e = err(ride(&f, &p, seam(None, Some(pick(a.x, a.y, None)))));
    assert!(e.contains("exit point") && e.contains("plunge column"), "the moved plunge is judged: {e}");
    // The slab over the automatic first end: a start at the last end
    // moves the retract under it.
    let mut f = f0;
    f.extend(slab(a.x, a.y, 6.0));
    let e = err(ride(&f, &p, seam(Some(pick(b.x, b.y, None)), None)));
    assert!(e.contains("start point") && e.contains("retract column"), "the moved retract is judged: {e}");
}
