//! The operation page: the Wire Contour's fields (#532) - the side and
//! the start hole on Geometry, the two workpiece planes on Heights, the
//! machine's condition on Passes, and the taper angle (#536) and the
//! 4-axis block (#537: the pairing rule, the twist tolerance and the
//! sync pairs as numbers) on Geometry. Every value edits the page draft;
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
use camcraft_app::operation::{WireContourOp, WireFourAxis, WirePairingRule, WireSide, WireSyncPair};

const SIDES: [&str; 4] = [
    "Die - the wire runs inside the profile",
    "Punch - the wire runs outside the profile",
    "Left - an open run's kerf left of travel",
    "Right - an open run's kerf right of travel",
];
const SIDE_OF: [WireSide; 4] = [WireSide::Die, WireSide::Punch, WireSide::Left, WireSide::Right];
const PAIRINGS: [&str; 2] = ["By arc length from the start hole", "By sync points"];

/// The 4-axis block (#537): the rule, the tolerance and the sync pairs.
fn four_axis_fields(ui: &mut Ui, u: Unit, f: &mut WireFourAxis) {
    let mut pi = usize::from(f.pairing == WirePairingRule::SyncPoints);
    prop_row(ui, "Pairing", |ui| combo(ui, "wire.pairing", &mut pi, &PAIRINGS));
    f.pairing = if pi == 1 { WirePairingRule::SyncPoints } else { WirePairingRule::ArcLength };
    prop_row(ui, "Twist tolerance", |ui| num(ui, u, &mut f.tolerance));
    hint(ui, "pick two profiles: the upper on the top plane, the lower on the bottom plane; where they turn against each other the wire stands this much farther off them at most, to keep its kerf out of the surface between");
    if f.pairing != WirePairingRule::SyncPoints {
        return;
    }
    let mut drop = None;
    for (i, s) in f.sync.iter_mut().enumerate() {
        for (which, q) in [("upper", &mut s.upper), ("lower", &mut s.lower)] {
            match q {
                Some(xy) => {
                    prop_row(ui, &format!("Sync {} {which} X", i + 1), |ui| num(ui, u, &mut xy[0]));
                    prop_row(ui, &format!("Sync {} {which} Y", i + 1), |ui| num(ui, u, &mut xy[1]));
                }
                None => {
                    if ui.button(format!("State sync {} {which} point", i + 1)).clicked() {
                        *q = Some([0.0, 0.0]);
                    }
                }
            }
        }
        if ui.button(format!("Remove sync pair {}", i + 1)).clicked() {
            drop = Some(i);
        }
    }
    if let Some(i) = drop {
        f.sync.remove(i);
    }
    if ui.button("Add sync pair").clicked() {
        f.sync.push(WireSyncPair { upper: Some([0.0, 0.0]), lower: Some([0.0, 0.0]) });
    }
    hint(ui, "each sync pair is one point of the upper profile and the point of the lower profile the wire passes through with it; between pairs the profiles are walked at the same share of their length");
}

pub(super) fn fields(ui: &mut Ui, world: &World, params: &mut OpParams, cx: &mut Cx<'_>) {
    let (u, tab, op_ctx) = (cx.u, cx.tab, cx.op_ctx);
    let OpParams::WireContour(p) = params else {
        return;
    };
    let WireContourOp { top_z, bottom_z, condition, side, start, burnout, clearance, taper_deg, four_axis, .. } = p;

    if tab == SEC_GEO {
        let mut si = SIDE_OF.iter().position(|s| s == side).unwrap_or(0);
        prop_row(ui, "Side", |ui| combo(ui, "wire.side", &mut si, &SIDES));
        *side = SIDE_OF.get(si).copied().unwrap_or(WireSide::Die);
        prop_row(ui, "Run-out clearance", |ui| num(ui, u, clearance));
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
        hint(ui, "or pick one open run reaching the stock's edge at both ends (a cut-off, a slot): left / right puts the kerf beside its travel, threaded outside the stock the run-out clearance off its edge");
        // #536: a constant taper, degrees from vertical.
        prop_row(ui, "Taper angle", |ui| {
            ui.add(DragValue::new(taper_deg).speed(0.1).range(-45.0..=45.0).suffix("°"));
        });
        hint(ui, "0 cuts vertical; a positive taper widens the profile downward from the top plane (a die's relief), a negative one narrows it - the machine's edm.max_taper_deg bounds it");
        // #537: two profiles paired into a ruled surface.
        let mut on = four_axis.is_some();
        check_row(ui, "4-axis (upper and lower profiles)", &mut on);
        match (on, four_axis.as_mut()) {
            (true, Some(f)) => four_axis_fields(ui, u, f),
            (true, None) => *four_axis = Some(WireFourAxis::default()),
            (false, _) => *four_axis = None,
        }
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
        // #540: no-core burnout - the pocket's interior burnt out by
        // offset rings, the wire at least the clearance from the walls.
        let mut on = burnout.is_some();
        check_row(ui, "Burn out the pocket (no core)", &mut on);
        match (on, burnout.as_mut()) {
            (true, Some(c)) => {
                prop_row(ui, "Wall clearance", |ui| num(ui, u, c));
            }
            (true, None) => *burnout = Some(0.5),
            (false, _) => *burnout = None,
        }
        hint(ui, "a burnout leaves no slug to drop; the wall clearance may not be under the kerf offset (wire radius + gap)");
    }
}
