//! K5 (#4178): the entry stage plans each entry with a proof (law 5), names
//! each fallback (law 4), and refuses by name when no entry is safe. Never
//! a plunge into material that the user did not choose.

use super::common::{curve, square_region, tool};
use camcraft_pathkernel::entry::{
    ArcChoice, Entry, EntryChoice, EntryKind, EntryPlanner, EntryStage, Heights, Hole, MaterialChoice, NO_MATERIAL_ENTRY,
    NO_SAFE_ENTRY, Proof,
};
use camcraft_pathkernel::guide::Family;
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{Bounds, KernelRegion, MaterialSide, Point2, Region, Selection, Verdict, WorkPlane, region};

pub fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn rect(x0: f64, y0: f64, w: f64, h: f64) -> Vec<Point2> {
    vec![p(x0, y0), p(x0 + w, y0), p(x0 + w, y0 + h), p(x0, y0 + h)]
}

/// The planner: material top 5 above the level, clearance 2, retract 10;
/// arc leads of radius 5 and 90 degrees; chords within 0.01 mm.
pub fn stage() -> EntryStage {
    EntryStage::new(Heights::new(5.0, 2.0, 10.0).unwrap(), ArcChoice::new(5.0, 90.0).unwrap(), 0.01).unwrap()
}

/// A closed pocket `w` by `h` from the origin, in a stock that covers it.
pub fn pocket(w: f64, h: f64, plunges: bool) -> KernelRegion {
    let bounds = Bounds::new(Some(square_region(-10.0, -10.0, 80.0)), Region::empty(), 0.0, -5.0).unwrap();
    let sel = Selection::face(rect(0.0, 0.0, w, h), vec![]).unwrap();
    region(&sel, MaterialSide::Inside, &bounds, WorkPlane::setup_xy(0.0).unwrap(), tool(3.0, plunges), 0.0).unwrap()
}

/// The open band round a 20 mm part in a 60 mm stock from (-20, -20),
/// with the keep-out `keep`.
pub fn band(keep: Region) -> KernelRegion {
    let bounds = Bounds::new(Some(square_region(-20.0, -20.0, 60.0)), keep, 0.0, -5.0).unwrap();
    let sel = Selection::face(rect(0.0, 0.0, 20.0, 20.0), vec![]).unwrap();
    region(&sel, MaterialSide::Outside, &bounds, WorkPlane::setup_xy(0.0).unwrap(), tool(3.0, false), 0.0).unwrap()
}

/// A closed ring cut through `xy`, counter-clockwise.
pub fn ring(xy: &[(f64, f64)]) -> Cut {
    Cut::new(curve(xy, true, 0, Family::Ring), None).unwrap()
}

/// The ring of the 40 mm pocket, 10 mm in from its walls.
pub fn pocket_ring() -> Cut {
    ring(&[(10.0, 10.0), (30.0, 10.0), (30.0, 30.0), (10.0, 30.0)])
}

/// The ring on the stock outline of [`band`], from (10, -20).
pub fn edge_ring() -> Cut {
    ring(&[(10.0, -20.0), (40.0, -20.0), (40.0, 40.0), (-20.0, 40.0), (-20.0, -20.0)])
}

fn choice(k: EntryKind) -> EntryChoice {
    EntryChoice::new(k, 1.0, vec![]).unwrap()
}

fn plan(cut: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, &'static str> {
    let s: &dyn EntryPlanner = &stage();
    s.entry(cut, r, cleared, e).map_err(|e| e.reason())
}


/// The entry and its fallbacks as (asked, used) names.
fn done(v: Verdict<Entry>) -> (Entry, Vec<(&'static str, &'static str)>, Vec<String>) {
    match v {
        Verdict::Done { out, fallbacks, notes } => {
            (out, fallbacks.iter().map(|f| (f.asked(), f.used())).collect(), notes.iter().map(|n| n.text().to_owned()).collect())
        }
        other => panic!("the entry stage gives only done verdicts: {other:?}"),
    }
}

fn last_descent(e: &Entry) -> Point2 {
    e.descent().last().unwrap().at()
}

#[test]
fn a_closed_pocket_with_no_hole_and_no_helix_or_ramp_is_refused_by_name() {
    for plunges in [false, true] {
        let r = pocket(40.0, 40.0, plunges);
        for k in [EntryKind::Air, EntryKind::ArcLead, EntryKind::Predrilled, EntryKind::ClearedColumn] {
            let s: &dyn EntryPlanner = &stage();
            let e = s.entry(&pocket_ring(), &r, &Region::empty(), &choice(k)).unwrap_err();
            assert_eq!(e.reason(), NO_SAFE_ENTRY, "{k:?}");
            assert_eq!(e.place(), Some(p(10.0, 10.0)));
            assert!(e.fix_text().is_some_and(|f| f.contains("helix or ramp")));
        }
    }
}

#[test]
fn an_asked_helix_enters_the_closed_pocket_in_the_centre_area() {
    let r = pocket(40.0, 40.0, false);
    let (e, fallbacks, notes) = done(plan(&pocket_ring(), &r, &Region::empty(), &choice(EntryKind::Helix)).unwrap());
    assert_eq!((e.kind(), fallbacks.len(), notes.len()), (EntryKind::Helix, 0, 0));
    assert!(matches!(e.proof(), Proof::IntoMaterial { .. }));
    assert_eq!(e.approach()[0].depth(), 7.0);
    let end = e.descent().last().unwrap();
    assert_eq!((end.at(), end.depth()), (p(10.0, 10.0), 0.0));
    let center = r.planar().unwrap().center().offset(0.01).unwrap();
    assert!(e.descent().iter().all(|q| center.contains(q.at())), "the helix leaves the centre area");
    assert!(e.descent().windows(2).all(|w| w[1].depth() < w[0].depth()), "the helix goes down on each move");
    assert!(e.descent().windows(2).all(|w| w[0].at() != w[1].at()), "no move straight down");
}

#[test]
fn an_asked_helix_that_does_not_fit_falls_back_to_a_named_ramp() {
    let r = pocket(40.0, 6.4, false);
    let line = Cut::new(curve(&[(5.0, 3.2), (35.0, 3.2)], false, 0, Family::Row), None).unwrap();
    let (e, fallbacks, _) = done(plan(&line, &r, &Region::empty(), &choice(EntryKind::Helix)).unwrap());
    assert_eq!(e.kind(), EntryKind::Ramp);
    assert_eq!(fallbacks, vec![("helix", "ramp")]);
    assert_eq!(last_descent(&e), p(5.0, 3.2));
    assert!(e.descent().windows(2).all(|w| w[0].at() != w[1].at()), "no move straight down");
    // At 0.01 degrees the ramp needs too many passes: refused, never a plunge.
    let flat = EntryChoice::new(EntryKind::Helix, 1.0, vec![]).unwrap().with_material(MaterialChoice::new(0.01, None).unwrap());
    let r = pocket(40.0, 6.4, true);
    assert_eq!(plan(&line, &r, &Region::empty(), &flat).unwrap_err(), NO_MATERIAL_ENTRY);
}

#[test]
fn a_plunge_needs_the_users_choice_and_a_plunging_tool() {
    let (e, fallbacks, _) = done(plan(&pocket_ring(), &pocket(40.0, 40.0, true), &Region::empty(), &choice(EntryKind::Plunge)).unwrap());
    assert_eq!((e.kind(), fallbacks.len()), (EntryKind::Plunge, 0));
    let (e, fallbacks, _) = done(plan(&pocket_ring(), &pocket(40.0, 40.0, false), &Region::empty(), &choice(EntryKind::Plunge)).unwrap());
    assert_eq!(e.kind(), EntryKind::Helix);
    assert_eq!(fallbacks, vec![("plunge", "helix")]);
    // A ramp asked with a plunging tool: never a plunge in its place.
    let (e, _, _) = done(plan(&pocket_ring(), &pocket(40.0, 40.0, true), &Region::empty(), &choice(EntryKind::Ramp)).unwrap());
    assert_eq!(e.kind(), EntryKind::Ramp);
}

#[test]
fn a_predrilled_hole_serves_the_closed_pocket() {
    let r = pocket(40.0, 40.0, false);
    let hole = Hole::new(p(20.0, 20.0), 4.0).unwrap();
    let asked = EntryChoice::new(EntryKind::Predrilled, 1.0, vec![hole]).unwrap();
    let (e, fallbacks, _) = done(plan(&pocket_ring(), &r, &Region::empty(), &asked).unwrap());
    assert_eq!((e.kind(), fallbacks.len()), (EntryKind::Predrilled, 0));
    assert_eq!(e.proof(), &Proof::InHole(hole));
    assert_eq!(last_descent(&e), p(20.0, 20.0));
    let lead: Vec<Point2> = e.lead().unwrap().pts().iter().map(|q| q.at()).collect();
    assert_eq!(lead, vec![p(20.0, 20.0), p(10.0, 10.0)]);
    // Asked air: the hole serves, and the fallback is named.
    let air = EntryChoice::new(EntryKind::Air, 1.0, vec![hole]).unwrap();
    let (_, fallbacks, _) = done(plan(&pocket_ring(), &r, &Region::empty(), &air).unwrap());
    assert_eq!(fallbacks, vec![("air entry", "pre-drilled hole")]);
    // A hole narrower than the tool does not serve a tool that cannot plunge.
    let narrow = EntryChoice::new(EntryKind::Predrilled, 1.0, vec![Hole::new(p(20.0, 20.0), 2.0).unwrap()]).unwrap();
    assert_eq!(plan(&pocket_ring(), &r, &Region::empty(), &narrow).unwrap_err(), NO_SAFE_ENTRY);
}

#[test]
fn a_cleared_column_serves_the_closed_pocket() {
    let r = pocket(40.0, 40.0, false);
    let cleared = square_region(14.0, 14.0, 12.0);
    let (e, fallbacks, _) = done(plan(&pocket_ring(), &r, &cleared, &choice(EntryKind::ClearedColumn)).unwrap());
    assert_eq!((e.kind(), fallbacks.len()), (EntryKind::ClearedColumn, 0));
    let Proof::InColumn(col) = e.proof() else { panic!("a column proof") };
    // The tool disc of radius 3 fits in the cleared square from (14, 14) to (26, 26).
    let at = last_descent(&e);
    assert!((17.0 - 0.01..=23.0 + 0.01).contains(&at.x) && (17.0 - 0.01..=23.0 + 0.01).contains(&at.y), "{at:?}");
    assert!(col.region().offset(0.001).unwrap().contains(at));
    assert_eq!(e.lead().unwrap().pts().last().unwrap().at(), p(10.0, 10.0));
}

#[test]
fn an_open_pocket_enters_in_air_with_an_arc_lead() {
    let r = band(Region::empty());
    let air = r.planar().unwrap().air().clone();
    for asked in [EntryKind::ArcLead, EntryKind::Helix] {
        let (e, fallbacks, notes) = done(plan(&edge_ring(), &r, &Region::empty(), &choice(asked)).unwrap());
        assert_eq!((e.kind(), fallbacks.len(), notes.len()), (EntryKind::ArcLead, 0, 0), "{asked:?}");
        assert!(matches!(e.proof(), Proof::InAir(_)));
        let at = last_descent(&e);
        assert!(air.contains(at), "the descent is in the air band: {at:?}");
        assert!(at.y <= -20.0 - 3.0 - 1.0 + 1e-6, "clear of the stock by the tool radius and the approach: {at:?}");
        let lead = e.lead().unwrap().pts();
        assert_eq!((lead[0].at(), lead.last().unwrap().at()), (at, p(10.0, -20.0)));
        assert!(lead.iter().all(|q| q.at().y <= -20.0 + 0.01 && q.depth() == 0.0), "the lead stays outside the stock");
    }
}

/// The distance from `q` to the box from `lo` to `hi`.
fn off_box(q: Point2, lo: Point2, hi: Point2) -> f64 {
    let dx = (lo.x - q.x).max(q.x - hi.x).max(0.0);
    let dy = (lo.y - q.y).max(q.y - hi.y).max(0.0);
    dx.hypot(dy)
}

#[test]
fn an_air_entry_at_the_stock_edge_keeps_off_a_clamp() {
    let free = done(plan(&edge_ring(), &band(Region::empty()), &Region::empty(), &choice(EntryKind::ArcLead)).unwrap()).0;
    // A clamp from (3, -26) to (4, -25) stands where the free entry comes down.
    let (lo, hi) = (p(3.0, -26.0), p(4.0, -25.0));
    assert!(off_box(last_descent(&free), lo, hi) < 3.0, "the clamp is in the way of the free entry");
    let r = band(square_region(3.0, -26.0, 1.0));
    let (e, _, notes) = done(plan(&edge_ring(), &r, &Region::empty(), &choice(EntryKind::ArcLead)).unwrap());
    assert_eq!(e.kind(), EntryKind::ArcLead);
    let moves = e.descent().iter().chain(e.lead().unwrap().pts());
    for q in moves {
        assert!(off_box(q.at(), lo, hi) >= 3.0 - 0.01, "the tool touches the clamp at {:?}", q.at());
    }
    assert!(notes.iter().any(|n| n.contains("flown at 2.500 mm")), "{notes:?}");
}
