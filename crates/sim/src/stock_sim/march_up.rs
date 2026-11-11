//! The column read from below (#637 E2b): `march`'s mirror, rising
//! from the world floor to the LOWEST material centre and reading the
//! material's floor from there. After a baked flip (`field_turn`) the
//! field holds cuts on both sides - the earlier setup's solids are
//! monotone downward in the new frame, the next setup's monotone
//! upward - so a column's material is one interval `[max(mesh floor,
//! lower cut's top), min(mesh top, upper cut's bottom)]`: the top is
//! `march`'s, the floor is this module's.
//!
//! ## The laws here
//!
//! - **Same probe, same proof.** The ascent sphere-traces the same
//!   keyed probe as the descent, so every sign is the cell centre's
//!   exact parity-and-claim sign and a material/air bracket is a proof;
//!   the step budget and the stall floor are `march`'s, mirrored.
//! - **The floor is read two ways, the HIGHER binding.** At the lowest
//!   material centre: the cut layer's downward entry (`cut_downs` -
//!   the distance DOWN from the centre to the ray's entry into a swept
//!   solid, a flipped bake's `cut_ups` record moved whole), and the
//!   shell's lowest crossing in the bracket (exact for any plane,
//!   lattice-aligned or not). A cut top above the mesh floor is the
//!   floor; a mesh floor above the cut's top is. Neither on record is
//!   `Unreadable` - a surface the field cannot place is never read as
//!   the cell boundary (a flipped bounded tool's bottom face).
//! - **Over-report, never under.** A broken probe promise answers the
//!   last proven-air height, inside the bracket: the floor read low is
//!   material over-reported, the field's blessed direction.
//!
//! Provenance: sphere-tracing of a signed-distance field along a ray
//! (the published marching law); this codebase's own parity sign
//! (`solid_rest`); the mirror of `march` by construction. Nothing from
//! vendor binaries.

use super::field::{cell_of, center_of, SdfField};
use super::march::{face, probe, shell_in, BRACKET_WALK, MAX_MARCH_STEPS, STEP_MIN};

/// How one column's ascent ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ReadUp {
    /// The floor of the lowest material, bracketed inside its cell.
    Floor(f64),
    /// The ray reached the world ceiling with material never found.
    NoMaterial,
    /// The march could not finish inside its budget, or the floor's
    /// surface is one the field cannot place: no verdict.
    Unreadable,
}

/// The bracket's floor, from a material point above an air one: the
/// walk descends to the LOWEST material centre, then reads the floor
/// from the downward entry on record there and the shell's crossing
/// below it (`face`, the higher binding). `None`: neither on record.
fn refine_down(f: &SdfField, cx: &Option<Vec<f64>>, x: f64, y: f64, mat: f64, air: f64, cuts: bool) -> Option<f64> {
    let cell = f.cell();
    let (ix, iy) = (cell_of(x, cell), cell_of(y, cell));
    let mut j = cell_of(mat, cell);
    for _ in 0..BRACKET_WALK {
        let cm = center_of(j, cell);
        let pm = probe(f, cx, x, y, cm, cuts);
        if !pm.is_finite() || pm >= 0.0 {
            return Some(air); // the probe's promise broke: no verdict, not a film
        }
        let below = j.saturating_sub(1);
        let ca = center_of(below, cell);
        let pa = probe(f, cx, x, y, ca, cuts);
        if pa.is_nan() {
            return Some(air);
        }
        if pa >= 0.0 {
            let entry = if cuts {
                let d = f.down_at([ix, iy, j]);
                (d.is_finite() && d >= 0.0 && d < cell).then_some(cm - d)
            } else {
                None
            };
            return face(entry, shell_in(cx, ca, cm, false), false);
        }
        if below == j {
            return Some(air); // the lattice's own end
        }
        j = below; // the floor stands below this cell's centre - descend
    }
    Some(air)
}

/// Sphere-trace up `(x, y)` from `z_lo` until the steel's floor (or a
/// bail state names why there is none). `air` tracks the lowest probe
/// known to stand in air, so every bracket is honest.
pub(crate) fn march_up(
    f: &SdfField,
    cx: &Option<Vec<f64>>,
    x: f64,
    y: f64,
    z_lo: f64,
    z_hi: f64,
    cuts: bool,
) -> ReadUp {
    if cx.is_none() {
        return ReadUp::Floor(z_lo); // Open answers material, the `inside` law
    }
    let step_min = f.cell() * STEP_MIN;
    let (mut z, mut air) = (z_lo, z_lo);
    for _ in 0..MAX_MARCH_STEPS {
        let phi = probe(f, cx, x, y, z, cuts);
        if !phi.is_finite() {
            return ReadUp::Unreadable; // a field that cannot say how far
        }
        if phi < 0.0 {
            return refine_down(f, cx, x, y, z, air, cuts).map_or(ReadUp::Unreadable, ReadUp::Floor);
        }
        if phi == 0.0 {
            return ReadUp::Floor(z); // the probe landed ON the surface
        }
        if z + phi >= z_hi {
            // The final probe is the ceiling itself: past it the field
            // has no steel to offer and the march has no world left.
            let p = probe(f, cx, x, y, z_hi, cuts);
            return if p < 0.0 {
                refine_down(f, cx, x, y, z_hi, air.min(z), cuts).map_or(ReadUp::Unreadable, ReadUp::Floor)
            } else if p == 0.0 {
                ReadUp::Floor(z_hi)
            } else {
                ReadUp::NoMaterial
            };
        }
        air = z;
        z += phi.max(step_min);
    }
    ReadUp::Unreadable
}
