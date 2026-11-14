//! The operation owns its mark (#1595): the program verdict's credit for
//! the mark an Engrave or a Trace on the finished part was asked to cut.
//!
//! Provenance: our own law over this codebase's column deviation
//! (`deviate`) and its declared-credit walk (`edge_break::credit_floors`,
//! #303); the sweep is the column engine's own atoms. Nothing from
//! vendor binaries or output.
//!
//! The law: the part the program must leave is the model WITH every
//! declared mark cut. A mark is what its operation's own cutter sweeps,
//! no deeper than the operation's budget under the model's top in each
//! column. A column the verdict convicted that the mark's cutter swept
//! is judged against that floor instead - `max(swept, model top -
//! budget)`: cut down to it (within `tol`) is the finished part; cut
//! below it - by the mark itself past its budget, or by another
//! operation under the mark - is a gouge charged at its depth under
//! the floor. A column no mark swept keeps the model's verdict. Rest is
//! untouched: a mark is permission to remove, never a demand.

use super::cut::{simulate, SimOp};
use super::deviate::Deviation;
use super::dexel::Dexel;
use super::edge_break::credit_floors;

use crate::geometry::Triangle;

/// One operation's declared mark: its own moves and cutter, and how far
/// under the part's surface it may cut (mm).
#[derive(Clone, Copy, Debug)]
pub struct Mark<'a> {
    pub op: SimOp<'a>,
    pub budget: f64,
}

/// What `mark`'s cutter sweeps on `dex`'s lattice: `+inf` where it never
/// reaches. `None` when the sweep cannot be read (the engine's own
/// refusals) - no credit then, the sharp verdict stands.
fn swept(dex: &Dexel, mark: &Mark) -> Option<Dexel> {
    let mut s = Dexel {
        x0: dex.x0,
        y0: dex.y0,
        cell: dex.cell,
        nx: dex.nx,
        ny: dex.ny,
        stock_z: f64::INFINITY,
        tiles: vec![None; dex.tiles.len()],
    };
    simulate(&mut s, &[mark.op]).ok()?;
    Some(s)
}

/// Re-judge `dev`'s gouge columns against the declared `marks`; returns
/// how many columns the marks cleared or re-charged. `dex` is the
/// verdict's own lattice (the tops `dev` was read from) and `facets`
/// the part it judged. A mark with no positive finite budget declares
/// nothing; a lattice that does not match `dev` credits nothing - the
/// verdict then stands as it was, never looser.
pub fn credit_marks(dev: &mut Deviation, dex: &Dexel, facets: &[Triangle], marks: &[Mark]) -> usize {
    if dev.gouge_cells == 0 {
        return 0;
    }
    let sweeps: Vec<(Dexel, f64)> = marks
        .iter()
        .filter(|m| m.budget.is_finite() && m.budget > 0.0)
        .filter_map(|m| swept(dex, m).map(|s| (s, m.budget)))
        .collect();
    if sweeps.is_empty() {
        return 0;
    }
    credit_floors(dev, dex, facets, |ix, iy, p, _tol| {
        let k = ix + iy * dex.nx;
        sweeps
            .iter()
            .map(|(s, budget)| (s.top_at(k), budget))
            .filter(|(z, _)| z.is_finite())
            .map(|(z, budget)| z.max(p - budget))
            .fold(f64::INFINITY, f64::min)
    })
}
