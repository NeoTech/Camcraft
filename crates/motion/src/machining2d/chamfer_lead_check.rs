//! Chamfer lead's exact cone/body adapter to the existing solid verifier.
//!
//! Provenance: this module only adapts lead moves and the Chamfer seam
//! allowance to `verify::solid_hits` / `stack_hits`; the assembly sweep,
//! cone profile and mesh law stay in `camcraft-sim`.

use crate::cl::{MoveKind, PathBuilder};
use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};
use crate::verify::{Cutter, SolidLaw, Warning, solid_hits, stack_hits};

use super::arcs::{Seg, push_segs};
use super::chamfer::ChamferParams;
use super::contour::Lead;

/// Exact verification inputs for a requested Chamfer lead. The verifier
/// sweeps the cone, flute, shank, modelled body and holder; no radius
/// approximation is introduced here.
#[derive(Clone, Copy, Debug)]
pub struct ChamferLeadSafety<'a> {
    facets: &'a [Triangle],
    cutter: &'a Cutter,
    law: SolidLaw,
}

impl<'a> ChamferLeadSafety<'a> {
    #[must_use]
    pub fn new(facets: &'a [Triangle], cutter: &'a Cutter, law: SolidLaw) -> Self {
        Self { facets, cutter, law }
    }
}

pub(super) struct LeadProbe<'a> {
    pub path: &'a [Point2],
    pub entry: Point2,
    pub segs_in: &'a [Seg],
    pub segs_out: &'a [Seg],
    pub shaped_in: Lead,
    pub shaped_out: Lead,
}

pub(super) fn full_tool_holds(
    candidate: LeadProbe<'_>,
    p: &ChamferParams,
    safety: Option<&ChamferLeadSafety<'_>>,
) -> Result<(), String> {
    if candidate.shaped_in.size() == 0.0 && candidate.shaped_out.size() == 0.0 {
        return Ok(());
    }
    let safety = safety.ok_or_else(|| {
        "the selected model and complete cutter were not supplied for full lead collision checking"
            .to_owned()
    })?;
    if candidate.shaped_in.size() > 0.0 {
        let Some(&next) = candidate.path.get(1) else {
            return Err("the lead-in has no cut transition".into());
        };
        let mut b = PathBuilder::new(Point3::new(
            candidate.entry.x,
            candidate.entry.y,
            p.clearance_z,
        ));
        b.push(
            MoveKind::Plunge,
            Point3::new(candidate.entry.x, candidate.entry.y, p.tip_z()),
            Some(p.plunge_feed),
        );
        let terminal = push_checked_lead(
            &mut b,
            candidate.segs_in,
            p.tip_z(),
            p.feed,
            p.minimum_cone_radius + p.width,
            SeamEnd::AtEnd,
        );
        let cut_move = b.moves_len();
        push_segs(&mut b, &[Seg::Line(vec![next])], p.tip_z(), p.feed);
        let mut exempt = vec![cut_move];
        if let Some(at) = terminal {
            exempt.push(at);
        }
        verify_probe(&b.finish(), safety, &exempt)?;
    }
    if candidate.shaped_out.size() > 0.0 {
        let Some(&before_last) =
            candidate.path.get(candidate.path.len().saturating_sub(2))
        else {
            return Err("the lead-out has no cut transition".into());
        };
        let Some(&last) = candidate.path.last() else {
            return Err("the lead-out has no selected endpoint".into());
        };
        let mut b = PathBuilder::new(Point3::new(before_last.x, before_last.y, p.tip_z()));
        push_segs(&mut b, &[Seg::Line(vec![last])], p.tip_z(), p.feed);
        let terminal = push_checked_lead(
            &mut b,
            candidate.segs_out,
            p.tip_z(),
            p.feed,
            p.minimum_cone_radius + p.width,
            SeamEnd::AtStart,
        );
        let mut exempt = vec![0];
        if let Some(at) = terminal {
            exempt.push(at);
        }
        verify_probe(&b.finish(), safety, &exempt)?;
    }
    Ok(())
}

fn verify_probe(
    path: &crate::cl::Toolpath,
    safety: &ChamferLeadSafety<'_>,
    exempt_gouges: &[usize],
) -> Result<(), String> {
    for warning in solid_hits(path, safety.facets, safety.cutter, &safety.law) {
        if matches!(warning, Warning::Gouge { at } if exempt_gouges.contains(&at)) {
            continue;
        }
        if matches!(warning, Warning::ShankHit { .. } | Warning::RapidInPart { .. }) {
            return Err(format!("full body collision: {warning}"));
        }
        return Err(format!("full cone collision: {warning}"));
    }
    if let Some(warning) =
        stack_hits(path, safety.facets, safety.cutter, &safety.law)
            .into_iter()
            .next()
    {
        return Err(format!("full body collision: {warning}"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeamEnd {
    AtStart,
    AtEnd,
}

/// Keep the full shape, but isolate its cone-contact piece as an exact,
/// shorter line or arc move. Only that seam piece and the selected cut
/// transition may enter the authored chamfer volume.
fn push_checked_lead(
    b: &mut PathBuilder,
    segs: &[Seg],
    z: f64,
    feed: f64,
    contact: f64,
    seam: SeamEnd,
) -> Option<usize> {
    match seam {
        SeamEnd::AtStart => push_checked_lead_start(b, segs, z, feed, contact),
        SeamEnd::AtEnd => push_checked_lead_end(b, segs, z, feed, contact),
    }
}

fn push_checked_lead_start(
    b: &mut PathBuilder,
    segs: &[Seg],
    z: f64,
    feed: f64,
    contact: f64,
) -> Option<usize> {
    let (first, after) = segs.split_first()?;
    match first {
        Seg::Line(points) => {
            let (&to, after_points) = points.split_first()?;
            let from = Point2::new(b.at().x, b.at().y);
            let d = (to.x - from.x).hypot(to.y - from.y);
            let at = b.moves_len();
            if d > contact && contact > 0.0 {
                let join = Point2::new(
                    from.x + (to.x - from.x) * contact / d,
                    from.y + (to.y - from.y) * contact / d,
                );
                b.push(MoveKind::Feed, Point3::new(join.x, join.y, z), Some(feed));
                b.push(MoveKind::Feed, Point3::new(to.x, to.y, z), Some(feed));
            } else {
                b.push(MoveKind::Feed, Point3::new(to.x, to.y, z), Some(feed));
            }
            if !after_points.is_empty() {
                push_segs(b, &[Seg::Line(after_points.to_vec())], z, feed);
            }
            push_segs(b, after, z, feed);
            Some(at)
        }
        Seg::Arc { center, ccw, to } => {
            let from = Point2::new(b.at().x, b.at().y);
            let radius = (from.x - center.x).hypot(from.y - center.y);
            let a0 = (from.y - center.y).atan2(from.x - center.x);
            let a1 = (to.y - center.y).atan2(to.x - center.x);
            let sweep = if *ccw {
                (a1 - a0).rem_euclid(std::f64::consts::TAU)
            } else {
                (a0 - a1).rem_euclid(std::f64::consts::TAU)
            };
            let length = radius * sweep;
            let at = b.moves_len();
            if length > contact && radius > 0.0 && contact > 0.0 {
                let direction = if *ccw { 1.0 } else { -1.0 };
                let join_angle = a0 + direction * contact / radius;
                let join = Point2::new(
                    center.x + radius * join_angle.cos(),
                    center.y + radius * join_angle.sin(),
                );
                b.push_arc(
                    MoveKind::Feed,
                    Point3::new(join.x, join.y, z),
                    Some(feed),
                    *center,
                    *ccw,
                );
            }
            b.push_arc(
                MoveKind::Feed,
                Point3::new(to.x, to.y, z),
                Some(feed),
                *center,
                *ccw,
            );
            push_segs(b, after, z, feed);
            Some(at)
        }
    }
}

fn push_checked_lead_end(
    b: &mut PathBuilder,
    segs: &[Seg],
    z: f64,
    feed: f64,
    contact: f64,
) -> Option<usize> {
    let (last, before) = segs.split_last()?;
    push_segs(b, before, z, feed);
    match last {
        Seg::Line(points) => {
            let (&to, before_points) = points.split_last()?;
            if !before_points.is_empty() {
                push_segs(b, &[Seg::Line(before_points.to_vec())], z, feed);
            }
            let from = Point2::new(b.at().x, b.at().y);
            let d = (to.x - from.x).hypot(to.y - from.y);
            if d > contact && contact > 0.0 {
                let join = Point2::new(
                    to.x - (to.x - from.x) * contact / d,
                    to.y - (to.y - from.y) * contact / d,
                );
                b.push(MoveKind::Feed, Point3::new(join.x, join.y, z), Some(feed));
            }
            let at = b.moves_len();
            b.push(MoveKind::Feed, Point3::new(to.x, to.y, z), Some(feed));
            (b.moves_len() > at).then_some(at)
        }
        Seg::Arc { center, ccw, to } => {
            let from = Point2::new(b.at().x, b.at().y);
            let radius = (from.x - center.x).hypot(from.y - center.y);
            let a0 = (from.y - center.y).atan2(from.x - center.x);
            let a1 = (to.y - center.y).atan2(to.x - center.x);
            let sweep = if *ccw {
                (a1 - a0).rem_euclid(std::f64::consts::TAU)
            } else {
                (a0 - a1).rem_euclid(std::f64::consts::TAU)
            };
            let length = radius * sweep;
            if length > contact && radius > 0.0 && contact > 0.0 {
                let direction = if *ccw { 1.0 } else { -1.0 };
                let join_angle = a1 - direction * contact / radius;
                let join = Point2::new(
                    center.x + radius * join_angle.cos(),
                    center.y + radius * join_angle.sin(),
                );
                b.push_arc(
                    MoveKind::Feed,
                    Point3::new(join.x, join.y, z),
                    Some(feed),
                    *center,
                    *ccw,
                );
            }
            let at = b.moves_len();
            b.push_arc(
                MoveKind::Feed,
                Point3::new(to.x, to.y, z),
                Some(feed),
                *center,
                *ccw,
            );
            (b.moves_len() > at).then_some(at)
        }
    }
}
