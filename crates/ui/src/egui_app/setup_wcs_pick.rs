//! The Setups page's work-offset picker (#196, #342): which work offset
//! the post writes for the setup's one WCS, named by the setup's post
//! (#931, `setup::offset_name`; slice 3 lists the post's whole range). It is only the label -
//! the WCS origin, and every height measured from it, stay put.
//! An unset setup IS `G54` - one `G54` entry, stored unset, so the file
//! format is unchanged (#329). A pick queues the undoable
//! `SetSetupWorkOffset`; nothing here writes the world.
//!
//! Provenance: our own UI over RS-274 / ISO 6983 work-offset selection;
//! no vendor material consulted.

use egui::{ComboBox, Ui};


use crate::state::{CommandKind, State};

/// The entries the combo lists: offsets 1-6 (slice 3 of #931 widens it
/// to the setup's post's maximum).
const ENTRIES: usize = 6;

/// The combo text for a stored pick: the setup's post's name for it,
/// plain - unset reads like an explicit offset 1.
#[must_use]
pub(super) fn pick_text(world: &flecs_ecs::prelude::World, setup: &str, slot: Option<usize>) -> String {
    let n = u32::try_from(slot.unwrap_or(0)).map_or(u32::MAX, |s| s.saturating_add(1));
    camcraft_app::setup::offset_name(world, Some(setup), n)
}

/// A stored pick in the combo's terms: `G54` (slot 0) and unset are one
/// entry, so both read `None` and choosing `G54` on either queues nothing.
fn normalized(slot: Option<usize>) -> Option<usize> {
    slot.filter(|i| *i != 0)
}

/// One setup row's picker: label, combo, and the queued command when the
/// user picks a different entry.
pub(super) fn picker(ui: &mut Ui, state: &mut State, setup: &str) {
    let current = normalized(camcraft_app::setup::work_offset(&state.world, setup));
    let mut chosen = current;
    ui.horizontal(|ui| {
        ui.label("    Work offset");
        ComboBox::from_id_salt(("setup_wcs", setup))
            .selected_text(pick_text(&state.world, setup, current))
            .show_ui(ui, |ui| {
                for i in 0..ENTRIES {
                    let w = pick_text(&state.world, setup, Some(i));
                    ui.selectable_value(&mut chosen, normalized(Some(i)), w);
                }
            })
            .response
            .on_hover_text(
                "Which of G54-G59 the posted program uses for this setup. It only names the \
                 controller's register - the origin stays where it is placed.",
            );
    });
    if chosen != current {
        state.pending.push_back(CommandKind::SetSetupWorkOffset {
            setup: setup.to_owned(),
            slot: chosen,
        });
    }
}
