//! Finding 5 of the #4123 review: the load limit has its domain, every
//! reading and every over-limit record is valid, each reading above the
//! limit is named once, and each name agrees with a reading (law 6).

use camcraft_pathkernel::Point2;
use camcraft_pathkernel::guide::GuideSet;
use camcraft_pathkernel::load::{BAD_LOAD, Controlled, DOUBLE_OVERLOAD, OverLimit, Reading, STRAY_OVERLOAD, UNNAMED_OVERLOAD};
use camcraft_pathkernel::order::{OrderedSet, StartSide};
use camcraft_pathkernel::verdict::NO_REASON;

fn set() -> OrderedSet {
    OrderedSet::new(&GuideSet::default(), vec![], Point2::new(0.0, 0.0), StartSide::Inside).unwrap()
}

fn control(readings: Vec<Reading>, over: Vec<OverLimit>, limit: f64) -> Result<Controlled, &'static str> {
    Controlled::new(set(), readings, over, limit).map_err(|e| e.reason())
}

const P: Point2 = Point2 { x: 2.0, y: 3.0 };

#[test]
fn the_limit_has_its_domain() {
    for limit in [0.0, -0.5, 1.5, f64::NAN, f64::INFINITY] {
        assert_eq!(control(vec![], vec![], limit).unwrap_err(), BAD_LOAD, "{limit}");
    }
    assert_eq!(control(vec![], vec![], 0.5).unwrap().limit(), 0.5);
}

#[test]
fn readings_and_records_are_valid_by_their_type() {
    for (at, load) in [(Point2::new(f64::NAN, 0.0), 0.5), (P, f64::NAN), (P, -0.1), (P, 1.2)] {
        assert!(Reading::new(at, load).is_err(), "{at:?} {load}");
        assert!(OverLimit::new(at, load, "full width").is_err(), "{at:?} {load}");
    }
    assert_eq!(OverLimit::new(P, 1.0, " ").unwrap_err().reason(), NO_REASON);
}

#[test]
fn a_reading_over_the_limit_is_named_once() {
    let hot = vec![Reading::new(P, 1.0).unwrap()];
    assert_eq!(control(hot.clone(), vec![], 0.5).unwrap_err(), UNNAMED_OVERLOAD);
    let named = OverLimit::new(P, 1.0, "full-width first ring").unwrap();
    assert_eq!(control(hot.clone(), vec![named], 0.5).unwrap().over().len(), 1);
    assert_eq!(control(hot, vec![named, named], 0.5).unwrap_err(), DOUBLE_OVERLOAD);
    // Within the tolerance band of the limit is not over it.
    assert!(control(vec![Reading::new(P, 0.52).unwrap()], vec![], 0.5).is_ok());
}

#[test]
fn a_record_must_agree_with_a_reading_over_the_limit() {
    let hot = vec![Reading::new(P, 1.0).unwrap()];
    // Another place, another load, or a reading that is not over the limit: each is a stray record.
    let elsewhere = OverLimit::new(Point2::new(9.0, 9.0), 1.0, "full width").unwrap();
    assert_eq!(control(hot.clone(), vec![OverLimit::new(P, 1.0, "x").unwrap(), elsewhere], 0.5).unwrap_err(), STRAY_OVERLOAD);
    let other_load = OverLimit::new(P, 0.9, "full width").unwrap();
    assert_eq!(control(hot, vec![other_load], 0.5).unwrap_err(), UNNAMED_OVERLOAD);
    let cool = vec![Reading::new(P, 0.3).unwrap()];
    assert_eq!(control(cool, vec![OverLimit::new(P, 0.3, "x").unwrap()], 0.5).unwrap_err(), STRAY_OVERLOAD);
}
