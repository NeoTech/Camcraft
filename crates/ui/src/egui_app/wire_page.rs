//! The operation page: the Wire Contour's fields (#532) - the side and
//! the start hole on Geometry, the two workpiece planes on Heights, the
//! machine's condition on Passes. Every value edits the page draft;
//! Apply rides the command history (EditOperationParams, undoable),
//! whose door refuses a condition the machine does not state. No Feeds
//! tab: a wire cut states no feed - the control's generator sets it.
//!
//! A minimal page: the tab law wants every kind's sections drawn, so
//! the fields exist. Picking the start hole in the viewport and the
//! wire EDM's own page layout are #535.
//!
//! Provenance: our own UI over our own wire parameters (general wire
//! EDM practice, cited in `motion::wire_edm`); no vendor text.

use super::fields::*;
use camcraft_app::operation::{WireContourOp, WireSide};

const SIDES: [&str; 2] = ["Die - the wire runs inside the profile", "Punch - the wire runs outside the profile"];

pub(super) fn fields(ui: &mut Ui, world: &World, params: &mut OpParams, cx: &mut Cx<'_>) {
    let (u, tab, op_ctx) = (cx.u, cx.tab, cx.op_ctx);
    let OpParams::WireContour(p) = params else {
        return;
    };
    let WireContourOp { top_z, bottom_z, condition, side, start } = p;
    if tab == SEC_GEO {
        let mut si = usize::from(*side == WireSide::Punch);
        prop_row(ui, "Side", |ui| combo(ui, "wire.side", &mut si, &SIDES));
        *side = if si == 1 { WireSide::Punch } else { WireSide::Die };
        let mut stated = start.is_some();
        check_row(ui, "Start hole stated", &mut stated);
        match (stated, start.as_mut()) {
            (true, Some(s)) => {
                prop_row(ui, "Start hole X", |ui| num(ui, u, &mut s[0]));
                prop_row(ui, "Start hole Y", |ui| num(ui, u, &mut s[1]));
            }
            (true, None) => *start = Some([0.0, 0.0]),
            (false, _) => *start = None,
        }
        hint(ui, "pick the one closed profile to cut; the wire is threaded at the start hole - inside the profile for a die, outside it for a punch");
    }
    if tab == SEC_HEIGHTS {
        height_editor(ui, "Top of workpiece", world, op_ctx.as_deref(), top_z, "wire.top");
        height_editor(ui, "Bottom of workpiece", world, op_ctx.as_deref(), bottom_z, "wire.bottom");
        hint(ui, "the wire cuts through from top to bottom in one pass; its path is programmed on the bottom plane");
    }
    if tab == SEC_PASSES {
        // The conditions the op's machine states; one it no longer
        // states stays visible, named as such, until another is picked.
        let ids = operation::wire_op::conditions_of(world, op_ctx.as_deref());
        let mut names: Vec<String> = vec!["(none chosen)".into()];
        names.extend(ids.iter().cloned());
        let mut idx = ids.iter().position(|i| i == condition).map_or(0, |i| i + 1);
        if idx == 0 && !condition.is_empty() {
            names.push(format!("{condition} (not stated by the machine)"));
            idx = names.len() - 1;
        }
        let was = idx;
        prop_row(ui, "Condition", |ui| combo_owned(ui, "wire.condition", &mut idx, &names));
        if idx != was {
            *condition = idx.checked_sub(1).and_then(|i| ids.get(i)).cloned().unwrap_or_default();
        }
        hint(ui, "the machine's EDM condition states the wire diameter and the spark gap; the path stands wire radius + gap off the profile");
    }
}
