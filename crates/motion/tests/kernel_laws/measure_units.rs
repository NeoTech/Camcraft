//! #4154: the law measures on hand-made toolpaths with known readings.
//! The stock is a box 40 x 20 from Z -5 to Z 0, the tool radius is 2, and
//! the cut level is Z -1. Each case reads one law and nothing else.

use super::common::law_measure::{EntryKind, EntryStart, Spec, is_fallback, measure};
use camcraft_motion::cl::{ArcMove, ArcPlane, Move, MoveKind, Toolpath};
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::geometry::Point3;

const Z: f64 = -1.0;
const UP: f64 = 5.0;

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    let pts = vec![
        Point2::new(x0, y0),
        Point2::new(x1, y0),
        Point2::new(x1, y1),
        Point2::new(x0, y1),
    ];
    Region::new(vec![Loop::new(pts).expect("loop")]).expect("region")
}

fn spec() -> Spec {
    Spec::new(2.0, rect(-20.0, -10.0, 20.0, 10.0), 0.0, 90.0)
}

/// A path built move by move from a start point.
struct Path {
    at: Point3,
    path: Toolpath,
}

impl Path {
    fn at(x: f64, y: f64) -> Self {
        Path {
            at: Point3::new(x, y, UP),
            path: Toolpath::default(),
        }
    }
    fn go(mut self, kind: MoveKind, x: f64, y: f64, z: f64) -> Self {
        let to = Point3::new(x, y, z);
        let feed = (kind != MoveKind::Rapid).then_some(500.0);
        self.path.moves.push(Move::new(kind, self.at, to, feed));
        self.at = to;
        self
    }
    fn rapid(self, x: f64, y: f64, z: f64) -> Self {
        self.go(MoveKind::Rapid, x, y, z)
    }
    fn feed(self, x: f64, y: f64, z: f64) -> Self {
        self.go(MoveKind::Feed, x, y, z)
    }
    /// Down to the level at the point, as a vertical feed.
    fn plunge(self) -> Self {
        let (x, y) = (self.at.x, self.at.y);
        self.go(MoveKind::Plunge, x, y, Z)
    }
    fn lift(self) -> Self {
        let (x, y) = (self.at.x, self.at.y);
        self.rapid(x, y, UP)
    }
    fn note(mut self, n: &str) -> Self {
        self.path.notes.push(n.into());
        self
    }
}

#[test]
fn a_path_in_air_reads_nothing() {
    let p = Path::at(-30.0, -15.0).plunge().feed(-30.0, 15.0, Z).feed(30.0, 15.0, Z).path;
    let r = measure(&p, &spec()).expect("measure");
    assert_eq!(
        r.row(),
        "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 0 \
         (plunge 0 ramp 0 helix 0 rapid 0) | over 0 worst 0 full 0 | fallback 0"
    );
    assert_eq!(r.breaks(false), "-");
}

fn two_cuts_with_a_lift() -> Path {
    Path::at(-30.0, -15.0)
        .plunge()
        .feed(-30.0, 15.0, Z)
        .lift()
        .rapid(30.0, -15.0, UP)
        .plunge()
        .feed(30.0, 15.0, Z)
}

#[test]
fn a_lift_without_a_reason_breaks_the_link_rule() {
    let r = measure(&two_cuts_with_a_lift().path, &spec()).expect("measure");
    assert_eq!((r.lifts.count, r.named_lifts, r.re_entries), (1, 0, 1));
    assert_eq!(r.lifts.places, vec![Point2::new(-30.0, 15.0)]);
    assert_eq!(r.unnamed_lifts(), 1);
    assert_eq!(r.breaks(false), "link");
}

#[test]
fn a_lift_with_a_named_reason_is_in_the_tally() {
    let note = "link: the tool lifted to the clearance plane 1 time(s) instead of the \
                keep-down ring link - no in-plane link was proved";
    let r = measure(&two_cuts_with_a_lift().note(note).path, &spec()).expect("measure");
    assert_eq!((r.lifts.count, r.named_lifts, r.unnamed_lifts()), (1, 1, 0));
    assert!(r.fallbacks.is_empty(), "a link note is not a fallback");
    assert_eq!(r.breaks(false), "-");
}

#[test]
fn a_crossing_feed_is_read_with_its_place() {
    let p = Path::at(-40.0, -15.0)
        .plunge()
        .feed(-25.0, 15.0, Z)
        .feed(-25.0, -15.0, Z)
        .feed(-40.0, 15.0, Z)
        .path;
    let r = measure(&p, &spec()).expect("measure");
    assert_eq!(r.crossings.count, 1, "{:?}", r.crossings);
    assert_eq!(r.breaks(false), "link");
}

#[test]
fn a_feed_out_of_the_reach_is_read() {
    let mut s = spec();
    s.reach = Some(rect(-50.0, -20.0, -26.0, 20.0));
    let p = Path::at(-30.0, -15.0).plunge().feed(-30.0, 15.0, Z).feed(-22.0, 15.0, Z).path;
    let r = measure(&p, &s).expect("measure");
    assert_eq!(r.leaves.count, 1);
    assert!(r.leaves.places[0].x > -26.0);
}

#[test]
fn an_entry_in_air_and_one_into_solid() {
    let p = Path::at(-30.0, 0.0).plunge().lift().rapid(0.0, 0.0, UP).plunge().path;
    let r = measure(&p, &spec()).expect("measure");
    let starts: Vec<_> = r.entries.iter().map(|e| (e.kind, e.start)).collect();
    assert_eq!(
        starts,
        vec![
            (EntryKind::Plunge, EntryStart::Air),
            (EntryKind::Plunge, EntryStart::Solid)
        ]
    );
    assert_eq!(r.entries[1].place, Point2::new(0.0, 0.0));
    assert_eq!(r.breaks(false), "L5");
}

#[test]
fn a_ramp_and_a_helix_into_solid_are_not_plunges() {
    let mut p = Path::at(-10.0, 0.0).rapid(-10.0, 0.0, 0.5).feed(-4.0, 0.0, Z).lift();
    p = p.rapid(10.0, 0.0, 0.5);
    let helix = Move {
        arc: Some(ArcMove {
            center: Point2::new(8.0, 0.0),
            ccw: true,
            plane: ArcPlane::Xy,
        }),
        ..Move::new(MoveKind::Feed, p.at, Point3::new(10.0, 0.0, Z), Some(300.0))
    };
    p.path.moves.push(helix);
    let r = measure(&p.path, &spec()).expect("measure");
    let starts: Vec<_> = r.entries.iter().map(|e| (e.kind, e.start)).collect();
    assert_eq!(
        starts,
        vec![
            (EntryKind::Ramp, EntryStart::Solid),
            (EntryKind::Helix, EntryStart::Solid)
        ]
    );
    assert_eq!(r.breaks(false), "-");
}

#[test]
fn an_entry_in_a_hole_and_in_a_cleared_column() {
    let mut s = spec();
    s.holes = Some(rect(-13.0, -3.0, -7.0, 3.0));
    let p = Path::at(-10.0, 0.0)
        .plunge()
        .feed(-10.0, 6.0, Z)
        .lift()
        .rapid(-10.0, 5.0, UP)
        .plunge()
        .path;
    let r = measure(&p, &s).expect("measure");
    let starts: Vec<_> = r.entries.iter().map(|e| e.start).collect();
    assert_eq!(starts, vec![EntryStart::Hole, EntryStart::Column]);
}

#[test]
fn a_full_width_slot_reads_180_degrees() {
    let p = Path::at(-30.0, 0.0).plunge().feed(30.0, 0.0, Z).path;
    let r = measure(&p, &spec()).expect("measure");
    assert_eq!((r.over.count, r.full_width.count), (1, 1));
    assert!(r.worst_deg > 179.0, "worst {}", r.worst_deg);
    let at = r.full_width.places[0];
    // The first place: the centre on the stock edge, where the whole
    // front half of the rim meets the stock.
    assert!((-20.0..20.0).contains(&at.x), "the place is in the stock: {at:?}");
    assert_eq!(r.breaks(false), "L6");
    assert_eq!(r.breaks(true), "-", "a named slot cuts at full width by plan");
}

#[test]
fn a_side_cut_reads_its_engagement() {
    // The centre 1 mm outside the edge Y 10: a radial depth of 1 on a
    // radius of 2, that is acos(1 - 1/2) = 60 degrees.
    let p = Path::at(-30.0, 11.0).plunge().feed(30.0, 11.0, Z).path;
    let r = measure(&p, &spec()).expect("measure");
    assert!((55.0..70.0).contains(&r.worst_deg), "worst {}", r.worst_deg);
    assert_eq!((r.over.count, r.full_width.count), (0, 0));
}

#[test]
fn fallback_notes_are_told_from_other_notes() {
    assert!(is_fallback("entry: an axial dive on the floor instead of the helix"));
    assert!(!is_fallback("link: the tool lifted to the clearance plane 2 time(s) instead of x"));
    assert!(!is_fallback("pass start: Outside"));
}

#[test]
fn a_substituted_entry_with_no_note_breaks_law_4() {
    let mut s = spec();
    s.asked_entry = Some(EntryKind::Helix);
    let p = Path::at(0.0, 0.0).plunge().path;
    let r = measure(&p, &s).expect("measure");
    assert_eq!(r.substituted(), 1);
    assert_eq!(r.breaks(false), "L4 L5");
    let noted = Path::at(0.0, 0.0)
        .plunge()
        .note("entry: no room for the helix - an axial dive on the floor instead")
        .path;
    let r = measure(&noted, &s).expect("measure");
    assert_eq!((r.fallbacks.len(), r.breaks(false).as_str()), (1, "L5"));
}
