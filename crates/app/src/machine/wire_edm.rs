//! The machine check on a wire EDM (#532, part of #290): what a program
//! of wire cuts asks the machine, judged against what its file states.
//!
//! Provenance: our own rules over the umbrella's machine model (#290:
//! X/Y carry the lower guide and table, U/V the upper guide; the wire
//! is the machine's condition) and the unstated-limits law of
//! `assets/machines/AGENTS.md`. No vendor manual consulted.
//!
//! What is judged: the X/Y travel of every move against the stated
//! strokes, and the refusals by name - an op that is not wire EDM work,
//! a move that states a pose (a taper: #536 judges the angle and the
//! U/V reach it needs). What is NOT: the wire sweep against the part,
//! the slug, the start hole's clearance (wire verification, #534), and
//! any tool number, rpm or feed - a wire EDM has no changer and no
//! spindle, and the control's generator sets the rate. Those are
//! SKIPPED with a named note on the check, never failed and never
//! silent.

use flecs_ecs::prelude::{Entity, World};

use super::{move_extremes, say, Machine, MachineCheck};
use crate::model;
use crate::operation::Family;

/// [`super::check_ops`] for a machine of kind wire EDM.
pub(super) fn check_wire_ops(world: &World, machine: &Machine, ops: &[(String, Entity)]) -> Result<MachineCheck, String> {
    let mut out = MachineCheck::default();
    out.notes.push(format!(
        "{}: a wire EDM - only the X/Y travel is judged here; the wire's sweep against the part, the slug and the start hole are judged at the post by the wire laws (#534)",
        machine.model
    ));
    out.notes.push(format!(
        "{}: a wire EDM has no spindle and no tool changer - rpm, feed and tool numbers are not judged (the control's generator sets the rate from the condition)",
        machine.model
    ));
    for (name, e) in ops {
        let op = model::operation_of(world, e).ok_or_else(|| format!("{name}: lost its operation"))?;
        let kind = op.params.kind();
        let family = kind.info().family();
        if family != Family::WireEdm {
            say(
                &mut out,
                format!("{name}: a {kind} operation is {} work - a wire EDM cuts only wire EDM operations", family.work()),
            );
            continue;
        }
        let cache = model::toolpath_of(world, e).ok_or_else(|| format!("{name}: lost its toolpath"))?;
        let (_, origin) = crate::posting::wcs_context(world, op.setup.as_deref());
        for (i, mv) in cache.path.moves.iter().enumerate() {
            if mv.rotary.is_some() || mv.axis.is_some_and(|a| !crate::cl::is_vertical(&a)) {
                say(&mut out, format!("{name}: move {i} states a wire pose - a tapered cut is not judged on this machine yet"));
            }
            for p in move_extremes(mv) {
                for (k, w) in [p.x, p.y].iter().enumerate() {
                    let Some(ax) = machine.axis(k) else { continue };
                    let Some((lo, hi)) = ax.travel else { continue };
                    let v = (w - origin[k]) * ax.direction.sign();
                    if v < lo || v > hi {
                        say(&mut out, format!(
                            "{name}: move {i} reaches {v:.3} in {}, the stroke is {lo:.3}..{hi:.3}",
                            ax.name
                        ));
                    }
                }
            }
        }
    }
    Ok(out)
}
