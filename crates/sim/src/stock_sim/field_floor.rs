//! A field column's two sides (#637 steps B and E1): its top by the
//! down-march, its FLOOR exactly - from the shell where no cut reaches
//! it, and under a cut by the monotone law below. The column body's
//! commonest re-clamp is a part flipped onto a face an earlier setup
//! machined, holes drilled through it included.
//!
//! ## The laws here
//!
//! - **A spanning tool is monotone on a vertical ray.** Every tool
//!   `Stock::simulate` builds spans the verdict world above its tip
//!   (`spanning_flank`), so if its swept solid holds `(x, y, z)` it holds
//!   `(x, y, z')` for every `z' > z` in the world. The material left in a
//!   column is then ONE interval `[shell floor, tool bottom]` or nothing:
//!   a cut from above lowers the top, never raises the floor, and reaches
//!   the floor only by cutting through.
//! - **The floor is the shell's lowest crossing** (a mesh plane, exact to
//!   rounding) wherever a material centre stands in the column: the
//!   down-march's `Top` proves one, so a claim near the floor is a tool
//!   that passed close, not one that went below the top here.
//! - **A column with no material centre is read from the record.** Cut
//!   through, or a sliver thinner than a cell: `cut_ups` of the cell whose
//!   centre is the highest below the floor plane holds the ray's entry
//!   into the swept solid (`field_ray`, closed-form under a flat floor),
//!   so the tool's bottom is `centre + entry`. Material stands iff that
//!   bottom is not below the floor - the column body's own `top < floor`
//!   empty law, same strictness. A record that must exist and does not is
//!   refused by name, never read as air.
//! - **A bounded tool suspends the law.** A solid that does not span the
//!   world above its tip (the truncated taper, a bare ball, a lathe
//!   sweep, a posed axis) can leave material above a cut; the field marks
//!   itself (`bounded_cuts`) when one is subtracted, and a floor a cut
//!   came within a cell of is then refused by name (`CutFromBelow`), as
//!   step B did for every cut - never a guess.
//!
//! Provenance: ray-parity crossings of a watertight shell (the `solid_rest`
//! law this codebase already uses); monotonicity of an upward-spanning
//! solid along its axis (elementary set geometry); our own design (#637
//! research comments).

use super::field::{cell_of, center_of, SdfField};
use super::march::{march, world, Read};

/// One column's floor read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloorRead {
    /// The lowest face of the stock under the column, exact to rounding.
    Floor(f64),
    /// The column crosses no stock at all, or every bit of it was cut away.
    NoMaterial,
    /// A bounded tool's cut has claimed the column near its floor: an
    /// upward read would not be exact, so none is given.
    CutFromBelow,
    /// The column cannot be read (an `Open` parity column, a march that
    /// cannot finish, a record that should exist and does not).
    Unreadable,
}

/// One column's material, both sides (#637 E1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnSpan {
    /// Material stands from `floor` up to `top` (`top >= floor`).
    Material { top: f64, floor: f64 },
    /// No material stands on the column: air beside the stock, or cut
    /// through.
    Empty,
    /// A bounded tool's cut reached the floor: no exact read (the module
    /// law).
    CutFromBelow,
    /// The column cannot be read.
    Unreadable,
}

impl SdfField {
    /// The floor under `(x, y)`: `column_span`'s floor.
    #[must_use]
    pub fn floor_at(&self, x: f64, y: f64) -> FloorRead {
        match self.column_span(x, y) {
            ColumnSpan::Material { floor, .. } => FloorRead::Floor(floor),
            ColumnSpan::Empty => FloorRead::NoMaterial,
            ColumnSpan::CutFromBelow => FloorRead::CutFromBelow,
            ColumnSpan::Unreadable => FloorRead::Unreadable,
        }
    }

    /// The material standing on the column `(x, y)` after every cut: see
    /// the module law.
    #[must_use]
    pub fn column_span(&self, x: f64, y: f64) -> ColumnSpan {
        if !(x.is_finite() && y.is_finite()) {
            return ColumnSpan::Unreadable;
        }
        let Some(cross) = self.shell_crossings(x, y) else {
            return ColumnSpan::Unreadable;
        };
        let finite = cross.iter().copied().filter(|z| z.is_finite());
        let Some(floor) = finite.clone().reduce(f64::min) else {
            return ColumnSpan::Empty;
        };
        let ceiling = finite.reduce(f64::max).unwrap_or(floor);
        if self.bounded_cuts && self.cut_near(x, y, floor) {
            return ColumnSpan::CutFromBelow;
        }
        let (z_hi, z_lo) = world(self);
        match march(self, &Some(cross), x, y, z_hi, z_lo, true) {
            Read::Unreadable => ColumnSpan::Unreadable,
            // A material centre proves the interval reaches the shell
            // floor (the monotone law); a top under the floor breaks the
            // parity promise and is no verdict.
            Read::Top(top) if top.is_finite() && top >= floor => ColumnSpan::Material { top, floor },
            Read::Top(_) => ColumnSpan::Unreadable,
            Read::NoMaterial => self.sliver(x, y, floor, ceiling),
        }
    }

    /// Has a cut claimed a cell within one cell of the floor point? The
    /// swept tool came within a cell of any centre in the 3x3x3 block
    /// around it (a claim is `-d_tool` at a centre: a claim above `-cell`
    /// means the tool passed within a cell). Every point of a cell lies
    /// within sqrt(3)/2 cell of its centre, so this catches a point inside
    /// a cut near its wall and a cut thinner than a cell (review of #649),
    /// conservatively: a tool that passed close without reaching the
    /// floor counts too, the safe direction.
    fn cut_near(&self, x: f64, y: f64, floor: f64) -> bool {
        let (ix, iy, iz) = (cell_of(x, self.cell), cell_of(y, self.cell), cell_of(floor, self.cell));
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let key = [ix.saturating_add(dx), iy.saturating_add(dy), iz.saturating_add(dz)];
                    if self.cut_at(key) > -self.cell {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// The column holds no material centre: cut through, or a sliver
    /// thinner than a cell standing on the floor. The cell whose centre is
    /// the highest below the floor plane keeps the ray's entry into the
    /// swept solid: the tool's bottom on this column, the lowest any move
    /// proved (`field_ray`'s union law).
    fn sliver(&self, x: f64, y: f64, floor: f64, ceiling: f64) -> ColumnSpan {
        let cell = self.cell;
        let (ix, iy) = (cell_of(x, cell), cell_of(y, cell));
        let mut k = cell_of(floor, cell);
        if center_of(k, cell) >= floor {
            k = k.saturating_sub(1);
        }
        let key = [ix, iy, k];
        let u = self.up_at(key);
        if u.is_finite() && u >= 0.0 {
            let bottom = center_of(k, cell) + u;
            if !bottom.is_finite() {
                return ColumnSpan::Unreadable;
            }
            return if bottom < floor {
                ColumnSpan::Empty // cut through
            } else {
                ColumnSpan::Material { top: bottom.min(ceiling), floor }
            };
        }
        // No entry on record. An uncut stock thinner than a cell has no
        // centre either, and is the shell itself; anything else is a
        // record that should exist - refused, never read as air.
        let unclaimed = |z: i32| self.cut_at([ix, iy, z]) == f64::NEG_INFINITY;
        if unclaimed(k) && unclaimed(k.saturating_add(1)) && ceiling - floor < cell {
            return ColumnSpan::Material { top: ceiling, floor };
        }
        ColumnSpan::Unreadable
    }
}
