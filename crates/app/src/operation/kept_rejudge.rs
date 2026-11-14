//! The post's live re-judge of the parts beside an operation's source
//! (#1628): what generation read through `keep_law::parts_in_stock` (a
//! stock source) and `keep_law::neighbour_parts` (a part source), read
//! again against the parts as they stand now.
//!
//! Provenance: our own design (`docs/decisions/1628-kept-parts-live-judge.md`)
//! over the solid and mark laws generation runs (`generate::contain_into`,
//! `mark_law::mark_into`). No vendor material consulted.
//!
//! ## The law
//!
//! Those parts are not in the op's stamp (the stamp contracts stand:
//! workholding never stales a path, a stock-based path never reads the
//! model, a fidelity refusal keeps its name). So a part moved into the
//! cut, added, un-tagged as a fixture, or un-marked as raw stock only
//! after a clean generation leaves the op `Ok` with a verdict nobody gave.
//! On an aligned stock the program sim judges every part live; where it
//! cannot judge (a turned stock, a posed path), the post gate
//! (`posting_sim`) calls [`moved_part_problem`] for each program op: the
//! cached path is judged again, with the very laws and numbers generation
//! used, against each kept part as it stands. A cut found is a refusal
//! naming the op and the part; nothing churns the op's status.

use flecs_ecs::prelude::World;

use crate::cl::{Move, Toolpath};
use crate::schema::Operation;
use crate::verify;

use super::generate::{contain_into, depth_law_of, stock_to_leave_of, DepthLaw};
use super::{keep_law, mark_law};

/// Whether `op`'s kind can read parts beside its source at all - a cheap
/// door before a job is collected (most ops carry no such law).
fn may_keep(op: &Operation) -> bool {
    keep_law::is_stock_kept_kind(&op.params) || depth_law_of(&op.params, &op.src, true) != DepthLaw::None
}

/// `None` when the cached `path` of program op `name` cuts none of the
/// parts its part laws keep beside its source, as they stand now; else
/// the refusal, naming the op and the first part it enters. An op whose
/// inputs can no longer be read is refused too: no verdict is no pass.
#[must_use]
pub fn moved_part_problem(world: &World, name: &str, op: &Operation, path: &Toolpath) -> Option<String> {
    if !may_keep(op) {
        return None;
    }
    let job = match super::collect_job(world, op, name) {
        Ok(job) => job,
        Err(e) => return Some(format!("{name}: the parts beside its source cannot be read again ({e}); regenerate it")),
    };
    if job.keep_parts.is_empty() && job.neighbour_parts.is_empty() {
        return None;
    }
    let parts = keep_law::kept_parts_named(world, op.setup.as_deref(), &op.params, &op.src, job.part_solid);
    let geom = job.tool.geom();
    let holder = crate::tool_body::stack(&geom, job.body.as_ref()).with_neck(job.neck.clone());
    let cutter = verify::Cutter::of(&job.tool).with_body(&geom, &holder);
    let cutter = super::thread_op::root_cutter(cutter, &job.tool, &job.params);
    let stl = stock_to_leave_of(&job.params);
    let excess = job.material.excess;
    // Generation's split: a stock-source Engrave or Trace marks the parts
    // (budgeted); every other kept part takes the plain solid law.
    let marks = !job.part_solid && mark_law::is_mark_kind(&job.params);
    let budget = job.hs.top.zip(job.hs.bottom).map_or(0.0, |(t, b)| mark_law::budget(t, b, job.tool.diameter * 0.5));
    // The laws run in the frame generation judged in: a 3+2 op's local one.
    let local = job.frame.as_ref().map(|f| local_path(f, path));
    let judged = local.as_ref().unwrap_or(path);
    for (part, facets) in parts {
        let facets = job.frame.as_ref().map_or(facets.clone(), |f| f.local_facets(&facets));
        let judge = |c: &verify::Cutter| {
            let mut w = Vec::new();
            if marks {
                mark_law::mark_into(&mut w, judged, &facets, c, budget, excess);
            } else {
                contain_into(&mut w, judged, &facets, c, DepthLaw::Keep, stl, excess, &job.params, &[]);
            }
            w
        };
        let mut warns = judge(&cutter);
        // #266: a tap's fluted edge is read at its thread's core, as at generation.
        super::tap_core::acquit(&mut warns, &job.tool, &job.params, |core| {
            let g = core.geom();
            judge(&verify::Cutter::of(core).with_body(&g, &crate::tool_body::stack(&g, job.body.as_ref()).with_neck(holder.neck.clone())))
        });
        if let Some(w) = warns.iter().find(|w| w.blocks_post()) {
            // A local path's move numbers are not the posted path's: said only upright.
            let what = if local.is_none() { format!(" ({w})") } else { String::new() };
            return Some(format!(
                "{name} now cuts part '{part}'{what} - a part moved, added or no longer workholding since it was generated; regenerate it"
            ));
        }
    }
    None
}

/// A posted (world) path back in a 3+2 frame's local coordinates: every
/// move as its 3-D chords at the arc-fit tolerance (a tilted path is
/// already chords; an upright one's arcs give way to chords within the
/// same tolerance the verifier measures arcs by).
fn local_path(f: &crate::cl::TiltedFrame, path: &Toolpath) -> Toolpath {
    let moves = path
        .moves
        .iter()
        .flat_map(|m| {
            m.segments3(crate::machining2d::ARC_FIT_TOL)
                .into_iter()
                .map(|(a, b)| Move { from: f.to_local(a), to: f.to_local(b), arc: None, axis: None, ..*m })
        })
        .collect();
    Toolpath { moves, ..path.clone() }
}
