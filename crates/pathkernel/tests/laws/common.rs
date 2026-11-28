//! Shared builders of the law suite.

use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::{Loop, PlanePoint, Point2, Region, ToolShape};

pub fn pts(xy: &[(f64, f64)]) -> Vec<PlanePoint> {
    xy.iter().map(|&(x, y)| PlanePoint::flat(Point2::new(x, y)).expect("finite")).collect()
}

pub fn curve(xy: &[(f64, f64)], closed: bool, k: u32, family: Family) -> Curve {
    Curve::new(pts(xy), None, closed, k, family).expect("valid curve")
}

/// The square loop from (x0, y0) with side `s`, counter-clockwise.
pub fn square(x0: f64, y0: f64, s: f64) -> Loop {
    let p = |x, y| Point2::new(x, y);
    Loop::new(vec![p(x0, y0), p(x0 + s, y0), p(x0 + s, y0 + s), p(x0, y0 + s)]).expect("square")
}

pub fn square_region(x0: f64, y0: f64, s: f64) -> Region {
    Region::new(vec![square(x0, y0, s)]).expect("region")
}

pub fn tool(radius: f64, can_plunge: bool) -> ToolShape {
    ToolShape::disc(radius, 0.0, can_plunge, 20.0).expect("tool")
}
