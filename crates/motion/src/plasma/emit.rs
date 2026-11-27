//! The plasma cut's moves (#670): per contour in cut order, the torch-
//! off travel at the clearance plane, the descent to the cut plane,
//! the lit lead-in, contour and lead-out (or a small hole's centre
//! lead-in and overburn), and the torch spans that say so - then the
//! kerf law over every lit move of a closed contour.
//!
//! Provenance: see the module head (`super`): Hypertherm's direction
//! rule, the LinuxCNC Plasma Primer's small-hole rule, Tormach's
//! PathPilot G41/G42 entry rule; lead arcs are plain circle geometry.
//! Our own code.

use crate::cl::{CompSide, CompSpan, ContourRole, Move, MoveKind, PathBuilder, Toolpath, TorchSpan};
use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::Point3;
use crate::machining2d::GenError;
use crate::path_picks::{PathPicks, PickRole};

use crate::sheet::laws::{cover_law, kerf_law};
use crate::sheet::lead::{clear, cross, lead, push_piece, ride, LeadAsk, Piece, LEAD_TRIES};
use crate::sheet::nest::Contour;
use crate::sheet::ring::{along, centroid, kerf_ring, longest_mid, nearest, unit, walk_from};
use crate::sheet::{Lit, Voice};

use super::picks::{assign, length, run_on, Ask};
use super::{header, PlasmaParams, SMALL_HOLE_THICKNESS};

/// One contour, planned: where the torch pierces, what it rides lit,
/// the feed factor of the small-hole rule, and the programmed part
/// contour under control compensation.
struct Plan {
    pierce: Point2,
    lit: Vec<Piece>,
    small: Option<f64>,
    comp: Option<(CompSide, Vec<Piece>)>,
}

fn invalid(why: String) -> GenError {
    Voice::PLASMA.invalid(why)
}

/// Plan one closed contour (the kerf ring, its start, leads or the
/// small-hole rule, and the programmed contour under G41/G42). `ask` is
/// what the user's picks ask of this contour (#3334): the start lands on
/// the start pick's projection, and the cut runs on past its start to
/// the exit pick's projection. No pick keeps the automatic seam.
fn plan_closed(c: &Contour, p: &PlasmaParams, ask: Ask, forbidden: Option<&Region>, notes: &mut Vec<String>, label: &str) -> Result<Plan, GenError> {
    let r = p.kerf * 0.5;
    let ring = kerf_ring(c, r, label, Voice::PLASMA)?;
    let inner = c.role == ContourRole::Inner;
    let ccw = inner != p.reversed;
    let eq_d = 2.0 * (c.area / std::f64::consts::PI).sqrt();
    if inner && (eq_d < p.small_hole_dia || eq_d < SMALL_HOLE_THICKNESS * p.thickness) {
        // The Primer's small-hole rule: pierce near the centre, lead
        // in perpendicular, no lead-out, the overburn, the slow feed.
        let cpt = centroid(&c.pts);
        let cut = Region::new(vec![Loop::new(ring.clone())?])?;
        if !cut.contains(cpt) || !clear(forbidden, &[cpt]) {
            return Err(invalid(format!(
                "{label} is a small hole whose centre stands outside its own cut - the perpendicular lead-in from the centre has no room"
            )));
        }
        let (seg, s) = nearest(&ring, ask.start.map_or(cpt, |q| q.foot));
        if let Some(q) = ask.start {
            // The rule's lead-in is straight from the centre. To a picked
            // start it must stay inside the hole's own cut.
            let at = q.at(PickRole::Start);
            let line: Vec<Point2> =
                (0..=8).map(|i| f64::from(i) / 8.0).map(|f| Point2::new(cpt.x + (s.x - cpt.x) * f, cpt.y + (s.y - cpt.y) * f)).collect();
            if !clear(forbidden, &line) || line[..8].iter().any(|m| !cut.contains(*m)) {
                return Err(invalid(format!(
                    "{at}: the straight lead-in from the centre of {label} (a small hole) would cross the part - pick another point on the hole, or clear the start point"
                )));
            }
            notes.push(format!("plasma: {label} (small hole) is led in from its centre to the {at}"));
        }
        let walk = walk_from(&ring, seg, s, ccw);
        let mut lit = vec![Piece::Line(s)];
        lit.extend(ride(&walk));
        let tail = match ask.exit {
            None => along(&walk, p.overburn),
            Some(q) => {
                // The overburn keeps its law: the torch goes off at the
                // exit, and the exit stands at or past the overburn.
                let at = q.at(PickRole::Exit);
                let ext = run_on(&walk, nearest(&ring, q.foot).1);
                let run = length(s, &ext);
                if run + 1e-9 < p.overburn {
                    return Err(invalid(format!(
                        "{at}: the exit stands {run:.3} mm past the start of {label}, inside its {:.3} mm overburn - move the exit point farther along the cut, shorten the overburn, or clear the exit point",
                        p.overburn
                    )));
                }
                notes.push(format!(
                    "plasma: {label} (small hole) runs on {run:.3} mm past its start to the {at} - the torch goes off there, with no lead-out"
                ));
                ext
            }
        };
        lit.extend(tail.into_iter().map(Piece::Line));
        if p.control_comp {
            notes.push(format!(
                "plasma: {label} (small hole) rides the computer's kerf offset - its rule has no lead-out to carry the G40 cancel"
            ));
        }
        return Ok(Plan { pierce: cpt, lit, small: Some(p.small_hole_feed), comp: None });
    }
    let (seg, s) = ask.start.map_or_else(|| longest_mid(&ring), |q| nearest(&ring, q.foot));
    let walk = walk_from(&ring, seg, s, ccw);
    // #3334: the cut runs on past its start to the exit pick; `e` is
    // where it leaves the ring (the start itself with no exit pick).
    let ext = ask.exit.map_or_else(Vec::new, |q| run_on(&walk, nearest(&ring, q.foot).1));
    let e = ext.last().copied().unwrap_or(s);
    let before_e = if ext.len() >= 2 { ext[ext.len() - 2] } else { s };
    let t_in = unit(s, walk[1]);
    let t_out = if ext.is_empty() { unit(walk[walk.len() - 2], s) } else { unit(before_e, e) };
    // The waste side: the ring's interior is on the left of a CCW walk;
    // a hole's waste is its interior, an outline's its exterior.
    let waste = |t: Point2| {
        let left = Point2::new(-t.y, t.x);
        if inner == ccw { left } else { Point2::new(t.y, -t.x) }
    };
    let (n_in, n_out) = (waste(t_in), waste(t_out));
    let straight = p.control_comp;
    // A pick is never moved: a lead with no room at a picked point is
    // refused in the pick's name. The automatic seam keeps its words.
    let refuse = |end: &str| {
        let start = ask.start.map(|q| (q.at(PickRole::Start), "start"));
        let exit = ask.exit.map(|q| (q.at(PickRole::Exit), "exit"));
        match if end == "in" { start } else { exit.or(start) } {
            Some((at, what)) if end == "in" => invalid(format!(
                "{at}: the lead-in of {label} finds no room beside the part there - the pierce would fall on the part; pick another point on the profile, shorten the lead-in, or clear the {what} point"
            )),
            Some((at, what)) => invalid(format!(
                "{at}: the lead-out of {label} finds no room beside the part there - the torch would go off over the part; pick another point on the profile, shorten the lead-out, or clear the {what} point"
            )),
            None => invalid(format!(
                "{label}: the lead-{end} finds no room beside the part - shorten it, or cut this contour with the leads off"
            )),
        }
    };
    let tries = if straight { 1 } else { LEAD_TRIES };
    let ask_in = LeadAsk { s, t: t_in, n: n_in, len: p.lead_in, straight, entry: true };
    let ask_out = LeadAsk { s: e, t: t_out, n: n_out, len: p.lead_out, straight, entry: false };
    let (pierce, lead_in) = lead(ask_in, forbidden, tries, notes, ("plasma", label)).ok_or_else(|| refuse("in"))?;
    let (_, lead_out) = lead(ask_out, forbidden, tries, notes, ("plasma", label)).ok_or_else(|| refuse("out"))?;
    let mut lit = lead_in.clone();
    lit.extend(ride(&walk));
    lit.extend(ext.iter().copied().map(Piece::Line));
    lit.extend(lead_out.iter().copied());
    if let Some(q) = ask.start {
        notes.push(format!("plasma: {label} starts at the {} - the lead-in lands on its projection on the profile", q.at(PickRole::Start)));
    }
    if let Some(q) = ask.exit {
        notes.push(format!(
            "plasma: {label} runs on {:.3} mm past its start to the {} - the lead-out and the torch off are there",
            length(s, &ext),
            q.at(PickRole::Exit)
        ));
    }
    let comp = if p.control_comp {
        // G41/G42: the control offsets the PART contour; the engage and
        // cancel blocks are the straight leads to and from it.
        let (nseg, s_nom) = nearest(&c.pts, s);
        let nominal = walk_from(&c.pts, nseg, s_nom, ccw);
        let far = lead_out.last().map_or(s, |q| q.to());
        let mut prog = vec![Piece::Line(s_nom)];
        prog.extend(ride(&nominal));
        prog.push(Piece::Line(far));
        let left = cross(t_in, n_in) > 0.0;
        Some((if left { CompSide::Left } else { CompSide::Right }, prog))
    } else {
        None
    };
    Ok(Plan { pierce, lit, small: None, comp })
}

/// Lay the contours down in `order` and certify the result. `picks` are
/// the user's start and exit picks (#3334); empty lists are the automatic
/// seam.
pub(super) fn cut(cs: &[Contour], order: &[usize], part: &Region, p: &PlasmaParams, picks: &PathPicks) -> Result<Toolpath, GenError> {
    let r = p.kerf * 0.5;
    let asks = assign(cs, picks, p)?;
    // One tolerance for both laws (#1053): the kerf law's forbidden band
    // and the cover law's reach read the same model-aware figure, so
    // neither can fall back to the plain one alone.
    let tol = p.kerf_tol();
    let forbidden = if part.is_empty() { None } else { Some(part.offset((r - tol).max(0.0))?) };
    let mut id_of = vec![0u32; cs.len()];
    for (pos, &k) in order.iter().enumerate() {
        if let Some(slot) = id_of.get_mut(k) {
            *slot = u32::try_from(pos).map_err(|_| invalid("too many contours".into()))?;
        }
    }
    let mut notes: Vec<String> = Vec::new();
    let mut plans: Vec<(usize, Plan)> = Vec::new();
    for &k in order {
        let c = &cs[k];
        let label = format!("contour C{}", id_of[k]);
        let plan = if c.role == ContourRole::Open {
            notes.push(format!("plasma: open run C{} is cut on its centre line - no kerf side, no leads, not judged by the kerf law", id_of[k]));
            Plan { pierce: c.pts[0], lit: c.pts[1..].iter().copied().map(Piece::Line).collect(), small: None, comp: None }
        } else {
            plan_closed(c, p, asks.get(k).copied().unwrap_or_default(), forbidden.as_ref(), &mut notes, &label)?
        };
        plans.push((k, plan));
    }
    let Some((_, first)) = plans.first() else {
        return Err(GenError::Empty("plasma profile: no contour to cut".into()));
    };
    let mut b = PathBuilder::new(Point3::new(first.pierce.x, first.pierce.y, p.clearance_z));
    let mut rec = header(p);
    for (k, plan) in &plans {
        let c = &cs[*k];
        if b.at().z < p.clearance_z {
            b.push(MoveKind::Rapid, Point3::new(b.at().x, b.at().y, p.clearance_z), None);
        }
        b.push(MoveKind::Rapid, Point3::new(plan.pierce.x, plan.pierce.y, p.clearance_z), None);
        b.push(MoveKind::Rapid, Point3::new(plan.pierce.x, plan.pierce.y, p.cut_z), None);
        let on = b.moves_len();
        let feed = p.feed.map(|f| f * plan.small.unwrap_or(1.0));
        for q in &plan.lit {
            push_piece(&mut b, *q, p.cut_z, feed, None);
        }
        if b.moves_len() == on {
            // #871: part-way refusals hand back the contours cut so far.
            return Err(invalid(format!("contour C{} lays down no cut", id_of[*k])).with_ghost_moves(b.moves()));
        }
        let off = b.moves_len() - 1;
        if let Some((side, prog)) = &plan.comp {
            let mut pb = PathBuilder::new(Point3::new(plan.pierce.x, plan.pierce.y, p.cut_z));
            for q in prog {
                push_piece(&mut pb, *q, p.cut_z, feed, None);
            }
            let programmed: Vec<Move> = pb.finish().moves;
            let span = CompSpan { side: *side, radius: r, wear: false, first: on, last: off, programmed };
            if !span.is_sane(b.moves()) {
                return Err(invalid(format!(
                    "contour C{}: the G41/G42 span does not chain onto its cut - cut it with the computer's offset",
                    id_of[*k]
                ))
                .with_ghost_moves(b.moves()));
            }
            b.push_comp(span);
        }
        rec.torches.push(TorchSpan {
            contour: id_of[*k],
            role: c.role,
            parent: c.parent.and_then(|q| id_of.get(q).copied()),
            on,
            off,
            small: plan.small,
        });
    }
    b.push(MoveKind::Rapid, Point3::new(b.at().x, b.at().y, p.clearance_z), None);
    let mut path = b.finish();
    // #672: no edge may stand beyond the kerf's reach - a slot narrower
    // than the kerf that the offset bridged is refused, not left uncut.
    // #871: the laws judge the whole cut; a refusal hands it back.
    let lit: Vec<Lit> =
        rec.torches.iter().map(|t| Lit { contour: t.contour, role: t.role, on: t.on, off: t.off }).collect();
    cover_law(&path, &lit, cs, order, (p.kerf, tol), Voice::PLASMA, &|_, _| false).map_err(|e| e.with_ghost(&path))?;
    if let Some(f) = &forbidden {
        kerf_law(&path, &lit, f, Voice::PLASMA).map_err(|e| e.with_ghost(&path))?;
    } else {
        notes.push("plasma: no part silhouette to judge - roles read by nesting, the kerf law not run".into());
    }
    let holes = rec.torches.iter().filter(|t| t.role == ContourRole::Inner).count();
    let small = rec.torches.iter().filter(|t| t.small.is_some()).count();
    let mut head = vec![format!(
        "plasma: {} contour(s) cut inner first ({holes} hole(s), {small} by the small-hole rule), kerf {:.3} mm {}",
        rec.torches.len(),
        p.kerf,
        if p.control_comp { "compensated in the control (G41/G42)" } else { "offset in the computer" }
    )];
    head.push(if p.reversed {
        "plasma: cut direction REVERSED by the operation - outer contours counter-clockwise, holes clockwise (the square edge falls on the scrap side)".into()
    } else {
        "plasma: outer contours clockwise, holes counter-clockwise (the square edge on the part)".into()
    });
    head.extend(notes);
    path.notes.extend(head);
    if !rec.is_sane(&path.moves) {
        return Err(invalid("the torch spans do not sit on the moves - refused".into()).with_ghost(&path));
    }
    path.plasma = Some(rec);
    Ok(path)
}
