//! The ribbon's command table: the cells, the group plan, captions and
//! tips, split from panes.rs (#4393, the 400-line law); our own design notes.

use camcraft_app::operation;

use crate::machine_modes;
use crate::shortcut_keys::chord;
use crate::state::State;

use super::TYPES;
use super::gates::{NO_STOCK_TIP, browser_refusal, has_stock, no_tool_for, tool_law_note};

/// One ribbon cell: command identity plus the state it reads for enabled,
/// toggled, label and tip. The table is the whole group layout - command
/// for command since M6.2 divided the Create row into the catalog's
/// family sections; the only hand-typed group left is the anchors below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::egui_app) enum Cmd {
    New,
    Model,
    Project,
    Save,
    SaveAs,
    Post,
    /// The setups door (#322): opens the Setups page, the one place a
    /// document's first setup is created - a fresh document has no
    /// other way there.
    Setups,
    Stock,
    Origin,
    Tools,
    /// The machine door (the ribbon law): one cell that opens the
    /// workspaces of `machine_modes::MACHINE_MODES` and wears the
    /// current one's plain caption. Switching mounts the kind's
    /// built-in reference through `LoadBuiltinMachine`, so the change
    /// rides undo like every other setup move.
    Machine,
    Create(usize),
    /// The dropdown button of a folded section (M6.2): it carries the
    /// section's label, never a faceless chevron - the cell's whole
    /// caption is that label plus the combo's own arrow glyph.
    More(operation::Section),
    Undo,
    Redo,
    Run,
    Delete,
    Frame,
    Paths,
    Simulate,
    Deviate,
    /// The View door: one cell whose list toggles the dock's closable
    /// panels and resets the layout (layout.rs). View state, no command.
    View,
    /// The Theme door (#441): the user's look - the built-in and
    /// discovered themes plus the colour-vision remap - right of the
    /// View door. Opening it is view state; a pick queues `SelectTheme`
    /// / `SelectColorVision`, kept as a user preference (#877).
    Theme,
}

/// The four anchor groups: file work opens the row, setup follows, and
/// the run/judge groups close it. The middle is the registry's (below):
/// the old hand-typed eight-cell Create group was M1.4.3's debt, home
/// to kinds 0..=7 while the table grew - the registry counts the
/// kinds, this comment never must.
const FILE: (&str, &[Cmd]) =
    ("File", &[Cmd::New, Cmd::Model, Cmd::Project, Cmd::Save, Cmd::SaveAs, Cmd::Post]);
const SETUP: (&str, &[Cmd]) =
    ("Setup", &[Cmd::Setups, Cmd::Stock, Cmd::Origin, Cmd::Tools]);

/// The bar's whole op-shortcut budget (the ribbon law): at most this
/// many Create cells stand inline, and they are the user's pins. The
/// rest of the catalog lives behind the section doors at every width.
pub(super) const MAX_PINS: usize = 4;
const MODIFY: (&str, &[Cmd]) = ("Modify", &[Cmd::Undo, Cmd::Redo, Cmd::Run, Cmd::Delete]);
const INSPECT: (&str, &[Cmd]) =
    ("Inspect", &[Cmd::Frame, Cmd::Paths, Cmd::Simulate, Cmd::Deviate]);
const VIEW: (&str, &[Cmd]) = ("View", &[Cmd::View, Cmd::Theme]);

/// One ribbon group before layout: its caption and every cell it would
/// show in a row that never ran out of width. `section` marks the
/// registry-seeded Create groups - only those fold into a dropdown.
#[derive(Clone, Debug)]
pub(in crate::egui_app) struct Group {
    pub title: &'static str,
    pub cells: Vec<Cmd>,
    pub section: Option<operation::Section>,
}

/// The whole ribbon as data (M6.2's home law): the anchors, the user's
/// pinned kinds, then every catalog section the document's machine can
/// run. Hiding, not darkening - the mill document's table carries no
/// Turning cells to grey out; `KindInfo::admits` is the single word
/// this filter and the command door share (M13.5 B: per kind, so the
/// lathe's Drilling door fronts the one axial Drill cell). The
/// registry's `section` column is the only seed, and the row order is
/// the product order, so a section's cells list the kinds in table
/// sequence.
#[must_use]
pub(in crate::egui_app) fn ribbon_plan(
    machine: camcraft_app::machine::Kind,
    pinned: &[usize],
) -> Vec<Group> {
    ribbon_geometry(machine, pinned)
}

fn ribbon_geometry(
    machine: camcraft_app::machine::Kind,
    pinned: &[usize],
) -> Vec<Group> {
    let mut plan = vec![
        Group { title: FILE.0, cells: FILE.1.to_vec(), section: None },
        Group { title: SETUP.0, cells: SETUP.1.to_vec(), section: None },
    ];
    // Pinned kinds first among the Create groups, in pin order - and
    // only those the machine can run, so the hiding law has one more
    // surface saying the same thing (a pin to other work survives in
    // the list unread, waiting for the mill-turn document that shows
    // it again). These are the bar's only inline op cells, and their
    // number is its budget: the promise of a pin is that it stands
    // where the user put it, so the fold law starts Pinned inline,
    // unfolded, at most MAX_PINS deep.
    let pins: Vec<usize> = pinned
        .iter()
        .copied()
        .filter(|i| {
            operation::KindInfo::ALL
                .get(*i)
                .is_some_and(|k| k.admits(machine))
        })
        // The bar's op budget (the ribbon law): the first MAX_PINS
        // admitted pins stand inline. A document whose list outgrew
        // the cap - pinned on an earlier build, or one that ignored
        // it - still renders, it just shows the first four; the rest
        // wait behind their section doors, listed and reachable.
        .take(MAX_PINS)
        .collect();
    if !pins.is_empty() {
        plan.push(Group {
            title: "Pinned",
            cells: pins.iter().copied().map(Cmd::Create).collect(),
            section: None,
        });
    }
    plan.extend(operation::shown_for(machine).into_iter().filter_map(|s| {
        // One home per kind (the ribbon law's corollary): a pinned
        // kind stands on the bar and leaves its section's list - the
        // door holds "everything of this family not already in
        // reach", never a duplicate of what the user can see.
        let cells: Vec<Cmd> = s
            .kinds_for(machine)
            .into_iter()
            .filter(|i| !pins.contains(i))
            .map(Cmd::Create)
            .collect();
        (!cells.is_empty()).then_some(Group {
            title: s.label(),
            cells,
            section: Some(s),
        })
    }));
    plan.push(Group { title: MODIFY.0, cells: MODIFY.1.to_vec(), section: None });
    plan.push(Group { title: INSPECT.0, cells: INSPECT.1.to_vec(), section: None });
    plan.push(Group { title: VIEW.0, cells: VIEW.1.to_vec(), section: None });
    plan
}


pub(super) fn label_of(c: Cmd) -> &'static str {
    match c {
        Cmd::New => "New",
        Cmd::Model => "Model",
        Cmd::Project => "Open",
        Cmd::Save => "Save",
        Cmd::SaveAs => "Save As",
        Cmd::Post => "Post",
        Cmd::Setups => "Setups",
        Cmd::Stock => "Stock",
        Cmd::Origin => "Origin",
        Cmd::Tools => "Tools",
        Cmd::Machine => "Machine",
        Cmd::Create(_) => "op", // formatted by the group row; ribbon shows TYPES
        Cmd::Undo => "Undo",
        Cmd::Redo => "Redo",
        Cmd::Run => "Run",
        Cmd::Delete => "Delete",
        Cmd::Frame => "Frame",
        Cmd::Paths => "Paths",
        Cmd::Simulate => "Simulate",
        Cmd::Deviate => "Deviate",
        Cmd::View => "Panels",
        Cmd::Theme => "Theme",
        Cmd::More(s) => s.label(),
    }
}

/// The section door's short ribbon word: the cell is ~43 px wide at a
/// 1280-point window (the floor), so "Milling 2D" would not fit; the
/// group caption under it and the tip carry the full name, and the
/// dropdown lists every kind behind it. Plain English, deliberate -
/// never a cut-off one (#948). The section's own name stays on the
/// group caption ("MILLING 2D"), so the cell can say just the part
/// that differs.
fn door_label(s: operation::Section) -> &'static str {
    match s {
        operation::Section::Mill2D => "2D",
        operation::Section::Mill3D => "3D",
        operation::Section::Drilling => "Drill",
        operation::Section::MultiAxis => "Multi",
        other => other.label(),
    }
}

pub(in crate::egui_app) fn text_of(c: Cmd, state: &State) -> String {
    match c {
        // The overflow button wears the section's name plus an
        // ellipsis - "Milling 3D…" reads as "the rest of this is in
        // here", which is exactly the law the dropdown keeps (M6.2:
        // a labeled door, never a faceless chevron). The machine door
        // wears the same invitation: the ellipsis says "a list opens".
        Cmd::More(s) => door_label(s).to_owned(),
        Cmd::Machine => machine_modes::short_caption(camcraft_app::machine::kind_of(&state.world)).to_owned(),
        Cmd::View | Cmd::Theme => label_of(c).to_owned(),
        // The ribbon caption is registry data (M5.0.2) - imgui's strings,
        // same order as the Create cells below. The list here is built
        // from the table, so an out-of-range index means a table bug:
        // stop in debug, name it and stay on screen in release.
        Cmd::Create(i) => match camcraft_app::operation::KindInfo::ALL.get(i) {
            Some(k) => k.caption.to_owned(),
            None => {
                debug_assert!(false, "Create({i}) past the kind table");
                diag_warn!("ribbon: unknown op kind {i}");
                "op".to_owned()
            }
        },
        other => label_of(other).to_owned(),
    }
}

/// The tip table, `tips_match_the_imgui_table`'s guard. The strings
/// were imgui's verbatim (ribbon.rs:140-222) until M6.2 restructured
/// the ribbon; the promise retired that day (RECOMMENDED.md) and the
/// table became the product's own - the test still pins it row for
/// row, because a tip is a promise and promises want a guard, not
/// because the wording is borrowed. Every advertised key is live:
/// `shortcuts` implements them all, and the pin gesture names itself.
pub(in crate::egui_app) fn tip_of(c: Cmd, state: &State) -> String {
    if let Some(why) = browser_refusal(c, state) {
        return why.into();
    }
    match c {
        Cmd::New => format!("New document\t{}", chord("N")),
        Cmd::Model => format!("Open a mesh model\t{}", chord("O")),
        Cmd::Project => format!("Open a project\t{}", chord("Shift+O")),
        Cmd::Save => format!("Save the project\t{}", chord("S")),
        Cmd::SaveAs => format!("Save the project under a new name\t{}", chord("Shift+S")),
        Cmd::Post => format!("Post NC program (validation gate), then review it\t{}", chord("P")),
        Cmd::Setups => "Setups: create, activate and delete the document's setups".into(),
        Cmd::Stock => "Stock definition".into(),
        Cmd::Origin => "Work offset (WCS)".into(),
        Cmd::Tools => "Tool library".into(),
        // The cell's caption already says which workspace is running;
        // the tip says what a click does, and how to undo it.
        Cmd::Machine => format!(
            "Switch the workspace: {} (mounts the kind's built-in machine, undoable)",
            machine_modes::modes_phrase()
        ),
        Cmd::Create(i) => {
            // The tool-less swap first (imgui's law, ribbon.rs:138):
            // an empty tool list is the deeper trouble - nothing
            // lights until a tool exists, whatever the machine is.
            // Then the family truth (M5.2 stage A): since M6.2 the
            // plan already hides what the machine cannot run, so the
            // darkening branch is defense in depth for any surface
            // that renders a plan the door would refuse - it says so
            // and names the fix, the same pair CreateOperation
            // enforces at the command. The pin hint rides every live
            // tip: the gesture a ribbon cell answers to beyond the
            // click, so the tip table is where it belongs.
            if no_tool_for(i, state) {
                "No tool yet - add one (Setup > Tools)".into()
            } else {
                let machine = camcraft_app::machine::kind_of(&state.world);
                let dark = camcraft_app::operation::KindInfo::ALL
                    .get(i)
                    .is_some_and(|k| !k.admits(machine));
                if dark {
                    let f = camcraft_app::operation::KindInfo::ALL[i].family();
                    let (fword, mword) = (f.work(), f.machine());
                    // #670: plasma work has no mill-turn to fall back on.
                    let or_mt = if matches!(f, camcraft_app::operation::Family::Plasma | camcraft_app::operation::Family::WireEdm | camcraft_app::operation::Family::Sinker | camcraft_app::operation::Family::Waterjet) { "" } else { " or a mill-turn" };
                    format!(
                        "The document's machine is a {} - {fword} operations do not drive it; mount a {mword}{or_mt} (Setup > Machine)",
                        machine.word()
                    )
                } else if !has_stock(state) {
                    NO_STOCK_TIP.into()
                } else if let Some(reason) = tool_law_note(i, state) {
                    reason
                } else if i == 0 {
                    "Add a Face mill operation\tRight-click to pin/unpin".into()
                } else {
                    format!("Add a {} operation\tRight-click to pin/unpin", TYPES[i])
                }
            }
        }
        Cmd::More(s) => format!("The rest of the {} kinds", s.label()),
        Cmd::Undo => format!("Undo\t{}", chord("Z")),
        Cmd::Redo => format!("Redo\t{}", chord("Y")),
        Cmd::Run => "Generate the selected operation".into(),
        Cmd::Delete => "Delete the selection\tDel".into(),
        Cmd::Frame => "Frame all\tF".into(),
        Cmd::Paths => if state.show_all_paths {
            "Hide every toolpath".into()
        } else {
            "Show every cached toolpath".into()
        },
        Cmd::Simulate => "Simulate tool moves (transport bar)".into(),
        Cmd::Deviate => {
            "Judge the program's stock against the model (deviation map)".into()
        }
        Cmd::View => "Show or hide the browser, properties and machines panes, or reset the layout".into(),
        Cmd::Theme => "Pick the document's theme and colour-vision mode (undoable)".into(),
    }
}
