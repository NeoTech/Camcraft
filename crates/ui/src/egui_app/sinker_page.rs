//! The operation page: the Sinker Burn's fields (#541) - the burn point
//! and the machine condition on Geometry, the approach plane, work top
//! and cavity floor datum on Heights, the datum and depth on Passes, the
//! servo feed on Feeds. Every value edits the page draft; Apply rides
//! the command history (EditOperationParams, undoable). The full sinker
//! setup and op pages are #548's.
//!
//! Provenance: our own UI over our own sinker parameters (general
//! die-sink EDM practice); no vendor text.

use super::fields::*;
use camcraft_app::operation::{SinkerBurnOp, SinkerDatum};

const DATUMS: [&str; 3] = ["Work top (depth down)", "Cavity floor (stand-off up)", "Electrode tip (down from approach)"];

pub(super) fn fields(ui: &mut Ui, world: &World, params: &mut OpParams, cx: &mut Cx<'_>) {
    let (u, tab, op_ctx) = (cx.u, cx.tab, cx.op_ctx);
    let OpParams::SinkerBurn(p) = params else {
        return;
    };
    let SinkerBurnOp { clearance_z, top_z, floor_z, datum, depth, x, y, feed, condition } = p;
    if tab == SEC_GEO {
        prop_row(ui, "Burn at X", |ui| num(ui, u, x));
        prop_row(ui, "Burn at Y", |ui| num(ui, u, y));
        prop_row(ui, "EDM condition", |ui| ui.add(egui::TextEdit::singleline(condition).desired_width(90.0)));
        hint(ui, "the machine's own condition id - its spark gap places the electrode tip above the floor");
    }
    if tab == SEC_HEIGHTS {
        height_editor(ui, "Approach plane", world, op_ctx.as_deref(), clearance_z, "sinker.clearance");
        height_editor(ui, "Work top", world, op_ctx.as_deref(), top_z, "sinker.top");
        height_editor(ui, "Cavity floor datum", world, op_ctx.as_deref(), floor_z, "sinker.floor");
        hint(ui, "no burn passes the cavity floor datum or the model's floor - such a depth is refused");
    }
    if tab == SEC_PASSES {
        let mut di = match datum {
            SinkerDatum::WorkTop => 0,
            SinkerDatum::CavityFloor => 1,
            SinkerDatum::ElectrodeTip => 2,
        };
        prop_row(ui, "Depth from", |ui| combo(ui, "sinker.datum", &mut di, &DATUMS));
        *datum = match di {
            1 => SinkerDatum::CavityFloor,
            2 => SinkerDatum::ElectrodeTip,
            _ => SinkerDatum::WorkTop,
        };
        prop_row(ui, "Depth", |ui| num(ui, u, depth));
    }
    if tab == SEC_FEEDS {
        prop_row(ui, "Servo feed", |ui| num_feed(ui, u, feed));
    }
}
