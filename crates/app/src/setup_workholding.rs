//! A setup's workholding rpm limit (#1031): the rated maximum rpm of the
//! chuck or collet a lathe setup holds the work in. A chuck's jaws lose
//! grip force as the spindle speeds up, so its rating is a ceiling of its
//! own, beside the machine's spindle maximum and the tool's max rpm, and
//! it belongs to the setup (the same tool in another chuck has another
//! limit). `None`, the default and every earlier file, states none.
//!
//! ## What it does to a program
//!
//! - **Constant surface speed (G96).** The section's rpm ceiling, the
//!   `PARAM MaxSpindleSpeed` the lathe posts clamp to (Fanuc `G50 S`,
//!   Tormach `G96 D`), becomes the lower of the tool's own and the
//!   workholding's ([`css_of`]); the posts take the lower of that and
//!   the machine's, so the clamp is min(machine, tool, workholding).
//! - **Constant rpm (G97).** An operation that turns the work at an rpm
//!   over its setup's limit is refused by name at the program gate
//!   ([`program_problems`]), never silently capped: the cut was planned
//!   at that rpm, so the fix is the user's to choose.
//!
//! Provenance: our own design (#1031); the hazard (jaw grip force falls
//! with speed, so chucks carry a rated maximum rpm) is general machining
//! knowledge from the machine-tool handbooks' workholding chapters. No
//! vendor material consulted.

use flecs_ecs::prelude::World;

use crate::model;
use crate::post::Css;
use crate::setup;

/// The highest limit a setup may state, rpm: past any lathe spindle, so
/// a typo (an extra zero) is refused at the command, not posted.
pub const MAX_STATED_RPM: f64 = 100_000.0;

/// Why `rpm` cannot be a workholding limit; `None` when it can.
#[must_use]
pub fn invalid(rpm: f64) -> Option<String> {
    if !rpm.is_finite() || rpm < 1.0 {
        return Some(format!("a workholding maximum must be at least 1 rpm, got {rpm}"));
    }
    if rpm > MAX_STATED_RPM {
        return Some(format!("a workholding maximum of {rpm} rpm is past any lathe spindle (at most {MAX_STATED_RPM})"));
    }
    None
}

/// The workholding limit of a declared setup context (`None` resolves
/// to the first setup, the implicit-setup law); `None` when it states
/// none or the document has no setups.
#[must_use]
pub fn of(world: &World, declared: Option<&str>) -> Option<f64> {
    let name = setup::effective(world, declared)?;
    setup::def_of(world, &name)?.workholding_max_rpm.filter(|r| invalid(*r).is_none())
}

/// The CSS ask with the workholding folded into its rpm ceiling: the
/// lower of the tool's own and the setup's. `None` stays `None` (a G97
/// section; [`program_problems`] judges its rpm).
#[must_use]
pub fn css_of(world: &World, declared: Option<&str>, css: Option<Css>) -> Option<Css> {
    let mut c = css?;
    if let Some(hold) = of(world, declared) {
        c.max_rpm = Some(c.max_rpm.map_or(hold, |t| t.min(hold)));
    }
    Some(c)
}

/// The program-level clause: every unsuppressed operation that turns the
/// work at a constant rpm (no CSS) faster than its setup's workholding
/// maximum, named with its setup. Empty when no setup states a limit.
#[must_use]
pub fn program_problems(world: &World) -> Vec<String> {
    let mut out = Vec::new();
    for (name, e) in setup::operation_order(world) {
        let Some(op) = model::operation_of(world, &e) else {
            continue;
        };
        if op.suppressed {
            continue;
        }
        let declared = op.setup.as_deref();
        let Some(hold) = of(world, declared) else {
            continue;
        };
        if !crate::operation::spindle_law::turns(world, declared, &op.params) {
            continue;
        }
        let Some(tool) = model::find_by_name(world, &op.tool).and_then(|t| model::tool_of(world, &t)) else {
            continue; // the tool gate names a missing tool
        };
        if crate::cutting_turn::css_of(&op.params, &tool).is_some() {
            continue; // G96: clamped by css_of above
        }
        let rpm = crate::posting::posted_rpm(&tool);
        if rpm > hold {
            let setup_name = setup::effective(world, declared).unwrap_or_default();
            let label = model::operation_label(world, &name, &e);
            out.push(format!(
                "{label}: turns the work at {rpm:.0} rpm (G97), over setup '{setup_name}''s workholding maximum of {hold:.0} rpm - lower tool {}'s rpm or state the chuck's real rating",
                op.tool
            ));
        }
    }
    out
}
