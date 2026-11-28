//! The independent stock grid of the law measures: a column height for
//! each cell, cut down by the swept disc of the tool (#4154).
//! Provenance: a Z-buffer (dexel) stock model, as in the stock simulation
//! literature (Van Hook, "Real-time shaded NC milling display", SIGGRAPH
//! 1986), on the shape-mask grid of `common/mask_grid.rs`.

use camcraft_motion::cl::{ArcPlane, Move};
use camcraft_motion::geo2d::Point2;
use camcraft_motion::geometry::Point3;

use super::super::mask_grid::Grid;
use super::Spec;

/// A material column stands above the tool by more than this.
pub(super) const Z_EPS: f64 = 1e-3;
/// The most sample points of one move.
const MAX_STEPS: f64 = 200_000.0;

pub(super) struct Stock {
    pub grid: Grid,
    /// The column top of each cell (`NEG_INFINITY`: no material).
    pub top: Vec<f64>,
    /// The cells of the stock that the tool may cut (stock less part).
    pub cuttable: Vec<bool>,
    /// The cells of the stock at the start.
    pub stock0: Vec<bool>,
    /// The cells of a pre-drilled hole.
    pub hole: Vec<bool>,
    pub r: f64,
}

impl Stock {
    pub fn new(spec: &Spec) -> Result<Self, String> {
        let pad = 2.0 * spec.tool_r + 1.0;
        let grid = Grid::for_region(&spec.stock, spec.cell, pad)?;
        let stock0 = grid.rasterize_region(&spec.stock);
        let part = spec
            .part
            .as_ref()
            .map_or_else(|| vec![false; grid.len()], |p| grid.rasterize_region(p));
        let hole = spec
            .holes
            .as_ref()
            .map_or_else(|| vec![false; grid.len()], |h| grid.rasterize_region(h));
        let mut top = vec![f64::NEG_INFINITY; grid.len()];
        let mut cuttable = vec![false; grid.len()];
        for i in 0..grid.len() {
            let solid = (stock0[i] && !hole[i]) || part[i];
            if solid {
                top[i] = spec.top;
            }
            cuttable[i] = stock0[i] && !hole[i] && !part[i];
        }
        Ok(Stock {
            grid,
            top,
            cuttable,
            stock0,
            hole,
            r: spec.tool_r,
        })
    }

    /// Call `f` with each cell whose centre is within `rr` of `p`.
    pub fn for_disc(&self, p: Point2, rr: f64, mut f: impl FnMut(usize)) {
        let g = &self.grid;
        if !(p.x.is_finite() && p.y.is_finite() && rr.is_finite() && rr > 0.0) {
            return;
        }
        let span = |c: f64, o: f64, n: usize| -> (usize, usize) {
            let lo = ((c - rr - o) / g.cell - 0.5).floor().clamp(0.0, n as f64);
            let hi = ((c + rr - o) / g.cell - 0.5).ceil().clamp(0.0, n as f64);
            (lo as usize, (hi as usize).min(n.saturating_sub(1)))
        };
        let (ix0, ix1) = span(p.x, g.x0, g.w);
        let (iy0, iy1) = span(p.y, g.y0, g.h);
        let r2 = rr * rr + 1e-12;
        for iy in iy0..=iy1 {
            for ix in ix0..=ix1 {
                let c = g.center(ix, iy);
                if (c.x - p.x).powi(2) + (c.y - p.y).powi(2) <= r2 {
                    f(g.idx(ix, iy));
                }
            }
        }
    }

    /// The inner disc of the tool: its radius less half a cell, so that a
    /// tool that only touches a wall does not read the wall.
    fn inner(&self) -> f64 {
        (self.r - 0.5 * self.grid.cell).max(0.5 * self.grid.cell)
    }

    /// True if material stands above `z` under the tool at `p`.
    pub fn material(&self, p: Point2, z: f64) -> bool {
        let mut hit = false;
        self.for_disc(p, self.inner(), |i| hit |= self.top[i] > z + Z_EPS);
        hit
    }

    /// True if a cell under the tool at `p` is a pre-drilled hole.
    pub fn in_hole(&self, p: Point2) -> bool {
        let mut hit = false;
        self.for_disc(p, self.inner(), |i| hit |= self.hole[i]);
        hit
    }

    /// True if a cell under the tool at `p` was stock at the start.
    pub fn was_stock(&self, p: Point2) -> bool {
        let mut hit = false;
        self.for_disc(p, self.inner(), |i| hit |= self.stock0[i]);
        hit
    }

    /// Cut the swept disc of the tool from `a` to `b`.
    pub fn sweep(&mut self, a: Point3, b: Point3) {
        let mut mask = Vec::new();
        let seg = (Point2::new(a.x, a.y), Point2::new(b.x, b.y));
        self.grid
            .sweep_capsule(seg, self.r, (a.z, b.z), false, &mut mask, &mut self.top);
    }
}

/// The sample points of a move, `step` apart at most, with both ends. An
/// arc in the XY plane follows its circle; other moves are straight.
pub(super) fn points(m: &Move, step: f64) -> Vec<Point3> {
    let (a, b) = (m.from, m.to);
    let finite = [a.x, a.y, a.z, b.x, b.y, b.z].iter().all(|v| v.is_finite());
    if !finite || !(step.is_finite() && step > 0.0) {
        return Vec::new();
    }
    let arc = m.arc.filter(|c| c.plane == ArcPlane::Xy);
    let (a0, sweep, rad, c) = match arc {
        Some(arc) => {
            let c = arc.center;
            let a0 = (a.y - c.y).atan2(a.x - c.x);
            let a1 = (b.y - c.y).atan2(b.x - c.x);
            let tau = std::f64::consts::TAU;
            let mut s = a1 - a0;
            if arc.ccw {
                while s <= 1e-12 {
                    s += tau;
                }
            } else {
                while s >= -1e-12 {
                    s -= tau;
                }
            }
            ((a0), s, (a.x - c.x).hypot(a.y - c.y), c)
        }
        None => (0.0, 0.0, 0.0, Point2::new(0.0, 0.0)),
    };
    let len = if arc.is_some() {
        (sweep.abs() * rad).hypot(b.z - a.z)
    } else {
        ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt()
    };
    let n = (len / step).ceil().clamp(1.0, MAX_STEPS) as usize;
    (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            let z = a.z + t * (b.z - a.z);
            if arc.is_some() {
                let ang = a0 + t * sweep;
                Point3::new(c.x + rad * ang.cos(), c.y + rad * ang.sin(), z)
            } else {
                Point3::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y), z)
            }
        })
        .collect()
}
