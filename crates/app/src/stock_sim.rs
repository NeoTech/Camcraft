//! Document stock: the world-facing half of `stock_sim`. The engine
//! (dexel grid, sweeps, measures, deviation, traverse timing) lives
//! in camcraft-sim (crates/sim/src/stock_sim.rs, M4.1 R4); what stays
//! here is exactly the part that walks the world - building the stock
//! solid from the `Stock`/`Setup` entities and driving a whole
//! document through the sim: the tuple seam the design note promised,
//! the engine consumes (facets, stock, tool) data, this module hands
//! it over.

pub use camcraft_sim::stock_sim::*;

// The rotary clause (M13.1): the eye's chord law, the verdict's
// refusal, the per-kind lift.
#[path = "stock_sim_rotary.rs"]
pub mod rotary;
pub use rotary::{
    rotary_kind_lift, rotary_pose_tol, simulate_rotary_slice, ROTARY_KIND_UNMEASURED, ROTARY_VERDICT_REFUSAL,
};

// The lathe clause (#19): the nose and the spindle in the plan, the
// blade's refusal, the verdict's refusal.
#[path = "stock_sim_lathe.rs"]
pub mod lathe;
pub use lathe::{LathePlan, LATHE_HELIX_REFUSAL, LATHE_THREAD_REFUSAL, LATHE_VERDICT_REFUSAL};

// The carry across setups (#317): the machined body moved into the
// next setup's frame through the transform between the two.
#[path = "stock_sim_carry.rs"]
pub mod carry;
// The stock as the walk holds it, and the engine that carries it (#637).
#[path = "stock_sim_held.rs"]
pub mod held;
#[path = "stock_sim_breaks.rs"]
pub mod breaks;
// #1595: the marks an Engrave or Trace on the part declares.
#[path = "stock_sim_marks.rs"]
mod marks;
// The wire clause (#1110): a wire cut stays out of the plan, by name.
#[path = "stock_sim_wire.rs"]
pub mod wire;
#[path = "stock_sim_waterjet.rs"]
pub mod waterjet;
pub use carry::{
    CARRY_MACHINE_CHANGE_REFUSAL, CARRY_MACHINE_REFUSAL, carry_machine_refusal, incoming_body, machine_kind_name,
};
pub use held::{CARRY_ENGINE_ENV, CARRY_ENGINE_SWITCH_HONOURED, FIELD_CARRY_SHAPE_REFUSAL, engine as carry_engine};
pub use held::{engine_from_switch as carry_engine_from_switch, engine_note as carry_engine_note, with_engine as with_carry_engine};

use crate::geometry::Triangle;

#[path = "stock_grid.rs"]
mod grid;
pub use grid::stock_grid;

// The plan both the verdict and the field eye walk (#2528 split).
#[path = "stock_sim_plan.rs"]
mod plan;
pub(crate) use plan::{bull_corner, plan_program, plan_program_until_ungenerated, Planned};

/// Simulate the whole document's program: the operations in posting
/// order (through [`crate::posting::program_ops`] - so what the sim
/// judges IS what would post, one law shared, not two agreeing), each
/// op's CACHED moves through its tool's atom, against the stock as the
/// setup defines it. Move coordinates are pre-origin-shift world
/// millimeters, the same numbers the viewport draws and the post
/// shifts; the stock grid is built on that same frame. Refusals name
/// their reason (an op that never generated, a rotated stock, a grid
/// too fine) and change nothing.
pub fn simulate_document(
    world: &flecs_ecs::prelude::World,
) -> Result<ProgramSim, String> {
    let planned = plan_program(world)?;
    wire::refuse_wire_cuts(world)?;
    waterjet::refuse_waterjet_cuts(world)?;
    refuse_unjudged(&planned)?;
    // #864: a part imported coarser than promised tightens the verdict by
    // its excess; one too coarse for that is refused by name, not judged -
    // and so is a program with an op the gate refuses for it (#1056).
    let tol = crate::model_fidelity::program_tol(world)?;

    // Setup changes (#317): a setup that holds the part differently, or
    // is placed relative to the one before it, starts from the body the
    // earlier setups left, carried through the transform between them
    // (`carry`); two setups holding it alike keep the M3.2 A clip. The
    // verdict is read in the frame of the setup the program ends in.
    let walked = carry::walk(world, &planned, None)?;
    let model_facets = if walked.spanned {
        crate::setup_align::parts_facets_in(world, walked.frame.as_deref())
    } else {
        part_facets(world)?
    };
    let (mut dev, display) = walked.held.verdict(&model_facets, tol)?;
    // #303: the op owns its break - a chamfer's or deburr's declared
    // bevel is part of the finished part, judged by its own depth.
    let breaks = breaks::program_breaks(world, &planned, walked.spanned, &walked.frame, &model_facets, tol);
    credit_breaks(&mut dev, &display, &model_facets, &breaks);
    // #1595: and its mark - an Engrave's or Trace's own sweep, within
    // the budget the operation's own law holds it to.
    credit_marks(&mut dev, &display, &model_facets, &marks::program_marks(world, &planned, walked.spanned, &walked.frame));
    // #2400: declared pocket/slot corner reliefs, bounded by cutter radius and cut floor.
    let reliefs = program_corner_reliefs(world, &planned, walked.spanned, &walked.frame);
    credit_corner_reliefs(&mut dev, &display, &model_facets, &reliefs);
    // #320: a gouge on a block a later setup set aside for a new one.
    if let Some(left) = &walked.left {
        carry::fold_left(&mut dev, left);
    }
    let classes = color_classes(&dev);
    let surface = display.surface(&classes);
    Ok(ProgramSim { dev, surface })
}

/// The kinds the column verdict cannot judge, refused by name before
/// any material is cut.
///
/// The posed clause (M13.1 follow-up): the verdict is the COLUMN
/// engine, whose atoms stand upright - a 5-axis lean or a 3+2 frame's
/// tilted moves would be cut as if vertical, a silent wrong verdict.
/// The field arm does cut a posed move exactly, but its verdict reads
/// heights marched from above, and a tilted tool is precisely the tool
/// that can reach UNDER an overhang. So neither arm's height verdict
/// judges a posed program - refusal by name, which the post gate shows
/// as "not judged".
pub(crate) fn refuse_unjudged(planned: &[Planned]) -> Result<(), String> {
    for p in planned {
        // The lathe clause (#19): a turning program is the field
        // eye's to cut (`stock_sim::lathe`), never this height
        // field's to judge.
        if p.lathe.is_some() {
            return Err(format!("{}: {LATHE_VERDICT_REFUSAL}", p.name));
        }
        // The rotary clause (M13.1): named before the posed one, so
        // the gate says WHY this posed program is not judged here.
        if p.rotary.is_some() {
            return Err(format!("{}: {ROTARY_VERDICT_REFUSAL}", p.name));
        }
        if let Some(k) = first_posed(&p.moves) {
            return Err(format!("{}: move {k}: {POSED_REFUSAL}", p.name));
        }
    }
    Ok(())
}

pub(super) fn program_corner_reliefs(
    world: &flecs_ecs::prelude::World,
    planned: &[Planned],
    spanned: bool,
    frame: &Option<String>,
) -> Vec<crate::verify::CornerReliefZone> {
    planned
        .iter()
        .filter(|p| !spanned || crate::setup::effective(world, p.setup.as_deref()) == *frame)
        .flat_map(|p| p.corner_reliefs.iter().copied())
        .collect()
}

/// The field eye's refusal of a sinker electrode (#541), said once.
pub const ELECTRODE_FIELD_REFUSAL: &str = "the stock view has no sinker electrode atom yet - the program verdict (the column engine's box atom) judges the burn";

/// The part as the machine must leave it: every model's facets in
/// world, minus the stock (setup furniture, not design intent) - and
/// minus the workholding, which M3.2 B made models too. A vise is
/// steel the program must dodge, never a promise it must leave
/// standing at height: its facets belong to the collision gate, not
/// to the gouge verdict (counting them here would read a clamped job
/// as a part with a vise-shaped hole in it). Visibility is a display
/// choice; the verdict judges all the rest.
///
/// #326: in a document where a setup holds the part turned, the part is
/// read in the frame of the one setup the program machines in; a program
/// spanning setups that hold it differently is refused by name here - the
/// field view draws one setup's frame. The program verdict
/// (`simulate_document`) carries the body across such setups (#317) and
/// reads the part in the frame the program ends in instead.
pub(crate) fn part_facets(world: &flecs_ecs::prelude::World) -> Result<Vec<Triangle>, String> {
    if !crate::setup_align::any_aligned(world) {
        return Ok(part_facets_as_modelled(world));
    }
    let mut setups: Vec<Option<String>> = Vec::new();
    for (_, e) in crate::setup::operation_order(world) {
        let Some(op) = crate::model::operation_of(world, &e) else { continue };
        if op.suppressed {
            continue;
        }
        let eff = crate::setup::effective(world, op.setup.as_deref());
        if !setups.contains(&eff) {
            setups.push(eff);
        }
    }
    match setups.as_slice() {
        [] => Ok(crate::setup_align::parts_facets_in(world, crate::setup::active(world).as_deref())),
        [one] => Ok(crate::setup_align::parts_facets_in(world, one.as_deref())),
        many if many.iter().any(|s| crate::setup_align::alignment(world, s.as_deref()).is_some()) => Err(
            "the program spans setups that hold the part differently (part alignment) - the field view draws one setup's frame; the program verdict carries the stock between them"
                .into(),
        ),
        _ => Ok(part_facets_as_modelled(world)),
    }
}

fn part_facets_as_modelled(world: &flecs_ecs::prelude::World) -> Vec<Triangle> {
    use crate::geometry::transform_point;
    use crate::model;

    let mut model_facets: Vec<Triangle> = Vec::new();
    let skip = crate::stock_source::stock_only_model(world); // #1094
    for (_, e) in model::model_entities(world) {
        if !crate::stock_source::judged(world, &e, skip) {
            continue;
        }
        let tris = model::facets_of(world, &e).unwrap_or_default();
        let l = model::transform_of(world, &e).unwrap_or_default();
        model_facets.extend(tris.iter().map(|t| Triangle::new(
            transform_point(t.a, l.pos, l.rot),
            transform_point(t.b, l.pos, l.rot),
            transform_point(t.c, l.pos, l.rot),
        )));
    }
    model_facets
}

// The engine-law unit tests (the ball atom, the reach law, the
// envelope gate), split out by the 400-line law (M13.1).
#[cfg(test)]
#[path = "stock_sim_tests.rs"]
mod tests;
