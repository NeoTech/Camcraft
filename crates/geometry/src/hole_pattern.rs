//! Hole patterns from numbers (#3195): a point list, a line, a grid and a
//! full or partial bolt-hole circle, turned into hole centres.
//!
//! Provenance: plain trigonometry for hole patterns as taught in machining
//! handbooks (Smid, *CNC Programming Handbook*, bolt-hole circle
//! coordinates x = cx + r cos a, y = cy + r sin a); our own refusal rules.

use crate::geo2d::Point2;

/// The most holes one pattern makes. A larger pattern is a typing error
/// (a count with an extra digit), not a part, and is refused by name.
pub const MAX_HOLES: usize = 10_000;

/// Two holes nearer than this (mm) are the same place.
const SAME_PLACE: f64 = 1e-6;

/// How the holes of a bolt-hole circle are spaced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircleSpacing {
    /// Evenly round the full circle: 360 / count degrees apart.
    Full,
    /// A partial circle: this many degrees from one hole to the next
    /// (negative runs clockwise).
    Pitch(f64),
    /// A partial circle: the first hole at the start angle and the last
    /// hole this many degrees further (negative runs clockwise).
    Span(f64),
}

/// One hole pattern, in the XY plane of the coordinates it is given in.
#[derive(Clone, Debug, PartialEq)]
pub enum HolePattern {
    /// Each hole stated: the points in their own order.
    Points(Vec<Point2>),
    /// `count` holes from `start`, `pitch` apart, along a line at
    /// `angle_deg` from +X.
    Line { start: Point2, pitch: f64, count: u32, angle_deg: f64 },
    /// `count_x` by `count_y` holes from `start`: rows along the grid's
    /// X (`pitch_x` apart), stepped `pitch_y` along its Y, the grid
    /// turned `angle_deg` about `start`. The holes run along X first.
    Grid { start: Point2, pitch_x: f64, pitch_y: f64, count_x: u32, count_y: u32, angle_deg: f64 },
    /// `count` holes on a circle of `diameter` round `center`, the first
    /// at `start_deg` from +X, counter-clockwise for positive angles.
    BoltCircle { center: Point2, diameter: f64, count: u32, start_deg: f64, spacing: CircleSpacing },
}

fn finite(v: f64, name: &str) -> Result<f64, String> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(format!("the {name} is not a finite number"))
    }
}

fn finite_point(p: Point2, name: &str) -> Result<Point2, String> {
    if p.x.is_finite() && p.y.is_finite() {
        Ok(p)
    } else {
        Err(format!("the {name} is not a finite point"))
    }
}

/// A pitch for `count` holes: any finite value for one hole, else positive.
fn pitch(v: f64, count: u32, name: &str) -> Result<f64, String> {
    let v = finite(v, name)?;
    if count > 1 && v <= 0.0 {
        return Err(format!("the {name} must be more than 0 mm for {count} holes (it is {v})"));
    }
    Ok(v)
}

fn count(n: u32, name: &str) -> Result<usize, String> {
    let n = usize::try_from(n).map_err(|_| format!("the {name} is too large"))?;
    if n == 0 {
        return Err(format!("the {name} must be 1 or more"));
    }
    if n > MAX_HOLES {
        return Err(format!("the {name} {n} makes more than {MAX_HOLES} holes"));
    }
    Ok(n)
}

/// Snap float noise (cos 90 degrees) to a clean nanometre lattice so a
/// stated hole at 0 posts as 0, not as -0.0000000000000018.
fn clean(v: f64) -> f64 {
    let r = (v * 1e9).round() / 1e9;
    if r == 0.0 { 0.0 } else { r }
}

fn at(x: f64, y: f64) -> Point2 {
    Point2::new(clean(x), clean(y))
}

impl HolePattern {
    /// The hole centres, in pattern order.
    ///
    /// # Errors
    /// A sentence that names the field: a non-finite number, a count of
    /// 0, a pitch or diameter of 0 or less, a partial circle that closes
    /// on itself, two holes on the same place, or more than
    /// [`MAX_HOLES`] holes.
    pub fn points(&self) -> Result<Vec<Point2>, String> {
        match self {
            Self::Points(pts) => points(pts),
            Self::Line { start, pitch: p, count: n, angle_deg } => {
                let start = finite_point(*start, "line start")?;
                let p = pitch(*p, *n, "line pitch")?;
                let n = count(*n, "line count")?;
                let a = finite(*angle_deg, "line angle")?.to_radians();
                let (dx, dy) = (a.cos(), a.sin());
                Ok((0..n).map(|i| {
                    let d = p * i as f64;
                    at(start.x + d * dx, start.y + d * dy)
                }).collect())
            }
            Self::Grid { start, pitch_x, pitch_y, count_x, count_y, angle_deg } => {
                let start = finite_point(*start, "grid start")?;
                let nx = count(*count_x, "grid count_x")?;
                let ny = count(*count_y, "grid count_y")?;
                let total = nx.checked_mul(ny).filter(|t| *t <= MAX_HOLES).ok_or_else(|| {
                    format!("the grid of {nx} by {ny} makes more than {MAX_HOLES} holes")
                })?;
                let px = pitch(*pitch_x, *count_x, "grid pitch_x")?;
                let py = pitch(*pitch_y, *count_y, "grid pitch_y")?;
                let a = finite(*angle_deg, "grid angle")?.to_radians();
                let (c, s) = (a.cos(), a.sin());
                let mut out = Vec::with_capacity(total);
                for j in 0..ny {
                    for i in 0..nx {
                        let (u, v) = (px * i as f64, py * j as f64);
                        out.push(at(start.x + u * c - v * s, start.y + u * s + v * c));
                    }
                }
                Ok(out)
            }
            Self::BoltCircle { center, diameter, count: n, start_deg, spacing } => {
                bolt_circle(*center, *diameter, *n, *start_deg, *spacing)
            }
        }
    }
}

fn points(pts: &[Point2]) -> Result<Vec<Point2>, String> {
    if pts.is_empty() {
        return Err("the point list has no points".into());
    }
    if pts.len() > MAX_HOLES {
        return Err(format!("the point list has more than {MAX_HOLES} holes"));
    }
    for (i, p) in pts.iter().enumerate() {
        finite_point(*p, &format!("point {}", i + 1))?;
    }
    for (i, p) in pts.iter().enumerate() {
        if let Some(j) = pts[i + 1..].iter().position(|q| (q.x - p.x).hypot(q.y - p.y) < SAME_PLACE) {
            return Err(format!("points {} and {} are the same place", i + 1, i + j + 2));
        }
    }
    Ok(pts.iter().map(|p| at(p.x, p.y)).collect())
}

fn bolt_circle(center: Point2, diameter: f64, n: u32, start_deg: f64, spacing: CircleSpacing) -> Result<Vec<Point2>, String> {
    let center = finite_point(center, "circle center")?;
    let d = finite(diameter, "circle diameter")?;
    if d <= 0.0 {
        return Err(format!("the circle diameter must be more than 0 mm (it is {d})"));
    }
    let count_n = count(n, "circle count")?;
    let start = finite(start_deg, "circle start angle")?;
    let step = match spacing {
        CircleSpacing::Full => 360.0 / count_n as f64,
        CircleSpacing::Pitch(p) => {
            let p = finite(p, "circle angular pitch")?;
            if count_n > 1 && p == 0.0 {
                return Err("the circle angular pitch must not be 0 degrees".into());
            }
            if p.abs() * (count_n as f64 - 1.0) >= 360.0 - 1e-9 {
                return Err(format!(
                    "{count_n} holes {} degrees apart go a full turn or more - use the full circle",
                    p.abs()
                ));
            }
            p
        }
        CircleSpacing::Span(s) => {
            let s = finite(s, "circle span")?;
            if count_n < 2 {
                return Err("a circle span needs a hole count of 2 or more".into());
            }
            if s == 0.0 || s.abs() >= 360.0 {
                return Err(format!(
                    "the circle span must be more than 0 and less than 360 degrees (it is {s}) - use the full circle for 360"
                ));
            }
            s / (count_n as f64 - 1.0)
        }
    };
    let r = d / 2.0;
    Ok((0..count_n)
        .map(|i| {
            let a = (start + step * i as f64).to_radians();
            at(center.x + r * a.cos(), center.y + r * a.sin())
        })
        .collect())
}
