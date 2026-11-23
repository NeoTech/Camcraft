//! 2D Contour tab entry (#3453): the first tabbed level of an Outside
//! contour on a sheet enters at the start the untabbed levels entered at,
//! one stepdown at most below the floor that they left under the FULL tool
//! disc - not only the tool centre. The later tab windows are pinned as
//! they are (#3678 owns their dives). The oracle is the posted path alone:
//! the floor at a point is the lowest level pass (a feed at one height)
//! whose swept disc holds it, else the top of the stock.
//!
//! Gate map (M4.4): covers camcraft_motion::machining2d (contour, contour_start, contour_step, contour_link).

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::{Link, Links};
use camcraft_motion::machining2d::{ContourParams, Lead, Side, contour};

const D: f64 = 6.0;
const R: f64 = D / 2.0;
const TOP: f64 = 0.0;
const STEP: f64 = 4.5;

fn l_shape() -> Region {
    let pts = [(0.0, 0.0), (40.0, 0.0), (40.0, 15.0), (15.0, 15.0), (15.0, 40.0), (0.0, 40.0)];
    Region::new(vec![Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()).unwrap()]).unwrap()
}

/// The reporter's sheet (#3453): 18 mm, 0.3 mm through, stepdown 4.5,
/// four 10 x 3 tabs, a plunge entry.
fn params(lead: Lead, links: Links) -> ContourParams {
    ContourParams {
        tool_diameter: D,
        stock_to_leave: 0.0,
        clearance_z: 25.0,
        retract_z: 12.0,
        top_z: TOP,
        bottom_z: -18.3,
        stepdown: STEP,
        feed: 1950.0,
        plunge_feed: 600.0,
        climb: true,
        side: Side::Outside,
        lead,
        tolerance: 0.01,
        tab_count: 4,
        tab_width: 10.0,
        tab_height: 3.0,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        links,
    }
}

/// The stay-down ramp preset of the contour page (`project/links.rs`).
fn keep_down_ramp() -> Links {
    Links { transition: Some(Entry3D::Ramp), level: Link::KeepDown, ramp_angle: 3.0, ..Links::PLANAR }
}

fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let ll = dx * dx + dy * dy;
    let t = if ll > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / ll).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// Points of the tool disc at `c`: the centre and rings out to the rim.
fn disc(c: Point2) -> Vec<Point2> {
    let mut out = vec![c];
    for ring in 1..=6 {
        let rr = R * f64::from(ring) / 6.0 * 0.999;
        for k in 0..72 {
            let t = std::f64::consts::TAU * f64::from(k) / 72.0;
            out.push(Point2::new(c.x + rr * t.cos(), c.y + rr * t.sin()));
        }
    }
    out
}

/// One feed move down (a plunge or a ramp step), judged against the cuts
/// posted before it.
#[derive(Debug)]
struct Descent {
    at: Point2,
    from_z: f64,
    to_z: f64,
    /// The floor under the tool centre, and the highest floor under the disc.
    floor_centre: f64,
    floor_disc: f64,
    /// How far the disc reaches past the cut under its centre (mm): the
    /// width of the crescent of material that is higher than that floor.
    crescent: f64,
}

/// Every feed move that descends, with the floors before it.
fn descents(t: &Toolpath) -> Vec<Descent> {
    let mut cuts: Vec<(f64, Vec<(Point2, Point2)>)> = Vec::new();
    let mut out = Vec::new();
    for m in &t.moves {
        let flat = (m.from.z - m.to.z).abs() < 1e-12;
        if m.kind == MoveKind::Rapid {
            continue;
        }
        if m.to.z < m.from.z - 1e-9 {
            let at = Point2::new(m.to.x, m.to.y);
            let floor = |q: Point2| {
                cuts.iter()
                    .filter(|(_, ch)| ch.iter().any(|&(a, b)| seg_dist(q, a, b) <= R + 1e-6))
                    .map(|(z, _)| *z)
                    .fold(TOP, f64::min)
            };
            let fc = floor(at);
            let pts = disc(at);
            let fd = pts.iter().map(|&q| floor(q)).fold(f64::NEG_INFINITY, f64::max);
            let crescent = pts
                .iter()
                .filter(|&&q| floor(q) > fc + 1e-9)
                .map(|q| R - (q.x - at.x).hypot(q.y - at.y))
                .fold(0.0, f64::max);
            out.push(Descent { at, from_z: m.from.z, to_z: m.to.z, floor_centre: fc, floor_disc: fd, crescent });
        }
        if flat && m.kind == MoveKind::Feed {
            cuts.push((m.to.z, m.chords(0.001)));
        }
    }
    out
}

/// The heights the feed moves rise to (the tab hops and the lifts).
fn rises(t: &Toolpath) -> Vec<(f64, f64)> {
    t.moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && m.to.z > m.from.z + 1e-9)
        .map(|m| (m.from.z, m.to.z))
        .collect()
}

fn report(name: &str, t: &Toolpath) -> Vec<Descent> {
    let ds = descents(t);
    eprintln!("== {name}");
    for d in &ds {
        eprintln!(
            "descent at ({:8.3}, {:8.3}) Z {:8.3} -> {:8.3}  floor centre {:8.3} disc {:8.3}  below disc floor {:6.3}  crescent {:5.3}",
            d.at.x,
            d.at.y,
            d.from_z,
            d.to_z,
            d.floor_centre,
            d.floor_disc,
            d.floor_disc - d.to_z,
            d.crescent
        );
    }
    for (a, b) in rises(t) {
        eprintln!("rise Z {a:8.3} -> {b:8.3}");
    }
    for n in &t.notes {
        eprintln!("note: {n}");
    }
    ds
}

/// The descents onto the first tabbed level (Z -18), in cut order, and the
/// XY where the first level (Z -4.5) entered.
fn tabbed_entries(name: &str, links: Links, lead: Lead) -> (Point2, Vec<Descent>) {
    let t = contour(&l_shape(), &params(lead, links)).unwrap();
    let ds = report(name, &t);
    let first = ds.iter().find(|d| (d.to_z + 4.5).abs() < 1e-9).map(|d| d.at);
    let Some(first) = first else { panic!("{name}: no entry onto Z -4.5") };
    (first, ds.into_iter().filter(|d| (d.to_z + 18.0).abs() < 1e-9).collect())
}

fn same_xy(a: Point2, b: Point2) -> bool {
    (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6
}

/// Each window's (depth below the disc floor, crescent). The crescent is
/// measured against the floor under the centre, so a disc that stands
/// wholly in uncut stock reads 0 there and 18 below the disc floor.
fn readings(ds: &[Descent]) -> Vec<(f64, f64)> {
    ds.iter().map(|d| (d.floor_disc - d.to_z, d.crescent)).collect()
}

fn assert_readings(name: &str, got: &[(f64, f64)], want: &[(f64, f64)]) {
    assert_eq!(got.len(), want.len(), "{name}: tab windows {got:?}");
    for (g, w) in got.iter().zip(want) {
        assert!((g.0 - w.0).abs() < 1e-6 && (g.1 - w.1).abs() < 1e-3, "{name}: window readings {got:?}, want {want:?}");
    }
}

#[test]
fn the_first_tabbed_window_enters_where_the_levels_above_cut_its_lead_in() {
    // #3453: the reporter's shape (each level retracts and re-enters). On
    // origin/main window 0 entered at (46.000, 8.865), 18 mm below the disc
    // floor with a 2.5 mm crescent of uncut sheet.
    let name = "plunge, arc lead";
    let (first, ds) = tabbed_entries(name, Links::PLANAR, Lead::Arc { radius: 3.0 });
    assert!(same_xy(ds[0].at, first), "{name}: window 0 enters at {:?}, not at the untabbed start {first:?}", ds[0].at);
    assert!((ds[0].from_z - 2.0).abs() < 1e-9, "{name}: the dive starts at the plunge height");
    // Windows 1 to 3: their lead-in starts stand 3 mm outside the ring,
    // with a 2.5 mm crescent of the disc in stock that no level above cut:
    // an 18 mm plunge (#3678 owns it; these readings change with its fix).
    assert_readings(name, &readings(&ds), &[(2.7, 0.0), (18.0, 2.501), (18.0, 2.501), (18.0, 2.501)]);
    assert!(ds[0].to_z >= ds[0].floor_disc - STEP - 1e-6, "{name}: window 0 is one stepdown at most below the disc floor");
}

#[test]
fn the_first_tabbed_window_of_the_stay_down_ramp_enters_at_the_untabbed_start() {
    // The stay-down levels cut the lead-in only on the first level: the
    // disc floor at the start is -4.5 (#3678 owns the 13.5 mm plunge).
    let name = "keep-down ramp, arc lead";
    let (first, ds) = tabbed_entries(name, keep_down_ramp(), Lead::Arc { radius: 3.0 });
    assert!(same_xy(ds[0].at, first), "{name}: window 0 enters at {:?}, not at the untabbed start {first:?}", ds[0].at);
    assert_readings(name, &readings(&ds), &[(13.5, 2.501), (18.0, 2.501), (18.0, 2.501), (18.0, 2.501)]);
}

#[test]
fn a_tabbed_level_without_a_lead_enters_in_the_kerf_of_the_level_above() {
    // Lead None: every window dives on the ring, 2.7 mm below the tab-top
    // level, with the whole disc in its kerf - as before #3453.
    for (name, links) in [("plunge, no lead", Links::PLANAR), ("keep-down ramp, no lead", keep_down_ramp())] {
        let (first, ds) = tabbed_entries(name, links, Lead::None);
        assert!(same_xy(ds[0].at, first), "{name}: window 0 enters at {:?}, not at the untabbed start {first:?}", ds[0].at);
        assert_readings(name, &readings(&ds), &[(2.7, 0.0); 4]);
    }
}
