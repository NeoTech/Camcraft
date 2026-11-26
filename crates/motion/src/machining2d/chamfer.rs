//! 2D chamfer: a cone-ruled ride along the picked edge (M2.4 B).
//!
//! Provenance: the chamfer mill's cut law is machine-tool handbook
//! geometry - a conical cutter whose tangent generatrix lies ON the
//! chamfer face. The face joins the point `width` back from the wall
//! (in the top plane) to the point `width * tan(angle)` down the wall;
//! a cone of half-angle from the axis `90 - angle` degrees, riding the
//! wall with its virtual tip at the corner, has exactly that surface.
//! The included angle across such a tool's cone is `180 - 2 * angle`
//! (a 90-degree chamfer mill for a 45-degree chamfer, the shop's own
//! catalog law). Written clean-room from that geometry, no vendor
//! material; the CL convention (moves carry TIP positions) is this
//! codebase's own, so the posted Z is the corner depth.
//! A positive tip offset (`minimum_cone_radius`) moves the cone axis
//! toward air and lowers the virtual tip so the selected band remains
//! tangent to that same face.
//!
//! The path is one closed circuit per picked loop at one constant Z -
//! `top_z - (minimum_cone_radius + width) * tan(angle_deg)`, the virtual
//! tip the moves carry. The authored edge-break depth remains
//! `width * tan(angle_deg)`. The zero-radius ride keeps its old path.
//! The congruence doors (the tool's cone, its reach) stand in the
//! app's dispatch where the Tool is visible; here the law is stated
//! on the face alone. Prime Directive: refused before cut, never
//! approximated.

use crate::cl::{EdgeBreak, MoveKind, PathBuilder, Toolpath};
use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::Point3;

use super::chamfer_lead::plan_walks;
use super::chamfer_lead_check::ChamferLeadSafety;
use super::face::GenError;
use super::contour::Lead;
use super::levels::{Motion, run_level_walks, run_levels};

/// One chamfer ride's numbers. `top_z` is the plane the chamfer's wide
/// end opens in (the picked edge's top); both the authored break depth
/// and virtual cone-tip depth follow from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChamferParams {
    /// Horizontal extent of the chamfer face in the top plane.
    pub width: f64,
    /// The face's angle from that plane, degrees in (0, 90).
    pub angle_deg: f64,
    /// Inner radius of the selected cone band, measured from its axis.
    pub minimum_cone_radius: f64,
    /// `None` is the pre-v83 compatibility marker: closed paths keep
    /// no added lead and open paths keep their fixed width air extension.
    /// `Some(Lead::None)` is a newly explicit no-added-XY choice.
    pub lead: Option<Lead>,
    /// `None` mirrors the lead-in. `Some(Lead::None)` turns only the
    /// lead-out off.
    pub lead_out: Option<Lead>,
    pub clearance_z: f64,
    /// Hop plane; validated at or above `top_z`, never the corner -
    /// between the corner and the top the wall still stands.
    pub retract_z: Option<f64>,
    pub top_z: f64,
    pub feed: f64,
    pub plunge_feed: f64,
    /// Climb milling under M3 (#1555): the material stays on the RIGHT
    /// of travel - a boss edge rides clockwise, a bore edge
    /// counter-clockwise; conventional reverses each loop.
    pub climb: bool,
    /// The picked region is a bore's air (Side Inside), not the part's
    /// material: which side of each boundary the operation selected.
    /// False is the generator's Outside law for both Auto and Outside.
    pub inside: bool,
}

impl ChamferParams {
    pub(super) fn validate(&self) -> Result<(), GenError> {
        let mut finite = [
            self.width,
            self.angle_deg,
            self.clearance_z,
            self.top_z,
            self.feed,
            self.plunge_feed,
        ]
        .into_iter()
        .all(f64::is_finite);
        finite &= self.retract_z.is_none_or(f64::is_finite);
        if !finite {
            return Err(GenError::Invalid("parameters must be finite".into()));
        }
        if self.width <= 0.0 {
            return Err(GenError::Invalid("chamfer width must be positive".into()));
        }
        if !self.minimum_cone_radius.is_finite() {
            return Err(GenError::Invalid(
                "tip offset must be finite".into(),
            ));
        }
        if self.minimum_cone_radius < 0.0 {
            return Err(GenError::Invalid(
                "tip offset must be non-negative".into(),
            ));
        }
        if self.lead.is_none() && self.lead_out.is_some() {
            return Err(GenError::Invalid(
                "a legacy Chamfer lead-in cannot take an explicit lead-out; choose its lead-in first"
                    .into(),
            ));
        }
        if self.lead.is_some_and(|lead| !lead.is_sane())
            || self.lead_out.is_some_and(|lead| !lead.is_sane())
        {
            return Err(GenError::Invalid("bad Chamfer lead parameters".into()));
        }
        if !(0.0..90.0).contains(&self.angle_deg) {
            return Err(GenError::Invalid(
                "chamfer angle must be between 0 and 90 degrees".into(),
            ));
        }
        if self.feed <= 0.0 || self.plunge_feed <= 0.0 {
            return Err(GenError::Invalid("feeds must be positive".into()));
        }
        if self.clearance_z < self.top_z {
            return Err(GenError::Invalid("clearance plane below the chamfer's top plane".into()));
        }
        if self.retract_z.is_some_and(|rz| rz < self.top_z) {
            return Err(GenError::Invalid("retract plane sits inside the material".into()));
        }
        Ok(())
    }

    /// The depth of the authored edge break.
    #[must_use]
    pub fn break_depth(&self) -> f64 {
        if self.minimum_cone_radius == 0.0 {
            // Preserve the legacy subtraction and its rounding in EdgeBreak
            // metadata and the open-wall probes, not just in move positions.
            self.top_z - self.tip_z()
        } else {
            self.width * self.angle_deg.to_radians().tan()
        }
    }

    /// The lower edge of the authored chamfer, on the selected wall.
    #[must_use]
    pub fn corner_z(&self) -> f64 {
        if self.minimum_cone_radius == 0.0 {
            self.tip_z()
        } else {
            self.top_z - self.break_depth()
        }
    }

    /// The virtual cone tip that the CL move carries.
    #[must_use]
    pub fn tip_z(&self) -> f64 {
        self.top_z
            - (self.minimum_cone_radius + self.width)
                * self.angle_deg.to_radians().tan()
    }

    pub(super) fn legacy_leads(&self) -> bool {
        self.lead.is_none()
    }
}

/// Chamfer every loop of the picked region: one circuit per loop, in
/// `|area|` order for determinism (trace's ordering law), at the virtual
/// cone-tip depth. A positive radius offsets the ride on the selected
/// air side; zero takes the old path without an offset call.
pub fn chamfer(region: &crate::geo2d::Region, p: &ChamferParams) -> Result<Toolpath, GenError> {
    chamfer_impl(region, p, None)
}

/// Chamfer with exact selected-model and full tool-assembly checks for
/// every requested lead. The legacy wrapper keeps the old public API.
pub fn chamfer_with_lead_safety(
    region: &crate::geo2d::Region,
    p: &ChamferParams,
    safety: &ChamferLeadSafety<'_>,
) -> Result<Toolpath, GenError> {
    chamfer_impl(region, p, Some(safety))
}

fn chamfer_impl(
    region: &crate::geo2d::Region,
    p: &ChamferParams,
    safety: Option<&ChamferLeadSafety<'_>>,
) -> Result<Toolpath, GenError> {
    p.validate()?;
    if region.is_empty() {
        return Err(GenError::Empty("empty chamfer region".into()));
    }
    let mut loops: Vec<&Loop> = region.loops().iter().collect();
    loops.sort_by(|x, y| y.area().abs().total_cmp(&x.area().abs()));
    let z = p.tip_z();
    if !z.is_finite() {
        return Err(GenError::Invalid("virtual cone tip left the finite horizon".into()));
    }
    let mut original: Vec<Vec<Point2>> = Vec::with_capacity(loops.len());
    for l in &loops {
        // A stored loop keeps its region on the left: the material for a
        // boss edge (climb reverses it), the bore's air for an Inside
        // pick (climb keeps it) - material right of travel either way.
        let mut pass: Vec<Point2> = super::ring::walked(l.points(), p.climb == p.inside, false);
        if let Some(&start) = pass.first() {
            pass.push(start);
        }
        original.push(pass);
    }
    let passes = if p.minimum_cone_radius == 0.0 {
        original.clone()
    } else {
        let distance = if p.inside {
            -p.minimum_cone_radius
        } else {
            p.minimum_cone_radius
        };
        let offset = region.offset(distance).map_err(|e| {
            GenError::Invalid(format!(
                "the tip offset of {} mm failed: {e}",
                p.minimum_cone_radius
            ))
        })?;
        if offset.is_empty() || offset.loops().len() != loops.len() {
            return Err(GenError::Invalid(format!(
                "the tip offset of {} mm folds, disappears, or changes the selected loop count; pick a smaller offset or a gentler boundary",
                p.minimum_cone_radius
            )));
        }
        // Loop count alone misses a narrow branch (or inlet) disappearing
        // while its parent loop survives. Undo the offset with sharp joins:
        // this restores ordinary inset corners without rounding them away.
        // Every span of every original boundary must survive that round trip.
        // Two round-join flattenings (5 um each) and the micron lattice are
        // the only allowance; the chamfer width must never hide a lost span.
        let restored = offset.offset_sharp(-distance)?;
        let restored_edges: Vec<Vec<Point2>> = restored
            .loops()
            .iter()
            .map(|l| {
                let mut edge = l.points().to_vec();
                if let Some(&first) = edge.first() {
                    edge.push(first);
                }
                edge
            })
            .collect();
        let surviving = Region::sweep(&restored_edges, 0.011)?;
        for edge in &original {
            if !surviving.clip_polyline(edge, false)?.is_empty() {
                return Err(GenError::Invalid(format!(
                    "the tip offset of {} mm loses a selected boundary span; pick a smaller offset or a gentler boundary",
                    p.minimum_cone_radius
                )));
            }
        }
        let mut shifted: Vec<&Loop> = offset.loops().iter().collect();
        shifted.sort_by(|x, y| y.area().abs().total_cmp(&x.area().abs()));
        shifted
            .iter()
            .map(|l| {
                let mut pass: Vec<Point2> =
                    super::ring::walked(l.points(), p.climb == p.inside, false);
                if let Some(&start) = pass.first() {
                    pass.push(start);
                }
                pass
            })
            .collect()
    };

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    // The one level the ride owns: not a ladder - the cone's
    // generatrix meets the wall at exactly one depth.
    let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed);
    if p.legacy_leads() {
        run_levels(&mut b, &passes, &[z], &motion, None, None, None);
    } else {
        let planned = plan_walks(&passes, p, safety)?;
        run_level_walks(
            &mut b,
            &planned.passes,
            &[z],
            &motion,
            Some(&planned.runs),
            None,
            None,
        );
        for note in planned.notes {
            b.note_once(&note);
        }
    }
    let cur = b.at();
    b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);

    let mut path = b.finish();
    if !path.all_finite() {
        return Err(GenError::Invalid("path left the finite horizon".into()));
    }
    // The break this ride was asked to cut (#303): each picked loop at
    // the top plane, the face's depth and slope from the two authored
    // numbers - never read back off the moves, so the sim judges the
    // ride against the promise.
    let slope = p.angle_deg.to_radians().tan();
    path.breaks = original
        .iter()
        .map(|pass| EdgeBreak {
            edge: pass.iter().map(|q| Point3::new(q.x, q.y, p.top_z)).collect(),
            depth: p.break_depth(),
            slope,
            volume: None,
        })
        .filter(EdgeBreak::is_sane)
        .collect();
    Ok(path)
}
