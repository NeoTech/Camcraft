//! The plasma laws' plane geometry (#672): the closed ring of a cut, even-
//! odd containment, segment distance and proper crossing, the kerf-wide
//! nearness test and the freed-area test a rapid is judged by.
//!
//! Provenance: textbook computational geometry (the orientation predicate
//! for segment crossing, O'Rourke, "Computational Geometry in C"; even-odd
//! ray parity); our own code.

use crate::geo2d::Point2;

/// Two points this close (mm) are the same point: the CL's 1 µm lattice.
const SAME: f64 = 1e-6;

/// The closed ring a cut rides: the stretch from the first point the path
/// returns to, up to (not including) that return. A closed contour's lit
/// path is lead-in, ring back to its start, lead-out (or a small hole's
/// overburn), so the ring is what lies between the two visits of the
/// start; fewer than three distinct points, or no revisit, is no ring.
#[must_use]
pub(super) fn ring_of(path: &[Point2]) -> Option<&[Point2]> {
    let key = |p: &Point2| ((p.x / SAME).round() as i64, (p.y / SAME).round() as i64);
    let mut first: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();
    for (j, p) in path.iter().enumerate() {
        match first.get(&key(p)) {
            Some(&i) if j - i >= 3 => return Some(&path[i..j]),
            Some(_) => {}
            None => {
                first.insert(key(p), j);
            }
        }
    }
    None
}

/// |shoelace area| of a ring.
#[must_use]
pub(super) fn ring_area(ring: &[Point2]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|k| {
            let (a, b) = (ring[k], ring[(k + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        .abs()
        * 0.5
}

/// Does `ring` enclose every one of `probe`'s points (strictly inside)?
#[must_use]
pub(super) fn encloses(ring: &[Point2], probe: &[Point2]) -> bool {
    !probe.is_empty() && probe.iter().all(|p| inside(*p, ring))
}

/// Even-odd point in polygon; a point on the ring itself is not inside.
#[must_use]
pub(super) fn inside(p: Point2, ring: &[Point2]) -> bool {
    let n = ring.len();
    if n < 3 {
        return false;
    }
    let mut odd = false;
    for k in 0..n {
        let (a, b) = (ring[k], ring[(k + 1) % n]);
        if pt_seg(p, a, b) <= SAME {
            return false;
        }
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y);
            if p.x < x {
                odd = !odd;
            }
        }
    }
    odd
}

/// The first point of `a` whose segment comes within `gap` of `b`'s path.
#[must_use]
pub(super) fn near(a: &[Point2], b: &[Point2], gap: f64) -> Option<Point2> {
    for s in a.windows(2) {
        for t in b.windows(2) {
            if seg_dist(s[0], s[1], t[0], t[1]) < gap {
                return Some(s[0]);
            }
        }
    }
    None
}

/// Where ring `a` properly crosses ring `b` (two edges cutting through
/// each other), or `None`: touching and shared edges are not crossings.
#[must_use]
pub(super) fn rings_cross(a: &[Point2], b: &[Point2]) -> Option<Point2> {
    let (n, m) = (a.len(), b.len());
    for i in 0..n {
        let (p, q) = (a[i], a[(i + 1) % n]);
        for j in 0..m {
            let (r, s) = (b[j], b[(j + 1) % m]);
            if proper(p, q, r, s) {
                return Some(meet(p, q, r, s));
            }
        }
    }
    None
}

/// Where segment a-b enters the area closed by `ring` (an endpoint
/// strictly inside, or the segment cutting through an edge); `None` when
/// it stays outside - a rapid leaving from the ring's own edge is not a
/// crossing.
#[must_use]
pub(super) fn crosses_area(a: Point2, b: Point2, ring: &[Point2]) -> Option<Point2> {
    if ring.len() < 3 {
        return None;
    }
    if inside(a, ring) {
        return Some(a);
    }
    if inside(b, ring) {
        return Some(b);
    }
    let n = ring.len();
    for k in 0..n {
        let (c, d) = (ring[k], ring[(k + 1) % n]);
        if proper(a, b, c, d) {
            return Some(meet(a, b, c, d));
        }
    }
    None
}

fn cross(o: Point2, a: Point2, b: Point2) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Segments a-b and c-d cut through each other (no endpoint on the other).
fn proper(a: Point2, b: Point2, c: Point2, d: Point2) -> bool {
    let (d1, d2, d3, d4) = (cross(c, d, a), cross(c, d, b), cross(a, b, c), cross(a, b, d));
    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

/// The crossing point of two segments known to cross properly.
fn meet(a: Point2, b: Point2, c: Point2, d: Point2) -> Point2 {
    let den = (b.x - a.x) * (d.y - c.y) - (b.y - a.y) * (d.x - c.x);
    if den.abs() <= f64::EPSILON {
        return a;
    }
    let t = (((c.x - a.x) * (d.y - c.y)) - ((c.y - a.y) * (d.x - c.x))) / den;
    Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
}

/// Distance from `p` to segment a-b.
#[must_use]
pub(super) fn pt_seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a.x + t * dx, a.y + t * dy);
    ((p.x - qx).powi(2) + (p.y - qy).powi(2)).sqrt()
}

/// Distance between segments a-b and c-d (0 when they cross).
#[must_use]
pub(super) fn seg_dist(a: Point2, b: Point2, c: Point2, d: Point2) -> f64 {
    if proper(a, b, c, d) {
        return 0.0;
    }
    pt_seg(a, c, d).min(pt_seg(b, c, d)).min(pt_seg(c, a, b)).min(pt_seg(d, a, b))
}
