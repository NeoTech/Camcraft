//! #3329: the user's start and exit points on the 3D Finish's Parallel
//! raster at any pass direction (finishing3d::finish_seam), a module of
//! the `multiaxis` suite. The raster seam's corner law is the 5-Axis
//! Parallel's (`raster_seam.rs` beside this file); what is pinned here
//! is the 3D Finish's own: the pass direction, the waterline rings that
//! cut after the raster, the pattern refusal, and the exact columns at
//! the picked corners.

use super::*;
use camcraft_motion::cl::Toolpath;
use camcraft_motion::finishing3d::{finish_seamed, parallel_with_angle, raster_ends_kept, RasterEnds, RasterSeam};
use camcraft_motion::path_picks::PathPick;
use camcraft_motion::stock_sim::dist_seg_tri;

/// The step that samples each cut along its length.
const STEP: f64 = 0.25;

fn tri(a: (f64, f64, f64), b: (f64, f64, f64), c: (f64, f64, f64)) -> Triangle {
    Triangle::new(p3(a.0, a.1, a.2), p3(b.0, b.1, b.2), p3(c.0, c.1, c.2))
}

/// A 40 x 30 floor at z = 0 with a 10 x 10 x 4 block on it: shallow
/// ground for the raster, steep walls for the rings.
fn floor_and_block() -> Vec<Triangle> {
    let mut f = vec![tri((0.0, 0.0, 0.0), (40.0, 0.0, 0.0), (40.0, 30.0, 0.0)), tri((0.0, 0.0, 0.0), (40.0, 30.0, 0.0), (0.0, 30.0, 0.0))];
    let (x0, x1, y0, y1, h) = (15.0, 25.0, 10.0, 20.0, 4.0);
    f.push(tri((x0, y0, h), (x1, y0, h), (x1, y1, h)));
    f.push(tri((x0, y0, h), (x1, y1, h), (x0, y1, h)));
    for ((ax, ay), (bx, by)) in [((x0, y0), (x1, y0)), ((x1, y0), (x1, y1)), ((x1, y1), (x0, y1)), ((x0, y1), (x0, y0))] {
        f.push(tri((ax, ay, 0.0), (bx, by, 0.0), (bx, by, h)));
        f.push(tri((ax, ay, 0.0), (bx, by, h), (ax, ay, h)));
    }
    f
}

fn params(waterlines: bool) -> Finish3DParams {
    let mut p = finish(5.0, 0.0);
    p.pattern = FinishPattern::Parallel;
    p.stepover = Some(4.0);
    p.waterlines = waterlines;
    p
}

fn pick(x: f64, y: f64) -> Option<PathPick> {
    Some(PathPick::xy(camcraft_motion::geo2d::Point2::new(x, y)))
}

fn seam(start: Option<PathPick>, exit: Option<PathPick>) -> RasterSeam {
    RasterSeam { start, exit }
}

fn same(a: Point3, b: Point3) -> bool {
    (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9 && (a.z - b.z).abs() < 1e-9
}

fn first_cut(t: &Toolpath) -> Point3 {
    t.moves.iter().find(|m| m.kind != MoveKind::Rapid).map(|m| m.to).expect("a cut")
}

fn last_feed(t: &Toolpath) -> usize {
    t.moves.iter().rposition(|m| m.kind == MoveKind::Feed).expect("a feed")
}

/// The feeds that travel in XY: the raster's cuts and links at depth.
fn cuts(t: &Toolpath) -> Vec<(Point3, Point3)> {
    t.moves.iter().filter(|m| m.kind == MoveKind::Feed && (m.from.x - m.to.x).hypot(m.from.y - m.to.y) > 1e-9).map(|m| (m.from, m.to)).collect()
}

/// Every cut of `got` lies on a cut of `auto` along its whole length,
/// sampled every [`STEP`] mm, and the other way round.
fn same_cuts(auto: &Toolpath, got: &Toolpath) {
    let off = |c: Point3, segs: &[(Point3, Point3)]| {
        segs.iter().map(|&(a, b)| {
            let d = b - a;
            let l2 = d.dot(d);
            let t = if l2 > 0.0 { ((c - a).dot(d) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let e = c - p3(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t);
            e.dot(e).sqrt()
        }).fold(f64::INFINITY, f64::min)
    };
    for (from, to, what) in [(got, auto, "a picked cut leaves the automatic raster"), (auto, got, "an automatic cut is missing")] {
        let segs = cuts(to);
        for (a, b) in cuts(from) {
            let d = b - a;
            let n = ((d.dot(d).sqrt() / STEP).ceil() as usize).max(1);
            for i in 0..=n {
                let t = i as f64 / n as f64;
                let c = p3(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t);
                assert!(off(c, &segs) < 1e-6, "{what} at {c:?}");
            }
        }
    }
}

/// Every move of `t` as the ball centre's segment keeps at least the
/// radius less the tolerance from every facet. Exact: the distance from
/// the segment to each facet, no sampling.
fn no_move_gouges(t: &Toolpath, facets: &[Triangle]) {
    for (i, m) in t.moves.iter().enumerate() {
        let (a, b) = (p3(m.from.x, m.from.y, m.from.z + R), p3(m.to.x, m.to.y, m.to.z + R));
        let d = facets.iter().map(|f| dist_seg_tri(a, b, f)).fold(f64::INFINITY, f64::min);
        assert!(d > R - TOL, "move {i} ({:?}) comes {d:.4} mm from the part: {m:?}", m.kind);
    }
}

/// The ball's vertical column over tip `q` up to the clearance plane
/// keeps the full radius from every facet. Exact, as above.
fn column_clear(q: Point3, clearance: f64, facets: &[Triangle]) -> bool {
    let (a, b) = (p3(q.x, q.y, q.z + R), p3(q.x, q.y, clearance + R));
    facets.iter().all(|f| dist_seg_tri(a, b, f) >= R - 1e-6)
}

fn refusal(facets: &[Triangle], p: &Finish3DParams, angle: f64, s: RasterSeam) -> String {
    finish_seamed(facets, p, angle, &s).err().map(|e| format!("{e:?}")).unwrap_or_default()
}

#[test]
fn finish_without_a_pick_is_the_angled_raster_byte_for_byte() {
    let facets = floor_and_block();
    for w in [false, true] {
        for angle in [0.0, 90.0, 30.0] {
            let p = params(w);
            let (got, ends) = finish_seamed(&facets, &p, angle, &RasterSeam::default()).expect("the seamed raster");
            assert_eq!(got, parallel_with_angle(&facets, &p, angle).expect("the raster"), "angle {angle}, waterlines {w}");
            assert!(ends.is_none());
        }
    }
}

#[test]
fn finish_picks_choose_the_start_and_the_exit_corner_at_any_angle() {
    let facets = floor_and_block();
    let p = params(false);
    for angle in [90.0, 30.0] {
        let auto = parallel_with_angle(&facets, &p, angle).expect("auto");
        let (a, b) = (first_cut(&auto), auto.moves[last_feed(&auto)].to);
        assert!((a.x - b.x).hypot(a.y - b.y) > 20.0, "angle {angle}: the raster's ends stand apart: {a:?} {b:?}");
        let reversed = |got: &Toolpath, ends: Option<RasterEnds>, what: &str| {
            let ends = ends.expect("a raster");
            assert!(same(ends.start, b) && same(ends.exit, a), "angle {angle}, {what}: the whole raster runs back: {ends:?}");
            // The path plunges at the picked start and its last cut ends
            // at the picked exit; the tool rises straight up from there.
            assert!(same(first_cut(got), b), "angle {angle}, {what}: the cut starts at {:?}", first_cut(got));
            let k = last_feed(got);
            assert!(same(got.moves[k].to, a), "angle {angle}, {what}: the cut ends at {:?}", got.moves[k].to);
            let rise = &got.moves[k + 1];
            assert!(rise.from.x == rise.to.x && rise.from.y == rise.to.y && rise.to.z > rise.from.z, "angle {angle}: straight up: {rise:?}");
            // Both moved ends: the plunge column at the new start and the
            // retract column at the new exit.
            assert!(column_clear(ends.start, p.clearance_z, &facets) && column_clear(ends.exit, p.clearance_z, &facets), "angle {angle}: {ends:?}");
            same_cuts(&auto, got);
            no_move_gouges(got, &facets);
        };
        // A start at the automatic end.
        let (got, ends) = finish_seamed(&facets, &p, angle, &seam(pick(b.x + 0.5, b.y - 0.5), None)).expect("started");
        reversed(&got, ends, "start");
        // An exit at the automatic start.
        let (got, ends) = finish_seamed(&facets, &p, angle, &seam(None, pick(a.x - 0.5, a.y + 0.5))).expect("exited");
        reversed(&got, ends, "exit");
        // Both picks, the two ends of one order.
        let (got, ends) = finish_seamed(&facets, &p, angle, &seam(pick(b.x, b.y), pick(a.x, a.y))).expect("both");
        reversed(&got, ends, "both");
        // Picks that keep the automatic order keep the path, move for move.
        let (got, ends) = finish_seamed(&facets, &p, angle, &seam(pick(a.x, a.y), pick(b.x, b.y))).expect("kept");
        assert_eq!(got, auto, "angle {angle}: the automatic corners are the automatic path");
        assert_eq!(ends, Some(RasterEnds { start: a, exit: b }));
    }
}

#[test]
fn finish_with_waterlines_keeps_a_start_and_refuses_an_exit_by_name() {
    let facets = floor_and_block();
    let p = params(true);
    // The rings cut after the raster: a start pick holds, the first cut.
    let (got, ends) = finish_seamed(&facets, &p, 0.0, &seam(pick(40.0, 30.0), None)).expect("a start with rings");
    let ends = ends.expect("a raster");
    assert!(same(first_cut(&got), ends.start), "the cut starts at the corner: {:?}", first_cut(&got));
    // An exit pick cannot end the cut: the rings end it. Refused, never
    // moved; with the rings off the same pick ends the cut.
    let m = refusal(&facets, &p, 0.0, seam(None, pick(40.0, 30.0)));
    assert!(m.contains("exit point (40.000, 30.000)") && m.contains("the waterline rings cut after the raster") && m.contains("turn off the waterline rings"), "{m}");
    finish_seamed(&facets, &params(false), 0.0, &seam(None, pick(40.0, 30.0))).expect("the same exit without rings");
}

#[test]
fn finish_refuses_an_impossible_pick_by_name() {
    let facets = floor_and_block();
    let p = params(false);
    // Each pattern but Parallel refuses the pick by name.
    for pattern in [FinishPattern::Waterline, FinishPattern::ParallelMulti, FinishPattern::Scallop, FinishPattern::Radial, FinishPattern::Ramp] {
        let q = Finish3DParams { pattern, ..p };
        let m = refusal(&facets, &q, 0.0, seam(pick(0.0, 0.0), None));
        assert!(m.contains("start point (0.000, 0.000)") && m.contains("Parallel pattern only"), "{pattern:?}: {m}");
        let m = refusal(&facets, &q, 0.0, seam(None, pick(0.0, 0.0)));
        assert!(m.contains("exit point (0.000, 0.000)") && m.contains("Parallel pattern only"), "{pattern:?}: {m}");
    }
    // A pick in the middle of the raster: no corner is near it.
    let m = refusal(&facets, &p, 90.0, seam(pick(20.0, 26.0), None));
    assert!(m.contains("start point (20.000, 26.000)") && m.contains("only at a corner"), "{m}");
    // Start and exit at one corner: the zigzag cannot end where it began.
    let m = refusal(&facets, &p, 90.0, seam(pick(0.0, 0.0), pick(0.0, 0.0)));
    assert!(m.contains("exit point") && m.contains("a raster that starts at the start point ends at"), "{m}");
    // An angle outside one turn is the raster's own refusal.
    let m = refusal(&facets, &p, 400.0, seam(pick(0.0, 0.0), None));
    assert!(m.contains("direction angle"), "{m}");
    // An all-steep model has no raster to read a pick on.
    let wall = vec![tri((0.0, 0.0, 0.0), (20.0, 0.0, 0.0), (20.0, 0.0, 4.0)), tri((0.0, 0.0, 0.0), (20.0, 0.0, 4.0), (0.0, 0.0, 4.0))];
    let m = refusal(&wall, &params(true), 0.0, seam(pick(0.0, 0.0), None));
    assert!(m.contains("start point (0.000, 0.000)") && m.contains("no pass to read it on"), "{m}");
}

#[test]
fn a_moved_corner_in_the_finished_path_is_refused_by_name() {
    // A later stage (the area controls' clip in the application) that
    // cuts the picked corner away: the finished path is read again.
    let facets = floor_and_block();
    let p = params(false);
    let auto = parallel_with_angle(&facets, &p, 0.0).expect("auto");
    let (a, b) = (first_cut(&auto), auto.moves[last_feed(&auto)].to);
    let s = seam(pick(b.x, b.y), pick(a.x, a.y));
    let (got, ends) = finish_seamed(&facets, &p, 0.0, &s).expect("both picks");
    let ends: RasterEnds = ends.expect("a raster");
    raster_ends_kept(&got, ends, &s, "a clip", "clear it").expect("the path keeps its corners");
    let cut = got.moves.iter().position(|m| m.kind != MoveKind::Rapid).expect("a cut");
    let mut head = got.clone();
    head.moves.drain(cut..cut + 2);
    let m = raster_ends_kept(&head, ends, &s, "a clip", "clear it").err().map(|e| format!("{e:?}")).unwrap_or_default();
    assert!(m.contains("start point (") && m.contains("): a clip, so the cut starts at") && m.contains("clear it"), "{m}");
    let mut tail = got.clone();
    let k = last_feed(&tail);
    tail.moves.truncate(k);
    let m = raster_ends_kept(&tail, ends, &s, "a clip", "clear it").err().map(|e| format!("{e:?}")).unwrap_or_default();
    assert!(m.contains("exit point (") && m.contains("): a clip, so the cut ends at"), "{m}");
}
