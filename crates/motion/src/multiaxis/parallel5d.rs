//! 5-axis parallel (M5.1 stage C): the Z-following raster of
//! [`crate::finishing3d::parallel`'], with the spindle oriented at
//! every contact by the lead/lag law. The raster lines are the
//! pattern's "parallel" half and the scallop cos-law `s = √(8·r·tol)`
//! (or the explicit stepover) their side step - the stepdown rides
//! the same ring ladder's law, and steep flanks still finish with
//! the rings' own level changes inside `parallel`. What stage C adds
//! is again the pose: each line's tangent direction is the raster's,
//! the normal the facet's, and the lean keeps the shank off the
//! walls the raster follows.
//!
//! Provenance: parallel-path 5-axis finishing with tilt control is
//! standard CAM practice; built on this codebase's own support-scan
//! raster. No vendor material consulted.

use crate::cl::Toolpath;
use crate::finishing3d::{self, Finish3DParams};
use crate::geometry::Triangle;
use crate::machining2d::GenError;

use super::avoid::Shank;
use super::stamp::stamp_axes;
use super::tilt::AxisControl;

/// The raster with the tool leaning where the law says - and, with a
/// `shank` (M5.1 stage F), dodging the part inside the cone where the
/// law's pose would touch.
pub fn parallel5d(
    facets: &[Triangle],
    p: &Finish3DParams,
    c: &AxisControl,
    shank: Option<&Shank>,
) -> Result<Toolpath, GenError> {
    let base = finishing3d::parallel(facets, p)?;
    stamp_axes(&base, facets, p.tool_diameter * 0.5, c, shank)
}

/// [`parallel5d`] with the user's start and exit points (#3330): the
/// raster's passes ordered for the picks (`finishing3d::parallel_seamed`:
/// the corner nearest the start pick begins the cut, the exit pick
/// chooses where the last pass ends), then posed by the same stamp - the
/// tilt limit and the shank dodge judge the re-ordered path as every
/// other. Returns the posed path and the raster's ordered ends (tips of
/// the upright plan). An empty `seam` is [`parallel5d`], byte for byte.
pub fn parallel5d_seamed(
    facets: &[Triangle],
    p: &Finish3DParams,
    c: &AxisControl,
    shank: Option<&Shank>,
    seam: &finishing3d::RasterSeam,
) -> Result<(Toolpath, Option<finishing3d::RasterEnds>), GenError> {
    let (base, ends) = finishing3d::parallel_seamed(facets, p, seam)?;
    Ok((stamp_axes(&base, facets, p.tool_diameter * 0.5, c, shank)?, ends))
}
