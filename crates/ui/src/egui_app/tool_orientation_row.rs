//! The tool editor's lathe orientation row (#630): the LinuxCNC 1-9
//! code that says which point of a turning insert's or boring bar's
//! nose the operator touches off - the point the lathe posts write.
//! For a grooving blade (#641) the same code names the touched-off
//! corner of its tip edge, and only the OD blade codes are offered.
//! The draft field only; Apply carries it through `ChangeTool` like
//! every other tool number, so it is undoable.
//!
//! #650: the row speaks the ACTIVE setup's machine's turret frame. The
//! draft holds the canonical (chart, front-frame) code the tool stores;
//! the list and the closed box show each code by the number that
//! machine's tool table gives it (a rear turret's OD insert reads 3),
//! and the caption names the frame. A front machine reads as before.
//!
//! Provenance: LinuxCNC Lathe User Information (tool orientation
//! chart, public docs); our own widget.

use egui::Ui;

use camcraft_app::orientation;
use camcraft_app::tools::ToolKind;
use camcraft_app::turret::Turret;

use crate::state::State;

use super::fields::field;

/// The list's entries in the machine's frame: (canonical code, label),
/// in the order that machine numbers them.
pub(crate) fn choices(turret: Turret, blade: bool) -> Vec<(u8, String)> {
    if blade {
        let mut v: Vec<(u8, String)> =
            orientation::BLADE_FITTING.iter().map(|q| (*q, turret.blade_label(*q))).collect();
        v.sort_by_key(|(q, _)| turret.shown(*q));
        v
    } else {
        turret.codes_in_order().iter().map(|q| (*q, turret.label(*q))).collect()
    }
}

/// The closed box's text for a draft code (None = the kind's default).
pub(crate) fn shown_text(turret: Turret, kind: ToolKind, draft: Option<u8>) -> Option<String> {
    let default = orientation::default_for(kind)?;
    let label = |q: u8| if kind == ToolKind::Groove { turret.blade_label(q) } else { turret.label(q) };
    Some(match draft {
        Some(q) => label(q),
        None => format!("default: {}", label(default)),
    })
}

/// The row, shown for the kinds that ride a nose centre.
pub(super) fn orientation_row(ui: &mut Ui, state: &mut State) {
    let kind = state.tool_ui.tool_kind();
    let turret = camcraft_app::setup_mount::turret_of_active(&state.world);
    let Some(shown) = shown_text(turret, kind, state.tool_ui.orientation) else { return };
    let blade = kind == ToolKind::Groove;
    field(ui, if blade { "Touched-off corner" } else { "Lathe orientation" });
    let default_text = shown_text(turret, kind, None).unwrap_or_default();
    egui::ComboBox::from_id_salt("tool_lathe_orientation")
        .selected_text(shown)
        .width(200.0)
        .show_ui(ui, |ui| {
            let none = state.tool_ui.orientation.is_none();
            if ui.selectable_label(none, default_text).clicked() {
                state.tool_ui.orientation = None;
            }
            for (q, text) in choices(turret, blade) {
                let on = state.tool_ui.orientation == Some(q);
                if ui.selectable_label(on, text).clicked() {
                    state.tool_ui.orientation = Some(q);
                }
            }
        });
    ui.label(format!(
        "{} - {}",
        if blade {
            "which corner of the tip edge was touched off; the lathe posts write that corner, not the middle of the blade"
        } else {
            "where the touched-off tip sits on the nose (LinuxCNC tool table chart); the lathe posts write that point, not the nose centre"
        },
        turret.caption(),
    ));
}

#[cfg(test)]
mod tests {
    use camcraft_app::tools::catalog;

    use crate::state::ToolUi;

    #[test]
    fn the_editor_draft_carries_the_orientation_through_apply() {
        let insert = catalog().into_iter().find(|(n, _)| *n == "TN-1600-0R8").map(|(_, t)| t);
        let Some(mut t) = insert else { panic!("catalog insert") };
        assert_eq!(ToolUi::from_tool("TN", &t, true).to_tool().orientation, None, "unstated stays unstated");
        t.orientation = Some(6);
        let mut draft = ToolUi::from_tool("TN", &t, true);
        assert_eq!(draft.to_tool().orientation, Some(6), "a stated code survives an Apply");
        draft.orientation = Some(1);
        assert_eq!(draft.to_tool().orientation, Some(1), "the row's pick is what Apply sends");
    }

    #[test]
    fn a_rear_turret_numbers_the_row_in_its_own_frame() {
        use super::{choices, shown_text};
        use camcraft_app::tools::ToolKind;
        use camcraft_app::turret::Turret;
        // The OD insert's default reads 3 on a rear turret, 2 on a front.
        let rear = shown_text(Turret::Rear, ToolKind::Turning, None);
        assert_eq!(rear.as_deref(), Some("default: 3 - OD, cuts toward the chuck"));
        let front = shown_text(Turret::Front, ToolKind::Turning, None);
        assert_eq!(front.as_deref(), Some("default: 2 - OD, cuts toward the chuck"));
        assert_eq!(shown_text(Turret::Rear, ToolKind::Bore, None).as_deref(), Some("default: 2 - ID, cuts toward the chuck"));
        // Picking the rear list's "3" stores the canonical 2.
        let list = choices(Turret::Rear, false);
        let picked = list.iter().find(|(_, t)| t.starts_with("3 - ")).map(|(q, _)| *q);
        assert_eq!(picked, Some(2));
        let numbers: Vec<String> = list.iter().map(|(_, t)| t[..1].to_owned()).collect();
        assert_eq!(numbers.concat(), "123456789", "the list runs in the machine's own numbers");
        // An OD thread tip (canonical 6) reads 8 on the rear turret.
        assert_eq!(shown_text(Turret::Rear, ToolKind::Turning, Some(6)).as_deref(), Some("8 - points -X (neutral OD)"));
        // Blades: the rear corners are 3 / 4 / 8.
        let blade: Vec<String> = choices(Turret::Rear, true).into_iter().map(|(_, t)| t).collect();
        assert_eq!(blade, ["3 - left corner (-Z, toward the chuck)", "4 - right corner (+Z, toward the free end)", "8 - width centre"]);
        assert_eq!(
            shown_text(Turret::Front, ToolKind::Groove, None).as_deref(),
            Some("default: 2 - left corner (-Z, toward the chuck)")
        );
        assert_eq!(shown_text(Turret::Rear, ToolKind::EndMill, None), None);
    }
}
