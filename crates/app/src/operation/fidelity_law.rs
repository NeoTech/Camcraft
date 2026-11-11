//! What an operation kind's gouge verdicts need of its model's fidelity
//! (#864): the tolerance its envelope law is judged at, the smallest
//! tolerance any of its gouge laws holds, and whether those laws read a
//! coarse model's excess at all.
//!
//! Provenance: our own registry of our own verify laws; `finish_tol_of`
//! is the envelope budget `generate` always handed to `verify`, moved
//! here unchanged so the program gate reads the same number.

use super::material::GOUGE_TOL;
use super::{OpParams, Section};
use crate::verify::Mode;

/// The envelope law's budget for `params`: the op's own tolerance for the
/// kinds that law judges, `None` for every other kind.
pub(crate) fn finish_tol_of(params: &OpParams) -> Option<f64> {
    match params {
        OpParams::Finish3D { tolerance, .. }
        | OpParams::Contour5D { tolerance, .. }
        | OpParams::Parallel5D { tolerance, .. }
        | OpParams::Swarf { tolerance, .. }
        | OpParams::AdvancedSwarf { tolerance, .. }
        | OpParams::RotaryContour { tolerance, .. }
        | OpParams::RotaryParallel { tolerance, .. }
        // The rough ladder's tolerance is exactly this
        // law's budget - its only reading (E2), and
        // the stamped ladder reads it the same way
        // (E3): the positions are the pocket's, the
        // verdict measures them under the pose.
        | OpParams::RotaryOffset { tolerance, .. }
        | OpParams::MultiAxisRough { tolerance, .. }
        // Flow is the finish's law on turning lines:
        // the tolerance is the envelope budget, as
        // always in this group (M5.1 I). The
        // drive-curve contour reads it the same way -
        // its ONLY reading, the path itself comes from
        // the picks (M5.1 J).
        | OpParams::Flow5D { tolerance, .. }
        | OpParams::MultiAxisContour { tolerance, .. }
        // Facing reads it the same way - its ONLY
        // reading: the stations come from the
        // profile, the budget judges them (M5.2 A).
        // The profile kinds read it TWICE (the
        // kernel chords its ride at it) and stand
        // before this same budget (M5.2 B).
        | OpParams::Facing { tolerance, .. }
        | OpParams::ProfileRough { tolerance, .. }
        | OpParams::ProfileFinish { tolerance, .. }
        // The contour rides read it the same way the
        // profile kinds do - TWICE: the kernel chords
        // the ride at it, the door judges the promised
        // line at it, and the envelope scan measures
        // the same skin under this budget (M5.2 E).
        // Without it the verdict cannot measure, and
        // an unmeasurable Envelope op refuses.
        | OpParams::Chamfering { tolerance, .. }
        | OpParams::TaperTurning { tolerance, .. }
        // The boring bar reads it the contour twins'
        // way - TWICE, and from the inside: the
        // kernel chords the wall and the center curve
        // at quarters of it, and the envelope scan
        // measures the ride against the same
        // promise (M5.2 F).
        | OpParams::BoringTurning { tolerance, .. }
        | OpParams::MultiPassTurning { tolerance, .. } => Some(*tolerance),
        _ => None,
    }
}

/// The smallest tolerance a gouge law of this kind is held at: the
/// section and solid laws' `GOUGE_TOL`, and the envelope law's own budget
/// where the kind carries one. A coarse model's excess must leave all of
/// them positive, or the operation is refused instead of judged.
pub(crate) fn floor_of(params: &OpParams) -> f64 {
    finish_tol_of(params).map_or(GOUGE_TOL, |t| t.min(GOUGE_TOL))
}

/// Whether every gouge law that judges this kind reads a coarse model's
/// excess: the upright mill kinds, whose verdict is `verify`'s section
/// law (b), solid law (b2 - `generate::contain_into` for the containment
/// kinds) and envelope law (f), plus the lathe's axial hole verdict.
///
/// Not yet: the cone and engrave rides (their verdict is the generator's
/// own construction law), the cross drill (`cross_part`), the multi-axis
/// and rotary kinds (the stamp, lean and radial field laws), turning
/// (the meridian scans) and plasma. On a reduced-fidelity model those are
/// refused at the program gate rather than judged loosely.
pub(crate) fn excess_aware(params: &OpParams) -> bool {
    let info = params.kind().info();
    let upright_mill = matches!(info.section, Section::Mill2D | Section::Mill3D | Section::Drilling);
    let judged = matches!(info.mode, Mode::Area | Mode::Boundary)
        || (info.mode == Mode::Envelope && info.section == Section::Mill3D);
    let cross_drill = matches!(params, OpParams::Drill { cross, .. } if !cross.is_empty());
    upright_mill && judged && !cross_drill
}
