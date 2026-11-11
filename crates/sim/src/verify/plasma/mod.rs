//! Plasma through-cut laws (#672, part of #621): what replaces the gouge
//! and collision checks for a 2D plasma program. A plasma op cuts through,
//! so the hazards are not depth but ORDER and GAPS:
//!
//! - **Cut order (refusal).** A contour cut after a contour that encloses
//!   it (a hole after its outline, an island after its hole): the
//!   enclosing cut has freed the material it sits in.
//! - **Cut crossing (refusal).** A contour whose closed ring crosses an
//!   earlier contour's ring: the kerf runs through a part already cut -
//!   the through-cut's 2D gouge.
//! - **Kerf crossing (warning).** A cut whose centre line comes within one
//!   kerf of an earlier cut's: the torch crosses an open kerf, the arc
//!   voltage jumps and torch height control dives - it "can smash the
//!   torch into the material" (LinuxCNC Plasma Primer §12). This law
//!   covers only that torch-height hazard, so it warns (a shared-kerf
//!   common line is legal; a lead running over an earlier kerf is this
//!   hazard, not a gouge).
//! - **Tip-up (warning).** A rapid that crosses the area of a part already
//!   cut free: a freed part can tip up into the torch (Tormach 1300PL
//!   manual UM10720 §1.4.2, §4.12.8).
//! - **Malformed (refusal).** A program the laws cannot judge, refused by
//!   name - see [`shape`]: a bad kerf, a NaN point, a one-point cut that
//!   is not a centre-pierced small hole, a small hole that is not a hole,
//!   a duplicate id, a parent that is the contour itself, an open run or
//!   no cut, an inner contour with no parent, a closed contour whose cut
//!   never closes, a rapid list that does not match the cuts, and a
//!   stated parent the geometry disagrees with (enclosure is recomputed
//!   from the rings, never trusted from the record).
//!
//! Input is a neutral program - the cuts in order, each the torch-on
//! polyline (arcs chorded) with its contour id, role and enclosing
//! contour - built from a path's plasma record by
//! [`PlasmaProgram::from_toolpath`] (the CL's `PLASMA` / `TORCH ON C<id>
//! OUTER|INNER|OPEN [PARENT<id>] [SMALL..]` / `TORCH OFF`, #670) and
//! nothing of the operation's internals.
//!
//! Provenance: our own geometry over the cited plasma practice (UM10720,
//! the LinuxCNC Plasma Primer); no vendor implementation inspected.

mod geom;
mod program;
mod shape;

use crate::geo2d::Point2;

pub use program::CHORD_TOL;

/// What a contour is: a part's outline (or an island in a hole), a hole,
/// or an open path (a slit, not a closed area).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutRole {
    Outer,
    Inner,
    Open,
}

impl CutRole {
    /// The record's word, for the refusals.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            CutRole::Outer => "OUTER",
            CutRole::Inner => "INNER",
            CutRole::Open => "OPEN",
        }
    }
}

/// One torch-on span: a contour cut from pierce to torch-off.
#[derive(Clone, Debug, PartialEq)]
pub struct PlasmaCut {
    pub contour: u32,
    pub role: CutRole,
    /// The smallest closed contour enclosing this one (a hole's outline,
    /// an island's hole); `None` for a top-level contour. Stated by the
    /// record and verified against the rings by the laws.
    pub parent: Option<u32>,
    /// A centre-pierced small hole: its path may be a single point.
    pub small: bool,
    /// The torch path, lead-in to lead-out, in cut order.
    pub path: Vec<Point2>,
}

impl PlasmaCut {
    /// The closed ring this cut rides - the stretch of the path between
    /// the first point it returns to and that return (the leads and a
    /// small hole's overburn fall outside it); `None` when the path never
    /// closes (an open run, a one-point small hole).
    #[must_use]
    pub fn ring(&self) -> Option<&[Point2]> {
        geom::ring_of(&self.path)
    }
}

/// A plasma program: the kerf, the cuts in cut order, and the rapid that
/// reaches each cut's pierce - `rapids[i]` is (previous torch-off point,
/// pierce point) before `cuts[i]`, `None` when the program starts there.
/// `rapids` has one entry per cut.
#[derive(Clone, Debug, Default)]
pub struct PlasmaProgram {
    pub kerf: f64,
    pub cuts: Vec<PlasmaCut>,
    pub rapids: Vec<Option<(Point2, Point2)>>,
}

/// What the laws find.
#[derive(Clone, Debug, PartialEq)]
pub enum PlasmaFinding {
    /// Refusal: `contour` is cut after `enclosing`, which encloses it.
    CutOrder { contour: u32, enclosing: u32 },
    /// Refusal: `contour`'s ring crosses earlier `earlier`'s ring at `at`.
    CutCrossing { contour: u32, earlier: u32, at: Point2 },
    /// Warning: contour `contour` comes within a kerf of earlier `earlier`.
    KerfCrossing { contour: u32, earlier: u32, at: Point2 },
    /// Warning: the rapid before `contour` crosses freed part `freed`.
    TipUp { contour: u32, freed: u32, at: Point2 },
    /// Refusal: the program cannot be judged.
    Malformed(String),
}

impl PlasmaFinding {
    /// Does this finding block the post?
    #[must_use]
    pub fn refuses(&self) -> bool {
        matches!(
            self,
            PlasmaFinding::CutOrder { .. } | PlasmaFinding::CutCrossing { .. } | PlasmaFinding::Malformed(_)
        )
    }

    /// Where the finding is, when it has a place (mm).
    #[must_use]
    pub fn at(&self) -> Option<Point2> {
        match self {
            PlasmaFinding::CutCrossing { at, .. }
            | PlasmaFinding::KerfCrossing { at, .. }
            | PlasmaFinding::TipUp { at, .. } => Some(*at),
            PlasmaFinding::CutOrder { .. } | PlasmaFinding::Malformed(_) => None,
        }
    }

    /// The finding's text without its place (the caller adds `at` in its
    /// own units); the whole line in mm is the `Display`.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            PlasmaFinding::CutOrder { contour, enclosing } => format!(
                "cut order: contour C{contour} is cut after contour C{enclosing}, which encloses it - a hole after its outline (or an island after its hole) cuts into a part already freed"
            ),
            PlasmaFinding::CutCrossing { contour, earlier, .. } => {
                format!("cut crossing: contour C{contour}'s cut crosses contour C{earlier}'s - the kerf runs through a part already cut")
            }
            PlasmaFinding::KerfCrossing { contour, earlier, .. } => format!(
                "kerf crossing: contour C{contour} comes within a kerf of contour C{earlier}'s cut - the torch height control can dive into the open kerf"
            ),
            PlasmaFinding::TipUp { contour, freed, .. } => {
                format!("tip-up: the rapid into contour C{contour} crosses part C{freed}, already cut free")
            }
            PlasmaFinding::Malformed(why) => format!("malformed program: {why}"),
        }
    }
}

impl core::fmt::Display for PlasmaFinding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.describe())?;
        if let Some(q) = self.at() {
            write!(f, " near X{:.3} Y{:.3}", q.x, q.y)?;
        }
        Ok(())
    }
}

/// Judge a plasma program. Findings come in cut order; a malformed
/// program is one refusal and nothing else.
#[must_use]
pub fn judge(p: &PlasmaProgram) -> Vec<PlasmaFinding> {
    let parents = match shape::well_formed(p) {
        Ok(parents) => parents,
        Err(why) => return vec![PlasmaFinding::Malformed(why)],
    };
    let rings: Vec<Option<&[Point2]>> = p.cuts.iter().map(PlasmaCut::ring).collect();
    let mut out = Vec::new();
    for (i, c) in p.cuts.iter().enumerate() {
        // Cut order: anything enclosing this contour already cut.
        if let Some(parent) = parents.get(i).copied().flatten()
            && p.cuts[..i].iter().any(|e| e.contour == parent)
        {
            out.push(PlasmaFinding::CutOrder { contour: c.contour, enclosing: parent });
        }
        // Tip-up: the rapid into this cut over a part already cut free.
        if let Some(Some((a, b))) = p.rapids.get(i) {
            for (j, e) in p.cuts[..i].iter().enumerate().filter(|(_, e)| e.role == CutRole::Outer) {
                if let Some(ring) = rings.get(j).copied().flatten()
                    && let Some(at) = geom::crosses_area(*a, *b, ring)
                {
                    out.push(PlasmaFinding::TipUp { contour: c.contour, freed: e.contour, at });
                }
            }
        }
        for (j, e) in p.cuts[..i].iter().enumerate() {
            // Cut crossing: ring through ring is the 2D gouge, refused.
            if let (Some(Some(mine)), Some(Some(theirs))) = (rings.get(i), rings.get(j))
                && let Some(at) = geom::rings_cross(mine, theirs)
            {
                out.push(PlasmaFinding::CutCrossing { contour: c.contour, earlier: e.contour, at });
                continue;
            }
            // Kerf crossing: within a kerf of an earlier cut's centre line.
            if let Some(at) = geom::near(&c.path, &e.path, p.kerf) {
                out.push(PlasmaFinding::KerfCrossing { contour: c.contour, earlier: e.contour, at });
            }
        }
    }
    out
}
