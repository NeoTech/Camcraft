//! The entry of a dead-end pass of the Outside start-side plan (#3567):
//! a later pass that no lead-in reaches starts on a wall, with one
//! stepover of uncut stock in front of the tool, and enters by the ramp
//! along its own pass (the ramp fallback's zig-zag at the entry's ramp
//! angle) instead of a plunge or helix into that stock; the pass then
//! cuts the ramped stretch again at depth. A dead end too short for the
//! ramp sends the area to the Auto order, named. Provenance: the linear
//! ramp entry of public machining handbooks, on this crate's zig-zag
//! law (`links::plan_zig_capped`); no vendor material consulted.

use crate::cl::PathBuilder;
use crate::geo2d::Point2;
use crate::links::plan_zig_capped;

use super::equal_steps::cut_levels_for;
use super::descent::{Cuts, Lvl, fallback_ramp_angle, push, zig_from};
use super::ladder_links::LevelWalk;
use super::levels::{Cert, Motion};
use super::pocket::PocketParams;

/// How near (mm) a pass start must stand to a marked dead end to be it:
/// the marks are the plan's own points, so only float noise separates.
const SAME: f64 = 1e-9;

impl Motion {
    /// #3567: the dead-end starts of the start-side plan.
    pub(super) fn with_ramp_starts(mut self, starts: Vec<Point2>) -> Self {
        self.ramp_starts = starts;
        self
    }
}

fn marked(starts: &[Point2], e: Point2) -> bool {
    starts.iter().any(|q| (q.x - e.x).hypot(q.y - e.y) <= SAME)
}

/// The deepest descent (mm) a dead-end ramp takes at any cut level: from
/// the material top of that level (the level above; the entry top on the
/// first) plus the plunge height, down to the level. `None` when the
/// levels cannot be read.
fn deepest_descent(p: &PocketParams) -> Option<f64> {
    let a = &p.links.approach;
    let zs = cut_levels_for(p.top_z, a, p.bottom_z, p.stepdown).ok()?;
    let mut above = a.entry_top(p.top_z);
    let mut worst: f64 = 0.0;
    for z in zs {
        worst = worst.max(above + a.plunge_height.max(0.0) - z);
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

/// Enter the pass of `lv` by the ramp along it when its start is a
/// marked dead end below the material top; the tool stands
/// at `plane`. `None` when the start is not one (the ordinary pass link
/// posts the entry); a dead end the ramp cannot plan is named on the path
/// and left to the ordinary link.
pub(super) fn dead_end_ramp(b: &mut PathBuilder, lv: &LevelWalk, m: &Motion, cert: Option<&Cert<'_>>, plane: f64) -> Option<Cuts> {
    let &e = lv.pass.first()?;
    // The mark is the plan's proof that no cut of the level reaches the
    // start (`level_order::order`), so the column is not scanned again:
    // a scan of every earlier move at each mark costs seconds on a large
    // part, and a ramp over a column that is cut after all is harmless.
    if !marked(&m.ramp_starts, e) || lv.z >= lv.mat_top {
        return None;
    }
    let cen = cert?;
    let l = Lvl { cen, passes: lv.passes, pass: lv.pass, walk: lv.walk, mat_top: lv.mat_top };
    let from = (lv.mat_top + m.links.approach.plunge_height).min(plane);
    let Some(d) = zig_from(&l, lv.z, from, m) else {
        b.note_once(&format!(
            "entry: the dead-end pass at ({:.3}, {:.3}) is too short for a ramp along it - it takes the pass link's entry",
            e.x, e.y
        ));
        return None;
    };
    Some(push(b, &d, &l, lv.z, plane, m))
}

#[cfg(test)]
mod tests {
    use super::too_short_at;
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
        assert_eq!(too_short_at(&[long.clone()], &[s], &p), None);
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
}
