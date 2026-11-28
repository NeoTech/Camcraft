//! #4176 (second part): the kernel's trace, medial and helix guides
//! (`camcraft-pathkernel`, K2) against the recorded old output (the #4153
//! `kernel_*` goldens). Only the guide geometry is compared: the places of
//! the cut moves at a cut level, the radii of the rings and the radius,
//! pitch and hand of a thread helix. The order, the links, the entries and
//! the levels are stages 4 to 6. Where the old output is correct the
//! kernel guide lies on it. The named differences:
//! - Bore: the old rings are spaced evenly (`rc * k / count`); the kernel
//!   spaces them at the stepover from the wall, so the recipe gives the
//!   stepover `rc / count` to get the same rings;
//! - Bore and Circular: the old rings are 256 or 2^n chords; the kernel
//!   rings are exact circles with a chord count from the tolerance (#3756);
//! - the wide slot: the old plan cuts rows across the centre band (y = -4,
//!   -1, 2, 4); the medial guide is the one centre line (y = 0) of the
//!   band, inside the old rows: rows across a slot are the row family;
//! - Engrave with an avoid item: not compared, because the old keep-out
//!   comes from an avoid item with its own pad; the kernel takes the
//!   keep-out as a value. The case without avoid is compared.

use camcraft_motion::geo2d::Point2;
use camcraft_motion::pathkernel::guide::{
    Circle, Curve, GuideChoice, GuidePlanner, GuideSet, Guides, Hand, HelixChoice, RingRef, RingsChoice, TraceOffset,
};
use camcraft_motion::pathkernel::{
    Bounds, Chain, MaterialSide, Region, Selection, ToolShape, WorkPlane, chain_region, region,
};

const TOL: f64 = 1e-3;
/// Two paths agree where their points are this close (mm).
const AGREE: f64 = 0.02;

/// One cut move of a golden: its end, and the centre and the sense of an arc.
#[derive(Clone, Copy, Debug)]
struct Move {
    to: (f64, f64, f64),
    arc: Option<(Point2, bool)>,
}

/// The `Plunge`, `Feed` and `Arc` moves of a golden CL text (coordinates
/// in um): a plunge ends at the cut level.
fn moves(name: &str) -> Vec<Move> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    let text = std::fs::read_to_string(&file).expect("golden");
    let mut out = Vec::new();
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| w.get(i).and_then(|s| s.parse::<f64>().ok()).map(|v| v / 1000.0);
        let tag = |c: char| w.iter().find_map(|t| t.strip_prefix(c).and_then(|v| v.parse::<f64>().ok()));
        let (Some(x), Some(y), Some(z)) = (num(1), num(2), num(3)) else { continue };
        match w.first() {
            Some(&"Feed" | &"Plunge") => out.push(Move { to: (x, y, z), arc: None }),
            Some(&"Arc") => {
                let c = Point2::new(tag('I').unwrap_or(0.0) / 1000.0, tag('J').unwrap_or(0.0) / 1000.0);
                out.push(Move { to: (x, y, z), arc: Some((c, tag('S').unwrap_or(0.0) > 0.0)) });
            }
            _ => {}
        }
    }
    out
}

/// The ends of the cut moves at the level `z`.
fn at_level(name: &str, z: f64) -> Vec<Point2> {
    moves(name).into_iter().filter(|m| (m.to.2 - z).abs() < 5e-4).map(|m| Point2::new(m.to.0, m.to.1)).collect()
}

fn seg(q: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (q.x - a.x - t * dx).hypot(q.y - a.y - t * dy)
}

fn walk(c: &Curve) -> Vec<Point2> {
    let mut v: Vec<Point2> = c.pts().iter().map(|p| p.at()).collect();
    if c.closed()
        && let Some(&f) = v.first()
    {
        v.push(f);
    }
    v
}

fn to_set(set: &GuideSet, q: Point2) -> f64 {
    set.curves().iter().flat_map(|c| walk(c).windows(2).map(|w| seg(q, w[0], w[1])).collect::<Vec<_>>()).fold(f64::INFINITY, f64::min)
}

fn to_points(pts: &[Point2], q: Point2) -> f64 {
    pts.iter().map(|p| (p.x - q.x).hypot(p.y - q.y)).fold(f64::INFINITY, f64::min)
}

fn guide(r: &camcraft_motion::pathkernel::KernelRegion, g: GuideChoice) -> GuideSet {
    let v = Guides.guide(r, &g, TOL).expect("guide");
    assert!(v.postable(), "{v:?}");
    v.out().expect("out").clone()
}

fn xy() -> WorkPlane {
    WorkPlane::setup_xy(0.0).expect("plane")
}

fn tool(r: f64) -> ToolShape {
    ToolShape::disc(r, 0.0, true, 30.0).expect("tool")
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Point2> {
    vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]
}

fn disc(cx: f64, cy: f64, r: f64, n: u32) -> Vec<Point2> {
    (0..n).map(|k| f64::from(k) * std::f64::consts::TAU / f64::from(n)).map(|a| Point2::new(cx + r * a.cos(), cy + r * a.sin())).collect()
}

/// A slot of half-width `w` round (0, 0)-(`len`, 0), counter-clockwise.
fn stadium(len: f64, w: f64) -> Vec<Point2> {
    let half = |cx: f64, a0: f64| (0..=64).map(move |i| a0 + std::f64::consts::PI * f64::from(i) / 64.0).map(move |a| Point2::new(cx + w * a.cos(), w * a.sin()));
    half(len, -std::f64::consts::FRAC_PI_2).chain(half(0.0, std::f64::consts::FRAC_PI_2)).collect()
}

/// A trace on the chains themselves (offset 0): every old cut point is on a
/// kernel curve, and every kernel vertex is an old cut point.
fn trace_agrees(chains: Vec<Chain>, golden: &str, z: f64) {
    let r = chain_region(&Selection::new(chains).expect("sel"), xy(), tool(1.0), Region::empty()).expect("region");
    let set = guide(&r, GuideChoice::Trace(TraceOffset::new(0.0).expect("offset")));
    let old = at_level(golden, z);
    assert!(!old.is_empty(), "{golden}: no cut at {z}");
    for q in &old {
        assert!(to_set(&set, *q) <= AGREE, "{golden}: old point {q:?} is off the kernel trace");
    }
    for q in set.curves().iter().flat_map(|c| c.pts().iter().map(|p| p.at())) {
        assert!(to_points(&old, q) <= AGREE, "{golden}: kernel vertex {q:?} is not an old cut point");
    }
}

#[test]
fn kernel_trace_is_the_old_trace_of_the_plate_and_the_round_loop() {
    let plate = vec![Chain::closed(rect(-30.0, -20.0, 30.0, 20.0)).unwrap(), Chain::closed(rect(-6.0, -6.0, 6.0, 6.0)).unwrap()];
    trace_agrees(plate, "kernel_trace_plate_climb.cl", -2.0);
    trace_agrees(vec![Chain::closed(disc(0.0, 0.0, 15.0, 128)).unwrap()], "kernel_trace_round.cl", -2.0);
}

#[test]
fn kernel_engrave_lines_are_the_old_engrave_without_an_avoid_item() {
    let squares = vec![Chain::closed(rect(-28.0, -12.0, -12.0, 12.0)).unwrap(), Chain::closed(rect(12.0, -12.0, 28.0, 12.0)).unwrap()];
    trace_agrees(squares, "kernel_engrave_avoid_empty.cl", -2.0);
}

#[test]
fn kernel_slot_lines_are_the_old_slot_centre() {
    // A slot as wide as the tool: the trace of its centre segment.
    let line = vec![Chain::open(vec![Point2::new(0.0, 0.0), Point2::new(60.0, 0.0)]).unwrap()];
    trace_agrees(line, "kernel_slot_plunge.cl", -3.0);
    // The wide slot (18 mm, tool 10): the medial line of the centre band
    // (half-width 4) is the centre segment, inside the old rows.
    let outline = stadium(60.0, 9.0);
    let sel = Selection::face(outline, vec![]).expect("slot");
    let b = Bounds::new(None, Region::empty(), 0.0, -6.0).expect("bounds");
    let r = region(&sel, MaterialSide::Inside, &b, xy(), tool(5.0), 0.0).expect("region");
    let set = guide(&r, GuideChoice::Medial);
    assert_eq!(set.curves().len(), 1, "{set:?}");
    let pts: Vec<Point2> = set.curves()[0].pts().iter().map(|p| p.at()).collect();
    assert!(pts.iter().all(|q| q.y.abs() <= 2.0 * TOL), "{pts:?}");
    let (x0, x1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.x), b.max(q.x)));
    assert!(x0.abs() <= 0.1 && (x1 - 60.0).abs() <= 0.1, "{x0} {x1}");
    let old = at_level("kernel_slot_wide.cl", -3.0);
    let (lo, hi) = old.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.y), b.max(q.y)));
    assert!((lo + 4.0).abs() < 1e-3 && (hi - 4.0).abs() < 1e-3, "the old rows span the band: {lo} {hi}");
}

/// The radii of the old arcs about `c` at level `z`, largest first.
fn old_radii(golden: &str, c: Point2, z: f64) -> Vec<f64> {
    let mut r: Vec<f64> = moves(golden)
        .into_iter()
        .filter(|m| (m.to.2 - z).abs() < 5e-4)
        .filter_map(|m| m.arc.filter(|(o, _)| (o.x - c.x).hypot(o.y - c.y) < 1e-3).map(|_| (m.to.0 - c.x).hypot(m.to.1 - c.y)))
        .collect();
    r.sort_by(|a, b| b.total_cmp(a));
    r.dedup_by(|a, b| (*a - *b).abs() < 2e-3);
    r
}

fn ring_radii(set: &GuideSet) -> Vec<f64> {
    let mut r: Vec<f64> = set.curves().iter().map(|c| c.arc().expect("exact circle").radius()).collect();
    r.sort_by(|a, b| b.total_cmp(a));
    r
}

fn same(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 2e-3)
}

#[test]
fn kernel_circle_rings_have_the_old_radii_of_circular_and_bore() {
    // Circular: hole 15, tool 8, step 3: centre radius 11.
    let hole = |c: Point2, r: f64, t: f64| {
        let sel = Selection::face(disc(c.x, c.y, r, 1024), vec![]).expect("hole");
        let b = Bounds::new(None, Region::empty(), 0.0, -4.0).expect("bounds");
        region(&sel, MaterialSide::Inside, &b, xy(), tool(t), 0.0).expect("region")
    };
    let o = Point2::new(0.0, 0.0);
    let rings = |c: Point2, rc: f64, step: f64| {
        GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, step, Some(Circle::new(c, rc).expect("circle")), None).expect("rings"))
    };
    let set = guide(&hole(o, 15.0, 4.0), rings(o, 11.0, 3.0));
    for z in [-2.0, -4.0] {
        let old = old_radii("kernel_circular_auto.cl", o, z);
        assert!(same(&ring_radii(&set), &old), "Circular at {z}: kernel {:?} against old {old:?}", ring_radii(&set));
    }
    // Bore: hole 10 at (5, 5), tool 6: centre radius 7, three even rings.
    let c = Point2::new(5.0, 5.0);
    let set = guide(&hole(c, 10.0, 3.0), rings(c, 7.0, 7.0 / 3.0));
    for z in [-2.0, -4.0] {
        let old = old_radii("kernel_bore_auto.cl", c, z);
        assert!(same(&ring_radii(&set), &old), "Bore at {z}: kernel {:?} against old {old:?}", ring_radii(&set));
    }
}

#[test]
fn kernel_thread_helix_has_the_old_radius_pitch_and_hand() {
    let c = Point2::new(10.0, 20.0);
    for (golden, hand) in [("kernel_thread_internal_right_climb.cl", Hand::Right), ("kernel_thread_internal_left_climb.cl", Hand::Left)] {
        let arcs: Vec<Move> = moves(golden).into_iter().filter(|m| m.arc.is_some_and(|(o, _)| (o.x - c.x).hypot(o.y - c.y) < 1e-3)).collect();
        let last = arcs.last().copied().expect("helix arcs");
        let rho = (last.to.0 - c.x).hypot(last.to.1 - c.y);
        // The old helix: half turns of 0.75 mm (pitch 1.5); its hand from
        // the sense of the arcs and the sign of the climb.
        let pair = arcs.windows(2).last().expect("two arcs");
        let dz = pair[1].to.2 - pair[0].to.2;
        assert!((dz.abs() - 0.75).abs() < 1e-3, "{golden}: {dz}");
        let ccw = last.arc.is_some_and(|(_, s)| s);
        let old_hand = if ccw == (dz > 0.0) { Hand::Right } else { Hand::Left };
        assert_eq!(old_hand, hand, "{golden}");
        let sel = Selection::face(disc(c.x, c.y, 5.0, 1024), vec![]).expect("hole");
        let b = Bounds::new(None, Region::empty(), 0.0, -6.0).expect("bounds");
        let r = region(&sel, MaterialSide::Inside, &b, xy(), tool(1.0), 0.0).expect("region");
        let g = GuideChoice::Helix(HelixChoice::new(Circle::new(c, rho).expect("circle"), 1.5, 6.0, hand).expect("helix"));
        let set = guide(&r, g);
        let h = &set.curves()[0];
        let lead = h.lead().expect("lead");
        assert_eq!((lead.pitch(), lead.hand()), (1.5, hand), "{golden}");
        assert!(h.pts().iter().all(|p| ((p.at().x - c.x).hypot(p.at().y - c.y) - rho).abs() < 1e-9), "{golden}");
    }
}
