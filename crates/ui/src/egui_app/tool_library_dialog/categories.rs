//! #3897 (U6 of #3663): the tool categories on the left of the tool list,
//! as in the #3663 mock-up: one category for each family of tool kinds,
//! grouped under Milling, Hole making, Turning and Other, each with the
//! count of the shown tools. A pick is view state; it filters the rows.
//! #4335 (U9): every category shows, also one without a tool, and its
//! count is the count of the rows that match the search.
//! Provenance: our own grouping of our own tool kinds; no HSMWorks
//! material involved.

use egui::{RichText, Ui};

use camcraft_app::tools::{Tool, ToolKind};

use crate::state::ToolCategory;

/// The groups and their categories, in the order the column shows them.
pub(crate) const GROUPS: [(&str, &[ToolCategory]); 4] = [
    (
        "Milling",
        &[
            ToolCategory::FlatEndMills,
            ToolCategory::BallNose,
            ToolCategory::BullNose,
            ToolCategory::ChamferMills,
            ToolCategory::ThreadSlotForm,
        ],
    ),
    ("Hole making", &[ToolCategory::Drills, ToolCategory::Taps, ToolCategory::BoringBars]),
    ("Turning", &[ToolCategory::Turning]),
    ("Other", &[ToolCategory::Probes, ToolCategory::BeamAndWire]),
];

/// The label of the row that shows every category.
pub(crate) const ALL: &str = "All tools";

/// The category of each tool kind. A new kind must name its category here.
#[must_use]
pub(crate) fn category_of(kind: ToolKind) -> ToolCategory {
    match kind {
        ToolKind::EndMill => ToolCategory::FlatEndMills,
        ToolKind::BallNose => ToolCategory::BallNose,
        ToolKind::BullNose => ToolCategory::BullNose,
        ToolKind::Taper => ToolCategory::ChamferMills,
        ToolKind::ThreadMill
        | ToolKind::ThreadMillSingle
        | ToolKind::TSlot
        | ToolKind::Keyseat
        | ToolKind::Dovetail
        | ToolKind::Lollipop
        // #3369: the form mill joins the form cutters of this family.
        | ToolKind::FormMill => ToolCategory::ThreadSlotForm,
        ToolKind::Drill => ToolCategory::Drills,
        ToolKind::Tap => ToolCategory::Taps,
        ToolKind::Bore => ToolCategory::BoringBars,
        ToolKind::Turning | ToolKind::Groove | ToolKind::Thread | ToolKind::Form => ToolCategory::Turning,
        ToolKind::Probe => ToolCategory::Probes,
        ToolKind::Torch | ToolKind::Waterjet | ToolKind::Laser | ToolKind::Electrode | ToolKind::Wire => ToolCategory::BeamAndWire,
    }
}

/// The name the column shows for a category.
#[must_use]
pub(crate) fn label(c: ToolCategory) -> &'static str {
    match c {
        ToolCategory::FlatEndMills => "Flat end mills",
        ToolCategory::BallNose => "Ball nose",
        ToolCategory::BullNose => "Bull nose",
        ToolCategory::ChamferMills => "Chamfer mills",
        ToolCategory::ThreadSlotForm => "Thread, T-slot, undercut",
        ToolCategory::Drills => "Drills",
        ToolCategory::Taps => "Taps",
        ToolCategory::BoringBars => "Boring bars",
        ToolCategory::Turning => "Inserts and blades",
        ToolCategory::Probes => "Touch probes",
        ToolCategory::BeamAndWire => "Wire, electrode, torch, jet",
    }
}

/// The column: "All tools", then each group with all its categories and
/// the count of each (`counts`; `total` for all). Gives the new pick when
/// the user clicks a row.
pub(super) fn draw(ui: &mut Ui, counts: &[(ToolCategory, usize)], total: usize, picked: Option<ToolCategory>) -> Option<Option<ToolCategory>> {
    let count = |c: ToolCategory| counts.iter().find(|(k, _)| *k == c).map_or(0, |(_, n)| *n);
    let mut out = None;
    if row(ui, picked.is_none(), ALL, total) {
        out = Some(None);
    }
    for (group, cats) in GROUPS {
        ui.add_space(4.0);
        ui.label(RichText::new(group.to_uppercase()).small().weak());
        for c in cats {
            if row(ui, picked == Some(*c), label(*c), count(*c)) {
                out = Some(Some(*c));
            }
        }
    }
    out
}

/// One category row: the name as a selectable label, the count at the
/// column's right (none for an empty category).
fn row(ui: &mut Ui, selected: bool, text: &str, n: usize) -> bool {
    ui.horizontal(|ui| {
        let clicked = ui.selectable_label(selected, text).clicked();
        if n > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.weak(n.to_string()));
        }
        clicked
    })
    .inner
}

/// Does `t` pass the category pick?
#[must_use]
pub(super) fn passes(picked: Option<ToolCategory>, t: &Tool) -> bool {
    picked.is_none_or(|c| category_of(t.kind) == c)
}
