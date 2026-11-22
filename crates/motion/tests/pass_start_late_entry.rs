//! #3567: with "Pass starts at" Outside, a later pass of a 2D Offset
//! level must land in stock that an earlier pass already cleared, never
//! plunge at full width into uncut stock. The case is the band of #3316:
//! a stock box 100 x 40 around a round part of radius 15, tool D8,
//! stepover 4. The check is a 2-D material sim on a 0.25 mm grid that
//! counts the stock cut by the plunges and helices after the first floor
//! feed; a ramp along the pass (no steeper than the ramp angle, its
//! centre on a pass path) is the decided dead-end entry and is not
//! counted, but is pinned on its own: its angle, and the floor under it.
//!
//! Gate map: covers camcraft_motion::machining2d (pass_start,
//! level_rings, level_order, level_ramp_entry, pocket). Provenance: our
//! own regression of the review of PR #3353 (ported from PR #3439); the
//! radial-engagement measure of the public milling handbooks; no vendor
//! material consulted.

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::{DEFAULT_RAMP_ANGLE, Links};
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{CornerRelief, PassStart, PocketParams, PocketStrategy, pocket_started, rings_fallbacks};

const R: f64 = 4.0;
const STEP: f64 = 4.0;
const CELL: f64 = 0.25;
const PART: f64 = 15.0;

/// The ramp angle the band's entry takes: the op sets none, so the ramp
/// fallback's default (`links::DEFAULT_RAMP_ANGLE`).
const RAMP_DEG: f64 = DEFAULT_RAMP_ANGLE;

/// How much steeper (deg) than [`RAMP_DEG`] a move may be and still be a
/// ramp: float noise only. A plunge is 90 deg.
const RAMP_TOL_DEG: f64 = 0.05;

/// How near (mm) to a planar floor feed a ramp's centre must stay to lie
/// on the pass path: the walks are fitted to arcs within microns, the
/// ramp rides the raw points.
const ON_PATH: f64 = 0.02;

fn lp(pts: Vec<Point2>) -> Loop {
    let Ok(l) = Loop::new(pts) else { panic!("fixture loop") };
    l
}

fn rect(hx: f64, hy: f64) -> Loop {
    lp(vec![Point2::new(-hx, -hy), Point2::new(hx, -hy), Point2::new(hx, hy), Point2::new(-hx, hy)])
}

fn circle(r: f64) -> Loop {
    let n = 256;
    lp((0..n)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / f64::from(n);
            Point2::new(r * a.cos(), r * a.sin())
        })
        .collect())
}

fn params(climb: bool) -> PocketParams {
    PocketParams {
        blend_radius: None,
        tool_diameter: 2.0 * R,
        stepover: STEP,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: None,
        top_z: 0.0,
        bottom_z: -3.0,
        stepdown: 5.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb,
        strategy: PocketStrategy::Waterline,
        entry: Entry3D::Helix,
        helix_radius: 1.6,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: Links::PLANAR,
    }
}

/// The material grid: true where stock still stands.
struct Grid {
    lo: Point2,
    nx: usize,
    ny: usize,
    cells: Vec<bool>,
}

impl Grid {
    fn new(stock: &Region) -> Grid {
        let Some((lo, hi)) = stock.bounds() else { panic!("stock bounds") };
        let nx = ((hi.x - lo.x) / CELL).ceil() as usize + 1;
        let ny = ((hi.y - lo.y) / CELL).ceil() as usize + 1;
        let mut cells = vec![false; nx * ny];
        for j in 0..ny {
            for i in 0..nx {
                let p = Point2::new(lo.x + (i as f64 + 0.5) * CELL, lo.y + (j as f64 + 0.5) * CELL);
                cells[j * nx + i] = stock.contains(p);
            }
        }
        Grid { lo, nx, ny, cells }
    }

    fn cell(&self, i: usize, j: usize) -> Point2 {
        Point2::new(self.lo.x + (i as f64 + 0.5) * CELL, self.lo.y + (j as f64 + 0.5) * CELL)
    }

    /// The cell index window of the disc of radius `r` at `c`.
    fn window(&self, c: Point2, r: f64) -> (usize, usize, usize, usize) {
        let i0 = (((c.x - r - self.lo.x) / CELL).floor().max(0.0)) as usize;
        let j0 = (((c.y - r - self.lo.y) / CELL).floor().max(0.0)) as usize;
        let i1 = ((((c.x + r - self.lo.x) / CELL).ceil()).max(0.0) as usize).min(self.nx);
        let j1 = ((((c.y + r - self.lo.y) / CELL).ceil()).max(0.0) as usize).min(self.ny);
        (i0, j0, i1, j1)
    }

    /// Clear the disc at `c`; returns the area removed (mm^2).
    fn cut(&mut self, c: Point2) -> f64 {
        let (i0, j0, i1, j1) = self.window(c, R);
        let mut n = 0usize;
        for j in j0..j1 {
            for i in i0..i1 {
                let p = self.cell(i, j);
                if (p.x - c.x).hypot(p.y - c.y) <= R && self.cells[j * self.nx + i] {
                    self.cells[j * self.nx + i] = false;
                    n += 1;
                }
            }
        }
        n as f64 * CELL * CELL
    }

    /// The width of cut (mm) of the disc at `c` moving along `dir`: the
    /// longest run of standing stock on the leading half of its rim, read
    /// as the chord depth `R (1 - cos)` - the law of the shape-mask
    /// helper's `contact_width`, on this grid.
    fn width(&self, c: Point2, dir: (f64, f64)) -> f64 {
        const RIM: usize = 64;
        if dir.0.hypot(dir.1) <= 1e-12 {
            return 0.0;
        }
        let rim = R + 0.75 * CELL;
        let base = dir.1.atan2(dir.0) - std::f64::consts::FRAC_PI_2;
        let (mut best, mut run) = (0usize, 0usize);
        for k in 0..RIM {
            let a = base + (k as f64 + 0.5) * std::f64::consts::PI / RIM as f64;
            let q = Point2::new(c.x + rim * a.cos(), c.y + rim * a.sin());
            let i = (q.x - self.lo.x) / CELL;
            let j = (q.y - self.lo.y) / CELL;
            let stock = i >= 0.0
                && j >= 0.0
                && (i as usize) < self.nx
                && (j as usize) < self.ny
                && self.cells[j as usize * self.nx + i as usize];
            run = if stock { run + 1 } else { 0 };
            best = best.max(run);
        }
        let theta = (best as f64 * std::f64::consts::PI / RIM as f64).min(std::f64::consts::PI);
        R * (1.0 - theta.cos())
    }

    /// The stock area (mm^2) still standing within `r` of `c`.
    fn standing(&self, c: Point2, r: f64) -> f64 {
        let (i0, j0, i1, j1) = self.window(c, r);
        let mut n = 0usize;
        for j in j0..j1 {
            for i in i0..i1 {
                let p = self.cell(i, j);
                if self.cells[j * self.nx + i] && (p.x - c.x).hypot(p.y - c.y) <= r {
                    n += 1;
                }
            }
        }
        n as f64 * CELL * CELL
    }
}

/// The centre positions a move sweeps, no farther apart than half a
/// cell, each with the direction of its chord.
fn sweep_dirs(m: &Move) -> Vec<(Point2, (f64, f64))> {
    let mut out = Vec::new();
    for (a, b) in m.chords(0.005) {
        let n = ((b.x - a.x).hypot(b.y - a.y) / (CELL * 0.5)).ceil().max(1.0) as usize;
        for s in 0..=n {
            let t = s as f64 / n as f64;
            out.push((Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t), (b.x - a.x, b.y - a.y)));
        }
    }
    out
}

/// The centre positions a move sweeps (see [`sweep_dirs`]).
fn sweep(m: &Move) -> Vec<Point2> {
    sweep_dirs(m).into_iter().map(|(q, _)| q).collect()
}

fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - a.x - t * dx).hypot(p.y - a.y - t * dy)
}

fn is_floor(m: &Move, floor: f64) -> bool {
    m.kind != MoveKind::Rapid && (m.from.z - floor).abs() < 1e-9 && (m.to.z - floor).abs() < 1e-9
}

/// The XY chords of every planar floor feed of `path`: the pass paths.
fn floor_chords(path: &Toolpath, floor: f64) -> Vec<(Point2, Point2)> {
    path.moves.iter().filter(|m| is_floor(m, floor)).flat_map(|m| m.chords(0.005)).collect()
}

/// The steepest descent (deg) of a move, over its 3-D segments.
fn steepness(m: &Move) -> f64 {
    m.segments3(0.005)
        .iter()
        .map(|(a, b)| (a.z - b.z).abs().atan2((b.x - a.x).hypot(b.y - a.y)).to_degrees())
        .fold(0.0, f64::max)
}

/// #3567: a ramp along the pass - a feed that is not planar, no steeper
/// than the ramp angle, whose centre stays on a pass path (a planar
/// floor feed) all along. A plunge is too steep; a helix leaves the path.
fn is_ramp(m: &Move, paths: &[(Point2, Point2)]) -> bool {
    m.kind == MoveKind::Feed
        && steepness(m) <= RAMP_DEG + RAMP_TOL_DEG
        && sweep(m).iter().all(|&q| paths.iter().any(|&(u, v)| seg_dist(q, u, v) <= ON_PATH))
}

/// One cutting move below the top, after the first floor feed, that is
/// not a floor feed: the stock it cut, its widest cut (mm), how steep it
/// is, and whether it is a ramp along the pass.
struct Entry {
    cut: f64,
    width: f64,
    steep: f64,
    ramp: bool,
}

/// Each entry move of `path` (see [`Entry`]), read on the material grid
/// of `stock`, in path order.
fn entries(path: &Toolpath, stock: &Region, floor: f64, top: f64) -> Vec<Entry> {
    let paths = floor_chords(path, floor);
    let mut g = Grid::new(stock);
    let (mut seen, mut out) = (false, Vec::new());
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        if m.from.z.min(m.to.z) >= top - 1e-9 {
            continue;
        }
        let planar = is_floor(m, floor);
        let (mut got, mut width) = (0.0, 0.0f64);
        for (q, dir) in sweep_dirs(m) {
            width = width.max(g.width(q, dir));
            got += g.cut(q);
        }
        if seen && !planar {
            out.push(Entry { cut: got, width, steep: steepness(m), ramp: is_ramp(m, &paths) });
        }
        seen |= planar;
    }
    out
}

/// The stock (mm^2) that PLUNGES and HELICES cut after the first floor
/// feed: each entry move that is not a ramp along the pass (the #3567
/// maintainer decision: a ramp along the pass is not counted).
fn late_entry_cut(path: &Toolpath, stock: &Region, floor: f64, top: f64) -> f64 {
    entries(path, stock, floor, top).iter().filter(|e| !e.ramp).map(|e| e.cut).sum()
}

fn band_path() -> (Toolpath, Region, PocketParams) {
    let (region, stock) = band();
    let p = params(true);
    let path = match pocket_started(&region, &p, &[], &CornerRelief::default(), PassStart::Outside) {
        Ok(t) => t,
        Err(e) => panic!("{e:?}"),
    };
    // #3567: the start-side plan is kept, not swapped for the Auto order.
    assert_eq!(rings_fallbacks(&path.notes), 0, "{:?}", path.notes);
    (path, stock, p)
}

/// #3316 pin: a new pass of a split level starts with a lead-in from a
/// piece already cut, or (#3567) at a dead end by a ramp along its own
/// pass - never a plunge or helix at the far wall into uncut stock.
#[test]
fn outside_entries_after_the_first_land_in_cleared_stock() {
    let (path, stock, p) = band_path();
    let late = late_entry_cut(&path, &stock, p.bottom_z, p.top_z);
    assert!(late < 1.0, "the later entries cut {late:.2} mm^2 of uncut stock");
}

/// #3567: each entry move that cuts stock after the first floor feed is
/// no steeper than the ramp angle - a ramp cannot hide a plunge. And the
/// dead ends are entered at all: the ramps cut stock.
#[test]
fn outside_dead_end_entries_are_no_steeper_than_the_ramp_angle() {
    let (path, stock, p) = band_path();
    let all = entries(&path, &stock, p.bottom_z, p.top_z);
    let steep: Vec<f64> =
        all.iter().filter(|e| e.cut > 0.0).map(|e| e.steep).filter(|&s| s > RAMP_DEG + RAMP_TOL_DEG).collect();
    assert!(steep.is_empty(), "entries steeper than {RAMP_DEG} deg cut stock: {steep:?}");
    let ramped: f64 = all.iter().filter(|e| e.ramp).map(|e| e.cut).sum();
    assert!(ramped > 1.0, "no dead end was entered by a ramp along the pass ({ramped:.2} mm^2)");
}

/// The widest cut a pass may take: one stepover plus a quarter radius,
/// the engagement law of `pass_start_levels` (plus its 1.5-cell slack).
const ENGAGEMENT: f64 = STEP + 0.25 * R + 1.5 * CELL;

/// #3567: during the ramp along the pass the tool cuts the width of the
/// pass itself (one stepover beside the ring before), never the full tool
/// width - the same engagement law as the floor feeds of the band.
#[test]
fn outside_dead_end_ramps_cut_no_wider_than_the_pass() {
    let (path, stock, p) = band_path();
    let all = entries(&path, &stock, p.bottom_z, p.top_z);
    let widest = all.iter().filter(|e| e.ramp).map(|e| e.width).fold(0.0, f64::max);
    eprintln!("ramp entries: {}, widest ramp cut {widest:.3} mm", all.iter().filter(|e| e.ramp).count());
    assert!(widest > 0.0, "no ramp along the pass cut stock");
    assert!(widest <= ENGAGEMENT, "a ramp along the pass cuts {widest:.3} mm wide, over {ENGAGEMENT:.3} mm");
}

/// #3567: the floor under each ramp is cut at full depth too - the pass
/// runs over the ramped stretch at the floor, so no stock stands there
/// on the material grid of the floor feeds alone.
#[test]
fn outside_floor_under_each_dead_end_ramp_is_cut() {
    let (path, stock, p) = band_path();
    let paths = floor_chords(&path, p.bottom_z);
    let mut floor = Grid::new(&stock);
    for m in path.moves.iter().filter(|m| is_floor(m, p.bottom_z)) {
        for q in sweep(m) {
            floor.cut(q);
        }
    }
    let ramps: Vec<&Move> = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Feed && !is_floor(m, p.bottom_z) && m.from.z.min(m.to.z) < p.top_z)
        .filter(|m| is_ramp(m, &paths))
        .collect();
    assert!(!ramps.is_empty(), "no ramp along the pass to check");
    let left: f64 = ramps.iter().flat_map(|m| sweep(m)).map(|q| floor.standing(q, R - 2.0 * CELL)).sum();
    assert!(left < 1e-9, "{left:.2} mm^2 of stock stands on the floor under the ramps");
}

fn band() -> (Region, Region) {
    // The machining region reaches one tool radius past the stock, as an
    // Outside containment's area does; the part is the hole.
    let Ok(region) = Region::new(vec![rect(50.0 + R, 20.0 + R), circle(PART)]) else { panic!("region") };
    let Ok(stock) = Region::new(vec![rect(50.0, 20.0), circle(PART)]) else { panic!("stock") };
    (region, stock)
}
