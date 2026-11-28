//! The engagement law of the load stage (K3, #4026): the in-process stock
//! as a raster of the stage's stock reader, and the radial engagement ae/D
//! of each cut, read in the cut order on the stock that the earlier cuts
//! leave. Every mode of the stage reads with this one law (record "Stage 3",
//! invariant 3).
//! Provenance: the contact angle phi of the cutting edge and the radial
//! engagement ae/D = (1 - cos phi) / 2 (Altintas, "Manufacturing
//! Automation", 2000, ch. 2; Stori & Wright 2000, the engagement on the
//! in-process stock); the stock as a raster of material cells, read at
//! the rim of the swept disc, as in Choy & Chan (Computer-Aided Design 35,
//! 2003) and our own design record (`RECOMMENDED.md`, `## #4024`).

use crate::guide::Curve;
use crate::load::StockRead;
use crate::verdict::{Refusal, Stage};
use crate::{Point2, Region};

/// The raster cells across one tool radius.
const CELLS_PER_R: f64 = 20.0;
/// The most raster cells of one read: a coarser cell above it.
const MAX_CELLS: f64 = 4.0e6;
/// The reads along a cut across one tool radius.
const SAMPLES_PER_R: f64 = 8.0;

/// Refusal reason: the stock raster cannot be made for the region.
pub const NO_RASTER: &str = "the load stage cannot make a stock raster: the cutter-centre area is empty or out of range";

/// The in-process stock: material cells of side `h` from `(x0, y0)`.
#[derive(Clone, Debug)]
pub(crate) struct Stock {
    x0: f64,
    y0: f64,
    h: f64,
    nx: usize,
    ny: usize,
    cells: Vec<bool>,
}

/// One read of a cut: the curve's index, the place and the engagement.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Read {
    pub curve: usize,
    pub at: Point2,
    pub load: f64,
}

/// A count of cells along one side, or `None` when it is not finite or too large.
fn count(span: f64, h: f64) -> Option<usize> {
    let n = (span / h).ceil() + 1.0;
    (n.is_finite() && (1.0..=MAX_CELLS).contains(&n)).then_some(n as usize)
}

impl Stock {
    /// The raster of the stock round `center` (the cutter-centre area) for
    /// a tool of radius `r`: each cell is material where `reader` says so.
    /// A cell that the reader does not know is material: the read can then
    /// only be too high, never too low.
    pub fn new(center: &Region, reader: &dyn StockRead, r: f64) -> Result<Self, Refusal> {
        let bad = || Refusal::new(Stage::Load, NO_RASTER);
        let (lo, hi) = center.bounds().ok_or_else(bad)?;
        let m = 2.0 * r;
        let (w, t) = ((hi.x - lo.x) + 2.0 * m, (hi.y - lo.y) + 2.0 * m);
        let h = (r / CELLS_PER_R).max((w * t / MAX_CELLS).sqrt());
        if !(h.is_finite() && h > 0.0) {
            return Err(bad());
        }
        let (nx, ny) = (count(w, h).ok_or_else(bad)?, count(t, h).ok_or_else(bad)?);
        let all = nx.checked_mul(ny).ok_or_else(bad)?;
        let mut s = Self { x0: lo.x - m, y0: lo.y - m, h, nx, ny, cells: Vec::with_capacity(all) };
        for j in 0..ny {
            for i in 0..nx {
                let c = s.centre(i, j);
                s.cells.push(reader.material_at(c).unwrap_or(true));
            }
        }
        Ok(s)
    }

    fn centre(&self, i: usize, j: usize) -> Point2 {
        Point2::new(self.x0 + (i as f64 + 0.5) * self.h, self.y0 + (j as f64 + 0.5) * self.h)
    }

    /// The cell index at `p`, or `None` off the raster.
    fn cell(&self, p: Point2) -> Option<usize> {
        let (fx, fy) = ((p.x - self.x0) / self.h, (p.y - self.y0) / self.h);
        if !(fx >= 0.0 && fy >= 0.0 && fx.is_finite() && fy.is_finite()) {
            return None;
        }
        let (i, j) = (fx as usize, fy as usize);
        (i < self.nx && j < self.ny).then(|| j.saturating_mul(self.nx).saturating_add(i))
    }

    /// Clears each cell whose centre is within `r` of `c` (the tool disc).
    fn clear(&mut self, c: Point2, r: f64) {
        let span = |v: f64, v0: f64, n: usize| {
            let lo = ((v - r - v0) / self.h).floor().max(0.0) as usize;
            let hi = (((v + r - v0) / self.h).ceil().max(0.0) as usize).min(n);
            lo..hi
        };
        for j in span(c.y, self.y0, self.ny) {
            for i in span(c.x, self.x0, self.nx) {
                let q = self.centre(i, j);
                if (q.x - c.x).hypot(q.y - c.y) <= r
                    && let Some(k) = self.cells.get_mut(j.saturating_mul(self.nx).saturating_add(i))
                {
                    *k = false;
                }
            }
        }
    }

    /// ae/D at `c` for the unit motion direction `d`, after the tool disc
    /// at `c` is cleared: the contact angle is the part of the front half
    /// of the cutting edge (one read for each degree, three quarters of a
    /// cell outside the edge) that has material beside it. A read inside
    /// the edge would miss the side of the arc that the disc of the read
    /// before has just cleared, and read too low.
    fn engagement(&self, c: Point2, d: Point2, r: f64) -> f64 {
        let n = Point2::new(-d.y, d.x);
        let edge = r + 0.75 * self.h;
        let touch = (-90i32..=90)
            .filter(|deg| {
                let t = f64::from(*deg).to_radians();
                let (a, b) = (t.cos() * edge, t.sin() * edge);
                let p = Point2::new(c.x + a * d.x + b * n.x, c.y + a * d.y + b * n.y);
                self.cell(p).and_then(|k| self.cells.get(k)).copied().unwrap_or(false)
            })
            .count();
        let phi = (touch as f64).to_radians().min(std::f64::consts::PI);
        ((1.0 - phi.cos()) / 2.0).clamp(0.0, 1.0)
    }
}

/// The places of a curve at most `step` apart, each with its unit
/// direction; a closed curve runs back to its start, which is not read twice.
pub(crate) fn samples(c: &Curve, step: f64) -> Vec<(Point2, Point2)> {
    let mut line: Vec<Point2> = c.pts().iter().map(|p| p.at()).collect();
    if c.closed()
        && let Some(&f) = line.first()
    {
        line.push(f);
    }
    let mut out = Vec::new();
    for w in line.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b.x - a.x).hypot(b.y - a.y);
        if !(len > 0.0 && len.is_finite()) {
            continue;
        }
        let d = Point2::new((b.x - a.x) / len, (b.y - a.y) / len);
        let n = (len / step).ceil().clamp(1.0, 1e6) as usize;
        for k in 0..n {
            let t = k as f64 / n as f64;
            out.push((Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)), d));
        }
    }
    if !c.closed()
        && let (Some(&(_, d)), Some(&p)) = (out.last(), line.last())
    {
        out.push((p, d));
    }
    out
}

/// Reads each curve of `curves` in order on a copy of `stock`: the tool
/// disc at each place is cleared, then the material beside its edge is read.
pub(crate) fn read(stock: &Stock, curves: &[Curve], r: f64) -> Vec<Read> {
    let mut s = stock.clone();
    let mut out = Vec::new();
    for (i, c) in curves.iter().enumerate() {
        for (p, d) in samples(c, r / SAMPLES_PER_R) {
            s.clear(p, r);
            out.push(Read { curve: i, at: p, load: s.engagement(p, d, r) });
        }
    }
    out
}
