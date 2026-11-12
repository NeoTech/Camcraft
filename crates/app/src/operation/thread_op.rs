//! The thread-milling operation's app half (#894, external threads #985): the door that judges tool and thread against each other by name, the kernel plan, and the two readings the gouge laws take of a thread mill - its crests against the thread's own far circle, its tooth roots against the part.
//!
//! Provenance: ISO 724 / ISO 965-1 thread depths and public machine-tool
//! handbook thread milling, as `camcraft_tools::thread_mill` and
//! `machining2d::thread_mill` cite them; the door and the verdict
//! readings are our own design. No vendor material consulted.
//!
//! A thread is cut INTO the wall the model shows (CAD draws a tapped
//! hole at its minor diameter), so the tool's full cylinder against the
//! part would read every thread as a gouge. The two readings split the
//! tool where the thread splits it: the tooth crests may reach the
//! major circle and no further (the tangency law, against
//! [`crest_region`]), and the body under the teeth may not enter the
//! part at all (the solid law, with [`root_cutter`]). An external
//! thread (#985) mirrors it on a boss, which CAD draws at its major
//! diameter: the crests may reach in to the minor circle
//! ([`super::thread_boss`]), the roots stay off the drawn wall.

use camcraft_tools::thread_mill as form;

use crate::drilling::Hole;
use crate::geo2d::Region;
use crate::machining2d::{self, GenError, ThreadHole, ThreadParams};
use crate::tools::Tool;
use crate::verify::{Cutter, Warning};

use super::heights::ResolvedHeights;
use super::material_source::{disk_loop, rim_is};
use super::types::OpParams;

/// The thread's radial depth for `pitch`, mm: ISO 724's in a hole, the
/// ISO 965-1 design root on a boss.
fn depth_of(pitch: f64, external: bool) -> f64 {
    pitch * if external { machining2d::EXTERNAL_THREAD_DEPTH_PER_PITCH } else { machining2d::THREAD_DEPTH_PER_PITCH }
}

/// The door (#894): why `tool` cannot cut the thread `params` states, by
/// name, or `None`. Asked when an operation is created, edited or
/// re-tooled, and again at generate. `None` for another kind of
/// operation, for a tool that is no thread mill (the tool law's
/// family door names that one), and for a single-form mill whose pitch
/// is not stated yet (the generator names that one).
#[must_use]
pub fn door(tool: &Tool, params: &OpParams) -> Option<String> {
    let OpParams::Thread { pitch, infeed, holes, external, bottom_z, passes, .. } = params else {
        return None;
    };
    if !form::is_thread_mill(tool.kind) {
        return None;
    }
    if let Some(why) = form::form_refusal(tool) {
        return Some(why);
    }
    // What the features ask of the tool: air inside a hole; on a boss
    // (#985), a core under the thread and a mill that reaches its foot.
    let no_air = |depth: Option<f64>| {
        if *external {
            // A stated bottom is resolved at generate, which asks again.
            return super::thread_boss::door(tool, holes, depth, bottom_z.is_none().then_some(None));
        }
        holes.iter().find(|x| tool.diameter * 0.5 >= x.radius).map(|x| {
            format!(
                "a {} mm thread mill has no air to enter the {} mm drill hole at ({}, {}) - the mill must be smaller than the thread's minor diameter",
                tool.diameter,
                x.radius * 2.0,
                x.xy.x,
                x.xy.y
            )
        })
    };
    let pitch = match form::resolve_pitch(tool, *pitch) {
        Ok(p) => p,
        // A single-form mill with the pitch not stated yet is an
        // unfinished operation, not an impossible one: a new Thread
        // operation starts that way, and the generator refuses it by
        // name until the pitch is stated. Everything that needs no
        // pitch is still judged here.
        Err(_) if pitch.is_none() => return passes.refusal(None, *infeed).or_else(|| no_air(None)),
        Err(why) => return Some(why),
    };
    let h = depth_of(pitch, *external);
    if tool.diameter * 0.5 <= h {
        return Some(format!(
            "a {} mm thread mill is no wider than twice the {h} mm thread depth of a {pitch} mm pitch - its teeth leave no core",
            tool.diameter
        ));
    }
    if *infeed > h + 1e-9 {
        return Some(format!(
            "the radial infeed of {infeed} mm is more than the {h} mm thread depth a {pitch} mm pitch stands at"
        ));
    }
    // #988: the pass control against this thread's depth.
    if let Some(why) = passes.refusal(Some(h), *infeed) {
        return Some(why);
    }
    no_air(Some(h))
}

/// The kernel's parameters for this operation and tool, heights
/// resolved; every door refusal is the generator's refusal too.
pub(super) fn kernel_params(tool: &Tool, params: &OpParams, hl: &ResolvedHeights) -> Result<ThreadParams, GenError> {
    let OpParams::Thread { infeed, feed, plunge_feed, climb, holes, pitch, hand, external, passes, .. } = params else {
        return Err(GenError::Invalid("not a thread-milling operation".into()));
    };
    if !form::is_thread_mill(tool.kind) {
        return Err(GenError::Invalid(format!(
            "thread milling needs a thread mill or a single-form thread mill - this tool is a {}",
            tool.kind
        )));
    }
    // The pitch first: an unstated one on a single-form mill passes the
    // door (an unfinished operation) and is refused here, by name.
    let pitch = form::resolve_pitch(tool, *pitch).map_err(GenError::Invalid)?;
    if let Some(why) = door(tool, params) {
        return Err(GenError::Invalid(why));
    }
    // #985: the reach beside a boss again, now against the resolved bottom.
    if *external
        && let Some(why) = super::thread_boss::door(tool, holes, Some(depth_of(pitch, true)), Some(hl.bottom))
    {
        return Err(GenError::Invalid(why));
    }
    Ok(ThreadParams {
        holes: holes
            .iter()
            .map(|x| ThreadHole {
                xy: x.xy,
                radius: x.radius,
                top_z: x.entry_z,
                floor_z: hl.bottom.unwrap_or(x.floor_z),
                // A boss has no fold and no exit: the words are a hole's.
                stepped: x.stepped && !*external,
                // A stated bottom is the author's own floor; only the
                // hole's own exit takes the run-through allowance.
                through: x.through && hl.bottom.is_none() && !*external,
            })
            .collect(),
        pitch,
        tool_diameter: tool.diameter,
        rows: form::rows(tool),
        infeed: *infeed,
        external: *external,
        hand: *hand,
        climb: *climb,
        passes: *passes,
        clearance_z: hl.clearance,
        retract_z: hl.retract,
        feed: *feed,
        plunge_feed: *plunge_feed,
    })
}

/// The crest law's region: `region` with each threaded hole's rim moved
/// out from the minor circle to the major one (`depth` further), so the
/// tangency verdict convicts a tooth crest that passes the thread's own
/// major diameter and nothing short of it.
pub(super) fn crest_region(region: Region, holes: &[Hole], depth: f64) -> Result<Region, GenError> {
    if holes.is_empty() {
        return Ok(region);
    }
    let mut loops: Vec<_> = region.loops().iter().filter(|l| !holes.iter().any(|x| rim_is(l, *x))).cloned().collect();
    for x in holes {
        loops.push(disk_loop(x.xy, x.radius + depth)?);
    }
    Ok(Region::new(loops)?)
}

/// The root law's cutter: `cutter` narrowed by the thread depth for a
/// Thread operation, so the solid law holds the body under the teeth
/// out of the part - radially at the minor wall (a boss's drawn major
/// wall, by the external depth, #985), and at the tip with no allowance
/// at all. Every other operation, and a tool whose pitch does not
/// resolve (refused before any path exists), keeps its cutter.
#[must_use]
pub(super) fn root_cutter(cutter: Cutter, tool: &Tool, params: &OpParams) -> Cutter {
    let OpParams::Thread { pitch, external, .. } = params else {
        return cutter;
    };
    match form::resolve_pitch(tool, *pitch) {
        Ok(p) if cutter.radius > depth_of(p, *external) => Cutter { radius: cutter.radius - depth_of(p, *external), ..cutter },
        _ => cutter,
    }
}

/// The radius the program simulation sweeps for `tool` under `params`:
/// the tool's own certain removal (`tool_law::cutting_radius`), except
/// an external Thread operation (#985), whose tooth roots stand the
/// external depth in at the thread's own pitch - exactly on the boss's
/// drawn wall on the last pass, where the internal depth at the tool's
/// coarsest pitch would read 0.07 x pitch of the boss as cut away.
pub fn sim_radius(tool: &Tool, params: &OpParams) -> Result<f64, String> {
    if let OpParams::Thread { pitch, external: true, .. } = params
        && let Ok(p) = form::resolve_pitch(tool, *pitch)
    {
        let root = tool.diameter * 0.5 - depth_of(p, true);
        if root.is_finite() && root > 0.0 {
            return Ok(root);
        }
    }
    super::tool_law::cutting_radius(tool)
}

/// The crossing law stands down for a Thread operation: every radial
/// pass and axial level retraces the same helix, and the lead arcs of
/// successive passes leave the axis tangent to one another, so feed
/// moves "crossing in XY at shared depth" is the operation itself, not
/// a path defect. What a harmful move would be - a crest past the major
/// circle, the body in the part, a rapid in stock - the crest law, the
/// root law and the rapid law still judge. Every other kind keeps it.
pub(super) fn stand_down_crossings(params: &OpParams, warns: &mut Vec<Warning>) {
    if matches!(params, OpParams::Thread { .. }) {
        warns.retain(|w| !matches!(w, Warning::Crossing { .. }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cl::ThreadHand;
    use crate::geo2d::Point2;
    use crate::operation::Height;

    fn hole(radius: f64) -> Hole {
        Hole { xy: Point2::new(5.0, 5.0), radius, entry_z: 0.0, floor_z: -10.0, through: false, stepped: false }
    }

    fn thread(pitch: Option<f64>) -> OpParams {
        OpParams::Thread {
            clearance_z: Height::fixed(10.0),
            retract_z: None,
            top_z: Height::fixed(0.0),
            bottom_z: None,
            infeed: 0.2,
            feed: 300.0,
            plunge_feed: 100.0,
            climb: true,
            holes: vec![hole(4.0)],
            pitch,
            hand: ThreadHand::Right,
            external: false,
            passes: crate::machining2d::ThreadPasses::default(),
        }
    }

    /// Any kind that is not Thread: the registry's first row, seeded.
    fn other_kind() -> OpParams {
        (crate::operation::KindInfo::ALL[0].seed)(&crate::operation::Carried::of(&thread(None)))
    }

    #[test]
    fn the_crest_region_moves_each_rim_out_to_the_major_circle() {
        let square = crate::geo2d::Loop::new(vec![
            Point2::new(-20.0, -20.0),
            Point2::new(20.0, -20.0),
            Point2::new(20.0, 20.0),
            Point2::new(-20.0, 20.0),
        ])
        .expect("a square");
        let minor = disk_loop(Point2::new(5.0, 5.0), 4.0).expect("rim");
        let region = Region::new(vec![square, minor]).expect("plate with a hole");
        let crest = crest_region(region, &[hole(4.0)], 0.8).expect("crest region");
        let radii: Vec<f64> = crest
            .loops()
            .iter()
            .filter(|l| l.points().len() > 8)
            .map(|l| l.points().iter().map(|p| (p.x - 5.0).hypot(p.y - 5.0)).fold(0.0, f64::max))
            .collect();
        assert_eq!(radii.len(), 1, "the minor rim is replaced, not kept beside the major one");
        assert!((radii[0] - 4.8).abs() < 2e-6, "{radii:?}");
    }

    #[test]
    fn the_root_cutter_narrows_a_thread_mill_by_the_thread_depth_and_nothing_else() {
        let tool = form::thread_mill(6.0, 1.5, 0.0, 12.0);
        let cutter = Cutter::of(&tool);
        let root = root_cutter(cutter, &tool, &thread(None));
        assert!((root.radius - (3.0 - 0.5413 * 1.5)).abs() < 1e-12);
        assert_eq!(root.flute_len, cutter.flute_len);
        // An unresolvable pitch (refused before a path exists) keeps the full cutter.
        assert_eq!(root_cutter(cutter, &tool, &thread(Some(1.0))).radius, 3.0);
        assert_eq!(root_cutter(cutter, &tool, &other_kind()).radius, 3.0);
    }

    #[test]
    fn an_external_thread_narrows_the_roots_by_the_external_depth() {
        let tool = form::thread_mill(6.0, 1.5, 0.0, 12.0);
        let mut boss = thread(None);
        if let OpParams::Thread { external, .. } = &mut boss {
            *external = true;
        }
        let root = 3.0 - 0.6134 * 1.5;
        assert!((root_cutter(Cutter::of(&tool), &tool, &boss).radius - root).abs() < 1e-12);
        assert!((sim_radius(&tool, &boss).expect("root") - root).abs() < 1e-12);
        // An internal thread keeps the tool's own certain removal.
        assert!((sim_radius(&tool, &thread(None)).expect("core") - (3.0 - 0.5413 * 1.5)).abs() < 1e-12);
        assert_eq!(sim_radius(&tool, &other_kind()), Ok(3.0 - 0.5413 * 1.5));
    }

    #[test]
    fn only_a_thread_operation_stands_the_crossing_law_down() {
        let mut warns = vec![Warning::Crossing { moves: [1, 2] }, Warning::Gouge { at: 3 }];
        stand_down_crossings(&other_kind(), &mut warns);
        assert_eq!(warns.len(), 2);
        stand_down_crossings(&thread(None), &mut warns);
        assert_eq!(warns, vec![Warning::Gouge { at: 3 }]);
    }
}
