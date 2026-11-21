//! The facet allowance of a meshed round, for the cover law (#3208).
//!
//! A model's mesh draws a small fillet in a few chords: an r0.25 round
//! in four 22.5° facets. The wire path is the polygon offset of those
//! chords, so at each facet vertex it stands `k / cos(θ/2)` off the
//! profile, not `k` - 3.2 µm too far at a 0.162 mm offset - although
//! the true round (radius >= `k`) is one the wire cuts. A vertex is a
//! facet of such a round when:
//!
//! - it turns no more than [`MAX_FACET_TURN_DEG`];
//! - the circle through it and its two neighbours has a radius of at
//!   least the kerf offset `k`, and the chord stands no more than
//!   [`MAX_FACET_SAG`] off that circle (a mesh, not a drawn polygon);
//! - a neighbour vertex lies on the same circle in the same way (a run
//!   of facets, not a lone kink between long sides).
//!
//! Only such a vertex gets an allowance: the exact mitre excess
//! `k (1 / cos(θ/2) - 1)`. A sharp corner, a lone kink and a round under
//! the minimum get none, and the cover law refuses them as before.
//!
//! Provenance: plain circle geometry (the circumcircle of three points)
//! and the mitre of a textbook polygon offset. Our own code.

use crate::geo2d::Point2;

use super::ring::dist;

/// The largest turn (degrees) at a vertex that is read as a facet of a
/// meshed round: coarser than any mesh of a round we accept.
pub(super) const MAX_FACET_TURN_DEG: f64 = 30.0;
/// The largest chord sag (mm) of a facet: the distance between a chord
/// and its round. A mesh keeps it to hundredths of a millimetre; a
/// drawn polygon with sharp corners has more.
pub(super) const MAX_FACET_SAG: f64 = 0.02;
/// How near (relative to the radius, plus 1 µm) two facets' circles
/// must agree to be one round.
const SAME_CIRCLE: f64 = 0.02;

/// The circle through `a`, `b`, `c`: its centre and radius, or `None`
/// for collinear or non-finite points.
fn circle(a: Point2, b: Point2, c: Point2) -> Option<(Point2, f64)> {
    // Translate to `b` for precision on short chords.
    let (ax, ay, cx, cy) = (a.x - b.x, a.y - b.y, c.x - b.x, c.y - b.y);
    let d = 2.0 * (ax * cy - ay * cx);
    if d.is_nan() || d.abs() <= 1e-18 {
        return None;
    }
    let (a2, c2) = (ax * ax + ay * ay, cx * cx + cy * cy);
    let ux = (cy * a2 - ay * c2) / d;
    let uy = (ax * c2 - cx * a2) / d;
    let r = ux.hypot(uy);
    (ux.is_finite() && uy.is_finite() && r.is_finite()).then(|| (Point2::new(b.x + ux, b.y + uy), r))
}

/// The signed turn (radians) at `b` from `a` to `c`.
fn turn(a: Point2, b: Point2, c: Point2) -> f64 {
    let (ux, uy, vx, vy) = (b.x - a.x, b.y - a.y, c.x - b.x, c.y - b.y);
    (ux * vy - uy * vx).atan2(ux * vx + uy * vy)
}

/// A facet vertex's turn and circle, or `None` when the vertex at `i`
/// cannot be a facet of a meshed round of radius `k` or more.
fn facet(pts: &[Point2], closed: bool, i: usize, k: f64) -> Option<(f64, Point2, f64)> {
    let n = pts.len();
    let (h, j) = neighbours(n, closed, i)?;
    let (a, b, c) = (pts[h], pts[i], pts[j]);
    let t = turn(a, b, c);
    if t.is_nan() || t.abs() > MAX_FACET_TURN_DEG.to_radians() + 1e-9 || t == 0.0 {
        return None;
    }
    let (centre, r) = circle(a, b, c)?;
    let chord = dist(a, b).max(dist(b, c));
    let sag = r - (r * r - chord * chord * 0.25).max(0.0).sqrt();
    (r >= k && sag <= MAX_FACET_SAG).then_some((t, centre, r))
}

fn neighbours(n: usize, closed: bool, i: usize) -> Option<(usize, usize)> {
    if n < 3 || i >= n {
        return None;
    }
    if closed {
        Some(((i + n - 1) % n, (i + 1) % n))
    } else {
        (i > 0 && i + 1 < n).then(|| (i - 1, i + 1))
    }
}

/// The allowance (mm) past the offset `k` at each vertex of `pts`: the
/// mitre excess at a facet of a meshed round, zero elsewhere.
pub(super) fn facet_allowance(pts: &[Point2], closed: bool, k: f64) -> Vec<f64> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let Some((t, centre, r)) = facet(pts, closed, i, k) else {
                return 0.0;
            };
            let Some((h, j)) = neighbours(n, closed, i) else {
                return 0.0;
            };
            let tol = SAME_CIRCLE * r + 1e-6;
            let run = [h, j].into_iter().any(|m| {
                facet(pts, closed, m, k)
                    .is_some_and(|(tm, cm, rm)| tm.signum() == t.signum() && dist(cm, centre) <= tol && (rm - r).abs() <= tol)
            });
            if run {
                k * (1.0 / (t * 0.5).cos() - 1.0)
            } else {
                0.0
            }
        })
        .collect()
}
