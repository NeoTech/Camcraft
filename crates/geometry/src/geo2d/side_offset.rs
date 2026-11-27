//! The offset of an open polyline to one side (#4176): the curves at
//! distance `|d|` from the polyline on its left (`d > 0`) or its right
//! (`d < 0`), trimmed where the polyline comes back nearer than `|d|`.
//! Provenance: the offset curve as the boundary of the Minkowski sum of the
//! polyline and a disc, with the parts that are nearer to another part of
//! the polyline removed (the global invalid loops of Tiller and Hanson,
//! "Offsets of two-dimensional profiles", IEEE CG&A 4, 1984; Choi and
//! Park, "A pair-wise offset algorithm for 2D point-sequence curve", CAD
//! 31, 1999). The sum is [`Region::sweep`]; the side and end-cap tests are
//! ours. Written new; no vendor material consulted.

use super::{GeoError, Point2, Region};

/// One piece of a side offset: its points in the direction of the
/// polyline, and whether it is a closed loop.
#[derive(Clone, Debug, PartialEq)]
pub struct SidePiece {
    pub points: Vec<Point2>,
    pub closed: bool,
}

/// The tolerance (mm) of the end-cap and side tests.
const SIDE_TOL: f64 = 1e-6;

fn sub(a: Point2, b: Point2) -> (f64, f64) {
    (a.x - b.x, a.y - b.y)
}

/// The nearest place of `q` on `path`: the distance, the segment and the
/// parameter on it.
fn nearest(path: &[Point2], q: Point2) -> (f64, usize, f64) {
    let mut best = (f64::INFINITY, 0, 0.0);
    for (i, w) in path.windows(2).enumerate() {
        let (dx, dy) = sub(w[1], w[0]);
        let l2 = dx * dx + dy * dy;
        let t = if l2 > 0.0 { (((q.x - w[0].x) * dx + (q.y - w[0].y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let d = (q.x - w[0].x - t * dx).hypot(q.y - w[0].y - t * dy);
        if d < best.0 - SIDE_TOL {
            best = (d, i, t);
        }
    }
    best
}

fn unit(a: Point2, b: Point2) -> (f64, f64) {
    let (dx, dy) = sub(b, a);
    let l = dx.hypot(dy);
    if l > 0.0 { (dx / l, dy / l) } else { (0.0, 0.0) }
}

/// True when `q` (on the boundary of the swept area) is on the left of the
/// path and not on an end cap.
fn keep_left(path: &[Point2], q: Point2) -> bool {
    let n = path.len();
    let (_, i, t) = nearest(path, q);
    let (a, b) = (path[i], path[i + 1]);
    let u = unit(a, b);
    let v = sub(q, a);
    let at_start = i == 0 && t <= 0.0;
    let at_end = i + 2 == n && t >= 1.0;
    if at_start {
        return v.0 * u.0 + v.1 * u.1 >= -SIDE_TOL && u.0 * v.1 - u.1 * v.0 > 0.0;
    }
    if at_end {
        let w = sub(q, b);
        return w.0 * u.0 + w.1 * u.1 <= SIDE_TOL && u.0 * w.1 - u.1 * w.0 > 0.0;
    }
    if t > 0.0 && t < 1.0 {
        return u.0 * v.1 - u.1 * v.0 > 0.0;
    }
    // At an inner vertex: the side of the sum of the two left normals.
    let (c, prev, next) = if t <= 0.0 { (a, path[i - 1], b) } else { (b, a, path[i + 2]) };
    let (u1, u2) = (unit(prev, c), unit(c, next));
    let m = (-u1.1 - u2.1, u1.0 + u2.0);
    let w = sub(q, c);
    if m.0.hypot(m.1) > SIDE_TOL { w.0 * m.0 + w.1 * m.1 > 0.0 } else { u1.0 * w.1 - u1.1 * w.0 > 0.0 }
}

/// The offset of the open polyline `path` by `d`: to its left for `d > 0`,
/// to its right for `d < 0`. The pieces run in the direction of `path`.
/// The round joins are flattened as the region offset (5 um). Refused for
/// a path of fewer than two points, a point or a distance that is not
/// finite, or `d` of zero.
pub fn offset_side(path: &[Point2], d: f64) -> Result<Vec<SidePiece>, GeoError> {
    if path.len() < 2 || !path.iter().all(|p| p.x.is_finite() && p.y.is_finite()) {
        return Err(GeoError::Invalid("side offset: the path needs two or more finite points".into()));
    }
    if !d.is_finite() || d == 0.0 {
        return Err(GeoError::Invalid("side offset: the distance must be finite and not zero".into()));
    }
    // The right side of the path is the left side of the reversed path.
    let mut walk: Vec<Point2> = path.to_vec();
    walk.dedup_by(|a, b| (a.x - b.x).hypot(a.y - b.y) <= SIDE_TOL);
    if walk.len() < 2 {
        return Err(GeoError::Invalid("side offset: the path has no length".into()));
    }
    if d < 0.0 {
        walk.reverse();
    }
    let tube = Region::sweep(std::slice::from_ref(&walk), d.abs())?;
    let mut out = Vec::new();
    for l in tube.loops() {
        let v = l.points();
        let kept: Vec<bool> = v.iter().map(|q| keep_left(&walk, *q)).collect();
        if kept.iter().all(|k| *k) {
            out.push(SidePiece { points: v.to_vec(), closed: true });
            continue;
        }
        // Runs of kept points, read round the loop from a point that is out.
        let Some(s) = kept.iter().position(|k| !k) else { continue };
        let mut run: Vec<Point2> = Vec::new();
        for j in 1..=v.len() {
            let i = (s + j) % v.len();
            if kept[i] {
                run.push(v[i]);
            } else if !run.is_empty() {
                out.push(SidePiece { points: std::mem::take(&mut run), closed: false });
            }
        }
    }
    out.retain(|p| p.points.len() >= 2);
    // Each piece in the direction of the path (the walk for d < 0 is the
    // path reversed, so its pieces turn back).
    for p in &mut out {
        let place = |q: &Point2| {
            let (_, i, t) = nearest(&walk, *q);
            i as f64 + t
        };
        let forward = match (p.points.first(), p.points.last()) {
            (Some(a), Some(b)) => place(a) <= place(b),
            _ => true,
        };
        if forward == (d < 0.0) {
            p.points.reverse();
        }
    }
    out.sort_by(|a, b| {
        let key = |p: &SidePiece| p.points.first().map_or((0.0, 0.0), |q| (q.y, q.x));
        key(a).0.total_cmp(&key(b).0).then(key(a).1.total_cmp(&key(b).1))
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    #[test]
    fn a_straight_line_is_offset_to_its_left_and_its_right() {
        let line = [p(0.0, 0.0), p(10.0, 0.0)];
        let left = offset_side(&line, 2.0).unwrap();
        assert_eq!(left.len(), 1);
        assert!(left[0].points.iter().all(|q| (q.y - 2.0).abs() < 1e-6 && q.x > -1e-6 && q.x < 10.0 + 1e-6), "{left:?}");
        assert!(left[0].points.first().unwrap().x < left[0].points.last().unwrap().x, "in the direction of the path");
        let right = offset_side(&line, -2.0).unwrap();
        assert_eq!(right.len(), 1);
        assert!(right[0].points.iter().all(|q| (q.y + 2.0).abs() < 1e-6), "{right:?}");
        assert!(right[0].points.first().unwrap().x < right[0].points.last().unwrap().x);
        let span = |s: &[SidePiece]| s[0].points.iter().map(|q| q.x).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(x), b.max(x)));
        let (a, b) = span(&left);
        assert!(a.abs() < 1e-6 && (b - 10.0).abs() < 1e-6, "the ends are the normal offsets of the path ends: {a} {b}");
    }

    #[test]
    fn a_corner_is_round_outside_and_sharp_inside() {
        // An L turning left at (10, 0): the left side is the inner side.
        let l = [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)];
        let inner = offset_side(&l, 2.0).unwrap();
        assert_eq!(inner.len(), 1);
        assert!(inner[0].points.iter().any(|q| (q.x - 8.0).abs() < 1e-6 && (q.y - 2.0).abs() < 1e-6), "the sharp inner corner");
        let outer = offset_side(&l, -2.0).unwrap();
        assert_eq!(outer.len(), 1);
        let d = |q: &Point2| (q.x - 10.0).hypot(q.y);
        let round: Vec<&Point2> = outer[0].points.iter().filter(|q| q.x > 10.0 && q.y < 0.0).collect();
        assert!(!round.is_empty() && round.iter().all(|q| (d(q) - 2.0).abs() < 1e-2), "the round join");
    }

    #[test]
    fn a_hairpin_trims_the_inner_offset() {
        // Two legs 2 mm apart: the offset of 3 mm between them is trimmed away.
        let u = [p(0.0, 0.0), p(20.0, 0.0), p(20.0, 2.0), p(0.0, 2.0)];
        let between = offset_side(&u, 3.0).unwrap();
        for s in &between {
            assert!(s.points.iter().all(|q| nearest(&u, *q).0 >= 3.0 - 1e-3), "{s:?}");
        }
    }

    #[test]
    fn bad_input_is_refused() {
        assert!(offset_side(&[p(0.0, 0.0)], 1.0).is_err());
        assert!(offset_side(&[p(0.0, 0.0), p(1.0, 0.0)], 0.0).is_err());
        assert!(offset_side(&[p(0.0, 0.0), p(1.0, 0.0)], f64::NAN).is_err());
        assert!(offset_side(&[p(0.0, 0.0), p(f64::INFINITY, 0.0)], 1.0).is_err());
        assert!(offset_side(&[p(1.0, 1.0), p(1.0, 1.0)], 1.0).is_err());
    }
}
