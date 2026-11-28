//! Straight and circular walls of a native die opening on the ISO grid: their tangents,
//! bounds, and the exact distance between two walls. Provenance: our own implementation of
//! classical point, line and circle geometry (closest points lie at an end or on a common normal).

use std::f64::consts::TAU;

use super::super::contour::{Point, intersects};

type V = [f64; 2];

fn f(p: Point) -> V {
    [p[0] as f64, p[1] as f64]
}

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn cross(a: V, b: V) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn dist(a: V, b: V) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// One wall of the opening in grid units (0.00001 mm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub from: Point,
    pub to: Point,
    /// The arc centre and its turn: `true` is counter-clockwise (G03). `None` is a straight wall.
    pub arc: Option<(Point, bool)>,
}

impl Span {
    pub fn line(from: Point, to: Point) -> Self {
        Self { from, to, arc: None }
    }

    /// The same wall, travelled the other way.
    pub fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            arc: self.arc.map(|(c, ccw)| (c, !ccw)),
        }
    }

    /// The direction of travel at the wall point `p` (exact, not normalised).
    pub fn tangent_at(&self, p: Point) -> Point {
        match self.arc {
            None => [self.to[0] - self.from[0], self.to[1] - self.from[1]],
            Some((c, ccw)) => {
                let r = [p[0] - c[0], p[1] - c[1]];
                if ccw { [-r[1], r[0]] } else { [r[1], -r[0]] }
            }
        }
    }

    /// The squared radius at `p` in grid units, exact.
    pub fn radius2_at(c: Point, p: Point) -> i128 {
        i128::from(p[0] - c[0]).pow(2) + i128::from(p[1] - c[1]).pow(2)
    }

    pub fn radius(&self) -> f64 {
        self.arc.map_or(0.0, |(c, _)| dist(f(c), f(self.from)))
    }

    /// The turn of `q` about the centre from the start, in the travel direction, in [0, 2π).
    fn turn(&self, q: V) -> f64 {
        let Some((c, ccw)) = self.arc else { return 0.0 };
        let (a, b) = (sub(f(self.from), f(c)), sub(q, f(c)));
        let t = cross(a, b).atan2(dot(a, b));
        let t = if ccw { t } else { -t };
        t.rem_euclid(TAU)
    }

    /// The swept angle of an arc, in (0, 2π).
    pub fn sweep(&self) -> f64 {
        let t = self.turn(f(self.to));
        if t > 0.0 { t } else { TAU }
    }

    pub fn length(&self) -> f64 {
        match self.arc {
            None => dist(f(self.from), f(self.to)),
            Some(_) => self.radius() * self.sweep(),
        }
    }

    fn in_sweep(&self, q: V) -> bool {
        self.turn(q) <= self.sweep() + 1e-12
    }

    /// The distance from `p` to this wall.
    pub fn point_distance(&self, p: V) -> f64 {
        let (a, b) = (f(self.from), f(self.to));
        match self.arc {
            None => {
                let d = sub(b, a);
                let t = (dot(sub(p, a), d) / dot(d, d)).clamp(0.0, 1.0);
                dist(p, [a[0] + t * d[0], a[1] + t * d[1]])
            }
            Some((c, _)) => {
                let c = f(c);
                if p != c && self.in_sweep(p) {
                    (dist(p, c) - self.radius()).abs()
                } else {
                    dist(p, a).min(dist(p, b))
                }
            }
        }
    }

    /// The points where the circle of this arc meets the line through `a` and `b`, as line parameters.
    fn line_hits(&self, a: V, b: V) -> Vec<f64> {
        let Some((c, _)) = self.arc else { return Vec::new() };
        let (d, m) = (sub(b, a), sub(a, f(c)));
        let (qa, qb, qc) = (dot(d, d), 2.0 * dot(d, m), dot(m, m) - self.radius().powi(2));
        let disc = qb * qb - 4.0 * qa * qc;
        if disc < 0.0 {
            return Vec::new();
        }
        let root = disc.sqrt();
        vec![(-qb - root) / (2.0 * qa), (-qb + root) / (2.0 * qa)]
    }

    /// Whether the two walls cross or touch.
    fn meets(&self, other: &Span) -> bool {
        match (self.arc, other.arc) {
            (None, None) => intersects(self.from, self.to, other.from, other.to),
            (Some(_), None) => other.meets(self),
            (None, Some(_)) => {
                let (a, b) = (f(self.from), f(self.to));
                other.line_hits(a, b).into_iter().any(|t| {
                    (0.0..=1.0).contains(&t)
                        && other.in_sweep([a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])])
                })
            }
            (Some((c1, _)), Some((c2, _))) => {
                let (c1, c2) = (f(c1), f(c2));
                let (r1, r2, d) = (self.radius(), other.radius(), dist(c1, c2));
                if d == 0.0 {
                    return r1 == r2
                        && (self.in_sweep(f(other.from)) || other.in_sweep(f(self.from)));
                }
                if d > r1 + r2 || d < (r1 - r2).abs() {
                    return false;
                }
                let along = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
                let h = (r1 * r1 - along * along).max(0.0).sqrt();
                let u = [(c2[0] - c1[0]) / d, (c2[1] - c1[1]) / d];
                let m = [c1[0] + along * u[0], c1[1] + along * u[1]];
                [1.0, -1.0].into_iter().any(|s| {
                    let q = [m[0] - s * h * u[1], m[1] + s * h * u[0]];
                    self.in_sweep(q) && other.in_sweep(q)
                })
            }
        }
    }

    /// The circle points of this arc on a common normal with `other`: the
    /// only interior closest points that an end check does not see.
    fn normal_points(&self, other: &Span) -> Vec<V> {
        let Some((c, _)) = self.arc else { return Vec::new() };
        let (c, r) = (f(c), self.radius());
        let u = match other.arc {
            None => {
                let d = sub(f(other.to), f(other.from));
                let n = dot(d, d).sqrt();
                [-d[1] / n, d[0] / n]
            }
            Some((o, _)) => {
                let d = sub(f(o), c);
                let n = dot(d, d).sqrt();
                if n == 0.0 {
                    return Vec::new();
                }
                [d[0] / n, d[1] / n]
            }
        };
        [1.0, -1.0]
            .into_iter()
            .map(|s| [c[0] + s * r * u[0], c[1] + s * r * u[1]])
            .filter(|q| self.in_sweep(*q))
            .collect()
    }

    /// The least distance between the two walls, in grid units. Zero if they meet.
    pub fn distance(&self, other: &Span) -> f64 {
        if self.meets(other) {
            return 0.0;
        }
        let mut best = [other.from, other.to]
            .map(|p| self.point_distance(f(p)))
            .into_iter()
            .chain([self.from, self.to].map(|p| other.point_distance(f(p))))
            .fold(f64::INFINITY, f64::min);
        for q in self.normal_points(other) {
            best = best.min(other.point_distance(q));
        }
        for q in other.normal_points(self) {
            best = best.min(self.point_distance(q));
        }
        best
    }

    /// The exact axis extrema of the wall: its ends, and each quadrant point that an arc passes.
    pub fn extrema(&self) -> Vec<V> {
        let mut out = vec![f(self.from), f(self.to)];
        if let Some((c, _)) = self.arc {
            let (c, r) = (f(c), self.radius());
            for (dx, dy) in [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)] {
                let q = [c[0] + r * dx, c[1] + r * dy];
                if self.in_sweep(q) {
                    out.push(q);
                }
            }
        }
        out
    }

    /// Points along the wall no farther than `sag` grid units from it.
    pub fn chords(&self, sag: f64) -> Vec<V> {
        let Some((c, _)) = self.arc else { return vec![f(self.from)] };
        let (c, r, sweep) = (f(c), self.radius(), self.sweep());
        let step = 2.0 * (1.0 - (sag / r).min(1.0)).acos();
        let n = (sweep / step).ceil().clamp(1.0, 100_000.0) as usize;
        let a = sub(f(self.from), c);
        let sign = if self.arc.is_some_and(|(_, ccw)| ccw) { 1.0 } else { -1.0 };
        (0..n)
            .map(|k| {
                let (s, co) = (sign * sweep * k as f64 / n as f64).sin_cos();
                [c[0] + co * a[0] - s * a[1], c[1] + s * a[0] + co * a[1]]
            })
            .collect()
    }
}
