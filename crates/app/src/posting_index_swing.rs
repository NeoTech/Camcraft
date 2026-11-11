//! The index-swing clause of the program gate (#804): before a 4th axis
//! that turns the work about X indexes, the tool is retracted to the
//! machine's safe Z - and the tool tip there must clear the radius the
//! stock, the parts and the workholding that turns with them sweep about
//! the A centerline. Too short a clearance refuses the post by name.
//!
//! Provenance: our own design (issue #804). The geometry is plain: every
//! point of a vertical tool at height h above a line along X is at least
//! h from it, so the tip is the tool's nearest point to the A axis; the
//! swept radius is the farthest stated vertex from that line. Machine
//! numbers are the `.mch` file's (Tormach 770MX spec page, Tormach 4th-
//! axis comparison sheet 02/24) - no vendor implementation inspected.
//!
//! The laws:
//! - **Who is judged.** Every posted op whose machine states a rotary
//!   that turns the work about X ([`work_a`]): a G30 or a section start
//!   can turn A before any op, so every op's retract is an index pose.
//!   Machines without one are untouched.
//! - **Where the tip is.** WCS Y0 Z0 sits on the A centerline (the
//!   indexed post's setup law), so at the safe Z the tip is at program
//!   Z `nose_to_axis - tool length` (cutter, shank and holder: the same
//!   protrusion the magazine law measures). An unstated `nose_to_axis`
//!   refuses: the gate cannot show the turn clears.
//! - **What sweeps.** The farthest vertex, from the line through the
//!   WCS origin along X, of the setup's stock, its parts and its tagged
//!   fixtures that turn with the work (a chuck); fixed steel does not
//!   turn. Over the whole X range: the tool's X at the index is the G30
//!   position, which the program does not know.
//! - **The margin.** [`MARGIN_MM`] between the tip and the swept radius.

use crate::{fixture, model, setup, setup_align};
use flecs_ecs::prelude::World;

/// The clearance the tip must keep over the swept radius, mm (5 mm,
/// about 0.2 in): covers the nominal spec numbers behind `nose_to_axis`,
/// a tool-length measurement and the chuck's centring.
pub const MARGIN_MM: f64 = 5.0;

/// The work-turning rotary about X a machine states: its letter and
/// stated `nose_to_axis` (None = unstated). None when there is none.
fn work_a(m: &crate::machine::Machine) -> Option<(String, Option<f64>)> {
    m.axes
        .iter()
        .find(|a| {
            a.rotary
                && a.turns_part
                && a.axis.is_some_and(|d| d[0] * d[0] >= 0.99 * (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]))
        })
        .map(|a| (a.name.clone(), a.nose_to_axis))
}

/// The A an op indexes to: the first leaning move's tool axis read as a
/// turn about X (the post's own reading, a = atan2(-y, z)), else 0.
fn index_angle(path: &crate::cl::Toolpath) -> f64 {
    path.moves
        .iter()
        .find_map(|mv| mv.axis.filter(|a| !crate::cl::is_vertical(a)))
        .map_or(0.0, |a| (-a[1]).atan2(a[2]).to_degrees())
}

/// The farthest point of the setup's turning bodies from the A line
/// through `origin` along X, mm (0 with nothing stated).
fn swept_radius(world: &World, declared: Option<&str>, origin: [f64; 3]) -> f64 {
    let mut bodies: Vec<flecs_ecs::prelude::Entity> = Vec::new();
    bodies.extend(setup::stock_entity(world, declared));
    bodies.extend(
        model::model_entities(world)
            .into_iter()
            .filter(|(_, e)| setup_align::is_part(world, e))
            .map(|(_, e)| e),
    );
    bodies.extend(
        fixture::tagged(world, declared)
            .into_iter()
            .filter(|(_, e)| !fixture::def_of(world, e).is_some_and(|d| d.fixed))
            .map(|(_, e)| e),
    );
    let mut r = 0.0_f64;
    for e in bodies {
        for t in setup_align::facets_in(world, &e, declared).unwrap_or_default() {
            for p in [t.a, t.b, t.c] {
                let d = (p.y - origin[1]).hypot(p.z - origin[2]);
                if d.is_finite() {
                    r = r.max(d);
                }
            }
        }
    }
    r
}

/// The clause: one problem per op whose index pose cannot be shown to
/// clear, in program order.
pub(super) fn index_swing_problems(world: &World) -> Vec<String> {
    let Ok(ops) = super::program_ops(world) else {
        return Vec::new(); // the post refuses the program by name first
    };
    let mut out = Vec::new();
    for (name, e) in ops {
        let Some(op) = model::operation_of(world, &e) else { continue };
        let Some(m) = crate::machine::of_setup(world, op.setup.as_deref()).1 else { continue };
        let Some((letter, nose)) = work_a(&m) else { continue };
        let a = model::toolpath_of(world, &e).map_or(0.0, |c| index_angle(&c.path));
        let Some(nose) = nose else {
            out.push(format!(
                "{name}: {} turns the work on {letter} but states no nose_to_axis - the tool's clearance \
                 over the part swept by the {letter}{a:.3} index cannot be judged",
                m.model
            ));
            continue;
        };
        // A lost tool is the machine clause's refusal, by name, already.
        let Some(tool_e) = model::find_by_name(world, &op.tool) else { continue };
        let Some(tool) = model::tool_of(world, &tool_e) else { continue };
        let tool = match crate::tool_body::judged(world, &tool_e, tool) {
            Ok((t, _)) => t,
            Err(why) => {
                out.push(format!("{name}: tool {}: {why} - the {letter} index clearance cannot be judged", op.tool));
                continue;
            }
        };
        let len = tool.flute_len + tool.shank_len + tool.holder_len;
        let (_, origin) = super::wcs_context(world, op.setup.as_deref());
        let swept = swept_radius(world, op.setup.as_deref(), origin);
        let tip = nose - len;
        let clearance = tip - swept;
        if !(clearance.is_finite() && clearance >= MARGIN_MM) {
            out.push(format!(
                "{name}: the {letter}{a:.3} index turns the work under the tool - at the safe Z the tip of \
                 {} ({len:.3} mm long) is {tip:.3} mm above the {letter} centerline, the stock, part and \
                 turning fixtures sweep {swept:.3} mm, a clearance of {clearance:.3} mm against the \
                 {MARGIN_MM:.1} mm needed",
                op.tool
            ));
        }
    }
    out
}
