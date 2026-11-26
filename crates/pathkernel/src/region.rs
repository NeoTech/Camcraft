//! Stage 1, Region (K1, #4025): the cutter-centre area and its edges. This
//! step holds the types and their checked constructors; K1 adds `region()`.
//! The stage takes no start side: that is law 2 in the type system.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 1: Region"; the cutter-centre offset of a pocket boundary as in
//! Held, "On the Computational Geometry of Pocket Machining" (1991).

use crate::frame::ToolShape;
use crate::verdict::{Refusal, Stage};
use crate::{Loop, Region, finite, region_finite};

/// The side of the selection's closed loops where the material is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialSide {
    Inside,
    Outside,
}

/// The bounds of one plan: the stock footprint (if known), the keep-out
/// area (fixtures, Do not touch) and the top and bottom along the normal.
#[derive(Clone, Debug, PartialEq)]
pub struct Bounds {
    stock: Option<Region>,
    keep_out: Region,
    top: f64,
    bottom: f64,
}

/// Refusal reason: bounds whose top is below the bottom, or a value that is
/// not finite.
pub const BAD_BOUNDS: &str = "the bounds need a finite top at or above a finite bottom, and finite areas";

impl Bounds {
    pub fn new(stock: Option<Region>, keep_out: Region, top: f64, bottom: f64) -> Result<Self, Refusal> {
        let areas_ok = stock.as_ref().is_none_or(region_finite) && region_finite(&keep_out);
        if !finite(&[top, bottom]) || top < bottom || !areas_ok {
            return Err(Refusal::new(Stage::Region, BAD_BOUNDS));
        }
        Ok(Self { stock, keep_out, top, bottom })
    }
    #[must_use]
    pub fn stock(&self) -> Option<&Region> {
        self.stock.as_ref()
    }
    #[must_use]
    pub fn keep_out(&self) -> &Region {
        &self.keep_out
    }
    #[must_use]
    pub fn top(&self) -> f64 {
        self.top
    }
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.bottom
    }
}

/// The output of stage 1. Phase 1 has only a planar area; a later phase
/// adds a region that is not a planar area as a new variant.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum KernelRegion {
    Planar(PlanarRegion),
}

impl KernelRegion {
    /// The planar region, if this is one.
    #[must_use]
    pub fn planar(&self) -> Option<&PlanarRegion> {
        match self {
            Self::Planar(p) => Some(p),
        }
    }
}

/// A planar cutter-centre area with its edges.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarRegion {
    center: Region,
    material: MaterialSide,
    part_walls: Vec<Loop>,
    stock_edge: Vec<Loop>,
    air: Region,
    keep_out: Region,
    tool: ToolShape,
}

/// Refusal reason: an empty cutter-centre area (the tool does not fit).
pub const EMPTY_CENTER: &str = "the tool does not fit: the cutter-centre area is empty";
/// Refusal reason: a loop with a point that is not finite.
pub const NOT_FINITE: &str = "a loop of the region has a point that is not a finite number";
/// Refusal reason: a region with no wall and no stock edge.
pub const NO_EDGE: &str = "the region needs at least one part wall or stock edge";
/// Refusal reason: an edge loop with no area (all its points on one line).
pub const FLAT_EDGE: &str = "an edge loop of the region has no area";

impl PlanarRegion {
    /// The checked region. Refused for an empty or degenerate `center`, a
    /// point that is not finite, an edge loop with no area, or no edge at
    /// all. The tool is valid by its own type. K1 adds the partition check
    /// of `part_walls` and `stock_edge` against the boundary of `center`.
    pub fn new(
        center: Region,
        material: MaterialSide,
        part_walls: Vec<Loop>,
        stock_edge: Vec<Loop>,
        air: Region,
        keep_out: Region,
        tool: ToolShape,
    ) -> Result<Self, Refusal> {
        let edges_finite = part_walls.iter().chain(&stock_edge).all(|l| l.points().iter().all(|p| finite(&[p.x, p.y])));
        if !edges_finite || ![&center, &air, &keep_out].iter().all(|r| region_finite(r)) {
            return Err(Refusal::new(Stage::Region, NOT_FINITE));
        }
        let area = center.area();
        if center.is_empty() || !area.is_finite() || area <= 0.0 {
            return Err(Refusal::new(Stage::Region, EMPTY_CENTER).fix("use a smaller tool or a larger selection"));
        }
        if part_walls.is_empty() && stock_edge.is_empty() {
            return Err(Refusal::new(Stage::Region, NO_EDGE));
        }
        if part_walls.iter().chain(&stock_edge).any(|l| !l.area().is_finite() || l.area().abs() <= 0.0) {
            return Err(Refusal::new(Stage::Region, FLAT_EDGE));
        }
        Ok(Self { center, material, part_walls, stock_edge, air, keep_out, tool })
    }
    #[must_use]
    pub fn center(&self) -> &Region {
        &self.center
    }
    #[must_use]
    pub fn material(&self) -> MaterialSide {
        self.material
    }
    #[must_use]
    pub fn part_walls(&self) -> &[Loop] {
        &self.part_walls
    }
    #[must_use]
    pub fn stock_edge(&self) -> &[Loop] {
        &self.stock_edge
    }
    #[must_use]
    pub fn air(&self) -> &Region {
        &self.air
    }
    #[must_use]
    pub fn keep_out(&self) -> &Region {
        &self.keep_out
    }
    #[must_use]
    pub fn tool(&self) -> ToolShape {
        self.tool
    }
}
