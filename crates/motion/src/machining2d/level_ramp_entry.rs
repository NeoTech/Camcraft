//! The entry of a dead-end pass of the Outside start-side plan (#3567):
//! a later pass that no lead-in reaches starts on a wall, with one
//! stepover of uncut stock in front of the tool and the part behind it,
//! and enters by the ramp along its own pass (the ramp fallback's
//! zig-zag at the entry's ramp angle). That ramp cuts a slot of nearly
//! the full tool width at the ramp angle; the pass then cuts the ramped
//! stretch again at depth, at one stepover. A dead end too short for the
//! ramp sends the area to the Auto order, named. Provenance: the linear
//! ramp entry of public machining handbooks, on this crate's zig-zag law
//! (`links::plan_zig_capped`); no vendor material consulted.

use crate::cl::PathBuilder;
use crate::entry::Entry3D;
use crate::geo2d::Point2;
use crate::links::plan_zig_capped;

use super::arcs::ARC_FIT_TOL;
use super::descent::{Cuts, Lvl, fallback_ramp_angle, push, ramp_fallback};
use super::equal_steps::cut_levels_for;
use super::ladder_links::LevelWalk;
use super::levels::{Cert, Motion};
use super::pocket::PocketParams;

/// How near (mm) a pass start must stand to a marked dead end to be it:
/// the marks are the plan's own points, so only float noise separates.
const SAME: f64 = 1e-9;

impl Motion {
    /// #3316: prove keep-down pass links through cut material too (the
    /// start-side plan's link law; moved here with the #3567 entry).
    pub(super) fn with_cleared_links(mut self, on: bool) -> Self {
        self.cleared_links = on;
        self
    }

    /// #3567: the dead-end starts of the start-side plan.
    pub(super) fn with_ramp_starts(mut self, starts: Vec<Point2>) -> Self {
        self.ramp_starts = starts;
        self
    }
}

fn marked(starts: &[Point2], e: Point2) -> bool {
    starts.iter().any(|q| (q.x - e.x).hypot(q.y - e.y) <= SAME)
}

/// The deepest descent (mm) a dead-end ramp takes at any cut level, as
/// [`dead_end_ramp`] plans it: from the material top of that level (the
/// level above; the entry top on the first, and on each level when the
/// ramp goes to depth from the top) plus the plunge height, down to the
/// level. `None` when the levels cannot be read.
fn deepest_descent(p: &PocketParams) -> Option<f64> {
    let a = &p.links.approach;
    let zs = cut_levels_for(p.top_z, a, p.bottom_z, p.stepdown).ok()?;
    let top = a.entry_top(p.top_z);
    let mut above = top;
    let mut worst: f64 = 0.0;
    for z in zs {
        let from = if a.ramp_to_depth { top } else { above };
        worst = worst.max(from + a.plunge_height.max(0.0) - z);
        above = z;
    }
    (worst.is_finite() && worst > 0.0).then_some(worst)
}

/// The first dead-end start among `passes` whose pass is too short for
/// the ramp at the entry's angle over the deepest descent of any level
/// ([`deepest_descent`]): the plan cannot enter it without a plunge, so
/// the caller falls back. Levels that cannot be read count as too short.
pub(super) fn too_short_at(passes: &[Vec<Point2>], starts: &[Point2], p: &PocketParams) -> Option<Point2> {
    let a = &p.links.approach;
    let dz = deepest_descent(p).unwrap_or(f64::NAN);
    passes.iter().filter_map(|pass| pass.first().copied().map(|e| (e, pass))).find_map(|(e, pass)| {
        (marked(starts, e) && plan_zig_capped(pass, dz, fallback_ramp_angle(a), a.ramp_max_len).is_none()).then_some(e)
    })
}

/// The note when the ramp stands in for the user's entry at a dead end
/// (the #2265 stand-in form), once per operation; `None` for a Ramp entry.
pub(super) fn stand_in_note(entry: Entry3D, angle: f64) -> Option<String> {
    let asked = match entry {
        Entry3D::Ramp => return None,
        Entry3D::Plunge => "plunge",
        Entry3D::Helix => "helix",
        Entry3D::Arc => "arc",
    };
    Some(format!(
        "entry: a pass of the Outside order starts at a dead end in uncut stock - ramp along the pass at {angle:.1} deg instead of the {asked} entry"
    ))
}

/// Enter the pass of `lv` by the ramp along it when its start is a
/// marked dead end below the material top; the tool stands at `plane`.
/// `None` when the start is not one (the ordinary pass link posts the
/// entry). The mark is the plan's proof that no cut of the level reaches
/// the start (`level_order::order`), so the cut moves are not scanned
/// again. A dead end the ramp cannot plan is named on the path and left
/// to the ordinary link; the plan's own check (`too_short_at`) sends such
/// an area to the Auto order first, so this is a backstop.
pub(super) fn dead_end_ramp(b: &mut PathBuilder, lv: &LevelWalk, m: &Motion, cert: Option<&Cert<'_>>, plane: f64) -> Option<Cuts> {
    let &e = lv.pass.first()?;
    if !marked(&m.ramp_starts, e) || lv.z >= lv.mat_top {
        return None;
    }
    let cen = cert?;
    let a = &m.links.approach;
    let from = (lv.mat_top + a.plunge_height).min(plane);
    // The ramp rides the start of the pass thinned within the arc fit's
    // budget: a level curve of a meshed part has a vertex every few
    // microns, and a ramp posted on each one is tens of thousands of moves.
    let tan = fallback_ramp_angle(a).to_radians().tan();
    let drop = (m.top_z + a.plunge_height).max(from) - lv.z;
    let need = if tan.is_finite() && tan > 0.0 { (0.5 * drop / tan).max(a.ramp_max_len) } else { f64::INFINITY };
    let lead = thinned(lead_of(lv.pass, need), ARC_FIT_TOL);
    let l = Lvl { cen, passes: lv.passes, pass: &lead, walk: lv.walk, mat_top: lv.mat_top };
    let Some(d) = ramp_fallback(&l, lv.z, from, m) else {
        b.note_once(&format!(
            "entry: the dead-end pass at ({:.3}, {:.3}) is too short for a ramp along it - it takes the pass link's entry",
            e.x, e.y
        ));
        return None;
    };
    if let Some(n) = stand_in_note(m.entry, fallback_ramp_angle(&m.links.approach)) {
        b.note_once(&n);
    }
    Some(push(b, &d, &l, lv.z, plane, m))
}

/// The start of `pts` up to the first vertex at or past `len` mm of
/// travel (all of it when shorter): the stretch a zig-zag ramp can use.
fn lead_of(pts: &[Point2], len: f64) -> &[Point2] {
    let mut run = 0.0;
    for (i, w) in pts.windows(2).enumerate() {
        run += (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
        if run >= len {
            return &pts[..i + 2];
        }
    }
    pts
}

/// `pts` thinned by Douglas-Peucker to the vertices the polyline needs
/// within `tol` mm: each kept point is a vertex of `pts`, each dropped one
/// lies within `tol` of the chord that replaces it. Iterative, so a long
/// walk cannot overflow the stack.
fn thinned(pts: &[Point2], tol: f64) -> Vec<Point2> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let mut todo = vec![(0usize, n - 1)];
    while let Some((i, j)) = todo.pop() {
        let (a, b) = (pts[i], pts[j]);
        let far = (i + 1..j).map(|k| (seg_dist(pts[k], a, b), k)).fold((0.0, i), |m, d| if d.0 > m.0 { d } else { m });
        if far.0 > tol && far.1 > i {
            keep[far.1] = true;
            todo.push((i, far.1));
            todo.push((far.1, j));
        }
    }
    pts.iter().zip(keep).filter_map(|(p, k)| k.then_some(*p)).collect()
}

fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 && dd.is_finite() { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - a.x - t * dx).hypot(p.y - a.y - t * dy)
}

#[cfg(test)]
mod tests {
    use super::{lead_of, stand_in_note, thinned, too_short_at};
    use crate::entry::Entry3D;
    use crate::geo2d::Point2;
    use crate::links::Links;
    use crate::load::DEFAULT_OPTIMAL_LOAD;
    use crate::machining2d::pass_start::{PassStart, StartSide};
    use crate::machining2d::{PocketParams, PocketStrategy, rings_fallbacks};

    fn params() -> PocketParams {
        PocketParams {
            blend_radius: None,
            tool_diameter: 8.0,
            stepover: 4.0,
            stock_to_leave: 0.0,
            clearance_z: 10.0,
            retract_z: None,
            top_z: 0.0,
            bottom_z: -3.0,
            stepdown: 5.0,
            feed: 1000.0,
            plunge_feed: 300.0,
            climb: true,
            strategy: PocketStrategy::Waterline,
            entry: Entry3D::Helix,
            helix_radius: 1.6,
            optimal_load: DEFAULT_OPTIMAL_LOAD,
            blend_turns: false,
            links: Links::PLANAR,
        }
    }

    /// #3567: a marked dead end whose pass is too short for the ramp (a
    /// 3 deg ramp down 5 mm needs 95 mm of travel, 64 out-and-back legs at
    /// most) is found, so the plan falls back, and so is a marked one of
    /// zero length; a long one and unmarked short ones are not.
    #[test]
    fn a_dead_end_too_short_for_the_ramp_is_found() {
        let p = params();
        let s = Point2::new(1.0, 2.0);
        let short = vec![s, Point2::new(1.5, 2.0)];
        let long = vec![s, Point2::new(60.0, 2.0)];
        let lone = vec![Point2::new(9.0, 9.0), Point2::new(9.0, 9.0)];
        assert_eq!(too_short_at(std::slice::from_ref(&long), &[s], &p), None);
        assert_eq!(too_short_at(&[long, short.clone()], &[s], &p), Some(s));
        assert_eq!(too_short_at(&[short, lone], &[], &p), None);
        let zero = vec![s, s];
        assert_eq!(too_short_at(&[zero], &[s], &p), Some(s), "a zero-length dead end cannot ramp");
    }

    /// #3567: the fallback for a dead end too short to ramp is an Offset
    /// rings fallback by name (the op raises `Warning::StrategyFallback`).
    #[test]
    fn the_ramp_fallback_note_is_a_rings_fallback() {
        let note = StartSide::from(PassStart::Outside).ramp_fallback_note(1.0, -2.5);
        assert_eq!(rings_fallbacks(std::slice::from_ref(&note)), 1, "{note}");
        assert!(note.starts_with("Pass start Outside: a pass at X 1.000, Y -2.500"), "{note}");
    }

    /// #3567 review item 3: a Plunge, Helix or Arc entry that the ramp
    /// replaces at a dead end is named once, in the #2265 stand-in form;
    /// a Ramp entry needs no note.
    #[test]
    fn a_replaced_entry_is_named() {
        assert_eq!(stand_in_note(Entry3D::Ramp, 3.0), None);
        let n = stand_in_note(Entry3D::Helix, 3.0).unwrap_or_default();
        assert_eq!(
            n,
            "entry: a pass of the Outside order starts at a dead end in uncut stock - ramp along the pass at 3.0 deg instead of the helix entry"
        );
        assert!(stand_in_note(Entry3D::Plunge, 2.5).is_some_and(|n| n.ends_with("at 2.5 deg instead of the plunge entry")));
    }

    /// The ramp's lead: a dense straight walk thins to its two ends, a
    /// corner stays, each kept point is a vertex of the walk, and short or
    /// empty walks pass through. The lead stops at the first vertex past
    /// the length asked; a non-finite length takes the whole walk.
    #[test]
    fn the_ramp_lead_is_thinned_within_the_fit_budget() {
        let dense: Vec<Point2> = (0..=1000).map(|k| Point2::new(f64::from(k) * 0.01, 0.0)).collect();
        assert_eq!(thinned(&dense, 0.01), vec![Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
        let corner = vec![Point2::new(0.0, 0.0), Point2::new(5.0, 0.0), Point2::new(5.0, 5.0)];
        assert_eq!(thinned(&corner, 0.01), corner);
        assert!(thinned(&[], 0.01).is_empty());
        assert_eq!(thinned(&corner[..2], 0.01), corner[..2].to_vec());
        assert_eq!(lead_of(&dense, 2.0).len(), 201);
        assert_eq!(lead_of(&dense, f64::NAN).len(), dense.len());
        assert_eq!(lead_of(&dense, f64::INFINITY).len(), dense.len());
        assert!(lead_of(&[], 1.0).is_empty());
    }
}
