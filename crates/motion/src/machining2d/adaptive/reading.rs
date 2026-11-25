//! The planner's stock read over a finished plan (#3933): the Offset
//! rings spiral is not planned on the stock, so its worst contact is
//! read after the fact on the same grid and contact law the HSM plan
//! keeps its cuts to (`raster`, `quality::rim_for`), and named when it
//! passes the load. Provenance: the engagement (immersion) angle read
//! off an in-process stock model, as `quality` (Altintas, Manufacturing
//! Automation, ch. 2; Kloypayan & Lee 2002); the walk is the planner's
//! own. No vendor material consulted.

use crate::geo2d::{Point2, Region};
use crate::load::load_at_rim;

use super::raster::Raster;
use super::{GRID_PER_D, GenError};

/// The rim (mm) the read takes the contact angle on for a tool of
/// radius `r`, on the planner's grid (`quality::rim_for`).
pub(crate) fn read_rim(r: f64) -> f64 {
    crate::quality::rim_for(2.0 * r, 2.0 * r / GRID_PER_D)
}

/// The worst straight-wall load (ae/D) the walks `passes` read in order
/// on the material `reach` for a tool of radius `r`, and where. Each
/// pass is cut from its first point; its points before `read_from[i]`
/// are cut but not read (an entry's own core). A pass with no
/// `read_from` entry is read whole.
pub(crate) fn worst_load(reach: &Region, passes: &[Vec<Point2>], read_from: &[usize], r: f64) -> Result<(f64, Point2), GenError> {
    if !(r.is_finite() && r > 0.0) {
        return Err(GenError::Invalid("the read needs a positive tool radius".into()));
    }
    let d = 2.0 * r;
    let mut ras = Raster::from_region(reach, d / GRID_PER_D, r)?;
    let rim = crate::quality::rim_for(d, ras.h);
    let law = (r, rim, r / 10.0);
    let mut worst = (0.0f64, passes.first().and_then(|p| p.first()).copied().unwrap_or_default());
    for (i, pass) in passes.iter().enumerate() {
        if pass.is_empty() {
            continue;
        }
        let from = read_from.get(i).copied().unwrap_or(0).min(pass.len() - 1);
        if from > 0 {
            ras.walk_until(&pass[..=from], law, f64::INFINITY);
        }
        let w = ras.walk_until(&pass[from..], law, f64::INFINITY);
        if w.0 > worst.0 {
            worst = w;
        }
    }
    Ok((load_at_rim(worst.0, d, rim), worst.1))
}
