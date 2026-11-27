//! #4407 (review of #4383, class B): the exit never drops its lead-out in
//! silence, and both constructors of an exit check the retract on one path.

use crate::entry_stage::{band, edge_ring, pocket, pocket_ring, stage};
use camcraft_pathkernel::entry::{
    BAD_EXIT, EntryPlanner, Exit, LEAD_OUT_NAME, NO_LEAD_OUT_ROOM, RAPID_IN_MATERIAL, STRAIGHT_UP_NAME,
};
use camcraft_pathkernel::verdict::NO_REASON;
use camcraft_pathkernel::{Region, Stage};

/// The ring of the closed pocket ends in uncut material: no lead-out arc
/// fits, and the exit names the fallback. The ring on the stock edge has
/// its lead-out and no fallback.
#[test]
fn an_exit_with_no_room_for_its_lead_out_names_the_fallback() {
    let s: &dyn EntryPlanner = &stage();
    let x = s.exit(&pocket_ring(), &pocket(40.0, 40.0, false), &Region::empty()).unwrap();
    assert!(x.lead_out().is_none());
    let fb = x.fallback().expect("a named fallback");
    assert_eq!((fb.stage(), fb.asked(), fb.used(), fb.reason()), (Stage::Exit, LEAD_OUT_NAME, STRAIGHT_UP_NAME, NO_LEAD_OUT_ROOM));
    let led = s.exit(&edge_ring(), &band(Region::empty()), &Region::empty()).unwrap();
    assert!(led.lead_out().is_some() && led.fallback().is_none());
}

/// The fallback is only for an exit with no lead-out, and it names its
/// reason; the retract check is the same in both constructors.
#[test]
fn the_exit_constructors_share_one_checked_path() {
    let x = Exit::new(None, 5.0, 0.0).unwrap();
    assert_eq!(x.clone().without_lead_out(" ").unwrap_err().reason(), NO_REASON);
    assert!(x.without_lead_out(NO_LEAD_OUT_ROOM).unwrap().fallback().is_some());
    let s: &dyn EntryPlanner = &stage();
    let led = s.exit(&edge_ring(), &band(Region::empty()), &Region::empty()).unwrap();
    assert_eq!(led.without_lead_out(NO_LEAD_OUT_ROOM).unwrap_err().reason(), BAD_EXIT);
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(Exit::new(None, bad, 0.0).unwrap_err().reason(), BAD_EXIT, "{bad}");
        assert_eq!(Exit::proved(&pocket_ring(), None, bad, 0.0).unwrap_err().reason(), BAD_EXIT, "{bad}");
    }
    assert_eq!(Exit::proved(&pocket_ring(), None, 5.0, f64::NAN).unwrap_err().reason(), BAD_EXIT);
    assert_eq!(Exit::proved(&pocket_ring(), None, 4.0, 5.0).unwrap_err().reason(), RAPID_IN_MATERIAL);
    // #4415: `Exit::new` gets the material top and refuses a retract under
    // it, as `Exit::proved` does.
    assert_eq!(Exit::new(None, 4.0, 5.0).unwrap_err().reason(), RAPID_IN_MATERIAL);
    assert_eq!(Exit::new(None, 5.0, f64::NAN).unwrap_err().reason(), BAD_EXIT);
    assert!(Exit::new(None, 5.0, 5.0).is_ok(), "a retract at the top is out of the material");
}
