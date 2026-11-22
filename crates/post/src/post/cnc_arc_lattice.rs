//! The stream writer's arc law on the micrometre lattice (issue #3442).
//!
//! Provenance: original. The `.cnc` stream writes every position as
//! whole micrometres (`cnc::serialize`), and its reader takes an arc
//! record whose end equals its start as a full turn (the
//! `full_circle_raw` form; RS-274 / ISO 6983 read such a block the same
//! way). A short arc whose chord is under the lattice step would round to
//! that form and be read as a full circle. Only the writer still knows
//! the true sweep, so the writer decides:
//! - the rounded in-plane ends differ: the arc record, as before;
//! - they are equal and the arc turns more than half a circle (or its
//!   true ends are equal): the full turn within one micrometre, the arc
//!   record as before;
//! - they are equal on a short arc: the record is written as a `Feed` to
//!   its rounded end. The tool moves less than one micrometre in the
//!   plane, and the normal axis (a helix) still moves as written. The
//!   record stays (a zero-length `Feed` when nothing moves) so the
//!   stream keeps one record for each toolpath move.

use super::cnc::MoveRec;
use crate::cl::ArcPlane;

/// The lattice word of a length: micrometres, nearest.
fn um(v: f64) -> i64 {
    (v * 1000.0).round() as i64
}

/// True when the arc record `m`, which starts at `from`, must be written
/// as a straight `Feed`: its in-plane ends round to the same micrometre
/// while its true sweep is half a turn or less. A record with no arc, no
/// known start, or a non-finite value is never changed.
pub fn short_arc_on_lattice(from: Option<[f64; 3]>, m: &MoveRec) -> bool {
    let (Some((c0, c1)), Some(s)) = (m.center, from) else {
        return false;
    };
    let e = [m.x, m.y, m.z];
    // The right-handed in-plane frame (u, v), u x v = +normal, and the
    // centre in it (the centre pair is in word order: I J, I K, J K).
    let (u, v, cu, cv) = match m.plane {
        ArcPlane::Xy => (0, 1, c0, c1),
        ArcPlane::Xz => (2, 0, c1, c0),
        ArcPlane::Yz => (1, 2, c0, c1),
    };
    let vals = [s[u], s[v], e[u], e[v], cu, cv];
    if vals.iter().any(|x| !x.is_finite()) {
        return false;
    }
    if um(s[u]) != um(e[u]) || um(s[v]) != um(e[v]) {
        return false;
    }
    if s[u] == e[u] && s[v] == e[v] {
        return false; // a full turn stated as one record
    }
    let a0 = (s[v] - cv).atan2(s[u] - cu);
    let a1 = (e[v] - cv).atan2(e[u] - cu);
    let mut sweep = if m.ccw { a1 - a0 } else { a0 - a1 };
    if sweep < 0.0 {
        sweep += std::f64::consts::TAU;
    }
    sweep <= std::f64::consts::PI
}
