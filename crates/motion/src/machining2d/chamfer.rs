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
//!
//! The path is one closed circuit per picked loop at one constant Z -
//! `top_z - width * tan(angle_deg)`, the derived floor the operation
//! states instead of carrying - entered by the axial dive trace and
//! the slot post (a chamfer ride has no bounded air beside the start
//! the entry arms could certify against: the wall is the start).
//! The congruence doors (the tool's cone, its reach) stand in the
//! app's dispatch where the Tool is visible; here the law is stated
//! on the face alone. Prime Directive: refused before cut, never
//! approximated.

use crate::cl::{EdgeBreak, MoveKind, PathBuilder, Toolpath};
use crate::geo2d::{Loop, Point2};
use crate::geometry::Point3;

use super::contour::air_side;
use super::contour_run_side::{LIFT, SectionAt, Words, material_side, simple};
use super::face::GenError;
use super::levels::{Motion, run_levels};

const CHAMFER_WORDS: Words = Words {
    body: "selected model",
    both: "a chamfer on that edge would meet material on both sides; pick a boundary edge with air on one side",
};

/// One chamfer ride's numbers. `top_z` is the plane the chamfer's wide
/// end opens in (the picked edge's top); the corner depth follows from
/// it and is never an independent input - that is what makes the
/// operation's floor a law, not a second promise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChamferParams {
    /// Horizontal extent of the chamfer face in the top plane.
    pub width: f64,
    /// The face's angle from that plane, degrees in (0, 90).
    pub angle_deg: f64,
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
    /// material: which side of each stored loop the material stands on.
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

    /// The corner depth the ride posts - the derived floor, stated in
    /// one place so the panel, the door and the path cannot disagree.
    #[must_use]
    pub fn corner_z(&self) -> f64 {
        self.top_z - self.width * self.angle_deg.to_radians().tan()
    }
}

/// Chamfer every loop of the picked region: one circuit per loop, in
/// `|area|` order for determinism (trace's ordering law), at the
/// corner depth. The centers ride the loops exactly - the tip is on
/// the wall, the generatrix on the face - so no offset exists to get
/// wrong, and a picked edge that is not the chamfer's own wall is
/// caught by the stock simulation, not papered over here.
pub fn chamfer(region: &crate::geo2d::Region, p: &ChamferParams) -> Result<Toolpath, GenError> {
    p.validate()?;
    if region.is_empty() {
        return Err(GenError::Empty("empty chamfer region".into()));
    }
    let mut loops: Vec<&Loop> = region.loops().iter().collect();
    loops.sort_by(|x, y| y.area().abs().total_cmp(&x.area().abs()));
    let z = p.corner_z();
    if !z.is_finite() {
        return Err(GenError::Invalid("corner depth left the finite horizon".into()));
    }
    let mut passes: Vec<Vec<Point2>> = Vec::with_capacity(loops.len());
    for l in &loops {
        // A stored loop keeps its region on the left: the material for a
        // boss edge (climb reverses it), the bore's air for an Inside
        // pick (climb keeps it) - material right of travel either way.
        let mut pass: Vec<Point2> = super::ring::walked(l.points(), p.climb == p.inside, false);
        if let Some(&start) = pass.first() {
            pass.push(start);
        }
        passes.push(pass);
    }

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    // The one level the ride owns: not a ladder - the cone's
    // generatrix meets the wall at exactly one depth.
    let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed);
    run_levels(&mut b, &passes, &[z], &motion, None, None, None);
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
    path.breaks = passes
        .iter()
        .map(|pass| EdgeBreak {
            edge: pass.iter().map(|q| Point3::new(q.x, q.y, p.top_z)).collect(),
            depth: p.top_z - z,
            slope,
            volume: None,
        })
        .filter(EdgeBreak::is_sane)
        .collect();
    Ok(path)
}

/// Chamfer joined open edge chains, one safe approach and exit per
/// chain. Every pass stands on the selected wall only: it descends in
/// the air one chamfer width off the first end, feeds onto and through
/// every selected point, then feeds back to the air at the other end
/// before retracting. The material side is read from the model at the
/// chamfer's own heights, so climb never depends on an arbitrary edge
/// chain orientation. The `EdgeBreak` contains the selected chain alone;
/// the two air leads grant no extra credit.
pub fn chamfer_open_runs(
    runs: &[Vec<Point2>],
    p: &ChamferParams,
    section: &SectionAt<'_>,
) -> Result<Toolpath, GenError> {
    p.validate()?;
    if runs.is_empty() {
        return Err(GenError::Empty("a 2D Chamfer needs a selected edge chain".into()));
    }
    let z = p.corner_z();
    if !z.is_finite() {
        return Err(GenError::Invalid("corner depth left the finite horizon".into()));
    }
    let depth = p.top_z - z;
    let probe = LIFT.min(depth * 0.25);
    let mut passes: Vec<Vec<Point2>> = Vec::with_capacity(runs.len());
    let mut breaks: Vec<EdgeBreak> = Vec::with_capacity(runs.len());
    for pts in runs {
        let tangent = |a: Point2, b: Point2| -> Result<Point2, GenError> {
            let len = (b.x - a.x).hypot(b.y - a.y);
            if !len.is_finite() || len <= 1e-9 {
                return Err(GenError::Empty(
                    "a chamfer edge holds a collapsed segment - re-pick the edge".into(),
                ));
            }
            Ok(Point2::new((b.x - a.x) / len, (b.y - a.y) / len))
        };
        for pair in pts.windows(2) {
            tangent(pair[0], pair[1])?;
        }
        let run = simple(pts)?;
        let mut levels = Vec::new();
        for at in [z + probe, p.top_z - probe] {
            if let Some(region) = section(at)?
                && !region.is_empty()
            {
                levels.push((at, region));
            }
        }
        let material_left = material_side(&run, &levels, &CHAMFER_WORDS)?.ok_or_else(|| {
            GenError::Invalid(
                "the selected open chamfer edge has no model wall at the programmed chamfer depth - pick an edge on the part, or change Top of Part Z"
                    .into(),
            )
        })?;
        let mut pass = run.clone();
        // Material stays on the RIGHT for climb, as on every loop. The
        // model read above supplies the side that a feature chain's
        // arbitrary storage direction otherwise cannot.
        if p.climb == material_left {
            pass.reverse();
        }
        let (Some(&first), Some(&second), Some(&last), Some(&before_last)) =
            (pass.first(), pass.get(1), pass.last(), pass.get(pass.len() - 2))
        else {
            return Err(GenError::Empty("a chamfer edge needs at least two points".into()));
        };
        let entry_air = air_side(tangent(first, second)?, p.climb);
        let exit_air = air_side(tangent(before_last, last)?, p.climb);
        let mut ride = Vec::with_capacity(pass.len() + 2);
        ride.push(Point2::new(
            first.x + entry_air.x * p.width,
            first.y + entry_air.y * p.width,
        ));
        ride.extend(pass);
        ride.push(Point2::new(
            last.x + exit_air.x * p.width,
            last.y + exit_air.y * p.width,
        ));
        passes.push(ride);
        breaks.push(EdgeBreak {
            edge: run.iter().map(|q| Point3::new(q.x, q.y, p.top_z)).collect(),
            depth: p.top_z - z,
            slope: p.angle_deg.to_radians().tan(),
            volume: None,
        });
    }

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed);
    run_levels(&mut b, &passes, &[z], &motion, None, None, None);
    let cur = b.at();
    if cur.z < p.clearance_z {
        b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);
    }
    let mut path = b.finish();
    if !path.all_finite() {
        return Err(GenError::Invalid("open chamfer path left the finite horizon".into()));
    }
    path.breaks = breaks.into_iter().filter(EdgeBreak::is_sane).collect();
    Ok(path)
}
