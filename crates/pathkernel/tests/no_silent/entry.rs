//! #4407 (review of #4383, class B): the entry stage never recovers in
//! silence. A try that a checked constructor refuses keeps its named
//! reason: in the notes of the entry that fits, or in the final refusal.
//! A cut above the material top says why the asked entry is not flown.

use crate::common::{square_region, tool};
use crate::entry_stage::{edge_ring, p, pocket, pocket_ring, stage};
use camcraft_pathkernel::entry::{
    ABOVE, Entry, EntryChoice, EntryKind, EntryPlanner, Hole, LEAD_OFF_ROOM, NO_SAFE_ENTRY, Proof,
};
use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{
    Bounds, KernelRegion, MaterialSide, PlanePoint, Point2, Refusal, Region, Selection, Verdict, WorkPlane, region,
};

fn rect(x0: f64, y0: f64, w: f64, h: f64) -> Vec<Point2> {
    vec![p(x0, y0), p(x0 + w, y0), p(x0 + w, y0 + h), p(x0, y0 + h)]
}

/// The 40 mm pocket of `entry_stage::pocket` with a 10 mm island in its
/// middle (15 to 25): the legal room is not convex.
fn pocket_with_island() -> KernelRegion {
    let bounds = Bounds::new(Some(square_region(-10.0, -10.0, 80.0)), Region::empty(), 0.0, -5.0).unwrap();
    let sel = Selection::face(rect(0.0, 0.0, 40.0, 40.0), vec![rect(15.0, 15.0, 10.0, 10.0)]).unwrap();
    region(&sel, MaterialSide::Inside, &bounds, WorkPlane::setup_xy(0.0).unwrap(), tool(3.0, false), 0.0).unwrap()
}

fn plan(cut: &Cut, r: &KernelRegion, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
    let s: &dyn EntryPlanner = &stage();
    s.entry(cut, r, &Region::empty(), e)
}

fn notes(v: &Verdict<Entry>) -> Vec<String> {
    match v {
        Verdict::Done { notes, .. } => notes.iter().map(|n| n.text().to_owned()).collect(),
        _ => Vec::new(),
    }
}

/// The nearest hole (30, 14) has its straight lead through the island: its
/// lead is refused (LEAD_OFF_ROOM). The next hole (10, 34) serves, and the
/// refused try is a note; with no other hole, it is in the refusal.
#[test]
fn a_refused_try_is_named_in_the_notes_or_in_the_refusal() {
    let r = pocket_with_island();
    let (near, far) = (Hole::new(p(30.0, 14.0), 4.0).unwrap(), Hole::new(p(10.0, 34.0), 4.0).unwrap());
    let both = EntryChoice::new(EntryKind::Predrilled, 1.0, vec![near, far]).unwrap();
    let v = plan(&pocket_ring(), &r, &both).unwrap();
    let e = v.out().unwrap();
    assert_eq!((e.kind(), e.proof()), (EntryKind::Predrilled, &Proof::InHole(far)));
    let said = notes(&v);
    assert!(said.iter().any(|n| n.contains("pre-drilled hole was refused by its check") && n.contains(LEAD_OFF_ROOM)), "{said:?}");
    let only = EntryChoice::new(EntryKind::Predrilled, 1.0, vec![near]).unwrap();
    let refused = plan(&pocket_ring(), &r, &only).unwrap_err();
    assert_eq!((refused.reason(), refused.tried()), (NO_SAFE_ENTRY, &[LEAD_OFF_ROOM][..]));
    assert!(refused.to_string().contains("tries refused"), "{refused}");
    // An entry with no refused try has no such note (the pins of
    // `entry_stage` keep their counts).
    let clear = EntryChoice::new(EntryKind::Predrilled, 1.0, vec![far]).unwrap();
    assert!(notes(&plan(&pocket_ring(), &r, &clear).unwrap()).is_empty());
}

/// A ring of the pocket at depth `d` along the plane normal.
fn ring_at(d: f64) -> Cut {
    let pts = [(10.0, 10.0), (30.0, 10.0), (30.0, 30.0), (10.0, 30.0)].iter().map(|&(x, y)| PlanePoint::new(p(x, y), d).unwrap()).collect();
    Cut::new(Curve::new(pts, None, true, 0, Family::Ring).unwrap(), None).unwrap()
}

/// The stage of `entry_stage` has its material top 5 above the level. A
/// cut at depth 6 starts above it: the tool goes straight down in air.
/// The reason ABOVE is a named fallback where one is due, and a note where
/// the asked kind was not an air entry.
#[test]
fn a_cut_above_the_material_top_says_why_the_asked_entry_is_not_flown() {
    let r = pocket(40.0, 40.0, false);
    let cut = ring_at(6.0);
    let run = |k: EntryKind| plan(&cut, &r, &EntryChoice::new(k, 1.0, vec![]).unwrap()).unwrap();
    for k in [EntryKind::Helix, EntryKind::Ramp, EntryKind::ArcLead] {
        let v = run(k);
        assert_eq!(v.out().map(Entry::kind), Some(EntryKind::Air), "{k:?}");
        let said = notes(&v);
        assert!(said.iter().any(|n| n.contains(ABOVE) && n.contains(k.name())), "{k:?}: {said:?}");
    }
    let Verdict::Done { fallbacks, .. } = run(EntryKind::Predrilled) else { panic!("an entry") };
    assert_eq!(fallbacks.iter().map(|f| (f.asked(), f.reason())).collect::<Vec<_>>(), vec![("pre-drilled hole", ABOVE)]);
    let v = run(EntryKind::Air);
    assert!(notes(&v).is_empty(), "an asked air entry is the entry flown");
    // At the material top or below, the stage plans as before.
    assert_ne!(plan(&edge_ring(), &crate::entry_stage::band(Region::empty()), &EntryChoice::new(EntryKind::Helix, 1.0, vec![]).unwrap()).unwrap().out().map(Entry::kind), Some(EntryKind::Air));
}
