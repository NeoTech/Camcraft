//! K5 (#4178): the exit is a lead-out in the plane where there is room,
//! then the retract (no cut on the way out, no rapid in material); a lead
//! is bound to its room; the fallback rule of law 4; the stage's own
//! choices refuse bad values by name.

use super::common::{pts, square_region};
use super::entry_stage::{band, edge_ring, p, pocket, pocket_ring, stage};
use camcraft_pathkernel::entry::{
    ArcChoice, Area, BAD_ARC, BAD_EXIT, BAD_HEIGHTS, BAD_MATERIAL_CHOICE, EXIT_OFF_CUT, Entry, EntryChoice, EntryKind,
    EntryPlanner, Exit, Heights, LEAD_GAP, LEAD_NOT_FLAT, LEAD_OFF_ROOM, LEAD_SHORT, Lead, MaterialChoice,
    Proof, RAPID_IN_MATERIAL, UNPROVED_LEAD_OUT, named, needs_fallback,
};
use camcraft_pathkernel::{PlanePoint, Region, Verdict};

fn room() -> Area {
    Area::new(square_region(0.0, 0.0, 10.0)).unwrap()
}

#[test]
fn the_exit_leads_out_into_the_air_then_retracts() {
    let s: &dyn EntryPlanner = &stage();
    let x = s.exit(&edge_ring(), &band(Region::empty()), &Region::empty()).unwrap();
    assert_eq!(x.retract(), 10.0);
    let out = x.lead_out().expect("a lead-out in the air");
    assert_eq!(out[0].at(), p(10.0, -20.0));
    assert!(out.len() > 2 && out.iter().all(|q| q.at().y <= -20.0 + 0.01 && q.depth() == 0.0), "the lead-out cuts no stock");
}

#[test]
fn the_exit_of_a_ring_in_uncut_material_goes_straight_up() {
    let s: &dyn EntryPlanner = &stage();
    let r = pocket(40.0, 40.0, false);
    let x = s.exit(&pocket_ring(), &r, &Region::empty()).unwrap();
    assert_eq!((x.lead_out(), x.retract()), (None, 10.0));
    // With the pocket cleared round the ring, the lead-out has room.
    let x = s.exit(&pocket_ring(), &r, &square_region(0.0, 0.0, 40.0)).unwrap();
    let out = x.lead_out().expect("a lead-out in cleared material");
    assert_eq!(out[0].at(), p(10.0, 10.0));
}

#[test]
fn an_exit_is_refused_off_its_cut_or_below_the_top() {
    let lead = Lead::new(pts(&[(1.0, 1.0), (2.0, 1.0)]), &room()).unwrap();
    assert_eq!(Exit::proved(&pocket_ring(), Some(lead), 10.0, 5.0).unwrap_err().reason(), EXIT_OFF_CUT);
    assert_eq!(Exit::proved(&pocket_ring(), None, 4.0, 5.0).unwrap_err().reason(), RAPID_IN_MATERIAL);
    assert_eq!(Exit::proved(&pocket_ring(), None, f64::NAN, 5.0).unwrap_err().reason(), BAD_EXIT);
    assert!(Exit::proved(&pocket_ring(), None, 5.0, 5.0).is_ok());
    assert_eq!(Exit::new(Some(pts(&[(1.0, 1.0)])), 5.0, 0.0).unwrap_err().reason(), UNPROVED_LEAD_OUT);
}

#[test]
fn a_lead_stays_flat_and_in_its_room_and_starts_where_the_descent_ends() {
    assert_eq!(Lead::new(pts(&[(1.0, 1.0)]), &room()).unwrap_err().reason(), LEAD_SHORT);
    assert_eq!(Lead::new(pts(&[(1.0, 1.0), (12.0, 1.0)]), &room()).unwrap_err().reason(), LEAD_OFF_ROOM);
    let tilted = vec![PlanePoint::new(p(1.0, 1.0), 0.0).unwrap(), PlanePoint::new(p(2.0, 1.0), -1.0).unwrap()];
    assert_eq!(Lead::new(tilted, &room()).unwrap_err().reason(), LEAD_NOT_FLAT);
    let c = EntryChoice::new(EntryKind::Air, 1.0, vec![]).unwrap();
    let tool = super::common::tool(3.0, false);
    let lead = || Some(Lead::new(pts(&[(2.0, 2.0), (5.0, 5.0)]), &room()).unwrap());
    let led = |at| Entry::led(EntryKind::ArcLead, vec![], pts(&[at]), lead(), Proof::InAir(room()), &c, tool);
    assert!(led((2.0, 2.0)).is_ok());
    assert_eq!(led((3.0, 2.0)).unwrap_err().reason(), LEAD_GAP);
}

#[test]
fn a_fallback_is_named_for_another_kind_than_asked() {
    use EntryKind::*;
    assert!(!needs_fallback(Air, ArcLead) && !needs_fallback(ArcLead, Air) && !needs_fallback(Helix, Helix));
    // Where the tool need not enter material, a safer entry is no fallback.
    for used in [Air, ArcLead, Predrilled, ClearedColumn] {
        assert!(!needs_fallback(Helix, used) && !needs_fallback(Ramp, used) && !needs_fallback(Plunge, used));
    }
    assert!(needs_fallback(Helix, Ramp) && needs_fallback(Plunge, Helix) && needs_fallback(Air, Predrilled));
    assert!(needs_fallback(Air, Helix) && needs_fallback(Predrilled, ClearedColumn));
    let c = EntryChoice::new(Helix, 1.0, vec![]).unwrap();
    let into = Proof::IntoMaterial { area: room(), angle_or_pitch: camcraft_pathkernel::Positive::new(camcraft_pathkernel::Stage::Entry, 3.0).unwrap() };
    let ramp = Entry::new(Ramp, vec![], pts(&[(5.0, 5.0)]), into, &c, super::common::tool(3.0, false)).unwrap();
    let Verdict::Done { fallbacks, .. } = named(ramp, &c, "the helix does not fit", vec![]).unwrap() else { panic!("done") };
    assert_eq!(fallbacks.len(), 1);
    assert_eq!((fallbacks[0].asked(), fallbacks[0].used(), fallbacks[0].reason()), ("helix", "ramp", "the helix does not fit"));
}

#[test]
fn the_stage_choices_refuse_bad_values_by_name() {
    assert_eq!(Heights::new(5.0, -1.0, 10.0).unwrap_err().reason(), BAD_HEIGHTS);
    assert_eq!(Heights::new(5.0, 1.0, 4.0).unwrap_err().reason(), BAD_HEIGHTS);
    assert_eq!(Heights::new(f64::NAN, 1.0, 4.0).unwrap_err().reason(), BAD_HEIGHTS);
    assert_eq!(ArcChoice::new(0.0, 90.0).unwrap_err().reason(), BAD_ARC);
    assert_eq!(ArcChoice::new(1.0, 181.0).unwrap_err().reason(), BAD_ARC);
    assert_eq!(ArcChoice::new(1.0, 0.0).unwrap_err().reason(), BAD_ARC);
    for (angle, radius) in [(0.0, None), (90.0, None), (f64::NAN, None), (3.0, Some(0.0)), (3.0, Some(-1.0))] {
        assert_eq!(MaterialChoice::new(angle, radius).unwrap_err().reason(), BAD_MATERIAL_CHOICE);
    }
    let m = MaterialChoice::new(5.0, Some(1.0)).unwrap();
    assert_eq!((m.angle(), m.helix_radius()), (5.0, Some(1.0)));
    assert_eq!(MaterialChoice::default().angle(), 3.0);
}
