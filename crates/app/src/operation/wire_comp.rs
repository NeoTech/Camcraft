//! The Wire Contour's offset mode (#4134, the app side of #4076): the
//! command door's refusals of what the control's wire offset cannot
//! engage on, and the spans recorded over the generated wire-centre path
//! - one for each pass, its radius that pass's kerf offset.
//!
//! Provenance: our own glue over the motion crate's span builder
//! (`wire_edm::wire_comp_spans`, whose module head cites RS-274 / ISO
//! 6983 and the owner's rules on #4076) and the machine's condition
//! table (#531), the one source of each pass's wire and gap.

use crate::cl::Toolpath;
use crate::links::Compensation;
use crate::machining2d::GenError;
use crate::wire_edm::wire_comp_spans;

use super::types::WireContourOp;
use super::wire_op::WireCondition;

/// What the stated offset cannot ride (#4134), by name: `None` for
/// computer compensation, or when the cut is a vertical, one-profile,
/// untabbed closed cut with the straight lead. The command door and the
/// generator both ask it.
#[must_use]
pub fn refusal(op: &WireContourOp) -> Option<String> {
    let mode = op.compensation;
    if !mode.in_control() {
        return None;
    }
    let with = if op.side.is_open_side() {
        format!("an open run ({} of travel) - the offset engages on a closed profile's lead-in from its start hole", op.side.word())
    } else if op.four_axis.is_some() {
        "a 4-axis cut - the offset is programmed on one vertical profile".to_owned()
    } else if op.taper_deg != 0.0 {
        format!("a {:.3} degree taper - the offset is programmed on a vertical cut only", op.taper_deg)
    } else if op.burnout.is_some() {
        "a no-core burnout - the burnout's rings have no lead-in from the start hole".to_owned()
    } else if op.tabs.is_some() {
        "tabs - the tab cut-off has no straight lead-in to engage on".to_owned()
    } else if op.lead_arc_radius.is_some() {
        "a tangent arc lead - the offset engages on a straight lead-in, and G40 goes on a straight lead-out".to_owned()
    } else if op.picks.first(crate::path_picks::PickRole::Exit).is_some() {
        "an exit pick - each pass is programmed from the start hole and back to it".to_owned()
    } else {
        return None;
    };
    Some(format!(
        "wire contour: {} compensation is stated with {with}; set the compensation to computer or remove that setting",
        mode.label()
    ))
}

/// Record the offset spans over `path`, the wire-centre path that
/// generation made and judged: the rough pass at `cond`'s kerf offset,
/// each skim at its own condition's. Computer compensation leaves the
/// path as it is.
///
/// # Errors
/// [`refusal`]'s refusals, and the span builder's (a lead shorter than
/// the offset, a pass that does not return to the start hole, ...).
pub(super) fn apply(op: &WireContourOp, cond: &WireCondition, path: &mut Toolpath) -> Result<(), GenError> {
    if !op.compensation.in_control() {
        return Ok(());
    }
    if let Some(why) = refusal(op) {
        return Err(GenError::Invalid(why));
    }
    let mut radii = vec![cond.wire_diameter * 0.5 + cond.gap];
    radii.extend(cond.skims.iter().map(|s| s.wire_diameter * 0.5 + s.gap));
    wire_comp_spans(path, op.side, op.compensation, &radii)?;
    let registers = radii.iter().enumerate().map(|(i, k)| format!("pass {} {k:.4} mm", i + 1)).collect::<Vec<_>>().join(", ");
    path.notes.push(match op.compensation {
        Compensation::Wear => format!(
            "wire EDM: wear compensation - each pass posts the wire-centre path under G41 / G42; the register holds only the wear delta (the stream states each pass's kerf offset: {registers})"
        ),
        _ => format!(
            "wire EDM: control compensation - each pass posts the part contour under G41 / G42 with G40 on its lead-out; set each register to its pass's kerf offset ({registers})"
        ),
    });
    Ok(())
}
