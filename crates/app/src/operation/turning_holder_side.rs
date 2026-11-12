//! Issue #1102: which side of the spindle axis a lathe holder stands on for a turning path - a rigid holder never flips through the axis mid-path, so its +x (radial, away from the axis) is the side the path works on, read off the path's signed radii (the handbook turning convention: a front or a rear tool post, a facing pass running a little past centre); our own design, no vendor material consulted.
//!
//! The holder law reads a body point (x, y, z) at sqrt((X + x)^2 + y^2)
//! with X the tip's radius signed along the HOLDER's +x. The path's
//! signed meridian ([`SignedMeridian`]) takes its + side from the path's
//! first point off the axis, which need not be the holder's side: a rear
//! tool that starts a hair past centre puts every later tip at X < 0 of
//! that frame. Reading such a body along the frame's +x would mirror it
//! through the axis, so a holder that strikes could be judged clear.

use crate::cl::Toolpath;

use super::turning_rapids::SignedMeridian;

/// How far past the spindle axis a tip may run while its holder stays
/// on its own side (mm): a facing pass cleaning the centre pip, which
/// overruns by up to a nose radius (2.4 mm is the largest common ISO
/// turning nose). A path past centre farther than this, on both sides,
/// cannot tell which side holds its holder and is refused by name.
pub(super) const PAST_CENTRE: f64 = 2.5;
/// On-axis float dust: a tip this close to the axis stands on neither side.
const ON_AXIS: f64 = 1e-6;

/// The holder's side of `sm`'s signed radial on `path`: `1.0` when it
/// stands on the + side, `-1.0` on the - side. The side is the one the
/// path reaches farthest on; the other may hold only a past-centre
/// overrun ([`PAST_CENTRE`]) while the far side runs beyond it. Refuses
/// by name otherwise - the side a body is read on is never guessed.
pub(super) fn holder_side(sm: &SignedMeridian, path: &Toolpath) -> Result<f64, String> {
    side_of(path.moves.iter().flat_map(|m| [sm.sr(m.from).1, sm.sr(m.to).1]))
}

/// [`holder_side`] over the signed tip radii `xs`.
pub(super) fn side_of(xs: impl IntoIterator<Item = f64>) -> Result<f64, String> {
    let (mut pos, mut neg) = (0.0_f64, 0.0_f64);
    for x in xs {
        if !x.is_finite() {
            return Err("a tip's radius is not finite".into());
        }
        pos = pos.max(x);
        neg = neg.max(-x);
    }
    let (far, near) = if neg > pos { (neg, pos) } else { (pos, neg) };
    if near > ON_AXIS && (near > PAST_CENTRE || far <= PAST_CENTRE) {
        return Err(format!(
            "the path stands on both sides of the spindle axis ({:.3} mm on one, {:.3} mm on the other) - which side holds the holder cannot be told; a tip may run at most {PAST_CENTRE} mm past centre",
            far, near
        ));
    }
    Ok(if neg > pos { -1.0 } else { 1.0 })
}

#[cfg(test)]
#[path = "turning_holder_side_tests.rs"]
mod tests;
