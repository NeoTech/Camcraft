//! The post's user options (issue #34): today a lathe post's thread
//! output - "Threading: G76 cycle / G32 passes" (Fanuc Lathe) or "G76
//! cycle / G33 passes" (Tormach PathPilot Lathe, #612) - shown under the
//! post picker while such a post is the document's. A pick queues the
//! undoable `SetPostOptions`; the widget only reads the singleton.
//! Provenance: our own panel; the choice's meaning is the post's
//! (assets/posts/fanuc-lathe.cps, assets/posts/tormach-lathe.cps).

use camcraft_app::posting;
use camcraft_app::schema::{PostOptions, PostRef, ThreadingOutput};
use egui::Ui;

use crate::state::{CommandKind, State};

/// The pass word a lathe post writes when it does not write the cycle:
/// G32 (Fanuc) or G33 (LinuxCNC / PathPilot); `None` for a post with no
/// threading option.
fn pass_word(post: &PostRef) -> Option<&'static str> {
    match post {
        PostRef::Named(n) if n == "Fanuc Lathe" => Some("G32"),
        PostRef::Named(n) if n == "Tormach PathPilot Lathe" => Some("G33"),
        _ => None,
    }
}

/// The threading row, when the document posts through a lathe post
/// that reads the option.
pub(super) fn threading_picker(ui: &mut Ui, state: &mut State) {
    let Some(pass) = pass_word(&posting::post_of(&state.world)) else {
        return;
    };
    let now = posting::post_options(&state.world);
    let passes = format!("{pass} passes");
    let label = |t: ThreadingOutput| match t {
        ThreadingOutput::G32Passes => passes.clone(),
        ThreadingOutput::G76Cycle => "G76 cycle".to_owned(),
    };
    let mut pick: Option<ThreadingOutput> = None;
    ui.horizontal(|ui| {
        ui.label("Threading");
        egui::ComboBox::from_id_salt("post_threading")
            .selected_text(label(now.threading))
            .show_ui(ui, |ui| {
                for t in [ThreadingOutput::G76Cycle, ThreadingOutput::G32Passes] {
                    if ui.selectable_label(now.threading == t, label(t)).clicked() && now.threading != t {
                        pick = Some(t);
                    }
                }
            })
            .response
            .on_hover_text(format!("How a thread posts: one G76 multi-pass cycle the control runs, or every pass as {pass} moves"));
    });
    if let Some(t) = pick {
        state.pending.push_back(CommandKind::SetPostOptions(PostOptions { threading: t, ..now }));
    }
}
