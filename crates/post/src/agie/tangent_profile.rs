//! Native nominal line/arc words and exact grid tangency for a convex DXF Wire Contour punch.
use super::{AgieJob, AgieSide, Work, contour};
use camcraft_geometry::{
    geo2d::Point2,
    wire_round_profile::{TangentProfile, TangentSegment},
};

type Grid = [i64; 2];
const SCALE: f64 = 100_000.;

fn grid(p: Point2) -> Result<Grid, String> {
    let raw = [p.x, p.y];
    if raw.iter().any(|&v| {
        !v.is_finite() || v.abs() >= 10000. || ((v * SCALE).round() / SCALE - v).abs() > 1e-9
    }) {
        return Err(
            "The placed DXF joints and centers must fit the native five-decimal grid".into(),
        );
    }
    Ok(raw.map(|v| (v * SCALE).round() as i64))
}

fn mm(p: Grid) -> [f64; 2] {
    p.map(|n| n as f64 / SCALE)
}
fn delta(p: Grid, q: Grid) -> Grid {
    [q[0] - p[0], q[1] - p[1]]
}
fn dot(a: Grid, b: Grid) -> i128 {
    i128::from(a[0]) * i128::from(b[0]) + i128::from(a[1]) * i128::from(b[1])
}
fn cross(a: Grid, b: Grid) -> i128 {
    dot([-a[1], a[0]], b)
}

struct Segment {
    from: Grid,
    to: Grid,
    center: Option<Grid>,
}

impl Segment {
    fn new(s: &TangentSegment) -> Result<Self, String> {
        let from = grid(s.start)?;
        let to = grid(s.end)?;
        let center = s.center.map(grid).transpose()?;
        if from == to {
            return Err("The native DXF grid collapses a source segment".into());
        }
        if let Some(c) = center {
            let (a, b) = (delta(c, from), delta(c, to));
            // The source sweeps counterclockwise by at most a half turn.
            // Check the branch and the radius using the emitted integers.
            if dot(a, a) == 0 || dot(a, a) != dot(b, b) || cross(a, b) < 0 {
                return Err("The native arc endpoints must preserve the same radius and sweep on the ISO grid".into());
            }
        }
        Ok(Self { from, to, center })
    }

    fn tangent(&self, at: Grid) -> Grid {
        match self.center {
            Some(c) => {
                let r = delta(c, at);
                [-r[1], r[0]]
            }
            None => delta(self.from, self.to),
        }
    }
}

pub(super) fn prepare(
    job: &AgieJob,
    shape: &TangentProfile,
    geometry: String,
) -> Result<Work, String> {
    let rough = f64::from(contour::stock_offset_um(job)?) / 1000.;
    if job.side != AgieSide::Punch || job.stock_min[2] != -20.0 || !job.contour.is_empty() {
        return Err("A tangent DXF work needs one nominal through punch in 20 mm stock".into());
    }
    let entry = job
        .start
        .ok_or("The tangent DXF punch needs a 2 mm threading hole")?;
    let hole = grid(Point2::new(entry[0], entry[1]))?;
    let segments = shape
        .segments()
        .iter()
        .map(Segment::new)
        .collect::<Result<Vec<_>, _>>()?;
    for (a, b) in segments.iter().zip(segments.iter().cycle().skip(1)) {
        let (u, v) = (a.tangent(a.to), b.tangent(b.from));
        if a.to != b.from || cross(u, v) != 0 || dot(u, v) <= 0 {
            return Err("The native ISO grid must preserve every tangent DXF joint".into());
        }
    }
    let min = [-shape.support([-1., 0.]), -shape.support([0., -1.])];
    let max = [shape.support([1., 0.]), shape.support([0., 1.])];
    let stock_clearance = ((rough + 0.125) * 10.).ceil() / 10.;
    if (0..2).any(|i| {
        min[i] - stock_clearance < job.stock_min[i]
            || max[i] + stock_clearance > job.stock_max[i]
            || entry[i] - 1. < job.stock_min[i]
            || entry[i] + 1. > job.stock_max[i]
    }) {
        return Err("The whole DXF contour, rough wire and complete threading hole must clear the stock edges".into());
    }
    for (i, s) in segments.iter().enumerate() {
        if s.center.is_some() {
            continue;
        }
        let axis = if s.from[1] == s.to[1] {
            0
        } else if s.from[0] == s.to[0] {
            1
        } else {
            continue;
        };
        let mut seam = hole;
        seam[1 - axis] = s.from[1 - axis];
        let direction = delta(s.from, s.to);
        let sign = direction[axis].signum();
        if (seam[axis] - s.from[axis]) * sign < 300_000
            || (s.to[axis] - seam[axis]) * sign < 300_000
        {
            continue;
        }
        let normal = [direction[1].signum() as f64, -direction[0].signum() as f64];
        let xy = mm(seam);
        let face = xy[0] * normal[0] + xy[1] * normal[1];
        if shape.support(normal) > face + 1e-9
            || entry[0] * normal[0] + entry[1] * normal[1] - face < 3.
        {
            continue;
        }
        let mut points = vec![xy, mm(s.to)];
        let mut arc_centers = vec![None, None];
        for next in segments[i + 1..].iter().chain(&segments[..i]) {
            points.push(mm(next.to));
            arc_centers.push(next.center.map(mm));
        }
        points.push(xy);
        arc_centers.push(None);
        let margin = 3.0 + 0.125 + rough;
        return Ok(Work {
            side: job.side,
            geometry,
            points,
            arc_centers,
            entry,
            exit: None,
            min: std::array::from_fn(|i| min[i].min(entry[i]) - margin),
            max: std::array::from_fn(|i| max[i].max(entry[i]) + margin),
        });
    }
    Err("Choose an external perpendicular threading hole at least 3 mm from a cardinal straight and both its joints".into())
}
