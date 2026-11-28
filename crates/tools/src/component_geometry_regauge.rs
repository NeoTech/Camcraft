//! #4496: a holder component at another gauge length. The new outline is
//! the built-in holder's outline at the new length, and it is accepted
//! only when it is the holder's own outline with one straight cylindrical
//! section made longer or shorter (the body between the nose and the
//! flange). Then the nose keeps its steel from the nose face, the flange
//! keeps its steel from the gauge line, and the section between keeps the
//! radius that the old outline gives it: the new outline is never smaller
//! than the steel of the holder. A holder whose outline is not the
//! built-in holder's, or whose body the flange envelope covers (the
//! outline then shows no body section, and the hidden steel can be wider
//! than the library body), is refused by name.
//! Provenance: our own rule over our own holder library; no HSMWorks
//! material involved.

use super::{ComponentGeometry, GeometryError};
use crate::holder_library::HolderEntry;
use crate::spun::SpunPoint;

/// The slack (mm) of the outline compare: float dust of two library
/// outlines at different gauge lengths, far below a drawn step.
const TOL: f64 = 1e-6;

/// Why a holder is not drawn at another gauge length. Each refusal names
/// the reason.
#[derive(Clone, Debug, PartialEq)]
pub enum RegaugeError {
    /// The holder states no gauge length (no workpiece port below its MCS).
    NoGauge,
    /// The holder has no body outline.
    NoBody,
    /// The holder library refuses the new gauge length.
    Library(GeometryError),
    /// The outline is not the built-in holder's outline at the holder's
    /// gauge length.
    NotLibrary { name: String, gauge_len: f64 },
    /// The new outline does not come from one cylindrical section of the
    /// old outline: the outline shows no body to make longer or shorter.
    NoSection { name: String },
}

impl core::fmt::Display for RegaugeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoGauge => write!(f, "the holder states no gauge length"),
            Self::NoBody => write!(f, "the holder has no body outline"),
            Self::Library(e) => write!(f, "{e}"),
            Self::NotLibrary { name, gauge_len } => write!(
                f,
                "the outline is not the built-in holder '{name}' at its gauge length {gauge_len} mm, so the built-in outline can be smaller than this holder's steel - edit the outline itself"
            ),
            Self::NoSection { name } => write!(
                f,
                "the outline of '{name}' shows no body section that this gauge length makes longer or shorter: the flange envelope covers the body, and the hidden steel can be wider than the built-in body - edit the outline itself"
            ),
        }
    }
}

impl std::error::Error for RegaugeError {}

fn same(a: &[SpunPoint], b: &[SpunPoint]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(p, q)| (p.z - q.z).abs() <= TOL && (p.r - q.r).abs() <= TOL && (p.arc - q.arc).abs() <= TOL)
}

/// `pts` without the points inside one cylinder: an ER nut and the body
/// as wide as the nut are one section of the outline.
fn merged(pts: &[SpunPoint]) -> Vec<SpunPoint> {
    let mut out: Vec<SpunPoint> = Vec::with_capacity(pts.len());
    for p in pts {
        if let [.., a, b] = out.as_slice() {
            let inside = (a.r - b.r).abs() <= TOL
                && (b.r - p.r).abs() <= TOL
                && a.arc.abs() <= TOL
                && b.arc.abs() <= TOL
                && a.z < b.z
                && b.z < p.z;
            if inside {
                out.pop();
            }
        }
        out.push(*p);
    }
    out
}

/// `pts` with the straight cylindrical segment from point `i` to point
/// `i + 1` made `d` mm longer (shorter when `d` is negative): the points
/// above it move by `d`. `None` when that segment is not a cylinder off
/// the axis, or is too short to lose `-d`.
fn stretched(pts: &[SpunPoint], i: usize, d: f64) -> Option<Vec<SpunPoint>> {
    let (a, b) = (pts.get(i)?, pts.get(i + 1)?);
    let cylinder = a.r > TOL && (a.r - b.r).abs() <= TOL && a.arc.abs() <= TOL && b.z - a.z > TOL;
    if !cylinder || b.z - a.z + d < -TOL {
        return None;
    }
    let mut out: Vec<SpunPoint> = pts.iter().take(i + 1).copied().collect();
    out.extend(pts.iter().skip(i + 1).map(|p| SpunPoint { z: p.z + d, ..*p }));
    // A section that is gone leaves two points on one another.
    out.dedup_by(|q, p| (p.z - q.z).hypot(p.r - q.r) <= TOL);
    Some(out)
}

impl ComponentGeometry {
    /// This holder at gauge length `gauge_len`, with its workpiece port
    /// `port` on the new nose face: the built-in `entry`'s outline at that
    /// length, when this holder's outline is `entry`'s at its own gauge
    /// length and the new outline is that outline with one cylindrical
    /// section made longer or shorter. Else the refusal by name.
    pub fn regauged_holder(&self, entry: &HolderEntry, port: &str, gauge_len: f64) -> Result<Self, RegaugeError> {
        let old_g = self.port(port).map(|f| -f.origin()[2]).filter(|g| g.is_finite() && *g > 0.0).ok_or(RegaugeError::NoGauge)?;
        let body = self.body().ok_or(RegaugeError::NoBody)?;
        let name = entry.name();
        let was = Self::holder(entry, port, old_g).map_err(RegaugeError::Library)?;
        let was_body = was.body().ok_or(RegaugeError::NoBody)?;
        let old = body.profile().points();
        if (body.base() - was_body.base()).abs() > TOL || !same(old, was_body.profile().points()) {
            return Err(RegaugeError::NotLibrary { name, gauge_len: old_g });
        }
        let new = Self::holder(entry, port, gauge_len).map_err(RegaugeError::Library)?;
        let new_pts = merged(new.body().ok_or(RegaugeError::NoBody)?.profile().points());
        let old = merged(old);
        let d = gauge_len - old_g;
        let kept = (0..old.len()).any(|i| stretched(&old, i, d).is_some_and(|s| same(&s, &new_pts)));
        if kept { Ok(new) } else { Err(RegaugeError::NoSection { name }) }
    }
}

#[cfg(test)]
#[path = "component_geometry_regauge_tests.rs"]
mod tests;
