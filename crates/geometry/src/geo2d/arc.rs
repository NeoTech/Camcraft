//! Circular arcs as chord points, and the test that a polyline stays in a
//! region (#4178: the arc leads of the path kernel's entry and exit stage).
//! Provenance: the sagitta of a chord, s = r (1 - cos(t / 2)), from any
//! geometry handbook; the containment test rides [`Region::clip_polyline`].
//! Our own code, no vendor material consulted.

use super::{COORD_LIMIT, GeoError, Point2, Region};

/// The most chords that one arc may have. A finer request is refused, not
/// cut short: a caller that needs more asks for a coarser tolerance.
pub const MAX_ARC_CHORDS: usize = 4096;

/// The largest turn of one chord (radians): a coarse tolerance still gives
/// an arc that looks like an arc.
const MAX_CHORD_TURN: f64 = std::f64::consts::FRAC_PI_4;

/// A stretch of a path outside a region that is shorter than this (mm) is
/// a touch at one point, not a stretch.
const TOUCH: f64 = 1e-9;

/// The points of the arc about `center` that starts at `start` and turns by
/// `sweep` radians (positive is counter-clockwise), with no chord farther
/// than `chord_tol` from the arc. The first point is `start` exactly; the
/// last point is the end of the arc. Refused: a value that is not finite, a
/// radius of zero, a sweep of zero or of more than one full turn, a
/// tolerance of zero or less, or more than [`MAX_ARC_CHORDS`] chords.
pub fn arc_points(center: Point2, start: Point2, sweep: f64, chord_tol: f64) -> Result<Vec<Point2>, GeoError> {
    let values = [center.x, center.y, start.x, start.y, sweep, chord_tol];
    if values.iter().any(|v| !v.is_finite()) {
        return Err(GeoError::Invalid("non-finite arc value".into()));
    }
    if values[..4].iter().any(|v| v.abs() >= COORD_LIMIT) {
        return Err(GeoError::OutOfRange);
    }
    let r = (start.x - center.x).hypot(start.y - center.y);
    let full = std::f64::consts::TAU;
    if !(r > 0.0 && chord_tol > 0.0 && sweep != 0.0 && sweep.abs() <= full * (1.0 + 1e-12)) {
        return Err(GeoError::Invalid("an arc needs a radius, a sweep up to one turn and a tolerance above zero".into()));
    }
    let turn = if chord_tol < r { (2.0 * (1.0 - chord_tol / r).acos()).min(MAX_CHORD_TURN) } else { MAX_CHORD_TURN };
    let steps = (sweep.abs() / turn).ceil();
    if !(steps.is_finite() && steps >= 1.0 && steps <= MAX_ARC_CHORDS as f64) {
        return Err(GeoError::Invalid("the arc needs more chords than the limit".into()));
    }
    // `steps` is a whole number in [1, MAX_ARC_CHORDS]: the cast is exact.
    let n = steps as usize;
    let a0 = (start.y - center.y).atan2(start.x - center.x);
    let mut pts = Vec::with_capacity(n + 1);
    pts.push(start);
    for i in 1..=n {
        let a = a0 + sweep * (i as f64) / steps;
        pts.push(Point2::new(center.x + r * a.cos(), center.y + r * a.sin()));
    }
    Ok(pts)
}

impl Region {
    /// True when no stretch of the open polyline `path` lies outside this
    /// region. A single point is held when the region contains it. On the
    /// boundary, [`Region::contains`]' half-open rule decides: a caller
    /// that needs a definite answer there tests against the region grown
    /// by its tolerance. A non-finite or out-of-range vertex is refused.
    pub fn holds_path(&self, path: &[Point2]) -> Result<bool, GeoError> {
        match path {
            [] => Ok(true),
            [p] => {
                if !(p.x.is_finite() && p.y.is_finite()) {
                    return Err(GeoError::Invalid("non-finite path vertex".into()));
                }
                Ok(self.contains(*p))
            }
            _ => {
                let outside = self.clip_polyline(path, false)?;
                let len = |s: &Vec<Point2>| s.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).sum::<f64>();
                Ok(outside.iter().all(|s| len(s) <= TOUCH))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo2d::Loop;
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    fn annulus() -> Region {
        let sq = |a: f64, b: f64| {
            let Ok(l) = Loop::new(vec![p(a, a), p(b, a), p(b, b), p(a, b)]) else { panic!("loop") };
            l
        };
        let Ok(r) = Region::new(vec![sq(0.0, 10.0), sq(4.0, 6.0)]) else { panic!("region") };
        r
    }

    #[test]
    fn a_quarter_arc_starts_and_ends_on_its_circle_within_the_tolerance() {
        let Ok(pts) = arc_points(p(0.0, 0.0), p(5.0, 0.0), FRAC_PI_2, 0.01) else { panic!("arc") };
        assert_eq!(pts.first().copied(), Some(p(5.0, 0.0)));
        let end = pts.last().copied().unwrap_or_default();
        assert!((end.x).abs() < 1e-9 && (end.y - 5.0).abs() < 1e-9, "{end:?}");
        for w in pts.windows(2) {
            let mid = p(0.5 * (w[0].x + w[1].x), 0.5 * (w[0].y + w[1].y));
            assert!(5.0 - mid.x.hypot(mid.y) <= 0.01 + 1e-12, "a chord leaves the tolerance");
            assert!((w[0].x.hypot(w[0].y) - 5.0).abs() < 1e-9);
        }
        // A clockwise sweep turns the other way.
        let Ok(cw) = arc_points(p(0.0, 0.0), p(5.0, 0.0), -FRAC_PI_2, 0.01) else { panic!("arc") };
        let end = cw.last().copied().unwrap_or_default();
        assert!((end.y + 5.0).abs() < 1e-9, "{end:?}");
    }

    #[test]
    fn a_coarse_tolerance_still_turns_at_most_an_eighth_of_a_turn_a_chord() {
        let Ok(pts) = arc_points(p(0.0, 0.0), p(1.0, 0.0), PI, 10.0) else { panic!("arc") };
        assert_eq!(pts.len(), 5);
        let Ok(full) = arc_points(p(0.0, 0.0), p(1.0, 0.0), TAU, 10.0) else { panic!("arc") };
        assert_eq!(full.len(), 9);
    }

    #[test]
    fn a_bad_arc_is_refused() {
        let o = p(0.0, 0.0);
        assert!(arc_points(o, o, 1.0, 0.01).is_err(), "radius zero");
        assert!(arc_points(o, p(1.0, 0.0), 0.0, 0.01).is_err(), "sweep zero");
        assert!(arc_points(o, p(1.0, 0.0), 7.0, 0.01).is_err(), "more than one turn");
        assert!(arc_points(o, p(1.0, 0.0), 1.0, 0.0).is_err(), "tolerance zero");
        assert!(arc_points(o, p(f64::NAN, 0.0), 1.0, 0.01).is_err(), "not finite");
        assert!(arc_points(o, p(1e13, 0.0), 1.0, 0.01).is_err(), "out of range");
        assert!(arc_points(o, p(1000.0, 0.0), TAU, 1e-9).is_err(), "too many chords");
    }

    #[test]
    fn a_path_is_held_only_when_no_stretch_leaves_the_region() {
        let r = annulus();
        assert!(matches!(r.holds_path(&[p(1.0, 1.0), p(9.0, 1.0), p(9.0, 9.0)]), Ok(true)));
        assert!(matches!(r.holds_path(&[p(1.0, 1.0), p(9.0, 9.0)]), Ok(false)), "crosses the hole");
        assert!(matches!(r.holds_path(&[p(1.0, 1.0), p(12.0, 1.0)]), Ok(false)), "leaves the outline");
        assert!(matches!(r.holds_path(&[p(5.0, 5.0)]), Ok(false)), "a point in the hole");
        assert!(matches!(r.holds_path(&[p(1.0, 5.0)]), Ok(true)));
        assert!(matches!(r.holds_path(&[]), Ok(true)));
        assert!(r.holds_path(&[p(f64::NAN, 1.0)]).is_err());
        assert!(r.holds_path(&[p(1.0, 1.0), p(f64::NAN, 1.0)]).is_err());
    }
}
