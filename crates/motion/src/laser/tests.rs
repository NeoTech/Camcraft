//! The laser profile (#4081): the shared sheet laws under the laser's
//! voice - the same cut as the waterjet's for the same numbers, every
//! note and refusal in the laser's words, and the power door.

use super::*;
use crate::cl::MoveKind;

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    Loop::new(vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]).unwrap()
}

/// A 100 x 60 plate with a 10 x 10 square hole.
fn plate() -> (Vec<Loop>, Region) {
    let loops = vec![rect(0.0, 0.0, 100.0, 60.0), rect(70.0, 25.0, 80.0, 35.0)];
    let part = Region::new(loops.clone()).unwrap();
    (loops, part)
}

fn params() -> LaserParams {
    LaserParams {
        kerf: 0.2,
        control_comp: false,
        reversed: false,
        lead_shape: LeadShape::Arc,
        lead_in: 2.0,
        lead_out: 1.0,
        open_side: OpenSide::Center,
        tabs: None,
        power: 80.0,
        pierce_time: 0.3,
        linearize: None,
        feed: 1500.0,
        clearance_z: 5.0,
        cut_z: 0.0,
        model_excess: 0.0,
    }
}

#[test]
fn the_laser_cuts_what_the_waterjet_cuts_and_speaks_as_the_laser() {
    let (loops, part) = plate();
    let p = params();
    let path = laser_profile(&loops, &[], &part, &p).unwrap();
    let jet = waterjet_profile(&loops, &[], &part, &WaterjetParams { voice: Voice::WATERJET, ..p.as_sheet() }).unwrap();
    assert_eq!(path.moves, jet.moves, "the same sheet laws, the same moves");
    let pierces = path.moves.iter().filter(|m| m.kind == MoveKind::Rapid && m.to.z == 0.0 && m.from.z > 0.0).count();
    assert_eq!(pierces, 2, "one pierce for the hole, one for the outline");
    assert!(path.notes.iter().all(|n| n.starts_with("laser: ")), "{:?}", path.notes);
    assert!(!path.notes.iter().any(|n| n.contains("abrasive") || n.contains("jet ")), "{:?}", path.notes);
    assert!(path.notes.iter().any(|n| n.contains("2 contour(s) cut inner first (1 hole(s), 0 tab(s)), 2 pierce(s), kerf 0.200 mm")), "{:?}", path.notes);
    assert!(path.notes.iter().any(|n| n.contains("pierce time 0.30 s at each pierce; beam and dwell records")), "{:?}", path.notes);
    assert!(path.notes.iter().any(|n| n.contains("requested power 80.0 %")), "{:?}", path.notes);
}

#[test]
fn the_power_is_refused_by_name_outside_its_range() {
    let (loops, part) = plate();
    for bad in [0.0, -5.0, 100.5, f64::NAN] {
        let p = LaserParams { power: bad, ..params() };
        let err = laser_profile(&loops, &[], &part, &p).unwrap_err().to_string();
        assert!(err.contains("laser profile: the power must be more than 0 and at most 100 %"), "{bad}: {err}");
    }
    let p = LaserParams { power: 100.0, ..params() };
    assert!(laser_profile(&loops, &[], &part, &p).is_ok(), "full power is legal");
}

#[test]
fn the_shared_refusals_speak_as_the_laser_profile() {
    let (loops, part) = plate();
    let cases: [(LaserParams, &str); 4] = [
        (LaserParams { kerf: 0.0, ..params() }, "laser profile: the kerf width must be a positive number"),
        (LaserParams { clearance_z: -1.0, ..params() }, "laser profile: the clearance plane"),
        (
            LaserParams { tabs: Some(JetTabs { count: 2, width: 1.0, at: vec![Point2::new(50.0, 0.0)] }), ..params() },
            "laser profile: the tab count is 2 but 1 tab position(s)",
        ),
        (
            LaserParams { tabs: Some(JetTabs { count: 1, width: 1.0, at: vec![Point2::new(500.0, 500.0)] }), ..params() },
            "laser profile: tab 1",
        ),
    ];
    for (p, want) in cases {
        let err = laser_profile(&loops, &[], &part, &p).unwrap_err().to_string();
        assert!(err.contains(want), "want {want:?}: {err}");
    }
    let err = laser_profile(&[], &[], &Region::default(), &params()).unwrap_err().to_string();
    assert!(err.contains("laser profile: no contour to cut"), "{err}");
}

#[test]
fn an_open_run_and_a_missing_silhouette_say_so_in_the_lasers_words() {
    let chain = vec![Point2::new(0.0, 0.0), Point2::new(40.0, 0.0)];
    let path = laser_profile(&[], &[chain], &Region::default(), &params()).unwrap();
    assert!(path.notes.iter().any(|n| n.starts_with("laser: contour C0 is an open run, cut on its centre line")), "{:?}", path.notes);
    assert!(path.notes.iter().any(|n| n.starts_with("laser: no part silhouette to judge")), "{:?}", path.notes);
}
