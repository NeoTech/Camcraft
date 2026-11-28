//! Closed-loop and trimmed-piece band emission for 2D Contour "Do not touch" (#3344).
//! Provenance: clean-room per-level keep-out emission over `contour` helpers; no vendor material consulted.

use camcraft_sim::verify::{AVOID_PAD, AvoidSpec};

use crate::cl::PathBuilder;
use crate::entry::Entry3D;
use crate::geo2d::Point2;
use crate::links::Link;

use super::arcs::push_segs;
use super::contour::{ContourParams, Lead, PreparedBand};
use super::contour_avoid_clip::{Clip, clip_polyline, legal_regions, segs_clear_of};
pub(super) use super::contour_avoid_piece::AvoidSink;
use super::contour_avoid_piece::{check_pick_clear, plan_window_avoid, push_open_piece, rotate_clear_start};
use super::contour_comp::{self, SpanCtx};
use super::contour_depth::Ladder;
use super::contour_entry::{Engage, crosses, flatten, say, seam_leads, wall_arc, wall_arc_top};
use super::contour_link::{Stay, plan_links, push_stay, ring_entry, ring_link, tab_window_dive, wall_arc_hop};
use super::contour_peck::uncut_floor;
use super::contour_start;
use super::contour_step::{Step, engage_level};
use super::face::GenError;
use super::fillet::Legal;
use super::lead_ext::{extend_leads, retrace};
use super::levels::{Cert, Motion};
use super::ring::Ring;
use super::tab_bridge::{cached, fit_tabs};
use super::tab_clear::fit_loops;

/// Emit all levels & rings of one band while keeping every move clear of `avoid`.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_band_avoid(
    b: &mut PathBuilder,
    band: &PreparedBand,
    p: &ContourParams,
    lad: &Ladder,
    pick: Option<(usize, Point2)>,
    exit: Option<(usize, Point2)>,
    avoid: &AvoidSpec,
    sink: &mut AvoidSink,
) -> Result<(), GenError> {
    let inside = p.side == Side::Inside;
    let m = Motion::new(p.clearance_z, Some(p.retract_z), p.feed, p.plunge_feed)
        .with_entry(p.entry, p.helix_radius)
        .with_arc_radius(p.tool_diameter)
        .with_links(p.links, p.helix_radius);
    let base_legal = band.room.as_ref().unwrap_or(&band.center);
    let prog = &band.prog;
    let hop = p.retract_z.min(p.clearance_z);
    let pad = AVOID_PAD.max(p.tolerance.min(0.05) + 0.01);

    let mut dives: (f64, Vec<Vec<Point2>>) = (f64::NAN, Vec::new());
    let mut last_leads: Vec<Vec<Point2>> = Vec::new();
    let mut prior_at_z: (f64, Vec<Vec<Point2>>) = (f64::NAN, Vec::new());
    let mut unclipped_last: Vec<bool> = vec![false; band.loops.len()];

    let is_tabbed = |z: f64| p.tab_count > 0 && z < p.bottom_z + p.tab_height;
    let tab_levels: Vec<Vec<u64>> = (0..band.loops.len())
        .map(|li| lad.ring(li).iter().filter(|&&z| is_tabbed(z)).map(|z| z.to_bits()).collect())
        .collect();
    let legal_base_only = Legal { region: base_legal, inside, air_left: Some(p.climb) };
    let mut fits = fit_loops(&band.loops, &tab_levels, pick, p, &legal_base_only);
    let (sched, stay) = plan_links(p, lad, &band.plan, &mut sink.lead_notes);

    for (si, &(z, li)) in sched.iter().enumerate() {
        let l = &band.loops[li];
        let tabbed = is_tabbed(z);
        let hop_z = if tabbed { hop.max(p.bottom_z + p.tab_height) } else { hop };
        let same_level = si > 0 && sched[si - 1].0.to_bits() == z.to_bits();
        if prior_at_z.0.to_bits() != z.to_bits() {
            prior_at_z = (z, Vec::new());
        }

        let ko_cut = avoid.keepout_region(z, z, pad).map_err(GenError::Invalid)?;
        let rz = lad.ring(li);
        let rzi = rz.iter().position(|q| q.to_bits() == z.to_bits()).unwrap_or(0);
        let arc_top = wall_arc_top(p, rz, rzi, hop_z);
        let drop_hi = (if sink.first_entry || p.links.level == Link::Clearance { p.clearance_z } else { hop_z })
            .max(arc_top)
            .max(z);
        let ko_entry = avoid.keepout_region(z, drop_hi, pad).map_err(GenError::Invalid)?;
        let (cut_reg, entry_reg) = legal_regions(base_legal, inside, &ko_cut, &ko_entry)?;
        let legal_cut = Legal { region: &cut_reg, inside, air_left: Some(p.climb) };
        let legal_entry = Legal { region: &entry_reg, inside, air_left: Some(p.climb) };
        let center_avoid = if ko_entry.is_empty() { band.center.clone() } else { band.center.difference(&ko_entry)? };
        let cert_avoid = Cert::Region(&center_avoid);

        let pts: Vec<Point2> = super::ring::walked(l.points(), p.climb == inside, inside);
        let picked = pick.filter(|&(pi, _)| pi == li).map(|(_, q)| q);
        let exit_q = exit.filter(|&(ei, _)| ei == li).map(|(_, q)| q);
        check_pick_clear(picked, exit_q, &pts, &ko_cut, &ko_entry, avoid)?;

        match clip_polyline(&pts, &ko_cut, true) {
            Clip::AllBlocked => {
                if prog.is_some() {
                    return Err(contour_comp::refused_as(
                        p,
                        format!("cannot keep clear of protected 'Do not touch' {}: closed ring is blocked", avoid.names_list()),
                    )
                    .with_ghost_moves(b.moves()));
                }
                sink.blocked_any = true;
                unclipped_last[li] = false;
                continue;
            }
            Clip::Pieces(pieces) => {
                if prog.is_some() {
                    return Err(contour_comp::refused_as(
                        p,
                        format!("cannot keep clear of protected 'Do not touch' {}: partial trim breaks closed compensation ring", avoid.names_list()),
                    )
                    .with_ghost_moves(b.moves()));
                }
                sink.clipped_any = true;
                unclipped_last[li] = false;
                for piece in pieces {
                    push_open_piece(
                        b, &piece, z, hop_z, inside, tabbed, p, band, &legal_cut, &legal_entry,
                        &ko_cut, &ko_entry, &center_avoid, &cert_avoid, &m, &mut prior_at_z.1, &mut dives, &mut last_leads, sink,
                    )?;
                }
                continue;
            }
            Clip::Unclipped => {}
        }

        let seated = contour_start::seat(pts.clone(), picked, tabbed, prog.is_some(), p, &legal_entry, &mut sink.lead_notes);
        // #3453 (as `contour::emit_band`): an untabbed level of a tabbed ring
        // that shares no level starts at its first window.
        let alone = !tabbed
            && picked.is_none()
            && band.room.is_none()
            && !(0..band.loops.len()).any(|lj| lj != li && rz.iter().any(|&q| lad.walks(lj, q)));
        let seated = super::tab_clear::level_start(seated, pts, fits.get(&li).filter(|_| alone && !tab_levels[li].is_empty()));
        let seated = if picked.is_none() && !tabbed && p.lead == Lead::None && ko_entry.contains(seated[0]) {
            match rotate_clear_start(seated.clone(), &ko_entry) {
                Some(r) => r,
                None => {
                    sink.clipped_any = true;
                    unclipped_last[li] = false;
                    push_open_piece(
                        b, &seated, z, hop_z, inside, tabbed, p, band, &legal_cut, &legal_entry,
                        &ko_cut, &ko_entry, &center_avoid, &cert_avoid, &m, &mut prior_at_z.1, &mut dives, &mut last_leads, sink,
                    )?;
                    continue;
                }
            }
        } else {
            seated
        };

        let ap = &p.links.approach;
        let ring = Ring::new(seated);
        let ov = if tabbed || ap.overlap <= 0.0 {
            0.0
        } else if p.links.level == Link::KeepDown {
            say(&mut sink.lead_notes, "lead: the overlap stands down under the keep-down level link - the next level descends from the seam".into());
            0.0
        } else {
            ap.overlap.min(ring.len)
        };
        let keeps = if tabbed {
            cached(&mut fits, li, || fit_tabs(&ring, p, picked.is_some(), &legal_entry), &mut sink.lead_notes)
        } else {
            vec![(0.0, ring.len)]
        };

        let stay_in = if unclipped_last[li] { stay[si] } else { None };
        let stay_out = stay.get(si + 1).copied().flatten().is_some();
        unclipped_last[li] = !tabbed;

        for (k, &ke) in keeps.iter().enumerate() {
            let Some(mut window) = plan_window_avoid(&ring, ke, p, &legal_cut, &legal_entry, &ko_cut, &ko_entry, &prior_at_z.1, sink) else {
                continue;
            };
            if picked.is_some() && k == 0 {
                contour_start::lead_held(p, window.0[0], &window.3)?;
            }
            if let Some(q) = exit_q {
                window = contour_start::exit_at(window, q, p, &legal_entry, &mut sink.lead_notes)?;
            } else if ov > 0.0 {
                let (rpoly, rwalk) = retrace(&window.0, &window.1, ov);
                let (en, li_segs, lo_segs) = seam_leads(&rwalk, &rpoly, p, &legal_entry, &mut sink.lead_notes);
                let (en, li_segs, lo_segs) = extend_leads(en, li_segs, lo_segs, &rwalk, &rpoly, p, &legal_entry, &mut sink.lead_notes);
                let last = rpoly[rpoly.len() - 1];
                if !crosses(en, &li_segs, last, &lo_segs)
                    && segs_clear_of(en, &li_segs, &ko_cut)
                    && !ko_entry.contains(en)
                    && segs_clear_of(last, &lo_segs, &ko_cut)
                {
                    window = (rpoly, rwalk, en, li_segs, lo_segs);
                }
            }
            let (poly, walk, entry, mut lead, mut lead_off) = window;
            if prog.is_some() && !segs_clear_of(entry, &lead, &ko_cut) {
                return Err(contour_comp::refused_as(
                    p,
                    format!("cannot keep clear of protected 'Do not touch' {}: lead-in enters keep-out", avoid.names_list()),
                )
                .with_ghost_moves(b.moves()));
            }
            if prog.is_none() && (ko_entry.contains(entry) || !segs_clear_of(entry, &lead, &ko_cut)) {
                lead.clear();
            }
            let e2 = if lead.is_empty() { poly[0] } else { Point2::new(entry.x, entry.y) };
            if ko_entry.contains(e2) || ko_cut.contains(e2) {
                if prog.is_some() || picked.is_some() {
                    return Err(GenError::Invalid(format!(
                        "cannot cut clear of protected 'Do not touch' {}: entry ({:.3}, {:.3}) is blocked",
                        avoid.names_list(),
                        e2.x,
                        e2.y
                    ))
                    .with_ghost_moves(b.moves()));
                }
                sink.clipped_any = true;
                unclipped_last[li] = false;
                push_open_piece(
                    b, &poly, z, hop_z, inside, tabbed, p, band, &legal_cut, &legal_entry,
                    &ko_cut, &ko_entry, &center_avoid, &cert_avoid, &m, &mut prior_at_z.1, &mut dives, &mut last_leads, sink,
                )?;
                continue;
            }
            let here = vec![flatten(e2, &lead), flatten(poly.last().copied().unwrap_or(e2), &lead_off)];
            let link_leads: Vec<Vec<Point2>> = std::mem::replace(&mut last_leads, here.clone()).into_iter().chain(here).collect();
            let lf = ap.lead_feed(p.feed);
            let lead_top = if ap.lead_height > 0.0 && !lead.is_empty() { z + ap.lead_height } else { z };

            if let (Some(s), Some(&zp)) = (stay_in, rzi.checked_sub(1).and_then(|i| rz.get(i))) {
                let ko_stay = avoid.keepout_region(z, zp, pad).map_err(GenError::Invalid)?;
                let stay_ok = match s {
                    Stay::Laps(_) => segs_clear_of(poly[0], &walk, &ko_stay),
                    Stay::Plunge | Stay::Arc => !ko_stay.contains(poly[0]),
                };
                if stay_ok {
                    let (_, stay_reg) = legal_regions(base_legal, inside, &ko_stay, &ko_stay)?;
                    let legal_stay = Legal { region: &stay_reg, inside, air_left: Some(p.climb) };
                    push_stay(b, s, &poly, &walk, zp, z, p, &legal_stay, &mut sink.lead_notes);
                    sink.first_entry = false;
                    push_segs(b, &walk, z, p.feed);
                    if !stay_out {
                        push_segs(b, &lead_off, z, p.feed);
                    }
                    continue;
                }
            }

            let arm = if sink.first_entry { p.entry } else { p.links.arm(p.entry) };
            let ring_to_ring = k == 0 && same_level && !sink.first_entry;
            // #3830, #3917: a ring after another one is entered by the user's arm.
            let ring_in = ring_entry(p, ring_to_ring && !tabbed, inside, rzi == 0, arm, hop_z);
            let enter_ring = ring_in.is_some();
            let wall_arc_on = !inside && arm == Entry3D::Arc && k == 0 && !(ring_to_ring && p.links.pass == Link::KeepDown);
            let eng = Engage { p, center: &center_avoid, cert: &cert_avoid, plan: &band.plan, m: &m, legal: &legal_entry };

            if wall_arc_on && ring_to_ring && p.links.pass == Link::Clearance {
                wall_arc_hop(b, p, hop_z);
            }
            let mut arced = false;
            if wall_arc_on && !tabbed && p.lead != Lead::None {
                let first = lead.first().or(walk.first());
                if wall_arc(b, &eng, e2, z, arc_top, hop_z, first).is_ok() {
                    arced = true;
                }
            }
            let linked = !arced && ring_to_ring && {
                let cut = if dives.0.to_bits() == z.to_bits() { dives.1.as_slice() } else { &[] };
                ring_link(b, p, &legal_cut, &[&band.plan, cut, &link_leads, &prior_at_z.1], e2, z, hop_z, tabbed, &mut sink.lifted, enter_ring)
            };
            if !arced && !linked {
                if k == 0 {
                    let prev_z = if rzi == 0 { p.clearance_z } else { rz[rzi - 1] };
                    let arm = ring_in.as_ref().map_or(arm, |r| r.arm);
                    let opened = rzi >= 2 || (rzi == 1 && arm == p.entry);
                    let use_inside = inside && center_avoid.contains(e2);
                    let step = Step {
                        arm,
                        e2,
                        z,
                        prev_z,
                        first: sink.first_entry,
                        opened,
                        inside: use_inside,
                        tabbed,
                        lead_top,
                        uncut: uncut_floor(p, rz, rzi),
                        traverse: ring_in.as_ref().map(|r| r.plane),
                    };
                    if let Some(d) = engage_level(b, &eng, &step, &lead, &walk, &mut sink.said) {
                        if dives.0.to_bits() != z.to_bits() {
                            dives = (z, Vec::new());
                        }
                        dives.1.push(vec![d, e2]);
                    }
                } else {
                    tab_window_dive(b, p, e2, hop_z, lead_top, uncut_floor(p, rz, rzi));
                }
            }
            sink.first_entry = false;
            let at = b.at();
            let lead_at = b.moves_len();
            if at.x == e2.x && at.y == e2.y && at.z > z {
                super::lead_ext::push_lead_down(b, &lead, at.z, z, lf);
            } else {
                push_segs(b, &lead, z, lf);
            }
            let walk_at = b.moves_len();
            push_segs(b, &walk, z, p.feed);
            let out_at = b.moves_len();
            if !stay_out {
                let end_pt = poly[poly.len() - 1];
                let exit_end = flatten(end_pt, &lead_off).last().copied().unwrap_or(end_pt);
                if !segs_clear_of(end_pt, &lead_off, &ko_cut) || ko_entry.contains(exit_end) {
                    lead_off.clear();
                }
                push_segs(b, &lead_off, z, p.feed);
            }
            if let Some(prog) = prog {
                if walk_at == lead_at || b.moves_len() == out_at {
                    return Err(contour_comp::refused_as(
                        p,
                        format!(
                            "needs a lead-in and a lead-out clear of protected 'Do not touch' {} at ({:.3}, {:.3})",
                            avoid.names_list(),
                            e2.x,
                            e2.y
                        ),
                    )
                    .with_ghost_moves(b.moves()));
                }
                contour_comp::push_span(b, &SpanCtx { prog, p, picked }, walk_at - 1, out_at)
                    .map_err(|e| e.with_ghost_moves(b.moves()))?;
            }
            let end = poly[poly.len() - 1];
            prior_at_z.1.push(flatten(e2, &lead));
            prior_at_z.1.push(poly);
            prior_at_z.1.push(flatten(end, &lead_off));
        }
    }
    Ok(())
}

use super::contour_params::Side;
