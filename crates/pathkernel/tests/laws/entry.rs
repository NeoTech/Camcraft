//! Finding 4 of the #4123 review: each proof names its place, and each
//! descent move of the entry stays in that place (law 5). A proof for
//! another place is refused, for every proof kind.

use super::common::{pts, square_region, tool};
use camcraft_pathkernel::entry::{
    Area, BAD_AREA, BAD_CHOICE, Entry, EntryChoice, EntryKind, Hole, NO_DESCENT, NO_HOLE, OFF_PROOF, PLUNGE_NOT_ALLOWED, Proof,
    WRONG_PROOF,
};
use camcraft_pathkernel::{Point2, Positive, Region, Stage};

fn area(x0: f64) -> Area {
    Area::new(square_region(x0, 0.0, 10.0)).unwrap()
}

fn pitch() -> Positive {
    Positive::new(Stage::Entry, 1.0).unwrap()
}

fn entry(kind: EntryKind, at: (f64, f64), proof: Proof, choice: &EntryChoice, plunge: bool) -> Result<Entry, &'static str> {
    let approach = pts(&[(at.0, at.1)]);
    Entry::new(kind, approach, pts(&[at, at]), proof, choice, tool(3.0, plunge)).map_err(|e| e.reason())
}

#[test]
fn each_entry_kind_needs_its_own_proof() {
    let c = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
    let into = || Proof::IntoMaterial { area: area(0.0), angle_or_pitch: pitch() };
    assert!(entry(EntryKind::Air, (5.0, 5.0), Proof::InAir(area(0.0)), &c, false).is_ok());
    assert!(entry(EntryKind::ArcLead, (5.0, 5.0), Proof::InAir(area(0.0)), &c, false).is_ok());
    assert!(entry(EntryKind::ClearedColumn, (5.0, 5.0), Proof::InColumn(area(0.0)), &c, false).is_ok());
    assert!(entry(EntryKind::Helix, (5.0, 5.0), into(), &c, false).is_ok());
    assert!(entry(EntryKind::Ramp, (5.0, 5.0), into(), &c, false).is_ok());
    assert_eq!(entry(EntryKind::Helix, (5.0, 5.0), Proof::InAir(area(0.0)), &c, false).unwrap_err(), WRONG_PROOF);
    assert_eq!(entry(EntryKind::Air, (5.0, 5.0), Proof::InColumn(area(0.0)), &c, false).unwrap_err(), WRONG_PROOF);
    let hole = Hole::new(Point2::new(5.0, 5.0), 4.0).unwrap();
    assert_eq!(entry(EntryKind::Air, (5.0, 5.0), Proof::InHole(hole), &c, false).unwrap_err(), WRONG_PROOF);
    let none = Entry::new(EntryKind::Air, vec![], vec![], Proof::InAir(area(0.0)), &c, tool(3.0, false));
    assert_eq!(none.unwrap_err().reason(), NO_DESCENT);
}

#[test]
fn a_descent_outside_the_proved_area_is_refused_for_each_area_proof() {
    let c = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
    let far = (25.0, 5.0);
    assert_eq!(entry(EntryKind::Air, far, Proof::InAir(area(0.0)), &c, false).unwrap_err(), OFF_PROOF);
    assert_eq!(entry(EntryKind::ClearedColumn, far, Proof::InColumn(area(0.0)), &c, false).unwrap_err(), OFF_PROOF);
    let into = Proof::IntoMaterial { area: area(0.0), angle_or_pitch: pitch() };
    assert_eq!(entry(EntryKind::Ramp, far, into, &c, false).unwrap_err(), OFF_PROOF);
    // On the boundary of the proved area within the tolerance is in it.
    assert!(entry(EntryKind::Air, (10.0, 5.0), Proof::InAir(area(0.0)), &c, false).is_ok());
    assert_eq!(Area::new(Region::empty()).unwrap_err().reason(), BAD_AREA);
}

#[test]
fn a_predrilled_entry_stays_in_a_known_hole() {
    let h = Hole::new(Point2::new(5.0, 5.0), 4.0).unwrap();
    let c = EntryChoice::new(EntryKind::Helix, 2.0, vec![h]).unwrap();
    assert!(entry(EntryKind::Predrilled, (5.0, 5.0), Proof::InHole(h), &c, false).is_ok());
    // The tool of radius 3 in a hole of radius 4: the centre may move 1 mm off the hole centre.
    assert!(entry(EntryKind::Predrilled, (6.0, 5.0), Proof::InHole(h), &c, false).is_ok());
    // The moves at (0,0) with the proof of the hole at (5,5): refused.
    assert_eq!(entry(EntryKind::Predrilled, (0.0, 0.0), Proof::InHole(h), &c, false).unwrap_err(), OFF_PROOF);
    assert_eq!(entry(EntryKind::Predrilled, (6.5, 5.0), Proof::InHole(h), &c, false).unwrap_err(), OFF_PROOF);
    let other = Hole::new(Point2::new(6.0, 5.0), 4.0).unwrap();
    assert_eq!(entry(EntryKind::Predrilled, (6.0, 5.0), Proof::InHole(other), &c, false).unwrap_err(), NO_HOLE);
    let wider = Hole::new(Point2::new(5.0, 5.0), 5.0).unwrap();
    assert_eq!(entry(EntryKind::Predrilled, (5.0, 5.0), Proof::InHole(wider), &c, false).unwrap_err(), NO_HOLE);
}

#[test]
fn a_plunge_needs_the_users_choice_and_a_plunging_tool() {
    let into = || Proof::IntoMaterial { area: area(0.0), angle_or_pitch: Positive::new(Stage::Entry, 90.0).unwrap() };
    let asked = EntryChoice::new(EntryKind::Plunge, 0.0, vec![]).unwrap();
    let other = EntryChoice::new(EntryKind::Ramp, 0.0, vec![]).unwrap();
    assert!(entry(EntryKind::Plunge, (5.0, 5.0), into(), &asked, true).is_ok());
    assert_eq!(entry(EntryKind::Plunge, (5.0, 5.0), into(), &asked, false).unwrap_err(), PLUNGE_NOT_ALLOWED);
    assert_eq!(entry(EntryKind::Plunge, (5.0, 5.0), into(), &other, true).unwrap_err(), PLUNGE_NOT_ALLOWED);
    assert_eq!(EntryChoice::new(EntryKind::Air, -1.0, vec![]).unwrap_err().reason(), BAD_CHOICE);
    assert!(Hole::new(Point2::new(f64::NAN, 0.0), 1.0).is_err());
}
