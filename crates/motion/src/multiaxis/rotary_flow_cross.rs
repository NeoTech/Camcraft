//! The complete-set crossing law of the Rotary Flow kernel (#4032,
//! found in the safety-tier review of PR #4018): the kernel blends
//! only adjacent guide pairs, so the per-pair meet law in the band
//! loop never reads two guides that are NOT neighbours in the pick.
//! A guide that crosses a non-adjacent guide cannot tile one band
//! set: the bands of the two adjacent pairs fold through the same
//! ground, and the passes cut it again. This law reads every
//! non-adjacent pair of the complete set before the first band is
//! cut, and refuses a meet or a crossing by name - both guides and
//! the crossing's own spot. An axis-aligned box prefilter, padded by
//! the corner-lattice dust, skips each pair (and each segment pair)
//! that cannot come within the dust, so a clean set stays cheap. The
//! adjacent pairs keep their own law in the band loop
//! (`rotary_flow_set::meet_or_cross`), unchanged.
//!
//! Provenance: the closest points of two segments from Ericson,
//! "Real-Time Collision Detection" 5.1 (`rotary_flow_set`'s own
//! solve), and the separating-box test from the same text, chapter
//! 4.2. No vendor material consulted.

use crate::geometry::Point3;
use crate::machining2d::GenError;

use super::rotary_flow_set::{seg_seg_at, MEET_PAIR_CAP};

/// An axis-aligned box: the low and the high corner.
type Bounds = ([f64; 3], [f64; 3]);

fn bounds_of(pts: &[Point3]) -> Bounds {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for p in pts {
        for (i, x) in [p.x, p.y, p.z].into_iter().enumerate() {
            lo[i] = lo[i].min(x);
            hi[i] = hi[i].max(x);
        }
    }
    (lo, hi)
}

/// Can two boxes come within `pad` of each other? A NaN bound reads
/// as "can": the exact solve then decides, never the prefilter.
fn near(a: &Bounds, b: &Bounds, pad: f64) -> bool {
    (0..3).all(|i| !(a.0[i] > b.1[i] + pad || b.0[i] > a.1[i] + pad))
}

/// The #4032 law: no two guides of the complete set that are not
/// neighbours in the pick may meet or cross, within the
/// corner-lattice dust (`eps2` is its square). Refused by name at
/// the first meet found, in pick order. Adjacent pairs are left to
/// the band loop's own law, so their refusals read as before.
pub(super) fn refuse_crossed_set(chains: &[Vec<Point3>], eps2: f64) -> Result<(), GenError> {
    let pad = eps2.max(0.0).sqrt();
    let boxes: Vec<Bounds> = chains.iter().map(|c| bounds_of(c)).collect();
    for i in 0..chains.len() {
        for j in (i + 2)..chains.len() {
            if !near(&boxes[i], &boxes[j], pad) {
                continue;
            }
            meet_far_pair(&chains[i], &chains[j], i, j, eps2, pad)?;
        }
    }
    Ok(())
}

/// The exact meet test of one non-adjacent pair `i < j` (0-based).
fn meet_far_pair(
    a: &[Point3],
    b: &[Point3],
    i: usize,
    j: usize,
    eps2: f64,
    pad: f64,
) -> Result<(), GenError> {
    let (na, nb) = (a.len().saturating_sub(1), b.len().saturating_sub(1));
    if na == 0 || nb == 0 {
        return Ok(()); // clean_chain already refused the short chain
    }
    if na.saturating_mul(nb) > MEET_PAIR_CAP {
        return Err(GenError::Invalid(format!(
            "Rotary Flow: guide chains {} and {} hold too many points to prove that they do \
             not cross - simplify the model edges, or pick shorter chains",
            i + 1,
            j + 1
        )));
    }
    let sb: Vec<Bounds> = b.windows(2).map(bounds_of).collect();
    for (s, sa) in a.windows(2).enumerate() {
        let ba = bounds_of(sa);
        for (t, seg) in b.windows(2).enumerate() {
            if !near(&ba, &sb[t], pad) {
                continue;
            }
            let (d, w) = seg_seg_at(sa[0], sa[1], seg[0], seg[1]);
            if d * d <= eps2 {
                return Err(GenError::invalid_at(
                    format!(
                        "Rotary Flow: guide chains {} and {} meet or cross each other at \
                         ({:.3}, {:.3}, {:.3}), segment {} of chain {} - the two chains are not \
                         neighbours in the pick, so their bands fold through the same ground; \
                         pick guide chains that do not cross",
                        i + 1,
                        j + 1,
                        w.x,
                        w.y,
                        w.z,
                        s + 1,
                        i + 1
                    ),
                    w,
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p3(x: f64, y: f64, z: f64) -> Point3 {
        Point3::new(x, y, z)
    }

    #[test]
    fn a_crossing_between_non_adjacent_chains_is_refused_by_name() {
        // Chains 1 and 3 cross at (5, 0, 0); chain 2 stands clear of
        // both. The adjacent pairs (1-2, 2-3) never meet.
        let set = vec![
            vec![p3(0.0, 0.0, 0.0), p3(10.0, 0.0, 0.0)],
            vec![p3(0.0, 0.0, 9.0), p3(10.0, 0.0, 9.0)],
            vec![p3(5.0, -3.0, 0.0), p3(5.0, 3.0, 0.0)],
        ];
        let e = refuse_crossed_set(&set, 1e-12).unwrap_err();
        let msg = format!("{e}");
        assert!(msg.contains("guide chains 1 and 3 meet or cross"), "{msg}");
        assert!(msg.contains("(5.000, 0.000, 0.000)"), "{msg}");
        let at = e.location().expect("the refusal names its spot");
        assert!((at.x - 5.0).abs() < 1e-9 && at.y.abs() < 1e-9 && at.z.abs() < 1e-9, "{at:?}");
    }

    #[test]
    fn overlapping_boxes_without_a_crossing_pass() {
        // Chains 1 and 3 share a bounding box but stay 1 mm apart:
        // the prefilter lets them through, and the exact law passes.
        let set = vec![
            vec![p3(0.0, 0.0, 0.0), p3(10.0, 10.0, 0.0)],
            vec![p3(0.0, 0.0, 5.0), p3(10.0, 10.0, 5.0)],
            vec![p3(0.0, 0.0, 1.0), p3(10.0, 10.0, 1.0)],
        ];
        assert!(refuse_crossed_set(&set, 1e-12).is_ok());
        // An adjacent meet is left to the band loop's own law.
        let adjacent = vec![
            vec![p3(0.0, 0.0, 0.0), p3(10.0, 0.0, 0.0)],
            vec![p3(5.0, -3.0, 0.0), p3(5.0, 3.0, 0.0)],
        ];
        assert!(refuse_crossed_set(&adjacent, 1e-12).is_ok());
    }

    #[test]
    fn far_boxes_skip_the_pair_and_a_nan_box_reads_near() {
        let a: Bounds = ([0.0; 3], [1.0; 3]);
        let b: Bounds = ([3.0, 0.0, 0.0], [4.0, 1.0, 1.0]);
        assert!(!near(&a, &b, 0.5));
        assert!(near(&a, &b, 2.5));
        let nan: Bounds = ([f64::NAN; 3], [f64::NAN; 3]);
        assert!(near(&a, &nan, 0.0));
    }
}
