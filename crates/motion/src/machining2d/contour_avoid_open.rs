//! Open-chain and open-run 2D Contour planning with "Do not touch" protection (#3344).
//! Provenance: clean-room per-Z-level open polyline clipping, lead shrinking, and keep-out checks over `contour_chain` / `contour_run_side`; no vendor material consulted.

use camcraft_sim::verify::{AVOID_PAD, AvoidSpec};

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::entry::Entry3D;
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;
use crate::links::Link;
use crate::plunge_law;

use super::arcs::{Seg, fit_arcs_sided, lead_in, lead_out, push_segs, walk_tangent};
use super::contour::{ContourParams, Side, air_side};
use super::contour_avoid_clip::{Clip, clip_polyline, finalize_avoid, segs_clear_of, segs_cross_prior};
use super::contour_chain::contour_along_chain;
use super::contour_entry::{flatten, say};
use super::contour_run_side::{SectionAt, contour_open_run, open_run_ride};
use super::contour_run_stock::{contour_open_run_sided, sided_run_ride};
use super::face::GenError;

const LEAD_SHRINKS: usize = 4;

fn scaled_lead(lead: super::contour_params::Lead, f: f64) -> super::contour_params::Lead {
    use super::contour_params::Lead;
    match lead {
        Lead::None => Lead::None,
        Lead::Linear { length } => Lead::Linear { length: length * f },
        Lead::Arc { radius } => Lead::Arc { radius: radius * f },
        Lead::Tangential { radius } => Lead::Tangential { radius: radius * f },
        Lead::Perpendicular { length } => Lead::Perpendicular { length: length * f },
    }
}

fn open_piece_leads(
    walk: &[Seg],
    poly: &[Point2],
    p: &ContourParams,
    ko_cut: &Region,
    ko_entry: &Region,
    prior: &[Vec<Point2>],
    notes: &mut Vec<String>,
) -> (Point2, Vec<Seg>, Vec<Seg>) {
    let (Some(&s), Some(&e)) = (poly.first(), poly.last()) else {
        return (Point2::new(0.0, 0.0), Vec::new(), Vec::new());
    };
    let t0 = walk_tangent(walk, poly, true);
    let t1 = walk_tangent(walk, poly, false);
    let a = &p.links.approach;
    let out_shape = p.lead_out_shape();

    let (mut entry, mut lead) = (s, Vec::new());
    for k in 0..=LEAD_SHRINKS {
        let f = 0.5f64.powi(k as i32);
        let (cand_e, cand_segs) = lead_in(s, t0, air_side(t0, p.climb), scaled_lead(p.lead, f));
        if a.ext_in > 0.0 && a.ext_in.is_finite() {
            let dir = super::contour_entry::flatten(cand_e, &cand_segs);
            let u = dir
                .windows(2)
                .next()
                .map(|w| super::contour::unit(w[1].x - w[0].x, w[1].y - w[0].y))
                .unwrap_or(t0);
            let mut ext_ok = false;
            for ek in 0..=LEAD_SHRINKS {
                let len = a.ext_in * 0.5f64.powi(ek as i32);
                let q = Point2::new(cand_e.x - u.x * len, cand_e.y - u.y * len);
                let mut with = vec![Seg::Line(vec![cand_e])];
                with.extend(cand_segs.iter().cloned());
                if segs_clear_of(q, &with, ko_cut)
                    && !ko_entry.contains(q)
                    && !segs_cross_prior(q, &with, prior)
                {
                    entry = q;
                    lead = with;
                    ext_ok = true;
                    break;
                }
            }
            if ext_ok {
                break;
            }
        }
        if segs_clear_of(cand_e, &cand_segs, ko_cut)
            && !ko_entry.contains(cand_e)
            && !segs_cross_prior(cand_e, &cand_segs, prior)
        {
            entry = cand_e;
            lead = cand_segs;
            break;
        }
        if k == LEAD_SHRINKS && p.lead != super::contour_params::Lead::None {
            say(notes, "lead: open-contour lead-in enters protected 'Do not touch' zone - dropped".into());
        }
    }

    let mut lead_off = Vec::new();
    for k in 0..=LEAD_SHRINKS {
        let f = 0.5f64.powi(k as i32);
        let cand_segs = lead_out(e, t1, air_side(t1, p.climb), scaled_lead(out_shape, f));
        if a.ext_out > 0.0 && a.ext_out.is_finite() {
            let flat = flatten(e, &cand_segs);
            let (end_pt, u) = flat
                .windows(2)
                .last()
                .map(|w| (w[1], super::contour::unit(w[1].x - w[0].x, w[1].y - w[0].y)))
                .unwrap_or((e, t1));
            for ek in 0..=LEAD_SHRINKS {
                let len = a.ext_out * 0.5f64.powi(ek as i32);
                let q = Point2::new(end_pt.x + u.x * len, end_pt.y + u.y * len);
                let mut with = cand_segs.clone();
                with.push(Seg::Line(vec![q]));
                if segs_clear_of(e, &with, ko_cut)
                    && !segs_cross_prior(e, &with, prior)
                    && !super::contour_entry::crosses(entry, &lead, e, &with)
                {
                    lead_off = with;
                    break;
                }
            }
            if !lead_off.is_empty() {
                break;
            }
        }
        if segs_clear_of(e, &cand_segs, ko_cut)
            && !segs_cross_prior(e, &cand_segs, prior)
            && !super::contour_entry::crosses(entry, &lead, e, &cand_segs)
        {
            lead_off = cand_segs;
            break;
        }
        if k == LEAD_SHRINKS && out_shape != super::contour_params::Lead::None {
            say(notes, "lead: open-contour lead-out enters protected 'Do not touch' zone - dropped".into());
        }
    }

    (entry, lead, lead_off)
}

/// [`contour_along_chain`] with "Do not touch" protection across flutes, shank, and holder.
pub fn contour_along_chain_avoid(
    pts: &[Point2],
    p: &ContourParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_along_chain(pts, p);
    }
    p.validate()?;
    if pts.len() < 2 {
        return Err(GenError::Empty("a centerline run needs at least two points".into()));
    }
    if let Some(q) = p.picks.first(crate::path_picks::PickRole::Start).map(|k| k.xy) {
        return Err(GenError::Invalid(format!(
            "start point ({:.3}, {:.3}): an open chain starts at its end - a picked start is \
             taken only on closed contours yet; clear the start point",
            q.x, q.y
        )));
    }
    if let Some(q) = p.picks.first(crate::path_picks::PickRole::Exit).map(|k| k.xy) {
        return Err(GenError::Invalid(format!(
            "exit point ({:.3}, {:.3}): an open chain leaves at its end - a picked exit is \
             taken only on closed contours yet; clear the exit point",
            q.x, q.y
        )));
    }
    let poly: Vec<Point2> = if p.climb {
        pts.to_vec()
    } else {
        let mut v = pts.to_vec();
        v.reverse();
        v
    };
    let zs = p.levels()?;
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let mut notes = Vec::new();
    if p.blend_turns {
        b.note("turn blends: an open centerline run bounds no area to prove a fillet against - its corners kept sharp");
    }
    let arm = p.links.arm(p.entry);
    for (i, e) in [p.entry, arm].into_iter().enumerate() {
        if e != Entry3D::Plunge && !(i == 1 && e == p.entry) {
            let w = match e {
                Entry3D::Ramp => "ramp",
                Entry3D::Helix => "helix",
                Entry3D::Arc => "arc",
                Entry3D::Plunge => "plunge",
            };
            b.note(format!(
                "entry: an open centerline run bounds no air to certify the {w} against - dive \
                 at the run start, unchanged"
            ));
        }
    }
    let hop = p.retract_z.min(p.clearance_z);
    let lift_z = match p.links.level {
        Link::Clearance => p.clearance_z,
        Link::Retract => hop,
        Link::KeepDown => {
            if zs.len() > 1 {
                say(
                    &mut notes,
                    "link: an open centerline run bounds no area to prove a keep-down link \
                     against - lifted to the retract plane between levels"
                        .into(),
                );
            }
            hop
        }
    };
    let pad = AVOID_PAD.max(p.tolerance.min(0.05) + 0.01);
    let mut first = true;
    let mut clipped_any = false;
    let mut blocked_levels = 0usize;

    for &z in &zs {
        let ko_cut = avoid.keepout_region(z, z, pad).map_err(GenError::Invalid)?;
        let drop_z = if first { p.clearance_z } else { lift_z };
        let ko_entry = avoid.keepout_region(z, drop_z.max(z), pad).map_err(GenError::Invalid)?;
        let pieces = match clip_polyline(&poly, &ko_cut, false) {
            Clip::AllBlocked => {
                blocked_levels += 1;
                continue;
            }
            Clip::Unclipped => vec![poly.clone()],
            Clip::Pieces(ps) => {
                clipped_any = true;
                ps
            }
        };
        let mut prior_at_z: Vec<Vec<Point2>> = Vec::new();
        for piece in pieces {
            // Also trim ends out of ko_entry so vertical plunge points cannot hit overhead tools.
            let sub_pieces = match clip_polyline(&piece, &ko_entry, false) {
                Clip::Unclipped => vec![piece],
                Clip::AllBlocked => {
                    clipped_any = true;
                    continue;
                }
                Clip::Pieces(sp) => {
                    clipped_any = true;
                    sp
                }
            };
            for sub in sub_pieces {
                let raw_walk = fit_arcs_sided(&sub, p.tolerance, Some(p.climb));
                let walk = if segs_clear_of(sub[0], &raw_walk, &ko_cut) {
                    raw_walk
                } else {
                    vec![Seg::Line(sub[1..].to_vec())]
                };
                let (entry, lead, lead_off) = open_piece_leads(&walk, &sub, p, &ko_cut, &ko_entry, &prior_at_z, &mut notes);
                if ko_entry.contains(entry) || ko_cut.contains(entry) {
                    clipped_any = true;
                    continue;
                }
                if !first {
                    let cur = b.at();
                    if cur.z < lift_z {
                        b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, lift_z), None);
                    }
                }
                let cur = b.at();
                b.push(MoveKind::Rapid, Point3::new(entry.x, entry.y, cur.z), None);
                let cc = p.links.approach.center_cutting;
                plunge_law::guard(&mut b, cc, p.tool_diameter * 0.5, entry, z, p.entry_top(), "the dive at the lead-in");
                b.push(MoveKind::Plunge, Point3::new(entry.x, entry.y, z), Some(p.plunge_feed));
                push_segs(&mut b, &lead, z, p.feed);
                push_segs(&mut b, &walk, z, p.feed);
                push_segs(&mut b, &lead_off, z, p.feed);
                let end = sub[sub.len() - 1];
                prior_at_z.push(flatten(entry, &lead));
                prior_at_z.push(sub);
                prior_at_z.push(flatten(end, &lead_off));
                first = false;
            }
        }
    }
    let cur = b.at();
    b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);
    if blocked_levels > 0 || clipped_any {
        b.note(format!(
            "Do not touch ({}): skipped or trimmed blocked open-contour portion(s)",
            avoid.names_list()
        ));
    }
    for n in notes {
        b.note(n);
    }
    finalize_avoid(b.finish(), avoid)
}

/// [`contour_open_run`] with "Do not touch" protection.
pub fn contour_open_run_avoid(
    pts: &[Point2],
    p: &ContourParams,
    section: &SectionAt,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_open_run(pts, p, section);
    }
    let (ride, note) = open_run_ride(pts, p, section)?;
    let mut path = contour_along_chain_avoid(&ride, p, avoid)?;
    if !path.notes.contains(&note) {
        path.notes.push(note);
    }
    Ok(path)
}

/// [`contour_open_run_sided`] with "Do not touch" protection.
pub fn contour_open_run_sided_avoid(
    pts: &[Point2],
    p: &ContourParams,
    side: Side,
    stock: &SectionAt,
    part: &SectionAt,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_open_run_sided(pts, p, side, stock, part);
    }
    let (ride, note) = sided_run_ride(pts, p, side, stock, part)?;
    let mut path = contour_along_chain_avoid(&ride, p, avoid)?;
    if !path.notes.contains(&note) {
        path.notes.push(note);
    }
    Ok(path)
}
