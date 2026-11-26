//! The test parts of the kernel law readings (#4154): the disc of #3934
//! and the dog bone of #3741 (as in `pass_start_levels/ring_links.rs`) in
//! a stock box of 80 x 50, a flat tool of radius 4, one level at Z -2 under
//! a stock top at Z 0; and the table that pins the readings of a kind.

use super::common::law_measure::{EntryKind, Readings, Spec, measure};
use camcraft_motion::cl::Toolpath;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{PassStart, PocketParams, PocketStrategy, StartSide};

/// The tool radius.
pub const R: f64 = 4.0;
/// The stock top.
pub const TOP: f64 = 0.0;
/// The one cut level.
pub const Z: f64 = -2.0;
/// The ring step of the pocket kinds.
pub const STEP: f64 = 2.0;

/// The engagement that a step `s` of the tool radius `r` plans, in
/// degrees (`acos(1 - s / r)`).
pub fn planned_deg(s: f64, r: f64) -> f64 {
    (1.0 - s / r).clamp(-1.0, 1.0).acos().to_degrees()
}

pub fn region(loops: Vec<Vec<Point2>>) -> Region {
    let loops = loops.into_iter().map(|l| Loop::new(l).expect("loop")).collect();
    Region::new(loops).expect("region")
}

pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Point2> {
    vec![
        Point2::new(x0, y0),
        Point2::new(x1, y0),
        Point2::new(x1, y1),
        Point2::new(x0, y1),
    ]
}

fn disc_loop(cx: f64, r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(cx + r * a.cos(), r * a.sin())
        })
        .collect()
}

pub fn stock() -> Region {
    region(vec![rect(-40.0, -25.0, 40.0, 25.0)])
}

/// A disc of radius 12 in the middle of the box.
pub fn disc() -> Region {
    region(vec![disc_loop(0.0, 12.0)])
}

/// Two discs of radius 12 at X -15 and X 15, joined by a neck 8 mm wide.
pub fn dog_bone() -> Region {
    region(vec![disc_loop(-15.0, 12.0)])
        .union(&region(vec![disc_loop(15.0, 12.0)]))
        .and_then(|p| p.union(&region(vec![rect(-15.0, -4.0, 15.0, 4.0)])))
        .expect("part")
}

/// The test parts by name.
pub fn parts() -> [(&'static str, Region); 2] {
    [("disc", disc()), ("dog bone", dog_bone())]
}

/// The band the tool centre may use round a part: the stock grown by the
/// diameter (the Outside reach), less the part.
pub fn band(part: &Region) -> Region {
    stock()
        .offset(2.0 * R)
        .and_then(|s| s.difference(part))
        .expect("band")
}

/// The area that the tool centre may use when it clears round `part`.
pub fn centre_reach(part: &Region) -> Region {
    stock()
        .offset(R)
        .and_then(|s| s.difference(&part.offset(R).expect("grown part")))
        .expect("centre reach")
}

pub fn side(start: PassStart) -> StartSide {
    StartSide {
        start,
        stock: Some(stock()),
        around_part: true,
    }
}

/// The pocket parameters of the ring cases (as `pass_start_levels`).
pub fn pocket_params(strategy: PocketStrategy) -> PocketParams {
    PocketParams {
        tool_diameter: 2.0 * R,
        stepover: STEP,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        top_z: TOP,
        bottom_z: Z,
        stepdown: 2.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        strategy,
        entry: Entry3D::Helix,
        helix_radius: 1.5,
        optimal_load: 0.25,
        blend_turns: false,
        blend_radius: None,
        links: Links::PLANAR,
    }
}

/// The spec of a clearing round `part` in the stock box.
pub fn spec_round(part: &Region, limit_deg: f64) -> Spec {
    let mut s = Spec::new(R, stock(), TOP, limit_deg);
    s.part = Some(part.clone());
    s.reach = Some(centre_reach(part));
    s.asked_entry = Some(EntryKind::Helix);
    s
}

/// A closed prism of `outline` from `z0` to `z1`, as facets.
pub fn prism(outline: &Region, z0: f64, z1: f64) -> Vec<Triangle> {
    let mut out = Vec::new();
    for t in outline.triangles().expect("triangles") {
        let p = |q: Point2, z: f64| Point3::new(q.x, q.y, z);
        out.push(Triangle::new(p(t[0], z1), p(t[1], z1), p(t[2], z1)));
        out.push(Triangle::new(p(t[0], z0), p(t[2], z0), p(t[1], z0)));
    }
    for l in outline.loops() {
        let pts = l.points();
        for (k, &a) in pts.iter().enumerate() {
            let b = pts[(k + 1) % pts.len()];
            let (a0, a1) = (Point3::new(a.x, a.y, z0), Point3::new(a.x, a.y, z1));
            let (b0, b1) = (Point3::new(b.x, b.y, z0), Point3::new(b.x, b.y, z1));
            out.push(Triangle::new(a0, b0, b1));
            out.push(Triangle::new(a0, b1, a1));
        }
    }
    out
}

/// One pinned row: the case, the readings and the laws that they break.
pub type Row = (&'static str, &'static str, &'static str);

/// Measure each case and compare it with the pinned table. On a
/// difference the message lists the whole table as read, ready to pin.
pub fn pin(kind: &str, slot: bool, table: &[Row], cases: Vec<(String, Toolpath, Spec)>) {
    let read: Vec<(String, Readings)> = cases
        .into_iter()
        .map(|(name, path, spec)| {
            let r = measure(&path, &spec).unwrap_or_else(|e| panic!("{kind} {name}: {e}"));
            (name, r)
        })
        .collect();
    let got: Vec<(String, String, String)> = read
        .iter()
        .map(|(n, r)| (n.clone(), r.row(), r.breaks(slot)))
        .collect();
    let want: Vec<(String, String, String)> = table
        .iter()
        .map(|(n, r, b)| ((*n).to_string(), (*r).to_string(), (*b).to_string()))
        .collect();
    if got != want {
        let mut text = String::new();
        for ((n, row, b), (_, r)) in got.iter().zip(&read) {
            text.push_str(&format!("    (\"{n}\",\n     \"{row}\",\n     \"{b}\"),\n"));
            text.push_str(&format!(
                "    // places: lifts {:?} over {:?} full {:?} entries {:?} fallbacks {:?}\n",
                r.lifts.places, r.over.places, r.full_width.places, r.entries, r.fallbacks
            ));
        }
        panic!("{kind}: the readings of the old planner differ from the pinned table:\n{text}");
    }
}
