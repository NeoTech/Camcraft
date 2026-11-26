//! Shared builders of the guide planner pins (#4176): regions through the
//! public `region` stage and the measures that the pins read.

use camcraft_pathkernel::guide::{Curve, GuideSet, Mark, MarkKind, Span};
use camcraft_pathkernel::{Bounds, KernelRegion, MaterialSide, Point2, Region, Selection, ToolShape, WorkPlane, region};

pub fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

/// The counter-clockwise polygon of `n` points on a circle.
pub fn circle(cx: f64, cy: f64, r: f64, n: usize) -> Vec<Point2> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            p(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Point2> {
    vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]
}

pub fn tool(r: f64) -> ToolShape {
    ToolShape::disc(r, 0.0, true, 30.0).unwrap()
}

pub fn xy() -> WorkPlane {
    WorkPlane::setup_xy(0.0).unwrap()
}

/// A pocket (material inside) of one outline with holes, no stock.
pub fn pocket(outer: Vec<Point2>, holes: Vec<Vec<Point2>>, r: f64) -> KernelRegion {
    let sel = Selection::face(outer, holes).unwrap();
    let b = Bounds::new(None, Region::empty(), 0.0, -5.0).unwrap();
    region(&sel, MaterialSide::Inside, &b, xy(), tool(r), 0.0).unwrap()
}

/// The single outline of the union of some polygons.
pub fn outline(parts: &[Vec<Point2>]) -> Vec<Point2> {
    let mut u = Region::empty();
    for q in parts {
        u = u.union(&Region::new(vec![camcraft_pathkernel::Loop::new(q.clone()).unwrap()]).unwrap()).unwrap();
    }
    assert_eq!(u.loops().len(), 1, "one outline");
    u.loops()[0].points().to_vec()
}

/// The material round a part (2D Offset round the part), in a stock.
pub fn around(part: Vec<Point2>, stock: Vec<Point2>, r: f64) -> KernelRegion {
    let sel = Selection::face(part, vec![]).unwrap();
    let st = Region::new(vec![camcraft_pathkernel::Loop::new(stock).unwrap()]).unwrap();
    let b = Bounds::new(Some(st), Region::empty(), 0.0, -5.0).unwrap();
    region(&sel, MaterialSide::Outside, &b, xy(), tool(r), 0.0).unwrap()
}

pub fn center(r: &KernelRegion) -> &Region {
    r.planar().unwrap().center()
}

/// The distance from `q` to the segment `a`-`b`.
pub fn seg(q: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (q.x - a.x - t * dx).hypot(q.y - a.y - t * dy)
}

/// The points of a curve in the plane.
pub fn xy_of(c: &Curve) -> Vec<Point2> {
    c.pts().iter().map(|q| q.at()).collect()
}

/// The segments of a curve (the closing one of a closed curve included).
pub fn segs(c: &Curve) -> Vec<(Point2, Point2)> {
    let v = xy_of(c);
    let mut s: Vec<(Point2, Point2)> = v.windows(2).map(|w| (w[0], w[1])).collect();
    if c.closed() {
        s.push((v[v.len() - 1], v[0]));
    }
    s
}

/// The distance from `q` to the boundary of `r`.
pub fn to_edge(r: &Region, q: Point2) -> f64 {
    r.loops()
        .iter()
        .flat_map(|l| {
            let v = l.points();
            (0..v.len()).map(move |i| seg(q, v[i], v[(i + 1) % v.len()]))
        })
        .fold(f64::INFINITY, f64::min)
}

/// The distance from `q` to the nearest curve of `set`.
pub fn to_set(set: &GuideSet, q: Point2) -> f64 {
    set.curves().iter().flat_map(segs).map(|(a, b)| seg(q, a, b)).fold(f64::INFINITY, f64::min)
}

/// True when every point of every curve of `set` is in `area` or within
/// `tol` of its boundary.
pub fn all_inside(set: &GuideSet, area: &Region, tol: f64) -> bool {
    set.curves().iter().flat_map(xy_of).all(|q| area.contains(q) || to_edge(area, q) <= tol)
}

/// The two sets have the same count and each curve of one is within `tol`
/// of the same curve of the other, both ways (the law-1 measure).
pub fn same_set(a: &GuideSet, b: &GuideSet, tol: f64) -> bool {
    a.curves().len() == b.curves().len()
        && a.curves().iter().zip(b.curves()).all(|(x, y)| {
            x.k() == y.k()
                && x.closed() == y.closed()
                && xy_of(x).iter().all(|q| segs(y).iter().any(|(s, e)| seg(*q, *s, *e) <= tol))
                && xy_of(y).iter().all(|q| segs(x).iter().any(|(s, e)| seg(*q, *s, *e) <= tol))
        })
}

/// The marks of one kind.
pub fn marks(set: &GuideSet, kind: MarkKind) -> Vec<&Mark> {
    set.marks().iter().filter(|m| m.kind() == kind).collect()
}

/// The points of a marked stretch (all the curve's points for a whole mark).
pub fn mark_pts(set: &GuideSet, m: &Mark) -> Vec<Point2> {
    match m.span() {
        Span::Whole => xy_of(&set.curves()[m.curve()]),
        Span::Part(v) => v.iter().map(|q| q.at()).collect(),
    }
}
