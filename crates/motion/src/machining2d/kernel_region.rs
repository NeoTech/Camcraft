//! The caller side of the path kernel's stage 1 (K1, #4025): turns the
//! region inputs of the 2D planners (a picked area, picked loops and open
//! runs, Contour `Side`, `StartSide`'s stock and band) into the kernel's
//! `Selection`, `MaterialSide` and `Bounds`. No old planner calls it: with
//! the planner switch at its default, the output does not change. A kind
//! PR of K6 calls it on its kernel route; until then only the pins build a
//! kernel region beside the old path.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 1: Region" (the three material-side carriers map onto
//! `MaterialSide`) and `docs/decisions/4025-kernel-region.md`.

use crate::geo2d::{Loop, Point2, Region};
use camcraft_pathkernel::region::REGION_GEOMETRY;
use camcraft_pathkernel::{Bounds, Chain, KernelRegion, MaterialSide, Refusal, Selection, Stage, ToolShape, WorkPlane, region};

use super::contour_params::Side;
use super::pass_start::StartSide;

/// The selection of a picked area: each loop of `area` is one closed chain
/// (its outer boundaries and its holes).
pub fn selection_of_area(area: &Region) -> Result<Selection, Refusal> {
    Selection::new(area.loops().iter().map(|l| Chain::closed(l.points().to_vec())).collect::<Result<_, _>>()?)
}

/// The selection of picked closed loops and open runs, as the app's
/// selection geometry gives them.
pub fn selection_of_picks(loops: &[Loop], runs: &[Vec<Point2>]) -> Result<Selection, Refusal> {
    let closed = loops.iter().map(|l| Chain::closed(l.points().to_vec()));
    let open = runs.iter().map(|r| Chain::open(r.clone()));
    Selection::new(closed.chain(open).collect::<Result<_, _>>()?)
}

/// The material side of a Contour `Side`. `Auto` has none until the
/// caller resolves it (`auto_side`), so it gives `None`.
#[must_use]
pub fn contour_material(side: Side) -> Option<MaterialSide> {
    match side {
        Side::Inside => Some(MaterialSide::Inside),
        Side::Outside => Some(MaterialSide::Outside),
        Side::Auto => None,
    }
}

/// The selection and the material side of a pocket-family area. A band
/// round a part (`around_part`, with the stock known) is the part, cut on
/// its Outside; each other area is cut on its Inside. The start side is not
/// read: it never changes the region.
pub fn pocket_selection(area: &Region, side: &StartSide) -> Result<(Selection, MaterialSide), Refusal> {
    match side.stock.as_ref().filter(|_| side.around_part) {
        Some(stock) => {
            let part = stock.difference(area).map_err(|_| Refusal::new(Stage::Region, REGION_GEOMETRY))?;
            Ok((selection_of_area(&part)?, MaterialSide::Outside))
        }
        None => Ok((selection_of_area(area)?, MaterialSide::Inside)),
    }
}

/// The bounds of a plan: the stock of `side` (if known), the keep-out area
/// and the top and the bottom of the levels.
pub fn bounds_of(side: &StartSide, keep_out: Region, top: f64, bottom: f64) -> Result<Bounds, Refusal> {
    Bounds::new(side.stock.clone(), keep_out, top, bottom)
}

/// The kernel region of a pocket-family area: [`pocket_selection`], then
/// the kernel's `region`.
pub fn pocket_region(
    area: &Region,
    side: &StartSide,
    bounds: &Bounds,
    plane: WorkPlane,
    tool: ToolShape,
    leave: f64,
) -> Result<KernelRegion, Refusal> {
    let (sel, material) = pocket_selection(area, side)?;
    region(&sel, material, bounds, plane, tool, leave)
}

#[cfg(test)]
#[path = "kernel_region_tests.rs"]
mod tests;
