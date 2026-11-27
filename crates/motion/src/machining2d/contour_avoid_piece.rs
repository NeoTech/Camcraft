//! Helper routines for closed/clipped contour piece emission with "Do not touch" (#3344).
//! Provenance: clean-room per-level keep-out checking for contour segments, picks, and exit retracts; no vendor material consulted.

use camcraft_sim::verify::AvoidSpec;

use crate::cl::{MoveKind, PathBuilder};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;
use crate::links::LinkTally;

use super::arcs::{Seg, push_segs};
use super::contour::{ContourParams, PreparedBand};
use super::contour_avoid_clip::{Clip, clip_polyline, safe_walk, segs_clear_of, segs_cross_prior};
use super::contour_entry::{Engage, crosses, flatten, seam_leads};
use super::contour_link::{guard, ring_link};
use super::contour_step::{Said, Step, engage_level};
use super::fillet::{Legal, fillet_corners};
use super::fillet_tally::FilletTally;
use super::lead_ext::extend_leads;
use super::levels::{Cert, Motion};
use super::ring::{Ring, dist};
use super::face::GenError;

/// Mutable cross-band emission state shared by single-band and whole-model avoid contours.
pub(super) struct AvoidSink {
    pub said: Said,
    pub lead_notes: Vec<String>,
    pub tally: FilletTally,
    pub first_entry: bool,
    pub lifted: LinkTally,
    pub clipped_any: bool,
    pub blocked_any: bool,
}

impl Default for AvoidSink {
    fn default() -> Self {
        Self {
            said: Said::default(),
            lead_notes: Vec::new(),
            tally: FilletTally::default(),
            first_entry: true,
            lifted: LinkTally::default(),
            clipped_any: false,
            blocked_any: false,
        }
    }
}

fn poly_closest(pts: &[Point2], q: Point2) -> Point2 {
    let mut best = (f64::INFINITY, pts.first().copied().unwrap_or(q));
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let ll = dx * dx + dy * dy;
        let t = if ll > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / ll).clamp(0.0, 1.0) } else { 0.0 };
        let f = Point2::new(a.x + dx * t, a.y + dy * t);
        let d2 = (f.x - q.x).powi(2) + (f.y - q.y).powi(2);
        if d2 < best.0 {
            best = (d2, f);
        }
    }
    best.1
}

pub(super) fn check_pick_clear(
    picked: Option<Point2>,
    exit_q: Option<Point2>,
    pts: &[Point2],
    ko_cut: &Region,
    ko_entry: &Region,
    avoid: &AvoidSpec,
) -> Result<(), GenError> {
    if let Some(q) = picked {
        let f = poly_closest(pts, q);
        if ko_cut.contains(f) || ko_entry.contains(f) {
            return Err(GenError::Invalid(format!(
                "start point ({:.3}, {:.3}) is blocked by protected 'Do not touch' {}",
                q.x, q.y, avoid.names_list()
            )));
        }
    }
    if let Some(q) = exit_q {
        let f = poly_closest(pts, q);
        if ko_cut.contains(f) || ko_entry.contains(f) {
            return Err(GenError::Invalid(format!(
                "exit point ({:.3}, {:.3}) is blocked by protected 'Do not touch' {}",
                q.x, q.y, avoid.names_list()
            )));
        }
    }
    Ok(())
}

/// Rotate an unclipped closed ring if `Lead::None` landed its start inside `ko_entry`.
pub(super) fn rotate_clear_start(pts: Vec<Point2>, ko_entry: &Region) -> Option<Vec<Point2>> {
    if pts.len() < 3 || !ko_entry.contains(pts[0]) {
        return Some(pts);
    }
    let open = if dist(pts[0], pts[pts.len() - 1]) <= 1e-9 { &pts[..pts.len() - 1] } else { &pts[..] };
    let m = open.len();
    for i in 0..m {
        let a = open[i];
        let b = open[(i + 1) % m];
        let mid = Point2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
        if !ko_entry.contains(mid) {
            let mut v = Vec::with_capacity(m + 2);
            v.push(mid);
            for k in 1..=m {
                v.push(open[(i + k) % m]);
            }
            v.push(mid);
            return Some(v);
        }
    }
    None
}

/// One planned window of a ring: the polyline, its walk, the lead-in
/// start, the lead-in and the lead-out.
pub(super) type Window = (Vec<Point2>, Vec<Seg>, Point2, Vec<Seg>, Vec<Seg>);

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_window_avoid(
    ring: &Ring,
    ke: (f64, f64),
    p: &ContourParams,
    legal_cut: &Legal,
    legal_entry: &Legal,
    ko_cut: &Region,
    ko_entry: &Region,
    prior_z: &[Vec<Point2>],
    sink: &mut AvoidSink,
) -> Option<Window> {
    let poly = ring.arc_between(ke.0, ke.1);
    if poly.len() < 2 {
        return None;
    }
    let mut walk = safe_walk(&poly, p, legal_cut, ko_cut);
    if p.blend_turns {
        let (filleted, t) = fillet_corners(&walk, poly[0], p.tool_diameter * 0.5, p.tolerance, legal_cut);
        if segs_clear_of(poly[0], &filleted, ko_cut) {
            walk = filleted;
            sink.tally.add(t);
        }
    }
    let (mut entry, mut lead, mut lead_off) = seam_leads(&walk, &poly, p, legal_entry, &mut sink.lead_notes);
    let ext = extend_leads(entry, lead, lead_off, &walk, &poly, p, legal_entry, &mut sink.lead_notes);
    entry = ext.0;
    lead = ext.1;
    lead_off = ext.2;
    if !segs_clear_of(entry, &lead, ko_cut) || ko_entry.contains(entry) || segs_cross_prior(entry, &lead, prior_z) {
        lead.clear();
        entry = poly[0];
    }
    let end = poly[poly.len() - 1];
    let exit_end = flatten(end, &lead_off).last().copied().unwrap_or(end);
    if !segs_clear_of(end, &lead_off, ko_cut) || ko_entry.contains(exit_end) || segs_cross_prior(end, &lead_off, prior_z) {
        lead_off.clear();
    }
    Some((poly, walk, entry, lead, lead_off))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_open_piece(
    b: &mut PathBuilder,
    piece: &[Point2],
    z: f64,
    hop_z: f64,
    inside: bool,
    tabbed: bool,
    p: &ContourParams,
    band: &PreparedBand,
    legal_cut: &Legal,
    legal_entry: &Legal,
    ko_cut: &Region,
    ko_entry: &Region,
    center_avoid: &Region,
    cert_avoid: &Cert<'_>,
    m: &Motion,
    prior_z: &mut Vec<Vec<Point2>>,
    dives: &mut (f64, Vec<Vec<Point2>>),
    last_leads: &mut Vec<Vec<Point2>>,
    sink: &mut AvoidSink,
) -> Result<(), GenError> {
    let sub_pieces = if ko_entry.contains(piece[0]) || ko_entry.contains(piece[piece.len() - 1]) {
        match clip_polyline(piece, ko_entry, false) {
            Clip::Unclipped => vec![piece.to_vec()],
            Clip::AllBlocked => {
                sink.clipped_any = true;
                return Ok(());
            }
            Clip::Pieces(ps) => {
                sink.clipped_any = true;
                ps
            }
        }
    } else {
        vec![piece.to_vec()]
    };

    for sub in sub_pieces {
        let mut walk = safe_walk(&sub, p, legal_cut, ko_cut);
        if p.blend_turns {
            let (filleted, t) = fillet_corners(&walk, sub[0], p.tool_diameter * 0.5, p.tolerance, legal_cut);
            if segs_clear_of(sub[0], &filleted, ko_cut) {
                walk = filleted;
                sink.tally.add(t);
            }
        }
        let (mut entry, mut lead, mut lead_off) = seam_leads(&walk, &sub, p, legal_entry, &mut sink.lead_notes);
        let ext = extend_leads(entry, lead, lead_off, &walk, &sub, p, legal_entry, &mut sink.lead_notes);
        entry = ext.0;
        lead = ext.1;
        lead_off = ext.2;
        if !segs_clear_of(entry, &lead, ko_cut) || ko_entry.contains(entry) || segs_cross_prior(entry, &lead, prior_z) {
            lead.clear();
            entry = sub[0];
        }
        if ko_entry.contains(entry) || ko_cut.contains(entry) {
            sink.clipped_any = true;
            continue;
        }
        let end = sub[sub.len() - 1];
        let exit_end = flatten(end, &lead_off).last().copied().unwrap_or(end);
        if !segs_clear_of(end, &lead_off, ko_cut)
            || ko_entry.contains(exit_end)
            || segs_cross_prior(end, &lead_off, prior_z)
            || crosses(entry, &lead, end, &lead_off)
        {
            lead_off.clear();
        }
        let e2 = Point2::new(entry.x, entry.y);
        let here = vec![flatten(e2, &lead), flatten(end, &lead_off)];
        let link_leads: Vec<Vec<Point2>> = std::mem::replace(last_leads, here.clone()).into_iter().chain(here).collect();
        let ap = &p.links.approach;
        let lf = ap.lead_feed(p.feed);
        let lead_top = if ap.lead_height > 0.0 && !lead.is_empty() { z + ap.lead_height } else { z };
        let arm = if sink.first_entry { p.entry } else { p.links.arm(p.entry) };

        if !sink.first_entry {
            let cut_dives = if dives.0.to_bits() == z.to_bits() { dives.1.as_slice() } else { &[] };
            ring_link(b, p, legal_cut, &[&band.plan, cut_dives, &link_leads, prior_z], e2, z, hop_z, tabbed, &mut sink.lifted, false);
            if b.at().z > lead_top + 1e-9 {
                guard(b, p, e2, lead_top, p.entry_top(), "the dive onto clipped contour");
                b.push(MoveKind::Plunge, Point3::new(e2.x, e2.y, lead_top), Some(p.plunge_feed));
            }
        } else {
            // The bore side's entry room is the centre region less the
            // keep-out, never the whole centre: an entry point outside that
            // room grazes at itself, as `emit_band_avoid` does for a ring.
            let eng = Engage { p, center: center_avoid, cert: cert_avoid, plan: &band.plan, m, legal: legal_entry };
            let step = Step {
                arm,
                e2,
                z,
                prev_z: p.clearance_z,
                first: true,
                opened: false,
                inside: inside && center_avoid.contains(e2),
                tabbed,
                lead_top,
                // A piece's first entry is the path's first engagement, from clearance.
                uncut: None,
                traverse: None,
            };
            if let Some(d) = engage_level(b, &eng, &step, &lead, &walk, &mut sink.said) {
                if dives.0.to_bits() != z.to_bits() {
                    *dives = (z, Vec::new());
                }
                dives.1.push(vec![d, e2]);
            }
        }
        sink.first_entry = false;
        let at = b.at();
        if at.x == e2.x && at.y == e2.y && at.z > z {
            super::lead_ext::push_lead_down(b, &lead, at.z, z, lf);
        } else {
            push_segs(b, &lead, z, lf);
        }
        push_segs(b, &walk, z, p.feed);
        push_segs(b, &lead_off, z, p.feed);
        prior_z.push(flatten(e2, &lead));
        prior_z.push(sub);
        prior_z.push(flatten(end, &lead_off));
    }
    Ok(())
}
