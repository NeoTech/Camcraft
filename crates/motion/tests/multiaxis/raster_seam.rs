//! #3330: the user's start and exit points on the 5-axis parallel's
//! zigzag raster (finishing3d::raster_seam), a module of the
//! `multiaxis` suite.

use super::*;
use camcraft_motion::cl::Toolpath;
use camcraft_motion::finishing3d::{RasterEnds, RasterSeam, parallel, parallel_seamed};
use camcraft_motion::multiaxis::parallel5d_seamed;
use camcraft_motion::path_picks::PathPick;

/// The step that samples each move along its length: much finer than
/// any chord a gouge can take.
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

/// The raster's finish parameters: the Parallel pattern, a 4 mm
/// stepover, clearance at 20.
fn raster_params(waterlines: bool) -> Finish3DParams {
    let mut p = finish(5.0, 0.0);
    p.pattern = FinishPattern::Parallel;
    p.stepover = Some(4.0);
    p.waterlines = waterlines;
    p
}

fn pick(x: f64, y: f64) -> PathPick {
    PathPick::xy(camcraft_motion::geo2d::Point2::new(x, y))
}

fn seam(start: Option<PathPick>, exit: Option<PathPick>) -> RasterSeam {
    RasterSeam { start, exit }
}

fn run(facets: &[Triangle], p: &Finish3DParams, s: &RasterSeam) -> Result<(Toolpath, Option<RasterEnds>), GenError> {
    parallel5d_seamed(facets, p, &control(15.0, 0.0, Some(60.0)), None, s)
}

/// Each move's (kind, start centre, end centre): the tip backed up `R`
/// along the pose the machine holds (the pivot law).
fn centres(t: &Toolpath) -> Vec<(MoveKind, Point3, Point3)> {
    let mut held = [0.0, 0.0, 1.0];
    t.moves
        .iter()
        .map(|m| {
            let from = centre_of(m.from, held);
            held = m.axis.unwrap_or(held);
            (m.kind, from, centre_of(m.to, held))
        })
        .collect()
}

fn up(q: Point3) -> Point3 {
    p3(q.x, q.y, q.z + R)
}

/// `a + d·t` (the geometry type carries only the difference).
fn at(a: Point3, d: Point3, t: f64) -> Point3 {
    p3(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t)
}

fn near(a: Point3, b: Point3, tol: f64) -> bool {
    let d = a - b;
    d.dot(d).sqrt() < tol
}

/// The closest point to `c` on triangle `t` (the Voronoi-region walk
/// of the textbook closest-point query).
fn closest(c: Point3, t: &Triangle) -> Point3 {
    let (a, b, cc) = (t.a, t.b, t.c);
    let (ab, ac, ap) = (b - a, cc - a, c - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = c - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return at(a, ab, v);
    }
    let cp = c - cc;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return cc;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return at(a, ac, w);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return at(b, cc - b, w);
    }
    let den = 1.0 / (va + vb + vc);
    at(at(a, ab, vb * den), ac, vc * den)
}

/// The gap from ball centre `c` to the nearest facet, less the radius.
fn clearance(c: Point3, facets: &[Triangle]) -> f64 {
    facets.iter().map(|t| {
        let e = c - closest(c, t);
        e.dot(e).sqrt()
    }).fold(f64::INFINITY, f64::min) - R
}

/// Every move of `t`, sampled every [`STEP`] mm along its length: the
/// ball never sinks more than the tolerance into a facet. A chord
/// through the part (a gouge, a link through a wall) fails here.
fn no_move_gouges(t: &Toolpath, facets: &[Triangle]) {
    for (k, a, b) in centres(t) {
        let d = b - a;
        let n = ((d.dot(d).sqrt() / STEP).ceil() as usize).max(1);
        for i in 0..=n {
            let c = at(a, d, i as f64 / n as f64);
            let g = clearance(c, facets);
            assert!(g > -TOL, "a {k:?} move sinks {:.4} mm into the part at {c:?}", -g);
        }
    }
}

/// The ball centre segments of the raster cuts: the feed moves that are
/// not vertical (a vertical feed is a hop up, out of the cut).
fn raster_cuts(t: &Toolpath) -> Vec<(Point3, Point3)> {
    centres(t).into_iter().filter(|(k, a, b)| *k == MoveKind::Feed && (a.x - b.x).hypot(a.y - b.y) > 1e-9).map(|(_, a, b)| (a, b)).collect()
}

/// Every cut of `got` rides a cut of `auto` along its whole length
/// (sampled every [`STEP`] mm): the pick changes the order and the
/// direction, never the geometry.
fn cuts_ride_the_automatic_raster(auto: &Toolpath, got: &Toolpath) {
    let segs = raster_cuts(auto);
    let off = |c: Point3| segs.iter().map(|&(a, b)| {
        let d = b - a;
        let l2 = d.dot(d);
        let t = if l2 > 0.0 { ((c - a).dot(d) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let e = c - at(a, d, t);
        e.dot(e).sqrt()
    }).fold(f64::INFINITY, f64::min);
    for (a, b) in raster_cuts(got) {
        let d = b - a;
        let n = ((d.dot(d).sqrt() / STEP).ceil() as usize).max(1);
        for i in 0..=n {
            let c = at(a, d, i as f64 / n as f64);
            let e = off(c);
            assert!(e < 1e-6, "a cut stands {e} mm off the automatic raster at {c:?}");
        }
    }
    assert_eq!(raster_cuts(got).len(), raster_cuts(auto).len(), "the pick adds and drops no cut");
}

/// The first plunge's end centre: where the cut starts.
fn first_plunge(t: &Toolpath) -> Point3 {
    centres(t).into_iter().find(|(k, _, _)| *k == MoveKind::Plunge).map(|(_, _, b)| b).expect("a plunge")
}

/// The automatic raster's four corners, read off its ends: `a` where it
/// starts and `b` where it ends; the two picks that reverse it whole
/// must give back `b` and `a`.
fn auto_ends(facets: &[Triangle], p: &Finish3DParams) -> RasterEnds {
    run(facets, p, &RasterSeam::default()).expect("the automatic raster").1.expect("a raster")
}

#[test]
fn raster_without_a_pick_is_the_automatic_path_byte_for_byte() {
    let facets = floor_and_block();
    for w in [false, true] {
        let p = raster_params(w);
        let auto = parallel(&facets, &p).expect("the raster");
        let (got, ends) = parallel_seamed(&facets, &p, &RasterSeam::default()).expect("the seamed raster");
        assert_eq!(got, auto, "an empty seam is the automatic raster (waterlines {w})");
        assert!(ends.is_some());
        let axis = control(15.0, 0.0, Some(60.0));
        let (posed, _) = run(&facets, &p, &RasterSeam::default()).expect("posed");
        assert_eq!(posed, parallel5d(&facets, &p, &axis, None).expect("auto"));
        // The automatic order stands by itself, not only by its twin: it
        // plunges at its own corner, ball centre (40, 1, 3) (the end of
        // the first row; its direction is the rim row's parity).
        let s = first_plunge(&posed);
        assert!(near(s, p3(40.0, 1.0, 3.0), 1e-6), "the automatic cut starts at its own corner: {s:?}");
    }
}

#[test]
fn raster_start_point_starts_the_cut_at_the_nearest_corner() {
    let facets = floor_and_block();
    let p = raster_params(true);
    let (auto, _) = run(&facets, &p, &RasterSeam::default()).expect("auto");
    let e0 = auto_ends(&facets, &p);
    assert!(near(first_plunge(&auto), up(e0.start), 1e-6), "the automatic cut starts at its first corner");
    // The far end of the first pass: every pass runs the other way.
    let (x_far, y_first) = (40.0 - e0.start.x, e0.start.y);
    let (got, ends) = run(&facets, &p, &seam(Some(pick(x_far, y_first)), None)).expect("started");
    let ends = ends.expect("a raster");
    assert!((ends.start.y - y_first).abs() < 1e-6 && (ends.start.x - e0.start.x).abs() > 20.0, "the cut starts at the far end of the first pass: {ends:?}");
    assert!(near(first_plunge(&got), up(ends.start), 1e-6), "the path plunges at the chosen corner");
    let first_cut = raster_cuts(&got)[0];
    let auto_cut = raster_cuts(&auto)[0];
    assert!((first_cut.1.x - first_cut.0.x).signum() == -(auto_cut.1.x - auto_cut.0.x).signum(), "the first pass runs the other way");
    cuts_ride_the_automatic_raster(&auto, &got);
    no_move_gouges(&got, &facets);
    // The automatic end: the passes run in the reverse order and way.
    let (got, ends) = run(&facets, &p, &seam(Some(pick(e0.exit.x, e0.exit.y)), None)).expect("reversed");
    let ends = ends.expect("a raster");
    assert!(near(ends.start, e0.exit, 1e-6) && near(ends.exit, e0.start, 1e-6), "the whole raster runs back: {ends:?}");
    assert!(near(first_plunge(&got), up(e0.exit), 1e-6));
    cuts_ride_the_automatic_raster(&auto, &got);
    no_move_gouges(&got, &facets);
}

#[test]
fn raster_exit_point_ends_the_last_pass_at_the_nearest_corner() {
    let facets = floor_and_block();
    let p = raster_params(false);
    let (auto, _) = run(&facets, &p, &RasterSeam::default()).expect("auto");
    let e0 = auto_ends(&facets, &p);
    let (got, ends) = run(&facets, &p, &seam(None, Some(pick(e0.start.x, e0.start.y)))).expect("exited");
    let ends = ends.expect("a raster");
    assert!(near(ends.exit, e0.start, 1e-6), "the last pass ends at the picked corner: {ends:?}");
    // The last cut ends there, and the tool rises straight up from it.
    let c = centres(&got);
    let last = c.iter().rposition(|(k, a, b)| *k == MoveKind::Feed && (a.x - b.x).hypot(a.y - b.y) > 1e-9).expect("a cut");
    assert!(near(c[last].2, up(ends.exit), 1e-6), "the last cut ends at the exit corner: {:?}", c[last]);
    let rise = &c[last + 1];
    assert!(near(p3(rise.1.x, rise.1.y, 0.0), p3(rise.2.x, rise.2.y, 0.0), 1e-6) && rise.2.z > rise.1.z, "the tool leaves straight up: {rise:?}");
    cuts_ride_the_automatic_raster(&auto, &got);
    no_move_gouges(&got, &facets);
    // Start and exit together, the two ends of one order.
    let (got, ends) = run(&facets, &p, &seam(Some(pick(e0.exit.x, e0.exit.y)), Some(pick(e0.start.x, e0.start.y)))).expect("both");
    assert_eq!(ends, Some(RasterEnds { start: e0.exit, exit: e0.start }));
    no_move_gouges(&got, &facets);
}

#[test]
fn raster_impossible_picks_are_refused_by_name() {
    let facets = floor_and_block();
    let p = raster_params(true);
    let e0 = auto_ends(&facets, &p);
    let refusal = |s: RasterSeam| run(&facets, &p, &s).err().map(|e| format!("{e:?}")).unwrap_or_default();
    // A pick in the middle of the raster: no corner is near it.
    let m = refusal(seam(Some(pick(20.0, 26.0)), None));
    assert!(m.contains("start point (20.000, 26.000)") && m.contains("only at a corner") && m.contains("farther than"), "{m}");
    let m = refusal(seam(None, Some(pick(20.0, 26.0))));
    assert!(m.contains("exit point (20.000, 26.000)") && m.contains("farther than"), "{m}");
    // A pick with a Z is read in 3D: right over a corner, 30 mm high.
    let high = PathPick { z: Some(e0.start.z + 30.0), ..pick(e0.start.x, e0.start.y) };
    let m = refusal(seam(Some(high), None));
    assert!(m.contains("only at a corner"), "{m}");
    // Start and exit at the same corner: the zigzag cannot end where it began.
    let m = refusal(seam(Some(pick(e0.start.x, e0.start.y)), Some(pick(e0.start.x, e0.start.y))));
    assert!(m.contains("exit point") && m.contains("a raster that starts at the start point ends at"), "{m}");
    // A seam on a pattern that is not the one-direction raster.
    let mut multi = p;
    multi.pattern = FinishPattern::ParallelMulti;
    let m = parallel_seamed(&facets, &multi, &seam(Some(pick(0.0, 0.0)), None)).err().map(|e| format!("{e:?}")).unwrap_or_default();
    assert!(m.contains("one-direction Parallel raster only"), "{m}");
}

/// The floor with a thin plate leaning at 50 degrees over its
/// (0, 0) corner, z 10 to 20: steep both ways, so the raster never
/// rides it, but a plunge column under it crosses it.
fn floor_under_a_canopy() -> Vec<Triangle> {
    let mut f = vec![tri((0.0, 0.0, 0.0), (40.0, 0.0, 0.0), (40.0, 30.0, 0.0)), tri((0.0, 0.0, 0.0), (40.0, 30.0, 0.0), (0.0, 30.0, 0.0))];
    let x1 = 10.0 / 50f64.to_radians().tan();
    f.push(tri((0.0, -5.0, 10.0), (x1, -5.0, 20.0), (x1, 10.0, 20.0)));
    f.push(tri((0.0, -5.0, 10.0), (x1, 10.0, 20.0), (0.0, 10.0, 10.0)));
    f
}

/// The plunge column over tip `q`: the ball swept from the tip up to
/// the clearance plane every [`STEP`] mm keeps off every facet.
fn column_is_clear(q: Point3, top: f64, facets: &[Triangle]) -> bool {
    let n = (((top - q.z) / STEP).ceil() as usize).max(1);
    (0..=n).all(|i| clearance(p3(q.x, q.y, q.z + R + (top - q.z) * i as f64 / n as f64), facets) > -0.01)
}

#[test]
fn raster_picked_columns_under_a_canopy_stay_clear_or_are_refused_by_name() {
    let facets = floor_under_a_canopy();
    let mut p = raster_params(false);
    p.clearance_z = 30.0;
    // The corner under the plate is no raster corner: the raster's
    // column law trimmed every sample there. A pick at it is refused.
    let m = run(&facets, &p, &seam(Some(pick(0.0, 0.0)), None)).err().map(|e| format!("{e:?}")).unwrap_or_default();
    assert!(m.contains("start point (0.000, 0.000)") && m.contains("farther than"), "{m}");
    let m = run(&facets, &p, &seam(None, Some(pick(0.0, 0.0)))).err().map(|e| format!("{e:?}")).unwrap_or_default();
    assert!(m.contains("exit point (0.000, 0.000)") && m.contains("farther than"), "{m}");
    // A pick beside the plate starts at the first sample outside it,
    // and the column there, and at the exit, is clear to clearance.
    for s in [seam(Some(pick(6.0, 0.0)), None), seam(None, Some(pick(6.0, 0.0)))] {
        let (got, ends) = run(&facets, &p, &s).expect("a pick beside the plate");
        let ends = ends.expect("a raster");
        let at = if s.start.is_some() { ends.start } else { ends.exit };
        assert!(at.x > 6.0 && at.y.abs() < 4.0, "the corner is outside the plate's shadow: {at:?}");
        for q in [ends.start, ends.exit] {
            assert!(column_is_clear(q, p.clearance_z, &facets), "the column over {q:?} crosses the plate");
        }
        no_move_gouges(&got, &facets);
    }
}
