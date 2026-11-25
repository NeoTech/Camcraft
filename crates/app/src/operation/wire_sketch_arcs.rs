//! A selected DXF wire chain cut on its own lines and arcs (#3937): any
//! chain that is not an exact polygon - one with an arc, a circle or a
//! joint gap under the stated tolerance - goes to the motion crate's
//! analytic wire contour, and the verifier reads its product on a
//! waste-side ring. TEXT, MTEXT and DIMENSION entities of the drawing no
//! longer block the cut; they are counted and said on the path.
//!
//! Provenance: our own glue over `wire_edm::wire_contour_arcs` (whose
//! module head cites the public sources) and the selected-chain door
//! (#3180).

use crate::cl::{Move, Toolpath};
use crate::geo2d::{Loop, Point2};
use crate::machining2d::GenError;
use crate::sketch_data::SketchData;
use crate::wire_edm::{wire_contour_arcs, ArcChain};

use super::wire_op::{wire_params, WireCondition};
use super::wire_sketch::{single_pass, SelectedWireChain};
use super::{ResolvedHeights, WireContourOp, WireSide};

/// The skipped DXF entities that are annotation, never geometry.
const ANNOTATIONS: [&str; 3] = ["TEXT", "MTEXT", "DIMENSION"];

/// The count of annotation entities the import skipped. Any other
/// skipped entity (a HATCH, say) may be geometry whose clearance from the
/// wire cannot be checked: refused by name.
pub(super) fn annotations(data: &SketchData) -> Result<usize, String> {
    let mut ignored: usize = 0;
    for (kind, n) in &data.summary().skipped {
        if !ANNOTATIONS.contains(&kind.as_str()) {
            return Err(format!(
                "the DXF import skipped {n} {kind} entit{} whose clearance from the wire cannot be checked",
                if *n == 1 { "y" } else { "ies" }
            ));
        }
        ignored = ignored.saturating_add(*n);
    }
    if ignored != data.skipped_total() {
        return Err("the DXF import skipped entities whose clearance from the wire cannot be checked".into());
    }
    Ok(ignored)
}

/// The points that bound move `m` in XY: its ends, and for an XY arc each
/// axis extreme of its circle that the arc passes.
pub(super) fn extremes(m: &Move) -> Vec<Point2> {
    let (a, b) = (Point2::new(m.from.x, m.from.y), Point2::new(m.to.x, m.to.y));
    let mut out = vec![a, b];
    let Some(arc) = m.arc.filter(|a| a.plane.is_xy()) else { return out };
    let c = arc.center;
    let r = (a.x - c.x).hypot(a.y - c.y).max((b.x - c.x).hypot(b.y - c.y));
    let turn = |q: Point2| {
        let t = (q.y - c.y).atan2(q.x - c.x) - (a.y - c.y).atan2(a.x - c.x);
        (if arc.ccw { t } else { -t }).rem_euclid(std::f64::consts::TAU)
    };
    let sweep = turn(b);
    let sweep = if sweep > 0.0 { sweep } else { std::f64::consts::TAU };
    for (dx, dy) in [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)] {
        let q = Point2::new(c.x + r * dx, c.y + r * dy);
        if turn(q) <= sweep {
            out.push(q);
        }
    }
    out
}

impl SelectedWireChain {
    /// The chain as lines and arcs in world XY.
    fn arc_chain(&self) -> Result<ArcChain, GenError> {
        ArcChain::from_closed(&self.chain, |p| self.world_xy(p))
    }

    /// The product's section for the wire laws: the exact polygon, or the
    /// analytic chain on a waste-side ring within the laws' arc sagitta.
    pub(super) fn verify_ring(&self, w: &WireContourOp) -> Result<Loop, String> {
        if self.exact_polygon() {
            return self.nominal_ring(w);
        }
        single_pass(w)?;
        self.arc_chain()
            .and_then(|c| c.waste_ring(w.side, crate::verify::wire::ARC_SAG))
            .map_err(|e| e.to_string())
    }

    /// Cut a chain that is not an exact polygon on its own lines and arcs.
    /// `None` for an exact polygon, which keeps the polygon path.
    pub(super) fn generate_arcs(
        &self,
        op: &WireContourOp,
        cond: &WireCondition,
        hs: &ResolvedHeights,
    ) -> Result<Option<Toolpath>, GenError> {
        if self.exact_polygon() {
            return Ok(None);
        }
        single_pass(op).map_err(GenError::Invalid)?;
        // #3332: the exit pick is read on a polygon profile or an open
        // run; on drawn lines and arcs it is refused by name, never dropped.
        if op.picks.first(crate::path_picks::PickRole::Exit).is_some() {
            return Err(GenError::Invalid(
                "wire contour: an exit point is stated on a drawn profile with arcs - the exit point is taken on a polygon profile or an open run so far; clear the exit point".into(),
            ));
        }
        if !cond.skims.is_empty() || !matches!(op.side, WireSide::Die | WireSide::Punch) {
            return Err(GenError::Invalid(
                "selected DXF wire cuts require a single vertical pass with slug capture and no tabs".into(),
            ));
        }
        let chain = self.arc_chain()?;
        let p = wire_params(op, cond, None, hs, None)?;
        wire_contour_arcs(&chain, &p).map(Some)
    }

    /// Say on the path how many annotation entities were not cut.
    pub(super) fn annotate(&self, path: &mut Toolpath) {
        if self.ignored > 0 {
            path.notes.push(format!(
                "wire EDM: {} TEXT, MTEXT or DIMENSION entit{} of the drawing ignored - annotation is not cut, and the wire is not checked against it",
                self.ignored,
                if self.ignored == 1 { "y" } else { "ies" }
            ));
        }
    }
}
