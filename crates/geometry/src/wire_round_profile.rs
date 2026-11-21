//! Placed DXF round profiles and circumscribed preview polygons; classical circle and tangent geometry.
use crate::dxf::Primitive;
use crate::geo2d::{Loop, Point2};
use crate::geometry::{Point3, Triangle};
use crate::sketch_data::SketchData;
use std::f64::consts::PI;

const FIT: f64 = 1e-8;
const SAG: f64 = 0.000025;

#[derive(Clone, Copy, Debug)]
pub enum Spine {
    Circle(Point2),
    Stadium([Point2; 2]),
}

/// A circle or a tangent stadium from authoritative DXF primitives.
#[derive(Clone, Copy, Debug)]
pub struct RoundProfile {
    pub spine: Spine,
    pub radius: f64,
}

fn distance(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

impl RoundProfile {
    /// Read a source circle or prove that four drawing primitives form a capsule.
    /// A joint within the drawing tolerance alone does not establish tangency.
    pub fn from_sketch(data: &SketchData) -> Result<Self, String> {
        let failure = "Wire DXF requires one CIRCLE or one closed racetrack of two tangent lines and two semicircles";
        let chains = data.chains();
        let [chain] = chains.closed.as_slice() else {
            return Err(failure.into());
        };
        if data.skipped_total() != 0
            || !chains.open.is_empty()
            || !chains.rejected.is_empty()
            || !chain.nesting.as_ref().is_ok_and(|n| n.depth == 0)
            || chain.max_gap > FIT
        {
            return Err(failure.into());
        }
        if let [entity] = data.primitives()
            && let Primitive::Circle { center, radius } = entity.primitive
        {
            if !(0.5..=20.).contains(&radius) {
                return Err("Wire DXF circle radius must be from 0.5 to 20 mm".into());
            }
            return Ok(Self {
                spine: Spine::Circle(center),
                radius,
            });
        }
        if data.primitives().len() != 4 || chain.segments.len() != 4 {
            return Err(failure.into());
        }
        let mut arcs = Vec::new();
        let mut lines = Vec::new();
        for segment in &chain.segments {
            match segment.curve {
                Primitive::Line { start, end } => lines.push([start, end]),
                Primitive::Arc {
                    center,
                    radius,
                    sweep_angle,
                    ..
                } if (sweep_angle - PI).abs() * radius <= FIT => {
                    arcs.push((center, radius, segment.start, segment.end));
                }
                _ => return Err(failure.into()),
            }
        }
        let [(a, r, p, q), (b, s, u, v)] = arcs.as_slice() else {
            return Err(failure.into());
        };
        let shape = Self {
            spine: Spine::Stadium([*a, *b]),
            radius: *r,
        };
        if !(0.5..=20.).contains(r) || (r - s).abs() > FIT || distance(*a, *b) < 6.0 {
            return Err("Wire DXF requires equal cap radii from 0.5 to 20 mm and straight sides at least 6 mm long".into());
        }
        let [ar, br, bl, al] = shape.tangencies([*a, *b]);
        let close = |a, b| distance(a, b) <= FIT;
        if !close(*p, al)
            || !close(*q, ar)
            || !close(*u, br)
            || !close(*v, bl)
            || !lines.iter().any(|&[l, h]| close(l, ar) && close(h, br))
            || !lines.iter().any(|&[l, h]| close(l, bl) && close(h, al))
        {
            return Err(failure.into());
        }
        Ok(shape)
    }

    pub fn centers(&self) -> [Point2; 2] {
        match self.spine {
            Spine::Circle(c) => [c, c],
            Spine::Stadium(pair) => pair,
        }
    }

    pub fn place_centers(&mut self, transform: impl Fn(Point2) -> Point2) {
        self.spine = match self.spine {
            Spine::Circle(c) => Spine::Circle(transform(c)),
            Spine::Stadium(pair) => Spine::Stadium(pair.map(transform)),
        };
    }

    /// Tangencies in CCW order: first straight side, second cap, return side, first cap.
    pub fn tangencies(&self, [a, b]: [Point2; 2]) -> [Point2; 4] {
        let scale = self.radius / distance(a, b);
        let x = (b.y - a.y) * scale;
        let y = (a.x - b.x) * scale;
        [
            Point2::new(a.x + x, a.y + y),
            Point2::new(b.x + x, b.y + y),
            Point2::new(b.x - x, b.y - y),
            Point2::new(a.x - x, a.y - y),
        ]
    }

    /// Construct a circumscribed preview. Intersections of consecutive tangents
    /// enclose the entire true arc with radial excess at most 0.000025 mm.
    pub fn preview(&self) -> Result<Loop, String> {
        let n = (PI / (2. * (self.radius / (self.radius + SAG)).acos())).ceil();
        if !n.is_finite() || !(2.0..=1024.0).contains(&n) {
            return Err("The DXF arc exceeds the verified wire preview resolution".into());
        }
        let step = PI / n;
        let outer = self.radius / (step / 2.).cos();
        let [a, b] = self.centers();
        let [br, al] = match self.spine {
            Spine::Circle(c) => [
                Point2::new(c.x + self.radius, c.y),
                Point2::new(c.x - self.radius, c.y),
            ],
            Spine::Stadium(pair) => {
                let [_, br, _, al] = self.tangencies(pair);
                [br, al]
            }
        };
        let mut points = Vec::with_capacity(2 * n as usize);
        for (center, start) in [(b, br), (a, al)] {
            let angle = (start.y - center.y).atan2(start.x - center.x);
            for i in 0..n as usize {
                let (sin, cos) = (angle + (i as f64 + 0.5) * step).sin_cos();
                points.push(Point2::new(center.x + outer * cos, center.y + outer * sin));
            }
        }
        Loop::new(points).map_err(|e| e.to_string())
    }

    /// A convex fan at both ends and two outward triangles per wall.
    pub fn prism(&self, ring: &Loop, bottom: f64, top: f64) -> Vec<Triangle> {
        let [a, b] = self.centers();
        let mid = Point2::new((a.x + b.x) / 2., (a.y + b.y) / 2.);
        let at = |p: Point2, z| Point3::new(p.x, p.y, z);
        let mut facets = Vec::with_capacity(4 * ring.points().len());
        for (&p, &q) in ring
            .points()
            .iter()
            .zip(ring.points().iter().cycle().skip(1))
        {
            let (l, r, h, k) = (at(p, bottom), at(q, bottom), at(p, top), at(q, top));
            facets.extend([
                Triangle::new(at(mid, bottom), r, l),
                Triangle::new(at(mid, top), h, k),
                Triangle::new(l, r, k),
                Triangle::new(l, k, h),
            ]);
        }
        facets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_circle_preview_stays_outside_the_true_wire_boundary() {
        let center = Point2::new(3.5, -2.0);
        let shape = RoundProfile {
            spine: Spine::Circle(center),
            radius: 10.0,
        };
        let circle = shape.preview().unwrap();
        for (&p, &q) in circle
            .points()
            .iter()
            .zip(circle.points().iter().cycle().skip(1))
        {
            let signed =
                ((q.x - p.x) * (center.y - p.y) - (q.y - p.y) * (center.x - p.x)) / distance(p, q);
            assert!((signed - shape.radius).abs() < 1e-9);
            assert!(distance(center, p) <= shape.radius + SAG + 1e-9);
        }
    }

    #[test]
    fn the_preview_contains_each_true_cap_with_bounded_excess() {
        let shape = RoundProfile {
            spine: Spine::Stadium([Point2::new(1., -2.), Point2::new(13., 14.)]),
            radius: 10.,
        };
        let ring = shape.preview().unwrap();
        let [a, b] = shape.centers();
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        for (&p, &q) in ring
            .points()
            .iter()
            .zip(ring.points().iter().cycle().skip(1))
        {
            // A capsule is the convex hull of its two circles. A CCW edge
            // contains both discs if their centers clear its inner half-plane.
            let length = distance(p, q);
            for center in [a, b] {
                let signed =
                    ((q.x - p.x) * (center.y - p.y) - (q.y - p.y) * (center.x - p.x)) / length;
                assert!(signed >= shape.radius - 1e-9, "{signed}");
            }
            let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
            let spine = Point2::new(a.x + t * dx, a.y + t * dy);
            assert!(distance(p, spine) <= shape.radius + SAG + 1e-9);
        }
    }
}
