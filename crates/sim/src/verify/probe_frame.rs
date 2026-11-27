//! The tool frame of a tip and axis (split from `probe.rs` under the
//! 400-line law): local coordinates of world points; plain linear
//! algebra. No vendor source.

use crate::geometry::{Point3, Triangle};

/// The tool frame of a tip and axis: local coordinates of world points
/// (tip at the origin, tool axis +z).
#[derive(Clone, Copy)]
pub(in crate::verify) struct Frame {
    o: Point3,
    u: [f64; 3],
    v: [f64; 3],
    a: [f64; 3],
}

impl Frame {
    pub(in crate::verify) fn new(o: Point3, axis: Option<[f64; 3]>) -> Self {
        let Some(a) = axis else {
            return Self { o, u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0], a: [0.0, 0.0, 1.0] };
        };
        let e = if a[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
        let u = norm(cross(a, e));
        let v = cross(a, u);
        Self { o, u, v, a }
    }

    pub(in crate::verify) fn local(&self, p: Point3) -> [f64; 3] {
        let d = [p.x - self.o.x, p.y - self.o.y, p.z - self.o.z];
        [dot(d, self.u), dot(d, self.v), dot(d, self.a)]
    }

    pub(in crate::verify) fn tri(&self, t: &Triangle) -> [[f64; 3]; 3] {
        [self.local(t.a), self.local(t.b), self.local(t.c)]
    }

    pub(in crate::verify) fn seg(&self, a: Point3, b: Point3) -> [[f64; 3]; 2] {
        [self.local(a), self.local(b)]
    }

    /// The world point `h` up the axis from the tip.
    pub(in crate::verify) fn up(&self, h: f64) -> Point3 {
        Point3::new(self.o.x + self.a[0] * h, self.o.y + self.a[1] * h, self.o.z + self.a[2] * h)
    }

    /// The world displacement of local displacement `d`.
    pub(in crate::verify) fn end(&self, d: [f64; 3]) -> Point3 {
        let c = |k: usize| self.u[k] * d[0] + self.v[k] * d[1] + self.a[k] * d[2];
        Point3::new(c(0), c(1), c(2))
    }

    pub(in crate::verify) fn is_upright(&self) -> bool {
        self.a == [0.0, 0.0, 1.0]
    }

    /// The local XY of the line through `at` along `dir` when it runs
    /// along this frame's axis (within 1e-9 of parallel, either sense);
    /// `None` for any other line, or one that is not finite (#1205).
    pub(in crate::verify) fn axis_xy(&self, at: Point3, dir: [f64; 3]) -> Option<[f64; 2]> {
        let len = dot(dir, dir).sqrt();
        if !(len.is_finite() && len > 0.0 && at.is_finite()) {
            return None;
        }
        let c = dot(dir, self.a) / len;
        let p = self.local(at);
        (c.abs() >= 1.0 - 1e-9 && p[0].is_finite() && p[1].is_finite()).then_some([p[0], p[1]])
    }
}

fn cross(p: [f64; 3], q: [f64; 3]) -> [f64; 3] {
    [p[1] * q[2] - p[2] * q[1], p[2] * q[0] - p[0] * q[2], p[0] * q[1] - p[1] * q[0]]
}

fn dot(p: [f64; 3], q: [f64; 3]) -> f64 {
    p[0] * q[0] + p[1] * q[1] + p[2] * q[2]
}

fn norm(p: [f64; 3]) -> [f64; 3] {
    let l = dot(p, p).sqrt();
    if l > 0.0 { [p[0] / l, p[1] / l, p[2] / l] } else { [1.0, 0.0, 0.0] }
}
