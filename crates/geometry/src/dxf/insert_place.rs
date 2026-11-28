//! The placement of block primitives by an INSERT transform (#1865).
//! Provenance: plane affine maps (a 2x2 matrix and a translation), the conformal test for a
//! similarity, and the principal axes of the image of a circle `c + p cos t + q sin t`; our own
//! implementation.
//!
//! A line maps exactly. A circle or an arc stays exact under a similarity (equal X and Y
//! scale, any rotation); a mirror reverses the arc. Under unequal scales a circle becomes an
//! ellipse, laid as tangent arcs within the curve tolerance, as an ELLIPSE entity is.
use super::curves::{ellipse_arcs, fit_settings, tolerance};
use super::entities::{circle_range, coordinate};
use super::types::{DxfError, Primitive, State, error};
use crate::geo2d::Point2;
use std::f64::consts::TAU;

/// `(x, y, z)` to `(a x + b y + tx, c x + d y + ty, sz z + tz)`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Affine {
    m: [f64; 4],
    t: [f64; 2],
    sz: f64,
    tz: f64,
}

impl Affine {
    pub const IDENTITY: Self = Self { m: [1.0, 0.0, 0.0, 1.0], t: [0.0, 0.0], sz: 1.0, tz: 0.0 };

    /// The map `p -> at + M (p - base)`, Z by `at_z + sz (z - base_z)`.
    pub fn insert(m: [f64; 4], at: [f64; 3], sz: f64, base: [f64; 3]) -> Self {
        let [a, b, c, d] = m;
        Self {
            m,
            t: [at[0] - (a * base[0] + b * base[1]), at[1] - (c * base[0] + d * base[1])],
            sz,
            tz: at[2] - sz * base[2],
        }
    }

    /// `self` after `inner`.
    pub fn then(&self, inner: &Self) -> Self {
        let [a, b, c, d] = self.m;
        let [e, f, g, h] = inner.m;
        Self {
            m: [a * e + b * g, a * f + b * h, c * e + d * g, c * f + d * h],
            t: [a * inner.t[0] + b * inner.t[1] + self.t[0], c * inner.t[0] + d * inner.t[1] + self.t[1]],
            sz: self.sz * inner.sz,
            tz: self.sz * inner.tz + self.tz,
        }
    }

    pub fn z(&self, z: f64) -> f64 {
        self.sz * z + self.tz
    }

    pub fn linear(&self, x: f64, y: f64) -> (f64, f64) {
        let [a, b, c, d] = self.m;
        (a * x + b * y, c * x + d * y)
    }

    pub fn point(&self, p: Point2, context: &str) -> Result<Point2, DxfError> {
        let (x, y) = self.linear(p.x, p.y);
        Ok(Point2::new(coordinate(x + self.t[0], context)?, coordinate(y + self.t[1], context)?))
    }

    /// `Some(+1)` for an exact similarity that keeps the turn, `Some(-1)` for an exact mirror,
    /// else `None`. Only an exact matrix maps a circle to a circle without a stated bound: any
    /// difference d in a matrix entry moves a point of a circle of radius r by up to r d, which
    /// no relative threshold limits in drawing units. A near similarity thus takes the ellipse
    /// path, where the deviation is measured in drawing units against the curve tolerance.
    /// Equal INSERT scales give exact matrices, also when composed: `then` forms the two
    /// diagonal (and the two other) entries from the same products.
    fn similarity(&self) -> Option<f64> {
        let [a, b, c, d] = self.m;
        if a == d && b == -c {
            Some(1.0)
        } else if a == -d && b == c {
            Some(-1.0)
        } else {
            None
        }
    }
}

/// `primitive` of a block placed by `m`, in world coordinates.
pub(super) fn place(state: &mut State, m: &Affine, primitive: &Primitive, context: &str) -> Result<Vec<Primitive>, DxfError> {
    let (center, radius, start, sweep, full) = match *primitive {
        Primitive::Line { start, end } => {
            let (start, end) = (m.point(start, context)?, m.point(end, context)?);
            if start == end {
                return Err(error(context, "placed line has zero length"));
            }
            return Ok(vec![Primitive::Line { start, end }]);
        }
        Primitive::Circle { center, radius } => (center, radius, 0.0, TAU, true),
        Primitive::Arc { center, radius, start_angle, sweep_angle } => (center, radius, start_angle, sweep_angle, false),
    };
    let c = m.point(center, context)?;
    let [a, _, cc, _] = m.m;
    let (p, q) = (m.linear(radius, 0.0), m.linear(0.0, radius));
    if let Some(turn) = m.similarity() {
        let r = radius * a.hypot(cc);
        circle_range(c, r, context)?;
        if full {
            return Ok(vec![Primitive::Circle { center: c, radius: r }]);
        }
        // M = s R(phi) (or s R(phi) diag(1, -1)): the angle t goes to phi + t (or phi - t).
        let phi = cc.atan2(a);
        let start_angle = (phi + turn * start).rem_euclid(TAU);
        return Ok(vec![Primitive::Arc { center: c, radius: r, start_angle, sweep_angle: turn * sweep }]);
    }
    conic(state, c, (p, q), (start, sweep, full), context)
}

/// The curve `c + p cos t + q sin t` for t from `start` over `sweep`, in world units: a circle
/// or an arc when it is within the curve tolerance of one, else tangent arcs within it.
pub(super) fn conic(
    state: &mut State,
    c: Point2,
    (mut p, mut q): ((f64, f64), (f64, f64)),
    (start, sweep, full): (f64, f64, bool),
    context: &str,
) -> Result<Vec<Primitive>, DxfError> {
    // Run the curve with a rising parameter.
    let (mut start, mut sweep) = (start, sweep);
    if sweep < 0.0 {
        q = (-q.0, -q.1);
        (start, sweep) = (-start, -sweep);
    }
    // Shift the parameter by psi so that the axes are perpendicular, the major one first.
    let psi = 0.5 * (2.0 * (p.0 * q.0 + p.1 * q.1)).atan2(p.0 * p.0 + p.1 * p.1 - q.0 * q.0 - q.1 * q.1);
    let (s, k) = psi.sin_cos();
    (p, q) = ((p.0 * k + q.0 * s, p.1 * k + q.1 * s), (q.0 * k - p.0 * s, q.1 * k - p.1 * s));
    start -= psi;
    let (major, minor) = (p.0.hypot(p.1), q.0.hypot(q.1));
    circle_range(c, major, context)?;
    let turn = if p.0 * q.1 - p.1 * q.0 >= 0.0 { 1.0 } else { -1.0 };
    // The circle of the major radius is at most major - minor from the ellipse.
    if major - minor <= tolerance(state) {
        fit_settings(state, "INSERT");
        let axis = p.1.atan2(p.0);
        return Ok(vec![if full {
            Primitive::Circle { center: c, radius: major }
        } else {
            Primitive::Arc { center: c, radius: major, start_angle: (axis + turn * start).rem_euclid(TAU), sweep_angle: turn * sweep }
        }]);
    }
    let out = ellipse_arcs(state, "INSERT", (c, p, q), (start, sweep, full))?;
    if out.is_empty() {
        return Err(error(context, "placed curve has zero length"));
    }
    Ok(out)
}
