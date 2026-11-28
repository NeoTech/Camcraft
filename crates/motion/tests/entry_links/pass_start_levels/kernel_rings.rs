//! #4176: the kernel's ring guide (`camcraft-pathkernel`, K2) against the
//! recorded rings of the old planner (the #3750 goldens of
//! `shared_planner.rs`), on the dog bone and on the band round a disc.
//! Where the old rings are correct the kernel rings lie on them. The named
//! differences:
//! - the old 2D Offset also walks the stock outline; the kernel's rings
//!   round the part end at the last level, whose tool disc already cuts
//!   past the stock outline (the outline is air: #4025 decision 4);
//! - the old Inside and Outside plans of the band build their own pieces
//!   for each start side (#3316, the defect of #3750); the kernel has one
//!   ring set, and it is the old Auto set;
//! - the helix turns and the links are stages 6 and 5, not the guide;
//! - `offset2d_dogbone_auto_blind.cl` is not compared: without the stock
//!   the old plan rings the part out to the band's outer edge, and the
//!   kernel cannot tell the part walls from the stock edge (an Outside
//!   region needs the stock, #4025 decision 8).

use super::{R, STEP, band, rect, region};
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::machining2d::{PassStart, StartSide, kernel_bounds_of, kernel_pocket_region};
use camcraft_motion::pathkernel::guide::{GuideChoice, GuidePlanner, GuideSet, MarkKind, RingPlanner, RingRef, RingsChoice};
use camcraft_motion::pathkernel::{KernelRegion, MaterialSide, PlanarRegion, ToolShape, WorkPlane};

const TOL: f64 = 1e-3;
/// Two paths agree where their points are this close (mm): the polygon
/// offsets of the two planners share the 5 um arc flattening.
const AGREE: f64 = 0.02;

fn disc(cx: f64, r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(cx + r * a.cos(), r * a.sin())
        })
        .collect()
}

/// The dog bone of `shared_planner.rs`: the stock and the band.
fn dog_bone() -> (Region, Region) {
    let part = region(vec![disc(-15.0, 12.0)])
        .union(&region(vec![disc(15.0, 12.0)]))
        .and_then(|p| p.union(&region(vec![rect(-15.0, -4.0, 15.0, 4.0)])))
        .expect("part");
    let stock = region(vec![rect(-40.0, -25.0, 40.0, 25.0)]);
    let band = stock.offset(R).and_then(|s| s.difference(&part)).expect("band");
    (stock, band)
}

fn tool() -> ToolShape {
    ToolShape::disc(R, 0.0, true, 30.0).expect("tool")
}

fn xy() -> WorkPlane {
    WorkPlane::setup_xy(0.0).expect("plane")
}

fn rings(r: &KernelRegion, reference: RingRef, step: f64) -> GuideSet {
    let g = GuideChoice::Rings(RingsChoice::new(reference, 0.0, step, None, None).expect("choice"));
    let v = RingPlanner.guide(r, &g, TOL).expect("rings");
    assert!(v.postable(), "the kernel rings cover the area: {v:?}");
    v.out().expect("out").clone()
}

/// The cut polylines of a golden CL text: runs of `Feed` and `Arc` moves
/// at the cut level (z = -2 mm), split at each other move. The first point
/// of a run is where the entry reaches the cut level. Coordinates are in um.
fn cl_runs(name: &str) -> Vec<Vec<Point2>> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    let text = std::fs::read_to_string(&file).expect("golden");
    let mut runs: Vec<Vec<Point2>> = vec![Vec::new()];
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| w.get(i).and_then(|s| s.parse::<f64>().ok());
        let tag = |c: char| w.iter().find_map(|t| t.strip_prefix(c).and_then(|v| v.parse::<f64>().ok()));
        let run = runs.last_mut().expect("run");
        match (w.first(), num(1), num(2), num(3)) {
            (Some(&"Feed"), Some(x), Some(y), Some(z)) if (z + 2000.0).abs() < 0.5 => run.push(Point2::new(x / 1000.0, y / 1000.0)),
            (Some(&"Arc"), Some(x), Some(y), Some(z)) if (z + 2000.0).abs() < 0.5 => {
                let to = Point2::new(x / 1000.0, y / 1000.0);
                match (run.last().copied(), tag('I'), tag('J'), tag('S')) {
                    (Some(from), Some(i), Some(j), Some(s)) => run.extend(arc(from, to, Point2::new(i / 1000.0, j / 1000.0), s > 0.0)),
                    // The last helix turn: the tool reaches the cut level here.
                    _ => run.push(to),
                }
            }
            _ => runs.push(Vec::new()),
        }
    }
    runs.into_iter().filter(|r| r.len() >= 2).collect()
}

/// The points of an arc from `from` to `to` round `c` (counter-clockwise
/// when `ccw`), every 2 degrees, the end included.
fn arc(from: Point2, to: Point2, c: Point2, ccw: bool) -> Vec<Point2> {
    let a0 = (from.y - c.y).atan2(from.x - c.x);
    let a1 = (to.y - c.y).atan2(to.x - c.x);
    let tau = std::f64::consts::TAU;
    let sweep = if ccw { (a1 - a0).rem_euclid(tau) } else { -(a0 - a1).rem_euclid(tau) };
    let sweep = if sweep.abs() < 1e-9 { if ccw { tau } else { -tau } } else { sweep };
    let rad = (from.x - c.x).hypot(from.y - c.y);
    let n = ((sweep.abs() / 2f64.to_radians()).ceil() as usize).max(1);
    (1..=n).map(|i| {
        let a = a0 + sweep * i as f64 / n as f64;
        Point2::new(c.x + rad * a.cos(), c.y + rad * a.sin())
    }).collect()
}

/// The passes of one start side of `level_rings_band.txt`.
fn level_passes(start: &str) -> Vec<Vec<Point2>> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/level_rings_band.txt");
    let text = std::fs::read_to_string(&file).expect("golden");
    let mut out = Vec::new();
    let mut on = false;
    for line in text.lines() {
        if line.contains("passes=") {
            on = line.starts_with(start);
            continue;
        }
        if on && !line.starts_with("note:") {
            let pts = line.split_whitespace().filter_map(|t| {
                let (x, y) = t.split_once(',')?;
                Some(Point2::new(x.parse().ok()?, y.parse().ok()?))
            });
            out.push(pts.collect());
        }
    }
    out
}

fn seg(q: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (q.x - a.x - t * dx).hypot(q.y - a.y - t * dy)
}

fn to_runs(runs: &[Vec<Point2>], q: Point2) -> f64 {
    runs.iter().flat_map(|r| r.windows(2).map(move |w| seg(q, w[0], w[1]))).fold(f64::INFINITY, f64::min)
}

/// The regular (not clean-up) kernel rings as closed or open polylines.
fn kernel_runs(set: &GuideSet) -> (Vec<Vec<Point2>>, Vec<Vec<Point2>>) {
    let fills: Vec<usize> = set.marks().iter().filter(|m| m.kind() == MarkKind::Fill).map(|m| m.curve()).collect();
    let (mut reg, mut fill) = (Vec::new(), Vec::new());
    for (i, c) in set.curves().iter().enumerate() {
        let mut v: Vec<Point2> = c.pts().iter().map(|q| q.at()).collect();
        if c.closed() {
            v.push(v[0]);
        }
        if fills.contains(&i) { fill.push(v) } else { reg.push(v) }
    }
    (reg, fill)
}

/// The points of `of` farther than [`AGREE`] from the paths `from`.
fn far(of: &[Vec<Point2>], from: &[Vec<Point2>]) -> Vec<Point2> {
    of.iter().flatten().copied().filter(|q| to_runs(from, *q) > AGREE).collect()
}

/// [`far`] for the old runs, less the first point of each run: the end of
/// an entry (stage 6), not a ring.
fn far_old(old: &[Vec<Point2>], from: &[Vec<Point2>]) -> Vec<Point2> {
    old.iter().flat_map(|r| r.iter().skip(1)).copied().filter(|q| to_runs(from, *q) > AGREE).collect()
}

/// The distance from `q` to the outline of the rectangle `x0..x1, y0..y1`.
fn to_rect(q: Point2, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> f64 {
    let c = [Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)];
    (0..4).map(|i| seg(q, c[i], c[(i + 1) % 4])).fold(f64::INFINITY, f64::min)
}

#[test]
fn kernel_rings_lie_on_the_old_offset_rings_of_the_dog_bone() {
    let (stock, band_) = dog_bone();
    let side = StartSide { start: PassStart::Inside, stock: Some(stock), around_part: true };
    let b = kernel_bounds_of(&side, Region::empty(), 0.0, -2.0).expect("bounds");
    let k = kernel_pocket_region(&band_, &side, &b, xy(), tool(), 0.0).expect("region");
    assert_eq!(k.planar().map(PlanarRegion::material), Some(MaterialSide::Outside));
    let set = rings(&k, RingRef::Part, 2.0);
    let (reg, fill) = kernel_runs(&set);
    assert!(fill.is_empty(), "the 2 mm step covers the band: no clean-up ring");
    for n in ["auto", "inside", "outside", "inside_rings", "outside_rings", "inside_arc", "outside_arc"] {
        let old = cl_runs(&format!("offset2d_dogbone_{n}.cl"));
        // Every point of the kernel's one ring set is on each old plan.
        assert_eq!(far(&reg, &old).len(), 0, "{n}: a kernel ring leaves the old path");
        // The old plan's other points are its pass along the stock
        // outline (the outer edge of its band, rounded where it meets the
        // rings): each is within one step of the outline.
        let extra = far_old(&old, &reg);
        assert!(!extra.is_empty(), "{n}: the old outline pass is the named difference");
        assert!(extra.iter().all(|q| to_rect(*q, (-40.0, -25.0, 40.0, 25.0)) < 2.0), "{n}: an old point off the rings, away from the outline");
    }
}

#[test]
fn kernel_rings_of_the_band_are_the_old_auto_rings() {
    let (_, _, reach) = band();
    let side = StartSide { start: PassStart::Inside, stock: None, around_part: false };
    let b = kernel_bounds_of(&side, Region::empty(), 0.0, -2.0).expect("bounds");
    let pocket = rings(&kernel_pocket_region(&reach, &side, &b, xy(), tool(), 0.0).expect("region"), RingRef::Boundary, STEP);
    let walls = reach.loops().to_vec();
    let level = PlanarRegion::new(xy(), reach.clone(), MaterialSide::Inside, walls, vec![], Region::empty(), Region::empty(), tool());
    let level = rings(&KernelRegion::Planar(level.expect("level region")), RingRef::Boundary, STEP);
    let cases = [
        ("2D Pocket", &pocket, cl_runs("pocket_band_auto.cl"), [cl_runs("pocket_band_inside.cl"), cl_runs("pocket_band_outside.cl")]),
        ("level", &level, level_passes("Auto"), [level_passes("Inside"), level_passes("Outside")]),
    ];
    for (name, set, auto, sided) in cases {
        let (reg, fill) = kernel_runs(set);
        assert!(fill.is_empty(), "{name}: no clean-up ring on the band");
        assert_eq!((far(&reg, &auto).len(), far_old(&auto, &reg).len()), (0, 0), "{name}: the kernel rings are the old Auto rings");
        // The named difference: the old Inside and Outside sets are not the
        // Auto set (#3750); the kernel has the one set for each side.
        for old in &sided {
            assert!(!far(&reg, old).is_empty() || !far_old(old, &reg).is_empty(), "{name}: an old sided set is now the Auto set");
        }
    }
}
