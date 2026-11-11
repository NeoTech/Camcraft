//! The verdict march (M7.1 C): reading column heights off the field.
//! The descent and its bracket live here; its consumers - the
//! deviate/measure/sheet verdict readers - are `march_verdicts`,
//! split by the 400-line law.
//!
//! Provenance: sphere-tracing of a signed-distance field along a ray
//! is the published marching law for implicit surfaces; the exactness
//! here comes from this codebase's own parity (`column_crossings`, the
//! `solid_rest` reuse), which makes every probe's SIGN exact wherever
//! its magnitude is merely bounded. Nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **The verdict is a HEIGHT, in the column engine's units.** The
//!   march answers the Z of the top of remaining material on a cell
//!   center, so a `Deviation` or `StockReport` from either engine
//!   carries the same vocabulary - perpendicular offsets, the
//!   renderer's numbers, belong to the eikonal pass (M7.2), not here.
//! - **A center's sign is exact, so a bracket is a proof.** At a
//!   cell CENTER, `phi < 0` iff the point stands in remaining
//!   material: the mesh term rides live parity, and the cut layer's
//!   conservative magnitude cannot flip a sign the tool did not
//!   claim. `refine` climbs to the topmost material center of the
//!   column, then reads the surface from there: at the ray-entry
//!   distance the cut layer proves (`field_ray` - a conservative
//!   lower bound, so flanks and grooves answer at their own tilted
//!   geometry), or at the shell's own crossing inside that cell (the
//!   cached parity column, exact to rounding for ANY plane, aligned
//!   with the lattice or not - #637 E2b; until then the cell boundary
//!   stood in, exact only for a lattice-aligned plane), the lower of
//!   the two where both bind. The residue is sub-cell solver error,
//!   stated as the parity suite's tolerances, never as a silent
//!   agreement. A sign flip with neither a crossing nor a record is
//!   a surface the field cannot place: `Unreadable`, never the
//!   boundary as a guess (a flipped bounded tool's bottom face).
//!   `march_up` is this descent's mirror, reading the floor.
//! - **A march that cannot finish says so.** The step budget and the
//!   stall floor turn into `Unreadable`, whose verdict is NaN - the
//!   no-verdict value the column engine already uses for air, never a
//!   zero dressed up as a clean bill of health. `NoMaterial` (the ray
//!   reached the world floor without finding steel) is NaN too: the
//!   field's stock has a bottom the semi-infinite column does not,
//!   and a column past it holds nothing to judge.
//! - **An `Open` column answers its ceiling.** Parity `Open` is
//!   material everywhere (stage A's never-hide rule), and the
//!   watertight gate makes it unreachable; should a future mesh ever
//!   be built around it, the march reports the world's top height -
//!   rest over-reported, the blessed direction.

use super::field::{cell_of, center_of, signed, SdfField};
use super::kernel::{StockError, MAX_CELLS_PER_AXIS};

/// How one column's descent ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Read {
    /// The top of remaining material, bracketed inside its cell.
    Top(f64),
    /// The ray reached the world floor with material never found.
    NoMaterial,
    /// The march could not finish inside its budget: no verdict.
    Unreadable,
}

/// Descent budget: from the world ceiling, cap-sized steps finish a
/// sane column in dozens; 4096 says the field cannot answer.
pub(crate) const MAX_MARCH_STEPS: u32 = 4096;

/// Stall floor, in cells: below it the probe already stands at the
/// surface's own threshold, so the step keeps its size and the
/// bracket walk takes the verdict from there.
pub(crate) const STEP_MIN: f64 = 1.0 / 64.0;

/// Bracket-walk budget: a column's centers, plus slack: the walk
/// ends at the first adjacent material/air pair, so this only bounds
/// pathological non-monotone columns.
pub(crate) const BRACKET_WALK: u32 = 8192;

/// The world the march runs in: one band above and below the shell,
/// so every probe's magnitude is answerable from inside it.
pub(crate) fn world(f: &SdfField) -> (f64, f64) {
    let b = f.bounds();
    let pad = f.cap() + f.cell();
    (b.max.z + pad, b.min.z - pad)
}

/// The stock's own XY lattice, origin-locked like its grid: the cells
/// the shell's bounding box covers, one row of indices each way.
pub(crate) struct Lattice {
    pub(crate) i0: i32,
    pub(crate) j0: i32,
    pub(crate) x0: f64,
    pub(crate) y0: f64,
    pub(crate) nx: usize,
    pub(crate) ny: usize,
    pub(crate) cell: f64,
}

pub(crate) fn lattice_of(f: &SdfField) -> Result<Lattice, StockError> {
    let cell = f.cell();
    let b = f.bounds();
    let (i0, j0) = (cell_of(b.min.x, cell), cell_of(b.min.y, cell));
    let (mut i1, mut j1) = (
        (b.max.x / cell).ceil() as i32 - 1,
        (b.max.y / cell).ceil() as i32 - 1,
    );
    (i1, j1) = (i1.max(i0), j1.max(j0));
    let (nx, ny) = ((i1 - i0 + 1) as usize, (j1 - j0 + 1) as usize);
    if nx > MAX_CELLS_PER_AXIS || ny > MAX_CELLS_PER_AXIS {
        return Err(StockError::Invalid(format!(
            "grid would be {nx}x{ny} cells; at most {MAX_CELLS_PER_AXIS} per axis"
        )));
    }
    Ok(Lattice {
        i0,
        j0,
        x0: i0 as f64 * cell,
        y0: j0 as f64 * cell,
        nx,
        ny,
        cell,
    })
}

/// The keyed probe: the stock-after-cuts distance at `(x, y, z)` with
/// the column's crossings cached - the sign exact, the magnitude only
/// a bound (the laws above). `cuts` false walks the stock BEFORE any
/// cut, which is how a report says what the program removed.
pub(crate) fn probe(f: &SdfField, cx: &Option<Vec<f64>>, x: f64, y: f64, z: f64, cuts: bool) -> f64 {
    let cell = f.cell();
    let key = [cell_of(x, cell), cell_of(y, cell), cell_of(z, cell)];
    let mesh = signed(f.mag_at(key), SdfField::parity_of(cx, z));
    if cuts {
        mesh.max(f.cut_at(key))
    } else {
        mesh
    }
}

/// The shell's surface inside a bracket: the cached column's crossings
/// in `(lo, hi]` - the highest for a top read (`top`), the lowest for a
/// floor read - `None` where the parity never flips there. Exact to
/// rounding for any plane, lattice-aligned or not (#637 E2b).
pub(crate) fn shell_in(cx: &Option<Vec<f64>>, lo: f64, hi: f64, top: bool) -> Option<f64> {
    let it = cx.as_ref()?.iter().copied().filter(|z| z.is_finite() && *z > lo && *z <= hi);
    if top { it.reduce(f64::max) } else { it.reduce(f64::min) }
}

/// The surface between a material center `cm` and the air center
/// beside it, from the cut layer's entry (`cm + entry` when the ray
/// meets a solid inside this cell: `cut_ups` for a top read, `cut_downs`
/// for a floor read) and the shell's crossing in the bracket: the
/// material's own end is the nearer of the two to `cm` (`min` for a
/// top, `max` for a floor). Neither on record: `None` - the field
/// cannot place that surface, and it never guesses the boundary.
pub(crate) fn face(entry: Option<f64>, shell: Option<f64>, top: bool) -> Option<f64> {
    match (entry, shell) {
        (Some(e), Some(s)) => Some(if top { e.min(s) } else { e.max(s) }),
        (Some(v), None) | (None, Some(v)) => Some(v),
        (None, None) => None,
    }
}

/// The bracket's surface, from a material point below an air one:
/// `mat`'s own cell center certifies material (a probe's sign IS its
/// center's - the values are per cell), and the walk climbs to the
/// TOPMOST material center. Its height is read two ways, the lower
/// binding where both do (`face`):
///
/// - where the cut layer's ray-entry companion proves the vertical
///   ray MEETS a swept solid inside this cell, that entry is the
///   surface (conservative to below - `field_ray`'s direction), and
///   tilted flanks and curved grooves answer at their own geometry,
///   not at the lattice's convenience;
/// - where the shell crosses the bracket, that crossing is the mesh's
///   own surface, exact for every plane (#637 E2b).
///
/// A column never reports above its air center, so the estimate sits
/// inside the proven bracket; the residue is sub-cell solver error
/// (golden section and bisection floors), stated as the parity
/// suite's small tolerances, never as a silent agreement. `None`: the
/// sign flipped with neither a crossing nor a record - no verdict.
fn refine(f: &SdfField, cx: &Option<Vec<f64>>, x: f64, y: f64, mat: f64, air: f64, cuts: bool) -> Option<f64> {
    let cell = f.cell();
    let (ix, iy) = (cell_of(x, cell), cell_of(y, cell));
    for j in (cell_of(mat, cell)..).take(BRACKET_WALK as usize) {
        let cm = center_of(j, cell);
        let pm = probe(f, cx, x, y, cm, cuts);
        if !pm.is_finite() || pm >= 0.0 {
            return Some(air); // the probe's promise broke: no verdict, not a film
        }
        let ca = center_of(j.saturating_add(1), cell);
        let pa = probe(f, cx, x, y, ca, cuts);
        if pa.is_nan() {
            return Some(air);
        }
        if pa >= 0.0 {
            // The uncut walk (mesh only) has no ray to enter; the cut
            // walk answers at the entry when this cell's ray met a
            // solid inside it. An entry ABOVE the mesh's own crossing
            // (M13.1: a curved tool bottom grazing the stock top from
            // above) is air already: the crossing binds, as the
            // column's `min(top, cut)` does.
            let entry = if cuts {
                let u = f.up_at([ix, iy, j]);
                (u.is_finite() && u >= 0.0 && u < cell).then_some(cm + u)
            } else {
                None
            };
            return face(entry, shell_in(cx, cm, ca, true), true);
        }
        // else: the surface stands above this cell's center - climb.
    }
    Some(air)
}

/// Sphere-trace down `(x, y)` from `z_hi` until the steel's top (or a
/// bail state names why there is none). `air` tracks the highest
/// probe known to stand in air, so every bracket is honest.
pub(crate) fn march(
    f: &SdfField,
    cx: &Option<Vec<f64>>,
    x: f64,
    y: f64,
    z_hi: f64,
    z_lo: f64,
    cuts: bool,
) -> Read {
    if cx.is_none() {
        return Read::Top(z_hi); // Open answers material, the `inside` law
    }
    let step_min = f.cell() * STEP_MIN;
    let (mut z, mut air) = (z_hi, z_hi);
    for _ in 0..MAX_MARCH_STEPS {
        let phi = probe(f, cx, x, y, z, cuts);
        if !phi.is_finite() {
            return Read::Unreadable; // a field that cannot say how far
        }
        if phi < 0.0 {
            return refine(f, cx, x, y, z, air, cuts).map_or(Read::Unreadable, Read::Top);
        }
        if phi == 0.0 {
            return Read::Top(z); // the probe landed ON the surface
        }
        if z - phi <= z_lo {
            // The final probe is the floor itself: past it the field
            // has no steel to offer and the march has no world left.
            let p = probe(f, cx, x, y, z_lo, cuts);
            return if p < 0.0 {
                refine(f, cx, x, y, z_lo, air.max(z), cuts).map_or(Read::Unreadable, Read::Top)
            } else if p == 0.0 {
                Read::Top(z_lo)
            } else {
                Read::NoMaterial
            };
        }
        air = z;
        z -= phi.max(step_min);
    }
    Read::Unreadable
}

/// One column's remaining-material top, for `Stock::surface_z`.
pub(crate) fn column_top(f: &SdfField, x: f64, y: f64) -> f64 {
    if !(x.is_finite() && y.is_finite()) {
        return f64::NAN;
    }
    let (z_hi, z_lo) = world(f);
    match march(f, &f.shell_crossings(x, y), x, y, z_hi, z_lo, true) {
        Read::Top(z) => z,
        _ => f64::NAN,
    }
}
