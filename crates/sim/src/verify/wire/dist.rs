//! Plain 2D distances for the wire laws: point to point, point to
//! segment, segment to segment, and a move's clearance to the product.
//! Provenance: textbook Euclidean geometry (closest points of segments).
//! Our own code.

use super::Product;
use crate::geo2d::Point2;

pub(super) fn pp(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The point of segment `a`-`b` nearest `p`.
fn foot(p: Point2, a: Point2, b: Point2) -> Point2 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    if l2.is_nan() || l2 <= 0.0 {
        return a;
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0);
    Point2::new(a.x + t * dx, a.y + t * dy)
}

fn cross(o: Point2, a: Point2, b: Point2) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Whether segments `a`-`b` and `c`-`d` cross or touch (proper or not).
fn crosses(a: Point2, b: Point2, c: Point2, d: Point2) -> bool {
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

/// The distance between segments `a`-`b` and `c`-`d`, with the point of
/// `c`-`d` nearest the first.
pub(super) fn seg_seg(a: Point2, b: Point2, c: Point2, d: Point2) -> (f64, Point2) {
    if crosses(a, b, c, d) {
        let t = cross(c, d, a) / (cross(c, d, a) - cross(c, d, b));
        return (0.0, Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)));
    }
    let mut best = (f64::INFINITY, c);
    for (p, q) in [(a, foot(a, c, d)), (b, foot(b, c, d))] {
        let dd = pp(p, q);
        if dd < best.0 {
            best = (dd, q);
        }
    }
    for q in [c, d] {
        let dd = pp(q, foot(q, a, b));
        if dd < best.0 {
            best = (dd, q);
        }
    }
    best
}

/// How far the wire's centre walking `a`-`b` stays from the product,
/// and the product's point nearest it: 0 when the move stands in the
/// product. An empty product is infinitely clear.
pub(super) fn clearance(p: &Product<'_>, a: Point2, b: Point2) -> (f64, Point2) {
    if p.contains(a) {
        return (0.0, a);
    }
    if p.contains(b) {
        return (0.0, b);
    }
    let mut best = (f64::INFINITY, a);
    for l in p.boundary().loops() {
        let pts = l.points();
        for i in 0..pts.len() {
            let (c, d) = (pts[i], pts[(i + 1) % pts.len()]);
            let got = seg_seg(a, b, c, d);
            if got.0 < best.0 {
                best = got;
            }
        }
    }
    best
}

/// The clearance of [`clearance`], signed (#3211): a move that stands in
/// the product reads minus how far its deepest point stands inside (the
/// distance to the product's boundary), with that point - so the kerf's
/// entry there is its reach plus that distance, not the reach alone.
pub(super) fn entry(p: &Product<'_>, a: Point2, b: Point2) -> (f64, Point2) {
    let (d, at) = clearance(p, a, b);
    if d > 0.0 {
        return (d, at);
    }
    let at_t = |t: f64| Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    let inside = |t: f64| {
        let q = at_t(t);
        if p.contains(q) { to_boundary(p, q) } else { 0.0 }
    };
    // The inside distance changes by at most the walked length (it is
    // 1-Lipschitz): a coarse walk, then a fine walk around its best.
    const N: usize = 32;
    let best_of = |lo: f64, hi: f64| {
        (0..=N).map(|i| lo + (hi - lo) * i as f64 / N as f64).fold((0.0, lo), |(bd, bt), t| {
            let dd = inside(t);
            if dd > bd { (dd, t) } else { (bd, bt) }
        })
    };
    let (coarse, t0) = best_of(0.0, 1.0);
    let h = 1.0 / N as f64;
    let (fine, t1) = best_of((t0 - h).max(0.0), (t0 + h).min(1.0));
    let (deep, t) = if fine > coarse { (fine, t1) } else { (coarse, t0) };
    if deep > 0.0 { (-deep, at_t(t)) } else { (d, at) }
}

/// How far `q` stands from the product's boundary.
fn to_boundary(p: &Product<'_>, q: Point2) -> f64 {
    let mut best = f64::INFINITY;
    for l in p.boundary().loops() {
        let pts = l.points();
        for i in 0..pts.len() {
            best = best.min(pp(q, foot(q, pts[i], pts[(i + 1) % pts.len()])));
        }
    }
    if best.is_finite() { best } else { 0.0 }
}
