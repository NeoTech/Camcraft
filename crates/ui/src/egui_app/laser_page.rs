//! The operation page: the Laser Profile's fields (#4081) - the kerf,
//! compensation, direction, open-run side and arc linearization on
//! Geometry, the sheet's heights on Heights, the tabs on Passes, the
//! leads on Lead in / Lead out, the cut feed, the power and the pierce
//! dwell on Feeds. Posting refuses laser work until the laser post
//! exists. Every value edits the page draft; Apply rides the command
//! history (EditOperationParams, undoable).
//!
//! Provenance: our own UI over our own laser parameters (general
//! laser-cutting practice, RS-274 / ISO 6983); no vendor text.

use super::fields::*;
use camcraft_app::operation::{JetLead, JetSide, LaserOp};

const COMPS: [&str; 2] = ["In computer (offset path)", "In control"];
const DIRS: [&str; 2] = ["Outer CW, holes CCW", "Reversed: outer CCW, holes CW"];
const SIDES: [&str; 3] = ["On the centre line", "Left of travel", "Right of travel"];
const LEADS: [&str; 2] = ["Arc", "Straight"];

pub(super) fn fields(ui: &mut Ui, world: &World, params: &mut OpParams, cx: &mut Cx<'_>) {
    let (u, tab, op_ctx) = (cx.u, cx.tab, cx.op_ctx);
    let OpParams::LaserProfile(p) = params else {
        return;
    };
    let LaserOp {
        clearance_z,
        top_z,
        bottom_z,
        kerf,
        control_comp,
        reversed,
        lead,
        lead_in,
        lead_out,
        open_side,
        tabs,
        feed,
        power,
        pierce_time,
        linearize, picks: _,
    } = p;
    if tab == SEC_GEO {
        auto_opt(ui, "Kerf width", "Laser head", kerf, "the laser head's diameter", 0.2, |ui, v| num(ui, u, v));
        let mut ci = usize::from(*control_comp);
        prop_row(ui, "Compensation", |ui| combo(ui, "laser.comp", &mut ci, &COMPS));
        *control_comp = ci == 1;
        let mut di = usize::from(*reversed);
        prop_row(ui, "Direction", |ui| combo(ui, "laser.dir", &mut di, &DIRS));
        *reversed = di == 1;
        let mut si = match open_side {
            JetSide::Center => 0,
            JetSide::Left => 1,
            JetSide::Right => 2,
        };
        prop_row(ui, "Open run side", |ui| combo(ui, "laser.side", &mut si, &SIDES));
        *open_side = match si {
            1 => JetSide::Left,
            2 => JetSide::Right,
            _ => JetSide::Center,
        };
        let mut chords = linearize.is_some();
        check_row(ui, "Send arcs as lines", &mut chords);
        if chords {
            prop_row(ui, "Line tolerance", |ui| num(ui, u, linearize.get_or_insert(0.01)));
        } else {
            *linearize = None;
        }
        hint(ui, "pick the closed contours and open runs to cut; holes are cut before the outline that frees the part");
    }
    if tab == SEC_HEIGHTS {
        height_editor(ui, "Clearance (beam off)", world, op_ctx.as_deref(), clearance_z, "laser.clearance");
        height_editor(ui, "Top of sheet", world, op_ctx.as_deref(), top_z, "laser.top");
        height_editor(ui, "Bottom of sheet", world, op_ctx.as_deref(), bottom_z, "laser.bottom");
        hint(ui, "one through-cut at the sheet top - no depth steps");
    }
    if tab == SEC_PASSES {
        super::waterjet_page::tab_fields(ui, u, tabs);
    }
    if tab == SEC_LEADS {
        let mut li = usize::from(*lead == JetLead::Straight);
        prop_row(ui, "Lead shape", |ui| combo(ui, "laser.lead", &mut li, &LEADS));
        *lead = if li == 1 { JetLead::Straight } else { JetLead::Arc };
        prop_row(ui, "Lead-in length", |ui| num(ui, u, lead_in));
        prop_row(ui, "Lead-out length", |ui| num(ui, u, lead_out));
        if *lead_in == 0.0 {
            consequence(ui, "zero lead-in can place a pierce at the part edge on the kerf ring");
        } else {
            hint(ui, "the pierce is made in the scrap; a lead with no room is shortened, and a tight hole is pierced at its centre");
        }
    }
    if tab == SEC_FEEDS {
        prop_row(ui, "Cut feed", |ui| num_feed(ui, u, feed));
        prop_row(ui, "Power", |ui| num_unit(ui, power, "%"));
        prop_row(ui, "Pierce time", |ui| num_unit(ui, pierce_time, "s"));
        if !(power.is_finite() && *power > 0.0 && *power <= 100.0) {
            consequence(ui, "the power must be more than 0 and at most 100 % of the source's maximum");
        } else {
            hint(ui, "state the power and the feed for the material and the sheet thickness from the source's cut data");
        }
    }
}
