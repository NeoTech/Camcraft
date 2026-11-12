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

/// Which spindle an operation's stated speed drives, so what the
/// workholding limit judges in its section (#1174). Read from what the
/// lathe post does with the section, not from the kind's name:
/// `fanuc-lathe.cps` opens a plain section with `G97 S<rpm> M03` (or
/// `G96`) - the WORK spindle - and a `PARAM LiveTool` section (a path
/// stating C angles) with the machine's live-spindle word and `S<rpm>`
/// while the work spindle is put in C mode and fed as an axis.
enum WorkSpeed {
    /// The work turns at the section's stated speed: the turning family
    /// and the lathe's axial (on-centre) drill.
    Stated,
    /// A C-axis live-tool section: the stated rpm is the driven tool's;
    /// the work turns only on C, at most at the machine's C rate.
    OnC,
    /// Nothing the limit judges: a cross (radial) drill - a driven tool
    /// plunging toward a work held still (its moves state no C, its
    /// pose is fixed) - or a kind that does not turn the work.
    Not,
}

fn work_speed(world: &World, declared: Option<&str>, op: &crate::schema::Operation, e: &flecs_ecs::prelude::Entity) -> WorkSpeed {
    use flecs_ecs::prelude::*;
    // The post's own test: any move stating a C angle makes the section
    // live-tool work (the stream's PARAM LiveTool, `post::post`).
    let on_c = e
        .entity_view(world)
        .try_get::<&crate::schema::Toolpath>(|t| t.path.moves.iter().any(|m| m.rotary.is_some()))
        .unwrap_or(false);
    if on_c {
        return WorkSpeed::OnC;
    }
    if matches!(&op.params, crate::operation::OpParams::Drill { cross, .. } if !cross.is_empty()) {
        return WorkSpeed::Not;
    }
    if crate::operation::spindle_law::turns(world, declared, &op.params) {
        WorkSpeed::Stated
    } else {
        WorkSpeed::Not
    }
}

/// The fastest the machine states its C axis turns, deg/min: the larger
/// of its feed and rapid rates (the post positions C at the rapid and
/// caps a cut by the feed). `None` when the machine states neither.
fn c_rate(world: &World, declared: Option<&str>) -> Option<f64> {
    let (_, machine) = crate::machine::of_setup(world, declared);
    let c = machine?.axes.into_iter().find(|a| a.rotary && a.name.eq_ignore_ascii_case("C"))?;
    [c.feed, c.rapid].into_iter().flatten().filter(|r| r.is_finite() && *r > 0.0).reduce(f64::max)
}

/// The program-level clause: every unsuppressed operation during which
/// the WORK spindle turns faster than its setup's workholding maximum,
/// named with its setup - a constant-rpm (G97) section at its stated
/// rpm, a C-axis live-tool section at its machine's C rate (never the
/// live tool's own rpm). Empty when no setup states a limit.
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
        let setup_name = || setup::effective(world, declared).unwrap_or_default();
        match work_speed(world, declared, &op, &e) {
            WorkSpeed::Not => {}
            WorkSpeed::OnC => {
                let label = model::operation_label(world, &name, &e);
                match c_rate(world, declared) {
                    Some(rate) if rate / 360.0 > hold => out.push(format!(
                        "{label}: turns the work on the C axis at up to {:.0} rpm (the machine's C rate of {rate:.0} degrees per minute), over setup '{}''s workholding maximum of {hold:.0} rpm - state the chuck's real rating or a slower C rate",
                        rate / 360.0,
                        setup_name()
                    )),
                    Some(_) => {}
                    None => out.push(format!(
                        "{label}: turns the work on the C axis, but the machine states no C rate, so how fast the work turns cannot be held to setup '{}''s workholding maximum of {hold:.0} rpm - state the C axis's speeds in the machine file",
                        setup_name()
                    )),
                }
            }
            WorkSpeed::Stated => {
                let Some(tool) = model::find_by_name(world, &op.tool).and_then(|t| model::tool_of(world, &t)) else {
                    continue; // the tool gate names a missing tool
                };
                if crate::cutting_turn::css_of(&op.params, &tool).is_some() {
                    continue; // G96: clamped by css_of above
                }
                let rpm = crate::posting::posted_rpm(&tool);
                if rpm > hold {
                    let label = model::operation_label(world, &name, &e);
                    out.push(format!(
                        "{label}: turns the work at {rpm:.0} rpm (G97), over setup '{}''s workholding maximum of {hold:.0} rpm - lower tool {}'s rpm or state the chuck's real rating",
                        setup_name(),
                        op.tool
                    ));
                }
            }
        }
    }
    out
}
