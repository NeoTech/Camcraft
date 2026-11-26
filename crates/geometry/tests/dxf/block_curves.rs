//! A SPLINE or ELLIPSE in a block keeps the stated curve tolerance in world units after its
//! INSERT placement (#1865 review of PR #4052): the curve is fitted after the placement, and a
//! near-similarity never passes as an exact circle.
//! Provenance: hand-built fixtures from the public DXF BLOCK, INSERT, SPLINE, ELLIPSE and CIRCLE
//! group codes; the true curves are the exact affine images of the drawn ones.

use camcraft_geometry::dxf::{CURVE_TOLERANCE_MM, Document, Primitive, parse};
use camcraft_geometry::geo2d::Point2;
use std::f64::consts::TAU;

fn drawing(blocks: &str, entities: &str) -> String {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n\
         0\nSECTION\n2\nBLOCKS\n{blocks}0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
}

fn block(name: &str, body: &str) -> String {
    format!("0\nBLOCK\n8\n0\n2\n{name}\n70\n0\n10\n0\n20\n0\n30\n0\n3\n{name}\n{body}0\nENDBLK\n8\n0\n")
}

fn insert(name: &str, more: &str) -> String {
    format!("0\nINSERT\n8\n0\n2\n{name}\n10\n0\n20\n0\n30\n0\n{more}")
}

/// A full ELLIPSE at the origin, major axis (10, 0), with the axis ratio `ratio`.
fn ellipse(ratio: &str) -> String {
    format!("0\nELLIPSE\n8\n0\n10\n0\n20\n0\n11\n10\n21\n0\n40\n{ratio}\n41\n0\n42\n6.283185307179586\n")
}

/// The quadratic Bezier (0, 0), (5, 10), (10, 0) as a SPLINE.
const SPLINE: &str = "0\nSPLINE\n8\n0\n70\n8\n71\n2\n72\n6\n73\n3\n74\n0\n\
    40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n10\n0\n20\n0\n30\n0\n10\n5\n20\n10\n30\n0\n10\n10\n20\n0\n30\n0\n";

fn bezier(sx: f64, sy: f64, t: f64) -> Point2 {
    // The first control point is the origin: its term is zero.
    let (b, c) = (2.0 * t * (1.0 - t), t * t);
    Point2::new(sx * (5.0 * b + 10.0 * c), sy * 10.0 * b)
}

fn read(text: &str) -> Document {
    parse(text.as_bytes()).unwrap_or_else(|e| panic!("reads: {e}"))
}

fn ends(p: &Primitive) -> (Point2, Point2) {
    match *p {
        Primitive::Line { start, end } => (start, end),
        Primitive::Arc { center, radius, start_angle, sweep_angle } => {
            let at = |a: f64| Point2::new(center.x + radius * a.cos(), center.y + radius * a.sin());
            (at(start_angle), at(start_angle + sweep_angle))
        }
        Primitive::Circle { center, radius } => {
            let p = Point2::new(center.x + radius, center.y);
            (p, p)
        }
    }
}

fn distance(q: Point2, p: &Primitive) -> f64 {
    match *p {
        Primitive::Line { start, end } => {
            let (dx, dy) = (end.x - start.x, end.y - start.y);
            let t = (((q.x - start.x) * dx + (q.y - start.y) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            (start.x + t * dx - q.x).hypot(start.y + t * dy - q.y)
        }
        Primitive::Circle { center, radius } => ((q.x - center.x).hypot(q.y - center.y) - radius).abs(),
        Primitive::Arc { center, radius, start_angle, sweep_angle } => {
            let a = (q.y - center.y).atan2(q.x - center.x);
            let (lo, sweep) = if sweep_angle >= 0.0 { (start_angle, sweep_angle) } else { (start_angle + sweep_angle, -sweep_angle) };
            if (a - lo).rem_euclid(TAU) <= sweep {
                ((q.x - center.x).hypot(q.y - center.y) - radius).abs()
            } else {
                let (s, e) = ends(p);
                (q.x - s.x).hypot(q.y - s.y).min((q.x - e.x).hypot(q.y - e.y))
            }
        }
    }
}

/// Each sampled point of the true curve is within the tolerance of the output, the document
/// records the tolerance, and the pieces join end to end.
fn within_bound(doc: &Document, curve: impl Fn(f64) -> Point2) {
    assert_eq!(doc.curve_tolerance, Some(CURVE_TOLERANCE_MM), "{:?}", doc.primitives);
    let out: Vec<Primitive> = doc.primitives.iter().map(|p| p.primitive.clone()).collect();
    for i in 0..=4000 {
        let q = curve(i as f64 / 4000.0);
        let d = out.iter().map(|p| distance(q, p)).fold(f64::INFINITY, f64::min);
        assert!(d <= CURVE_TOLERANCE_MM * 1.01, "true point {q:?} is {d} mm from the output");
    }
    for w in out.windows(2) {
        let (_, a) = ends(&w[0]);
        let (b, _) = ends(&w[1]);
        assert!((a.x - b.x).hypot(a.y - b.y) < 1e-6, "{w:?}");
    }
}

/// The review case: a nearly round ELLIPSE (ratio 0.99995, major 10) is a circle within the
/// tolerance at its drawn size, but not at a scale of 100.
#[test]
fn a_scaled_near_round_ellipse_keeps_the_bound_in_world_units() {
    let doc = read(&drawing(&block("E", &ellipse("0.99995")), &insert("E", "41\n100\n42\n100\n")));
    within_bound(&doc, |u| Point2::new(1000.0 * (TAU * u).cos(), 999.95 * (TAU * u).sin()));
}

#[test]
fn a_nested_scaled_ellipse_keeps_the_bound_in_world_units() {
    let blocks = format!("{}{}", block("E", &ellipse("0.99995")), block("M", &insert("E", "41\n10\n42\n10\n")));
    let doc = read(&drawing(&blocks, &insert("M", "41\n10\n42\n10\n50\n30\n")));
    let (s, c) = 30f64.to_radians().sin_cos();
    within_bound(&doc, |u| {
        let (x, y) = (1000.0 * (TAU * u).cos(), 999.95 * (TAU * u).sin());
        Point2::new(c * x - s * y, s * x + c * y)
    });
}

#[test]
fn an_unequally_scaled_ellipse_keeps_the_bound_in_world_units() {
    let doc = read(&drawing(&block("E", &ellipse("0.5")), &insert("E", "41\n30\n42\n70\n")));
    within_bound(&doc, |u| Point2::new(300.0 * (TAU * u).cos(), 350.0 * (TAU * u).sin()));
}

#[test]
fn a_scaled_spline_keeps_the_bound_in_world_units() {
    for (sx, sy) in [(100.0, 100.0), (100.0, 40.0), (-100.0, 100.0)] {
        let doc = read(&drawing(&block("S", SPLINE), &insert("S", &format!("41\n{sx}\n42\n{sy}\n"))));
        within_bound(&doc, |t| bezier(sx, sy, t));
    }
}

#[test]
fn a_nested_scaled_spline_keeps_the_bound_in_world_units() {
    let blocks = format!("{}{}", block("S", SPLINE), block("M", &insert("S", "41\n10\n42\n4\n")));
    let doc = read(&drawing(&blocks, &insert("M", "41\n10\n42\n10\n")));
    within_bound(&doc, |t| bezier(100.0, 40.0, t));
}

/// The review case: radius 1000 with X and Y scales 1e8 and 1e8 + 5e-5. The true Y extremum
/// is 0.050003 mm off the circle of the X scale, thus the placement is no exact circle. Within
/// the tolerance its ellipse needs more arcs than the import budget: it is refused by name.
#[test]
fn a_near_similarity_is_no_exact_circle() {
    let body = "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n1000\n";
    let text = drawing(&block("C", body), &insert("C", "41\n100000000\n42\n100000000.00005\n"));
    let e = parse(text.as_bytes()).expect_err("no circle off by 0.05 mm");
    assert!(e.0.contains("INSERT") && e.0.contains("tolerance"), "{e}");
}

/// A near similarity within the tolerance is a circle, and the document records the bound.
#[test]
fn a_near_similarity_within_the_tolerance_records_it() {
    let body = "0\nCIRCLE\n8\n0\n10\n0\n20\n0\n40\n10\n";
    let doc = read(&drawing(&block("C", body), &insert("C", "41\n1\n42\n1.00005\n")));
    within_bound(&doc, |u| Point2::new(10.0 * (TAU * u).cos(), 10.0005 * (TAU * u).sin()));
    assert_eq!(doc.primitives.len(), 1, "{:?}", doc.primitives);
}
