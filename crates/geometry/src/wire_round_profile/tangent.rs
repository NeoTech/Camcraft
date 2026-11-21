//! Convex tangent DXF chains, using the turning-angle theorem and supporting tangents of a circle.
use super::{FIT, SAG, distance};
use crate::{
    dxf::Primitive,
    geo2d::{Loop, Point2},
    sketch_chains::ClosedChain,
};
use std::f64::consts::{PI, TAU};

/// One nominal directed line or CCW arc. Coordinates are in the same frame as the profile.
#[derive(Clone, Copy, Debug)]
pub struct TangentSegment {
    pub start: Point2,
    pub end: Point2,
    pub center: Option<Point2>,
    pub sweep: f64,
}

impl TangentSegment {
    fn direction(&self, at: Point2) -> [f64; 2] {
        let (x, y) = match self.center {
            Some(c) => (c.y - at.y, at.x - c.x),
            None => (self.end.x - self.start.x, self.end.y - self.start.y),
        };
        let len = x.hypot(y);
        [x / len, y / len]
    }

    /// Greatest projection of an exact segment in the unit normal direction.
    pub fn support(&self, normal: [f64; 2]) -> f64 {
        let project = |p: Point2| normal[0] * p.x + normal[1] * p.y;
        let ends = project(self.start).max(project(self.end));
        let Some(c) = self.center else { return ends };
        let start = (self.start.y - c.y).atan2(self.start.x - c.x);
        let angle = (normal[1].atan2(normal[0]) - start).rem_euclid(TAU);
        if angle <= self.sweep {
            ends.max(project(c) + distance(c, self.start))
        } else {
            ends
        }
    }
}

#[derive(Clone, Debug)]
pub struct TangentProfile {
    segments: Vec<TangentSegment>,
}

impl TangentProfile {
    pub fn segments(&self) -> &[TangentSegment] {
        &self.segments
    }

    pub(super) fn from_chain(chain: &ClosedChain) -> Result<Self, String> {
        let invalid = "The DXF punch needs a single convex loop of tangent lines and CCW arcs";
        let mut segments = Vec::with_capacity(chain.segments.len());
        let mut turn = 0.0;
        let mut lines = 0;
        for item in &chain.segments {
            let (start, end) = (item.start, item.end);
            if ![start.x, start.y, end.x, end.y]
                .iter()
                .all(|v| v.is_finite())
                || distance(start, end) < 0.001
            {
                return Err(invalid.into());
            }
            let (center, sweep) = match item.curve {
                Primitive::Line { .. } => {
                    lines += 1;
                    (None, 0.)
                }
                Primitive::Arc {
                    center,
                    radius,
                    sweep_angle,
                    ..
                } => {
                    if !center.x.is_finite()
                        || !center.y.is_finite()
                        || !(0.5..=20.).contains(&radius)
                        || !(0.0..=PI + 1e-12).contains(&sweep_angle)
                        || (distance(start, center) - radius).abs() > FIT
                        || (distance(end, center) - radius).abs() > FIT
                    {
                        return Err(invalid.into());
                    }
                    let alpha = (start.y - center.y).atan2(start.x - center.x);
                    let (s, c) = (alpha + sweep_angle).sin_cos();
                    if distance(
                        Point2::new(center.x + radius * c, center.y + radius * s),
                        end,
                    ) > FIT
                    {
                        return Err(invalid.into());
                    }
                    turn += sweep_angle;
                    (Some(center), sweep_angle)
                }
                _ => return Err(invalid.into()),
            };
            segments.push(TangentSegment {
                start,
                end,
                center,
                sweep,
            });
        }
        // For one simple C1 closed walk, positive arc turns totaling one
        // revolution make every tangent a supporting line of a convex body.
        if lines == 0 || (turn - TAU).abs() > 1e-9 {
            return Err(invalid.into());
        }
        for (current, next) in segments.iter().zip(segments.iter().cycle().skip(1)) {
            let a = current.direction(current.end);
            let b = next.direction(next.start);
            if distance(current.end, next.start) > FIT || (a[0] - b[0]).hypot(a[1] - b[1]) > 1e-9 {
                return Err(invalid.into());
            }
        }
        Ok(Self { segments })
    }

    pub(super) fn place(&mut self, transform: impl Fn(Point2) -> Point2) {
        for segment in &mut self.segments {
            segment.start = transform(segment.start);
            segment.end = transform(segment.end);
            segment.center = segment.center.map(&transform);
        }
    }

    pub fn support(&self, normal: [f64; 2]) -> f64 {
        self.segments
            .iter()
            .map(|s| s.support(normal))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub(super) fn preview(&self) -> Result<Loop, String> {
        // Adjacent tangent intersections enclose every arc. The small pad
        // covers joint arithmetic and the downstream 0.000001 mm clipping grid.
        const PAD: f64 = 0.000002;
        let mut vertices = Vec::new();
        for segment in &self.segments {
            let Some(c) = segment.center else { continue };
            let radius = distance(c, segment.start);
            let half_step = ((radius + PAD) / (radius + SAG - PAD)).acos();
            let count = (segment.sweep / (2.0 * half_step)).ceil().max(1.);
            if !count.is_finite() || count > (2048 - vertices.len()) as f64 {
                return Err("The DXF wire preview exceeds 2048 tangent vertices".into());
            }
            let step = segment.sweep / count;
            let start = (segment.start.y - c.y).atan2(segment.start.x - c.x);
            let outer = (radius + PAD) / (step * 0.5).cos();
            for i in 0..count as usize {
                let (s, cos) = (start + (i as f64 + 0.5) * step).sin_cos();
                vertices.push(Point2::new(c.x + outer * cos, c.y + outer * s));
            }
        }
        let ring = Loop::new(vertices).map_err(|e| e.to_string())?;
        for (&a, &b) in ring
            .points()
            .iter()
            .zip(ring.points().iter().cycle().skip(1))
        {
            let len = distance(a, b);
            let outward = [(b.y - a.y) / len, (a.x - b.x) / len];
            let allowance = a.x * outward[0] + a.y * outward[1] - self.support(outward);
            if !(PAD * 0.5..=SAG).contains(&allowance) {
                return Err(
                    "The tangent preview could not enclose the complete DXF profile".into(),
                );
            }
        }
        Ok(ring)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_oblique_arc_has_its_true_extremum_between_the_joins() {
        // This 143 degree arc starts at the bottom and ends at the 3:4 radius.
        let arc = TangentSegment {
            start: Point2::new(24., -5.),
            end: Point2::new(27., 4.),
            center: Some(Point2::new(24., 0.)),
            sweep: PI / 2. + 4.0_f64.atan2(3.),
        };
        assert_eq!(arc.support([1., 0.]), 29.);
        assert_eq!(arc.support([0., 1.]), 4.);
        assert_eq!(arc.support([0., -1.]), 5.);
    }
}
