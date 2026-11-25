//! #3330: the user's start and exit points on the drive-curve contour
//! (multiaxis::curve_seam), a module of the `multiaxis` suite. Every
//! pin that says "the cut geometry does not change" samples each cut
//! move along its length, in both directions: no picked cut leaves the
//! automatic path, and no automatic cut is left out.

use super::*;
use camcraft_motion::cl::Toolpath;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::geometry::cube;
use camcraft_motion::multiaxis::{contour_curve, contour_curve_seamed, CurveParams, CurveSeam};
use camcraft_motion::path_picks::PathPick;

/// The step that samples each cut move along its length: much finer
/// than any chord a gouge can take across the box.
const STEP: f64 = 0.25;

/// How far a sample may stand from the other path's ball centres: the
/// walk's chord budget (0.0025 mm) on each side, with room for the
/// stations that a seat vertex moves along the same locus.
const RIDE: f64 = 0.02;

fn box_part() -> Vec<Triangle> {
    cube(p3(0.0, 0.0, 0.0), p3(40.0, 40.0, 15.0)).facets.to_vec()
}

/// The box with a canopy slab above the middle of its y = 0 wall.
fn canopy_part() -> Vec<Triangle> {
    let mut f = box_part();
    f.extend(cube(p3(15.0, -8.0, 18.0), p3(25.0, -1.0, 20.0)).facets.iter().copied());
    f
}

/// The closed curve round the box's walls at mid height.
fn ring() -> Vec<Point3> {
    [(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0), (0.0, 0.0)].iter().map(|&(x, y)| p3(x, y, 7.5)).collect()
}

/// The open curve on the box's top front edge.
fn edge() -> Vec<Point3> {
    vec![p3(5.0, 0.0, 15.0), p3(20.0, 0.0, 15.0), p3(35.0, 0.0, 15.0)]
}

fn cp() -> CurveParams {
    CurveParams { tool_diameter: 6.0, clearance_z: 25.0, retract_z: None, feed: 500.0, plunge_feed: 100.0, climb: true }
}

fn pick(x: f64, y: f64) -> Option<PathPick> {
    Some(PathPick::xy(Point2::new(x, y)))
}

fn run(chains: &[Vec<Point3>], facets: &[Triangle], start: Option<PathPick>, exit: Option<PathPick>) -> Result<Toolpath, GenError> {
    contour_curve_seamed(chains, facets, &cp(), &AxisControl::default(), None, &CurveSeam { start, exit })
}

/// Each move's ball centre segment: the tip backed up R along the pose
/// the machine holds there (the pivot law).
fn centres(t: &Toolpath) -> Vec<(MoveKind, Point3, Point3)> {
    let mut held = [0.0, 0.0, 1.0];
    t.moves
        .iter()
        .map(|m| {
            let a = p3(m.from.x + R * held[0], m.from.y + R * held[1], m.from.z + R * held[2]);
            held = m.axis.unwrap_or(held);
            (m.kind, a, p3(m.to.x + R * held[0], m.to.y + R * held[1], m.to.z + R * held[2]))
        })
        .collect()
}

/// The ball centre segments of the cut moves.
fn cuts(t: &Toolpath) -> Vec<(Point3, Point3)> {
    centres(t).into_iter().filter(|(k, _, _)| *k == MoveKind::Feed).map(|(_, a, b)| (a, b)).collect()
}

/// Each chain's cut: the ball centres from its plunge to its last feed.
fn runs(t: &Toolpath) -> Vec<Vec<Point3>> {
    let mut out: Vec<Vec<Point3>> = Vec::new();
    for (k, _, b) in centres(t) {
        match k {
            MoveKind::Plunge => out.push(vec![b]),
            MoveKind::Feed => {
                if let Some(r) = out.last_mut() {
                    r.push(b);
                }
            }
            _ => {}
        }
    }
    out
}

fn off(segs: &[(Point3, Point3)], c: Point3) -> f64 {
    segs.iter()
        .map(|&(a, b)| {
            let d = b - a;
            let l2 = d.dot(d);
            let t = if l2 > 0.0 { ((c - a).dot(d) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let e = c - p3(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t);
            e.dot(e).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

/// Every cut of `got` rides a cut of `auto` along its whole length (no
/// chord, no gouge), and every cut of `auto` rides a cut of `got` (no
/// part of the curve is left uncut). Each move is sampled every STEP.
fn same_cut(auto: &Toolpath, got: &Toolpath) {
    let (a, g) = (cuts(auto), cuts(got));
    for (from, to, what) in [(&g, &a, "a picked cut leaves the automatic path"), (&a, &g, "an automatic cut is left out")] {
        for &(p, q) in from {
            let len = (q - p).dot(q - p).sqrt();
            let n = ((len / STEP).ceil() as usize).max(1);
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let c = p3(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t, p.z + (q.z - p.z) * t);
                let e = off(to, c);
                assert!(e < RIDE, "{what}: {e:.4} mm at {c:?}");
            }
        }
    }
}

fn near(c: Point3, x: f64, y: f64, z: f64) -> bool {
    (c.x - x).abs() < 0.05 && (c.y - y).abs() < 0.05 && (c.z - z).abs() < 1e-6
}

#[test]
fn curve_start_point_seats_a_closed_curve_at_the_pick() {
    let facets = box_part();
    let auto = contour_curve(&[ring()], &facets, &cp(), &AxisControl::default(), None).expect("automatic ring");
    assert!(!near(runs(&auto)[0][0], 20.0, -R, 7.5), "the fixture starts away from the pick");
    let got = run(&[ring()], &facets, pick(20.0, -1.0), None).expect("seated ring");
    let r = runs(&got);
    assert_eq!(r.len(), 1, "the pick seats the curve, it adds no run");
    let (first, last) = (r[0][0], r[0][r[0].len() - 1]);
    assert!(near(first, 20.0, -R, 7.5), "the cut does not start at the pick: {first:?}");
    assert!((first - last).dot(first - last).sqrt() < 1e-9, "the seated ring does not come home: {last:?}");
    same_cut(&auto, &got);
}

#[test]
fn curve_exit_point_carries_a_closed_curve_on_to_the_pick() {
    let facets = box_part();
    let auto = contour_curve(&[ring()], &facets, &cp(), &AxisControl::default(), None).expect("automatic ring");
    // Start and exit: the whole lap from the start, then on to the exit.
    let got = run(&[ring()], &facets, pick(20.0, -1.0), pick(-1.0, 20.0)).expect("picked ring");
    let r = runs(&got);
    let (first, last) = (r[0][0], r[0][r[0].len() - 1]);
    assert!(near(first, 20.0, -R, 7.5), "the cut does not start at the start pick: {first:?}");
    assert!(near(last, -R, 20.0, 7.5), "the cut does not leave at the exit pick: {last:?}");
    same_cut(&auto, &got);
    // Exit alone: the automatic seam, the whole lap, then on to the exit.
    let got = run(&[ring()], &facets, None, pick(-1.0, 20.0)).expect("exit-only ring");
    let r = runs(&got);
    assert_eq!(r[0][0], runs(&auto)[0][0], "an exit pick keeps the automatic start");
    assert!(near(r[0][r[0].len() - 1], -R, 20.0, 7.5), "the cut does not leave at the exit pick");
    same_cut(&auto, &got);
}

#[test]
fn curve_picks_on_an_open_curve_select_its_ends() {
    let facets = box_part();
    let auto = contour_curve(&[edge()], &facets, &cp(), &AxisControl::default(), None).expect("automatic edge");
    assert!(runs(&auto)[0][0].x < 5.1, "the automatic cut starts at x = 5");
    // The start pick beside the x = 35 end: the cut starts there.
    let got = run(&[edge()], &facets, pick(36.0, -1.0), None).expect("started edge");
    let r = &runs(&got)[0];
    assert!(r[0].x > 34.9 && r[r.len() - 1].x < 5.1, "the cut does not run from the picked end: {:?} .. {:?}", r[0], r[r.len() - 1]);
    same_cut(&auto, &got);
    // The exit pick beside the x = 5 end: the cut leaves there.
    let got = run(&[edge()], &facets, None, pick(4.0, -1.0)).expect("exit edge");
    let r = &runs(&got)[0];
    assert!(r[0].x > 34.9 && r[r.len() - 1].x < 5.1, "the cut does not leave at the picked end");
    same_cut(&auto, &got);
    // Both, at the two ends: the same cut.
    let both = run(&[edge()], &facets, pick(36.0, -1.0), pick(4.0, -1.0)).expect("both ends");
    assert_eq!(both, got, "start and exit at the two ends cut the curve once, end to end");
}

#[test]
fn curve_pick_acts_on_its_nearest_curve_only() {
    let facets = box_part();
    let auto = contour_curve(&[ring(), edge()], &facets, &cp(), &AxisControl::default(), None).expect("automatic pair");
    // In XY the pick stands nearer the ring's wall (1 mm) than the edge's
    // end; its Z (the edge's height) reads it in 3D, and the edge wins.
    let in_xy = run(&[ring(), edge()], &facets, pick(36.0, -1.0), None).expect("pick read in XY");
    assert_eq!(runs(&in_xy)[1], runs(&auto)[1], "an XY pick here seats the ring, not the edge");
    let at = Some(PathPick { xy: Point2::new(36.0, -1.0), z: Some(15.0) });
    let got = run(&[ring(), edge()], &facets, at, None).expect("pick on the edge");
    let (a, g) = (runs(&auto), runs(&got));
    assert_eq!(g.len(), 2);
    assert_eq!(g[0], a[0], "the ring, farther from the pick, keeps its automatic cut");
    assert!(g[1][0].x > 34.9, "the edge starts at the picked end");
}

#[test]
fn curve_impossible_picks_are_refused_by_name() {
    let facets = box_part();
    let cases = [
        (vec![ring()], pick(90.0, 90.0), None, "start point (90.000, 90.000): the nearest drive curve stands"),
        (vec![ring()], None, pick(90.0, 90.0), "exit point (90.000, 90.000): the nearest drive curve stands"),
        (vec![edge()], pick(20.0, -1.0), None, "start point (20.000, -1.000): drive curve 1 is open and its cut can start only at an end"),
        (vec![edge()], None, pick(20.0, -1.0), "exit point (20.000, -1.000): drive curve 1 is open and its cut can leave only at an end"),
        (vec![edge()], pick(36.0, -1.0), pick(34.0, -1.0), "exit point (34.000, -1.000): it is nearer the end of open drive curve 1 where the start point begins"),
    ];
    for (chains, start, exit, want) in cases {
        let e = run(&chains, &facets, start, exit).err().map(|e| format!("{e}"));
        assert!(e.as_deref().is_some_and(|m| m.contains(want)), "want {want:?}, got {e:?}");
    }
}

#[test]
fn curve_picked_columns_under_a_canopy_are_refused_by_name() {
    let facets = canopy_part();
    // The automatic seam and a pick with open air above it still cut.
    assert!(run(&[ring()], &facets, None, None).is_ok(), "the automatic ring cuts beside the canopy");
    assert!(run(&[ring()], &facets, pick(-1.0, 20.0), pick(-1.0, 20.0)).is_ok(), "a pick with open air above it cuts");
    let e = run(&[ring()], &facets, pick(20.0, -1.0), None).err().map(|e| format!("{e}"));
    assert!(
        e.as_deref().is_some_and(|m| m.contains("start point (20.000, -1.000): the plunge column over drive curve 1 at (")),
        "a plunge under the canopy: {e:?}"
    );
    let e = run(&[ring()], &facets, None, pick(20.0, -1.0)).err().map(|e| format!("{e}"));
    assert!(
        e.as_deref().is_some_and(|m| m.contains("exit point (20.000, -1.000): the retract column over drive curve 1 at (")),
        "a retract under the canopy: {e:?}"
    );
}

#[test]
fn curve_without_a_pick_is_the_automatic_path_byte_for_byte() {
    let facets = box_part();
    let chains = [ring(), edge()];
    let auto = contour_curve(&chains, &facets, &cp(), &AxisControl::default(), None).expect("automatic");
    assert_eq!(run(&chains, &facets, None, None).expect("empty seam"), auto);
}
