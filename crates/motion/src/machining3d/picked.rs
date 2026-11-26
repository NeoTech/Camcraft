//! The 3D Pocket with the user's start and exit points (#3329): the
//! plain or offset-shell ladder, with or without "Do not touch" items,
//! whose levels the picks seat (`level_seam`). Rest roughing and the
//! Adaptive plan take no pick: the application refuses them by name
//! before this runs, and a level the HSM plan spirals is refused here.
//!
//! Provenance: the ladder's own (`entry::run_plain_with`); no vendor
//! material consulted.

use camcraft_sim::verify::{AVOID_PAD, AvoidSpec, elevate_rapid_links};

use super::feed::{guard_input, home};
use super::floor_levels::{coarse_step, cut_levels};
use super::ladder::ladder;
use super::level_seam::LevelSeam;
use super::params::Pocket3DParams;

use crate::cl::Toolpath;
use crate::geo2d::Region;
use crate::geometry::Triangle;
use crate::machining2d::{AdaptiveMethod, GenError};
use crate::stock_sim::Columns;

/// [`super::pocket3d_open_prev`] (`shell: false`) or
/// [`super::pocket3d_offset_prev`] (`shell: true`), with `avoid` as
/// [`super::pocket3d_avoid`] has it, seated by `seam`. An empty seam is
/// that path, move for move.
// One slot per law the run threads, as `run_plain_with` has them.
#[allow(clippy::too_many_arguments)]
pub fn pocket3d_picked(
    facets: &[Triangle],
    footprint: &Region,
    stock: Option<&Region>,
    p: &Pocket3DParams,
    shell: bool,
    mask: Option<&Columns<'_>>,
    avoid: Option<&AvoidSpec>,
    seam: &LevelSeam,
) -> Result<Toolpath, GenError> {
    guard_input(facets, footprint, p)?;
    let zs = cut_levels(facets, p, coarse_step(p))?;
    let av = avoid.filter(|a| !a.is_empty());
    let picks = (!seam.is_empty()).then_some(seam);
    let stock = if shell { None } else { stock };
    let mut path = ladder(facets, footprint, p, &zs, None, mask, home(p), shell, None, stock, av, AdaptiveMethod::Spiral, p.stock_to_leave, picks)?;
    if let Some(a) = av {
        let _ = elevate_rapid_links(&mut path, a, None, AVOID_PAD);
    }
    Ok(path)
}
