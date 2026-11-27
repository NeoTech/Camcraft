//! The stock grid that the law checks own (#4027): a raster of the stock
//! footprint, written apart from any planner's stock reader (record "The
//! laws as checks": an independent stock grid). It walks a plan step by
//! step, reads each entry and each cut on the stock as it is at that step,
//! and clears the swept disc of the tool.
//! Provenance: our own design record; the radial engagement ae/D from the
//! contact angle of the cutting edge (Stori & Wright 2000).

use camcraft_pathkernel::entry::{EntryKind, Proof};
use camcraft_pathkernel::plan::{Plan, Step};
use camcraft_pathkernel::{Point2, Region};

use super::kit::{dist, poly};

/// The grid cell is this part of the tool radius.
pub const CELL: f64 = 0.1;
/// The read error of an engagement on the grid: one cell at each side of
/// the width, over the diameter.
pub const GRID_TOL: f64 = CELL;

pub struct Grid {
    x0: f64,
    y0: f64,
    h: f64,
    nx: usize,
    ny: usize,
    cells: Vec<bool>,
}

impl Grid {
    /// The raster of `stock` with the cell side `h`, or `None` for an
    /// empty stock or a bad cell side.
    pub fn new(stock: &Region, h: f64) -> Option<Self> {
        let (lo, hi) = stock.bounds()?;
        if !(h > 0.0 && h.is_finite()) {
            return None;
        }
        let nx = ((hi.x - lo.x) / h).ceil() as usize + 1;
        let ny = ((hi.y - lo.y) / h).ceil() as usize + 1;
        let mut g = Self { x0: lo.x, y0: lo.y, h, nx, ny, cells: Vec::with_capacity(nx * ny) };
        for j in 0..ny {
            for i in 0..nx {
                let c = g.centre(i, j);
                g.cells.push(stock.contains(c));
            }
        }
        Some(g)
    }

    fn centre(&self, i: usize, j: usize) -> Point2 {
        Point2::new(self.x0 + (i as f64 + 0.5) * self.h, self.y0 + (j as f64 + 0.5) * self.h)
    }

    /// The index and the centre of each cell whose centre is within `r` of `c`.
    fn disc(&self, c: Point2, r: f64) -> Vec<(usize, Point2)> {
        let span = |v: f64, v0: f64, n: usize| {
            let lo = ((v - r - v0) / self.h).floor().max(0.0) as usize;
            let hi = (((v + r - v0) / self.h).ceil().max(0.0) as usize).min(n);
            lo..hi
        };
        let mut out = Vec::new();
        for j in span(c.y, self.y0, self.ny) {
            for i in span(c.x, self.x0, self.nx) {
                let q = self.centre(i, j);
                if dist(q, c) <= r {
                    out.push((j * self.nx + i, q));
                }
            }
        }
        out
    }

    pub fn clear_disc(&mut self, c: Point2, r: f64) {
        for (k, _) in self.disc(c, r) {
            self.cells[k] = false;
        }
    }

    /// The share of the disc that holds material (cells off the grid are air).
    pub fn fill(&self, c: Point2, r: f64) -> f64 {
        let all = std::f64::consts::PI * r * r / (self.h * self.h);
        let hit = self.disc(c, r).iter().filter(|(k, _)| self.cells[*k]).count();
        if all > 0.0 { (hit as f64 / all).min(1.0) } else { 0.0 }
    }

    /// True when the cell at `p` holds material (off the grid: air).
    fn material(&self, p: Point2) -> bool {
        let (fx, fy) = ((p.x - self.x0) / self.h, (p.y - self.y0) / self.h);
        if !(fx >= 0.0 && fy >= 0.0) {
            return false;
        }
        let (i, j) = (fx as usize, fy as usize);
        i < self.nx && j < self.ny && self.cells[j * self.nx + i]
    }

    /// ae/D at `c` for the motion direction `d` (a unit vector), from the
    /// contact angle: the angle of the front half of the cutting edge that
    /// touches material, read each degree half a cell inside the edge, as
    /// `(1 - cos angle) / 2` (1 for a full-width slot).
    pub fn engagement(&self, c: Point2, d: Point2, r: f64) -> f64 {
        let n = Point2::new(-d.y, d.x);
        let edge = r - self.h / 2.0;
        let touch = (-90..=90)
            .filter(|deg| {
                let t = f64::from(*deg).to_radians();
                let (a, b) = (t.cos() * edge, t.sin() * edge);
                self.material(Point2::new(c.x + a * d.x + b * n.x, c.y + a * d.y + b * n.y))
            })
            .count();
        let angle = (touch as f64).to_radians().min(std::f64::consts::PI);
        (1.0 - angle.cos()) / 2.0
    }
}

/// One read of an entry descent point: the material share under the tool,
/// the proof, and how far the descent of its entry moves in the plane
/// (#4406: a descent into material that does not move is a plunge).
pub struct EntryRead {
    pub index: usize,
    pub at: Point2,
    pub fill: f64,
    pub into_material: bool,
    pub kind: EntryKind,
    /// The largest distance of a descent point from the end of the descent.
    pub travel: f64,
}

/// One read of a cut point: its engagement on the stock of that moment.
pub struct CutRead {
    pub index: usize,
    pub at: Point2,
    pub load: f64,
}

#[derive(Default)]
pub struct Walk {
    pub entries: Vec<EntryRead>,
    pub cuts: Vec<CutRead>,
}

/// The points of a polyline at most `step` apart, each with its unit direction.
pub fn samples(line: &[Point2], step: f64) -> Vec<(Point2, Point2)> {
    let mut out = Vec::new();
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = dist(a, b);
        if len <= 0.0 {
            continue;
        }
        let d = Point2::new((b.x - a.x) / len, (b.y - a.y) / len);
        let n = (len / step).ceil().max(1.0) as usize;
        for k in 0..n {
            let t = k as f64 / n as f64;
            out.push((Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)), d));
        }
    }
    if let (Some(&(_, d)), Some(&p)) = (out.last(), line.last()) {
        out.push((p, d));
    }
    out
}

/// Walks `plan` on a fresh grid of `stock` for a tool of radius `r`. An
/// entry is read before its descent clears the disc; a cut point is read
/// before the tool clears its disc there. `None` for an empty stock.
pub fn walk(plan: &Plan, stock: &Region, r: f64) -> Option<Walk> {
    let mut g = Grid::new(stock, r * CELL)?;
    let (mut w, mut ne, mut nc) = (Walk::default(), 0, 0);
    for s in plan.steps() {
        match s {
            Step::Entry(e) => {
                let into_material = matches!(e.proof(), Proof::IntoMaterial { .. });
                let end = e.descent().last().map(|p| p.at());
                let travel = end.map_or(0.0, |q| e.descent().iter().map(|p| dist(p.at(), q)).fold(0.0, f64::max));
                for p in e.descent() {
                    let read = EntryRead { index: ne, at: p.at(), fill: g.fill(p.at(), r), into_material, kind: e.kind(), travel };
                    w.entries.push(read);
                }
                for p in e.descent() {
                    g.clear_disc(p.at(), r);
                }
                ne += 1;
            }
            Step::Cut(c) => {
                for (p, d) in samples(&poly(c.curve()), r / 4.0) {
                    w.cuts.push(CutRead { index: nc, at: p, load: g.engagement(p, d, r) });
                    g.clear_disc(p, r);
                }
                nc += 1;
            }
            _ => {}
        }
    }
    Some(w)
}
