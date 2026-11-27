//! Stage 1 builder (K1, #4025): one region value from a selection, the
//! material side and the bounds. Each planner reads this value; the
//! material side is an input of its own, and no start side reaches it.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 1: Region"; the cutter-centre area as the Minkowski erosion of the
//! machinable area by the tool disc (Held, "On the Computational Geometry of
//! Pocket Machining", 1991, ch. 2), computed with the polygon offset and the
//! booleans of `camcraft-geometry`.

use crate::frame::{ToolShape, WorkPlane};
use crate::region::{Bounds, KernelRegion, MaterialSide, PlanarRegion};
use crate::region_check::oriented;
use crate::selection::Selection;
use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Loop, Point2, Region, seg_dist};

/// Refusal reason: an open chain, which encloses no area and so has no
/// inside or outside.
pub const OPEN_NEEDS_SIDE: &str = "an open chain encloses no area, so it has no inside or outside material side";
/// Refusal reason: a stock to leave that is not finite, or that cancels
/// the tool radius.
pub const BAD_LEAVE: &str = "the stock to leave must be a finite number greater than minus the tool radius";
/// Refusal reason: an Outside material side with no stock to bound it.
pub const NEEDS_STOCK: &str = "an Outside material side needs the stock: the area outside the selection has no limit";
/// Refusal reason: a selection that does not meet the stock.
pub const OFF_STOCK: &str = "the selection is outside the stock: there is no material to cut";
/// Refusal reason: a material side with no area (an Outside selection
/// that covers all of the stock).
pub const EMPTY_MATERIAL: &str = "the material side of the selection has no area";
/// Refusal reason: the polygon offset or a boolean refused its input.
pub const REGION_GEOMETRY: &str = "the region geometry cannot be computed: a value is out of range";

/// The distance (mm) within which a boundary vertex of the cutter-centre
/// area counts as a vertex of the stock outline: the 1 um of the K0 laws.
const ON_STOCK_TOL: f64 = 1e-3;

fn geo<T, E>(r: Result<T, E>) -> Result<T, Refusal> {
    r.map_err(|_| Refusal::new(Stage::Region, REGION_GEOMETRY))
}

/// True when `p` is on a loop of `r` within `tol`.
fn on_outline(r: &Region, p: Point2, tol: f64) -> bool {
    r.loops().iter().any(|l| {
        let pts = l.points();
        pts.iter().zip(pts.iter().cycle().skip(1)).any(|(a, b)| seg_dist(p, *a, *b) <= tol)
    })
}

/// True when the region has an area above the kernel tolerance.
fn has_area(r: &Region) -> bool {
    let a = r.area();
    !r.is_empty() && a.is_finite() && a > KERNEL_TOL
}

/// The area that the closed chains of `sel` enclose. Nested chains are
/// holes; chains that do not nest add up.
fn enclosed(sel: &Selection) -> Result<Region, Refusal> {
    let loops: Vec<Loop> = sel.chains().iter().map(|c| geo(Loop::new(c.points().to_vec()))).collect::<Result<_, _>>()?;
    geo(Region::new(loops))
}

/// The one region of a plan.
///
/// - `sel`: the picked chains, in the 2D coordinates of `plane`. Each
///   chain must be closed.
/// - `material`: `Inside` cuts the area that the chains enclose; `Outside`
///   cuts the stock round it.
/// - `bounds`: the stock (needed for `Outside`; for `Inside` the selection
///   must meet it when it is known) and the keep-out area.
/// - `tool`, `leave`: the cutter-centre area keeps the tool radius plus
///   `leave` from the selection, and the tool radius from the keep-out.
///
/// The centre of an `Outside` region reaches the stock outline: the disc
/// then sweeps past every stock edge and corner. Its loops on the stock
/// outline are the stock edges, each other loop is a part wall. The air is
/// the band of one tool radius outside the stock where the disc does not
/// touch it, less the part zone (the part grown by the tool radius plus
/// `leave`: a part can reach out of the stock, #4241). An `Inside` region is a closed area: its loops are all part
/// walls and it has no air. A pure function of its inputs.
pub fn region(
    sel: &Selection,
    material: MaterialSide,
    bounds: &Bounds,
    plane: WorkPlane,
    tool: ToolShape,
    leave: f64,
) -> Result<KernelRegion, Refusal> {
    if let Some(c) = sel.first_open() {
        let r = Refusal::new(Stage::Region, OPEN_NEEDS_SIDE).fix("pick a closed chain, or join the points back to the first one");
        return Err(match c.points().first() {
            Some(p) => r.at(*p),
            None => r,
        });
    }
    let r = tool.radius();
    let d = r + leave;
    if !leave.is_finite() || !d.is_finite() || d <= KERNEL_TOL {
        return Err(Refusal::new(Stage::Region, BAD_LEAVE));
    }
    let area = enclosed(sel)?;
    if !has_area(&area) {
        return Err(Refusal::new(Stage::Region, EMPTY_MATERIAL));
    }
    let keep = if bounds.keep_out().is_empty() { Region::empty() } else { geo(bounds.keep_out().offset(r))? };
    let stock = bounds.stock();
    if let Some(s) = stock
        && !has_area(&geo(area.intersect(s))?)
    {
        return Err(Refusal::new(Stage::Region, OFF_STOCK).fix("pick a selection that meets the stock"));
    }
    let mut part_zone = Region::empty();
    let (center, air) = match material {
        MaterialSide::Inside => (geo(area.offset(-d))?, Region::empty()),
        MaterialSide::Outside => {
            let Some(s) = stock.filter(|s| has_area(s)) else {
                return Err(Refusal::new(Stage::Region, NEEDS_STOCK).fix("define the stock of the setup"));
            };
            if !has_area(&geo(s.difference(&area))?) {
                return Err(Refusal::new(Stage::Region, EMPTY_MATERIAL));
            }
            let air = geo(geo(s.offset(2.0 * r))?.difference(&geo(s.offset(r))?))?;
            let zone = geo(area.offset(d))?;
            let air = geo(geo(air.difference(&keep))?.difference(&zone))?;
            part_zone = zone;
            (geo(s.difference(&part_zone))?, air)
        }
    };
    let mut center = if keep.is_empty() { center } else { geo(center.difference(&keep))? };
    if !has_area(&center) {
        return Err(Refusal::new(Stage::Region, crate::region::EMPTY_CENTER).fix("use a smaller tool or a larger selection"));
    }
    if oriented(&center).is_some() {
        center = geo(Region::new(center.loops().to_vec()))?;
    }
    let (mut walls, mut edge) = (Vec::new(), Vec::new());
    for l in center.loops() {
        let on_stock = match (material, stock) {
            (MaterialSide::Outside, Some(s)) => l.points().iter().all(|p| on_outline(s, *p, ON_STOCK_TOL)),
            _ => false,
        };
        if on_stock { edge.push(l.clone()) } else { walls.push(l.clone()) }
    }
    let keep_out = bounds.keep_out().clone();
    PlanarRegion::new(plane, center, material, walls, edge, air, keep_out, tool)?.with_part_zone(part_zone).map(KernelRegion::Planar)
}
