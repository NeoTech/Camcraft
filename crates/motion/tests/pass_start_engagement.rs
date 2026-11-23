//! #3567: a later pass of a 2D Offset "Pass starts at: Outside" plan must
//! not plunge or helix into uncut stock at a dead end. The case is the
//! hand trace of PR #3353: a 100 x 40 stock band around a round part of
//! radius 15, tool D8, stepover 4, whose rings split. Each later pass of
//! a split level starts on the part wall, one stepover away from the cut
//! already walked, so it has no cleared column of its own.
//!
//! The check is a 2-D material sim on a 0.25 mm grid. A later ENTRY that
//! is a plunge or a helix, or any descending feed steeper than the ramp
//! angle, counts; a ramp along the pass (no steeper than the ramp angle,
//! its centre on a pass path all along) is the chosen entry (maintainer
//! decision on #3567) and is not counted. The ramp is pinned on its own:
//! that one ran, its angle, the floor under it, and its width reading.
//! The ramp still cuts uncut stock at the dead end (a quarter of the tool
//! disc on this band); the pins measure that load, they do not claim it
//! away.
//!
//! Gate map: covers camcraft_motion::machining2d (pass_start,
//! level_rings, level_order, level_ramp_entry, descent). Provenance: our
//! own regression of the review of PR #3353; the radial-engagement
//! measure of the public milling handbooks; no vendor material consulted.

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::{DEFAULT_RAMP_ANGLE, Links};
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{CornerRelief, PassStart, PocketParams, PocketStrategy, pocket_started, rings_fallbacks};

#[path = "pass_start_engagement/grid.rs"]
mod grid;
use grid::{CELL, Grid, R, seg_dist, sweep, sweep_dirs};

const STEP: f64 = 4.0;
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
    /// The first entry move after a floor feed: the stock standing in the
    /// tool disc where it starts (mm^2), else `None`.
    first: Option<f64>,
    width: f64,
    steep: f64,
    ramp: bool,
}

/// Each entry move of `path` (see [`Entry`]), read on the material grid
/// of `stock`, in path order.
fn entries(path: &Toolpath, stock: &Region, floor: f64, top: f64) -> Vec<Entry> {
    let paths = floor_chords(path, floor);
    let mut g = Grid::new(stock);
    let (mut seen, mut out, mut after_floor) = (false, Vec::new(), true);
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        if m.from.z.min(m.to.z) >= top - 1e-9 {
            continue;
        }
        let planar = is_floor(m, floor);
        let first = (after_floor && !planar).then(|| g.standing(Point2::new(m.from.x, m.from.y), R));
        let (mut got, mut width) = (0.0, 0.0f64);
        for (q, dir) in sweep_dirs(m) {
            width = width.max(g.width(q, dir));
            got += g.cut(q);
        }
        if seen && !planar {
            let (steep, ramp) = (steepness(m), is_ramp(m, &paths));
            out.push(Entry { cut: got, first, width, steep, ramp });
        }
        after_floor = planar;
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
    // The pin cannot pass empty: the dead ends were entered by ramps.
    let ramps = entries(&path, &stock, p.bottom_z, p.top_z).iter().filter(|e| e.ramp && e.cut > 0.0).count();
    assert!(ramps > 0, "no ramp along the pass ran, so the pin proves nothing");
}

/// #3567 review item 2: the slope law of `is_ramp`. A straight descending
/// feed on the pass path is a ramp only at or under the ramp angle; a
/// steeper one (60 deg, vertical) counts like a plunge.
#[test]
fn a_steep_straight_feed_is_not_a_ramp() {
    use camcraft_motion::cl::PathBuilder;
    use camcraft_motion::geometry::Point3;
    let floor = -3.0;
    let run = 3.0 / 60f64.to_radians().tan();
    let shallow = 3.0 / RAMP_DEG.to_radians().tan();
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, 10.0));
    b.push(MoveKind::Rapid, Point3::new(0.0, 0.0, 0.0), None);
    b.push(MoveKind::Feed, Point3::new(run, 0.0, floor), Some(300.0));
    b.push(MoveKind::Feed, Point3::new(run, 0.0, 0.0), Some(300.0));
    b.push(MoveKind::Feed, Point3::new(run, 0.0, floor), Some(300.0));
    b.push(MoveKind::Rapid, Point3::new(run, 0.0, 0.0), None);
    b.push(MoveKind::Feed, Point3::new(run + shallow, 0.0, floor), Some(300.0));
    b.push(MoveKind::Feed, Point3::new(-20.0, 0.0, floor), Some(1000.0));
    b.push(MoveKind::Feed, Point3::new(200.0, 0.0, floor), Some(1000.0));
    let path = b.finish();
    let paths = floor_chords(&path, floor);
    let kinds: Vec<bool> =
        path.moves.iter().filter(|m| m.kind == MoveKind::Feed && !is_floor(m, floor)).map(|m| is_ramp(m, &paths)).collect();
    // 60 deg down, vertical up, vertical down: not ramps; 3 deg down: a ramp.
    assert_eq!(kinds, vec![false, false, false, true], "{kinds:?}");
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

/// #3567: the width READING of the ramps along the pass. It measures the
/// lateral width of the uncut stock on the leading half of the tool rim
/// while the ramp travels (the stepover band beside the ring cut before),
/// on the 2-D grid, by the shape-mask helper's rim law, which reads no Z.
/// It is not a proof that a ramp never cuts at the full tool width: in a
/// case where the ring before does not reach the dead end, the ramp cuts a
/// slot at the ramp angle. It also reads the share of the tool disc in
/// uncut stock where each ramp starts (a quarter on this band). The pin
/// holds the band reading to the engagement law of the floor feeds, so a
/// ramp that leaves its pass for wider stock is caught.
#[test]
fn outside_dead_end_ramps_width_reading_of_the_stepover_band() {
    let (path, stock, p) = band_path();
    let all = entries(&path, &stock, p.bottom_z, p.top_z);
    let cuts = |e: &Entry| e.ramp && e.cut > 0.0;
    let widest = all.iter().filter(|e| cuts(e)).map(|e| e.width).fold(0.0, f64::max);
    let disc = std::f64::consts::PI * R * R;
    // Where each ramp starts: the share of the tool disc in uncut stock.
    let starts: Vec<String> =
        all.iter().filter(|e| e.ramp).filter_map(|e| e.first).map(|a| format!("{:.0} %", 100.0 * a / disc)).collect();
    eprintln!("ramp moves that cut: {}, widest band reading {widest:.3} mm; first disc in stock: {starts:?}", all.iter().filter(|e| cuts(e)).count());
    assert!(widest > 0.0, "no ramp along the pass cut stock");
    assert!(widest <= ENGAGEMENT, "a ramp along the pass reads {widest:.3} mm of band, over {ENGAGEMENT:.3} mm");
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

/// #3567 review items 1 and 4: a dead end too short for the ramp at the
/// entry's angle (here 0.01 deg, so no pass is long enough) is never
/// entered by a silent plunge: the area falls back to the Auto order with
/// the named rings-fallback note, and that fallback plan is pure Auto -
/// its moves equal the Auto run's byte for byte, with no ramp in them.
#[test]
fn a_dead_end_too_short_for_the_ramp_falls_back_to_pure_auto() {
    let (region, _) = band();
    let mut p = params(true);
    p.links.approach.ramp_angle = 0.01;
    let run = |start: PassStart| match pocket_started(&region, &p, &[], &CornerRelief::default(), start) {
        Ok(t) => t,
        Err(e) => panic!("{e:?}"),
    };
    let (outside, auto) = (run(PassStart::Outside), run(PassStart::Auto));
    assert_eq!(rings_fallbacks(&outside.notes), 1, "{:?}", outside.notes);
    assert!(
        outside.notes.iter().any(|n| n.contains("too short for a ramp entry along it") && n.starts_with("Pass start Outside")),
        "{:?}",
        outside.notes
    );
    assert_eq!(format!("{:?}", outside.moves), format!("{:?}", auto.moves), "the fallback plan is not the Auto plan");
}
