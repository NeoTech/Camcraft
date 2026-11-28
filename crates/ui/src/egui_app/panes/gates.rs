//! The ribbon's gates: enabled, toggled, the refusal of a dark Create cell
//! and its route, and the cell glyphs, split from panes.rs (#4393); our own design notes.

use camcraft_app::model;
use camcraft_app::operation;

use crate::actions;
use crate::state::{CommandKind, SetupPage, State, UiTarget};

use crate::egui_app::icons::Icon;
use crate::egui_app::op_icons::op_icon;

use super::cmd::Cmd;

/// #1431: a File cell the browser build cannot run - darkened, its tip
/// the reason (`browser_files`), the same words the command refuses with.
pub(in crate::egui_app) fn browser_refusal(c: Cmd, state: &State) -> Option<&'static str> {
    if !state.browser_build {
        return None;
    }
    let action = match c {
        Cmd::Model => CommandKind::OpenModelDialog,
        Cmd::Project => CommandKind::OpenProjectDialog,
        Cmd::Post => CommandKind::PostProgramDialog,
        _ => return None,
    };
    crate::browser_files::refusal(&action, camcraft_app::posting::post_options(&state.world).per_op)
}

pub(in crate::egui_app) fn enabled(c: Cmd, state: &State) -> bool {
    if browser_refusal(c, state).is_some() {
        return false;
    }
    match c {
        Cmd::Undo => state.history.can_undo(),
        Cmd::Redo => state.history.can_redo(),
        Cmd::Run => state.selected_operation().is_some(),
        Cmd::Delete => state.has_selection(),
        // The family gate by pairing (M3.2 E, M5.2 A) is the command
        // door's law; since M6.2 the ribbon's plan already hides
        // what the machine cannot run, so this half stands as defense
        // in depth - the live gate is the tool list: a kind with no
        // tool cannot cut whatever its section says.
        Cmd::Create(i) => {
            let machine = camcraft_app::machine::kind_of(&state.world);
            let kind_admits = camcraft_app::operation::KindInfo::ALL
                .get(i)
                .is_none_or(|k| k.admits(machine));
            // The tool law (M10.1 B): darken, never hide - the cell
            // stays to explain itself, the fix is a library click away.
            // #216: every ribbon-made op machines the active setup's stock
            // (`SrcRef::Stock`) - without one the door refuses, so the
            // cell darkens and says so instead of failing on the click.
            kind_admits && !no_tool_for(i, state) && has_stock(state) && tool_law_note(i, state).is_none()
        }
        // A dropdown button is live whenever it is on the ribbon; the
        // kinds behind it carry their own gates.
        Cmd::More(_) => true,
        Cmd::Simulate => state.has_toolpaths(),
        // The same gate as Simulate: a program with no generated move
        // has nothing to judge - the cell stays dark until there is.
        Cmd::Deviate => state.has_toolpaths(),
        Cmd::Copy => state.selected_operation().is_some(),
        Cmd::Paste => state.operation_clipboard.is_some(),
        _ => true,
    }
}

/// Why a disabled menu row is disabled (#4484), for the gates that have
/// no refusal of their own; the Create rows and the browser build speak
/// through `tip_of`. `None` when the command is live.
pub(in crate::egui_app) fn disabled_reason(c: Cmd, state: &State) -> Option<&'static str> {
    if enabled(c, state) {
        return None;
    }
    match c {
        Cmd::Undo => Some("Nothing to undo"),
        Cmd::Redo => Some("Nothing to redo"),
        Cmd::Run | Cmd::Copy => Some("Select an operation first"),
        Cmd::Delete => Some("Select something first"),
        Cmd::Simulate | Cmd::Deviate => Some("Generate a toolpath first"),
        Cmd::Paste => Some("Copy an operation first"),
        _ => None,
    }
}

/// The refusal a gated Create answer - a cell on the bar or a row
/// behind a door - speaks when it is clicked anyway: named, loud on
/// the status line, pointing at the fix. It reads the same two truths
/// `enabled` reads, in the same order of depth (no tool first, then
/// the machine's family), phrased as `actions::add_operation` phrases
/// its own refusal. Silence on a click reads as a dead button - the
/// law `State::apply_operation_params` already keeps for the pane -
/// so a gate that can be clicked must be able to answer. `None` when
/// the command is actually live, so no caller overwrites a status an
/// enabled command already owns.
fn gate_note(c: Cmd, state: &State) -> Option<String> {
    let Cmd::Create(i) = c else { return None };
    if enabled(c, state) {
        return None;
    }
    if no_tool_for(i, state) {
        return Some(String::from(
            "cannot add an operation: no tool in the document - add one (Setup > Tools)",
        ));
    }
    let machine = camcraft_app::machine::kind_of(&state.world);
    let dark = camcraft_app::operation::KindInfo::ALL
        .get(i)
        .filter(|k| !k.admits(machine));
    match dark {
        Some(k) => {
            let f = k.family();
            let (fword, mword) = (f.work(), f.machine());
            let or_mt = if matches!(f, camcraft_app::operation::Family::Plasma | camcraft_app::operation::Family::WireEdm | camcraft_app::operation::Family::Sinker | camcraft_app::operation::Family::Waterjet) { "" } else { " or a mill-turn" };
            Some(format!(
                "cannot add a {} operation: {fword} operations do not drive the document's {} - mount a {mword}{or_mt} (Setup > Machine)",
                k.label,
                machine.word(),
            ))
        }
        None if !has_stock(state) => Some(String::from(
            "cannot add an operation: no stock in the setup - set up stock first (Setup > Stock)",
        )),
        None => tool_law_note(i, state).map(|r| {
            let label = camcraft_app::operation::KindInfo::ALL.get(i).map_or("", |k| k.label);
            format!("cannot add a {label} operation: {r}")
        }),
    }
}

/// Where a refused Create click leads (#191): the place that fixes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::egui_app) enum GateRoute {
    /// No tool, or none that fits the kind: the Tool library page.
    Tools,
    /// The mounted machine cannot run the kind's family: the machine
    /// door's workspace list.
    Machine,
    /// The active setup has no stock (#216): the Stock page.
    Stock,
}

/// The dark Create cell's tip when the setup has no stock (#216).
pub(super) const NO_STOCK_TIP: &str = "No stock yet - set up stock first (Setup > Stock)";

/// Does the active setup have stock? Every ribbon-made operation
/// machines it (`SrcRef::Stock`), so without it `CreateOperation`
/// refuses - the same resolution the command door makes.
pub(super) fn has_stock(state: &State) -> bool {
    let active = camcraft_app::setup::active(&state.world);
    let name = camcraft_app::setup::stock_entity_name(&state.world, active.as_deref());
    model::find_by_name(&state.world, &name).is_some_and(|e| model::is_stock(&state.world, &e))
}

/// The fix for a dark Create cell, when it has one ([`gate_note`]'s
/// cases, in its order).
pub(in crate::egui_app) fn gate_route(c: Cmd, state: &State) -> Option<GateRoute> {
    let Cmd::Create(i) = c else { return None };
    if enabled(c, state) {
        return None;
    }
    if no_tool_for(i, state) {
        return Some(GateRoute::Tools);
    }
    let machine = camcraft_app::machine::kind_of(&state.world);
    if camcraft_app::operation::KindInfo::ALL.get(i).is_some_and(|k| !k.admits(machine)) {
        return Some(GateRoute::Machine);
    }
    if !has_stock(state) {
        return Some(GateRoute::Stock);
    }
    tool_law_note(i, state).map(|_| GateRoute::Tools)
}

/// A dark Create cell was clicked anyway (#191): say why on the status
/// line, then take the user to the fix rather than leave them at the
/// error - the Tool library, or the machine door opened on its list.
/// View state only; nothing is created or queued.
pub(in crate::egui_app) fn follow_gate(c: Cmd, state: &mut State) {
    let Some(why) = gate_note(c, state) else { return };
    match gate_route(c, state) {
        Some(GateRoute::Tools) => {
            actions::target(state, UiTarget::Tool(String::new()));
            state.status = format!("{why} - opened the tool library");
        }
        Some(GateRoute::Stock) => {
            actions::target(state, UiTarget::Stock);
            state.status = format!("{why} - opened the Stock page");
        }
        Some(GateRoute::Machine) => {
            state.ribbon_more = None;
            state.ribbon_view_open = false;
            state.ribbon_theme_open = false;
            state.ribbon_recent_open = None;
            let setup = camcraft_app::setup::active(&state.world).unwrap_or_default();
            actions::target(state, UiTarget::SetupNode(setup, SetupPage::Machine));
            state.status = format!("{why} - opened the Machine page");
        }
        None => state.status = why,
    }
}

/// The tool law's darkening (M10.1 B): the app's own refusal sentence
/// for creating kind `i` against the active setup's tools, or `None`
/// when a fitting tool is there. The predicate lives in app (the same
/// one `CreateOperation` enforces); the ribbon only renders it.
/// The empty-rack gate of a Create cell - waived for a kind that
/// mounts no library tool (the wire EDM's wire is the machine's, #535).
pub(super) fn no_tool_for(i: usize, state: &State) -> bool {
    let needs = camcraft_app::operation::KindInfo::ALL.get(i).is_none_or(|k| k.mounts_tool());
    needs && state.tool_names().is_empty()
}

pub(super) fn tool_law_note(i: usize, state: &State) -> Option<String> {
    camcraft_app::operation::KindInfo::ALL
        .get(i)
        .and_then(|k| camcraft_app::operation::tool_gate::create_refusal(&state.world, k))
}

pub(in crate::egui_app) fn toggled(c: Cmd, state: &State) -> bool {
    match c {
        Cmd::Paths => state.show_all_paths,
        Cmd::Simulate => state.sim.is_some(),
        Cmd::Deviate => state.program_sim.is_some(),
        // The open dropdown's button stands toggled while it is open
        // (M6.2): the accent fill says "this door is the one you
        // opened".
        Cmd::More(s) => state.ribbon_more == Some(s),
        Cmd::Machine => state.ribbon_mode_open,
        Cmd::View => state.ribbon_view_open,
        Cmd::Theme => state.ribbon_theme_open,
        Cmd::Model | Cmd::Project => {
            state.ribbon_recent_open.is_some() && state.ribbon_recent_open == crate::egui_app::recent_drop::kind_of(c)
        }
        _ => false,
    }
}

/// Command to bundled glyph: the same table the retired imgui icons.rs
/// store keyed (and ribbon.rs's `Item.icon` chose per cell), so the ribbon,
/// the browser's `spec.icon` slots and the store name one set of pixels.
pub(in crate::egui_app) fn icon_of(c: Cmd, state: &State) -> Option<Icon> {
    match c {
        Cmd::New => Some(Icon::New),
        Cmd::Model => Some(Icon::OpenModel),
        Cmd::Project => Some(Icon::OpenProject),
        Cmd::Save => Some(Icon::Save),
        Cmd::SaveAs => Some(Icon::Save),
        Cmd::Post => Some(Icon::Post),
        Cmd::Setups => Some(Icon::Setup),
        Cmd::Stock => Some(Icon::Stock),
        Cmd::Origin => Some(Icon::WorkOffset),
        Cmd::Tools => Some(Icon::Tools),
        // Icons stay a UI table (asset names do not belong in the app),
        // but an exhaustive match on the kind, not on the index: a new
        // registry row with no icon here stops at the compiler, never
        // silently borrows the last glyph (M5.0.2 - the old `_ =>` arm
        // was one of the three silent-drift catch-alls).
        Cmd::Create(i) => match camcraft_app::operation::KindInfo::ALL.get(i) {
            Some(k) => Some(op_icon(k.ty)),
            None => {
                debug_assert!(false, "Create({i}) past the kind table");
                diag_warn!("ribbon: unknown op kind {i}");
                Some(Icon::Generate)
            }
        },
        // A section door wears its family's signature glyph (#207: with
        // no pins, glyph-less doors left every Create cell of a fresh
        // install text-only - users read it as missing icons). No new
        // asset: an op glyph that says what the family makes, distinct
        // from the machine door's beside it; the caption still names
        // the section.
        Cmd::More(s) => Some(match s {
            operation::Section::Mill2D => Icon::Pocket,
            operation::Section::Mill3D => Icon::Finish3D,
            operation::Section::Drilling => Icon::Drill,
            operation::Section::MultiAxis => Icon::MultiAxisRough,
            // #661: turning has glyphs of its own now - the facing cut.
            operation::Section::Turning => Icon::Facing,
            // #670: the plasma section cuts 2D profiles - the contour's
            // glyph (the machine door's own choice, no new asset).
            operation::Section::Plasma => Icon::Contour,
            // #532: the wire cuts a profile too - the same glyph the
            // wire EDM's machine door wears (no new asset; the op page
            // and its own glyph are #535).
            operation::Section::WireEdm => Icon::Contour,
            // #541: the sinker burns straight down - the drill's glyph
            // (no new asset).
            operation::Section::Sinker => Icon::Drill,
            // #1883: the waterjet section wears its op's own glyph.
            operation::Section::Waterjet => Icon::WaterjetProfile,
        }),
        // The machine door wears its workspace's glyph (#207), from the
        // existing set: the face mill for Milling, the turned bar for
        // Turning, the rotary bar for Mill-turning.
        Cmd::Machine => Some(match camcraft_app::machine::kind_of(&state.world) {
            camcraft_app::machine::Kind::Lathe => Icon::RotaryContour,
            camcraft_app::machine::Kind::MillTurn => Icon::RotaryParallel,
            camcraft_app::machine::Kind::Mill => Icon::Face,
            // #530: the wire cuts a contour; the sinker burns a pocket.
            camcraft_app::machine::Kind::WireEdm => Icon::Contour,
            camcraft_app::machine::Kind::SinkerEdm => Icon::Pocket,
            // #669: a plasma table cuts 2D profiles - the contour glyph.
            camcraft_app::machine::Kind::Plasma => Icon::Contour,
            camcraft_app::machine::Kind::Waterjet => Icon::ToolWaterjet,
            camcraft_app::machine::Kind::Laser => Icon::ToolLaser,
        }),
        Cmd::Undo => Some(Icon::Undo),
        Cmd::Redo => Some(Icon::Redo),
        Cmd::Run => Some(Icon::Generate),
        Cmd::Delete => Some(Icon::Delete),
        Cmd::Frame => Some(Icon::FrameAll),
        Cmd::Paths => Some(Icon::Toolpaths),
        Cmd::Simulate => Some(Icon::Simulate),
        // The stock glyph, reused (RECOMMENDED.md's no-new-asset law):
        // what the cell shows IS the judged stock.
        Cmd::Deviate => Some(Icon::Stock),
        // The machine door's no-new-asset law: the caption names the
        // door, the ellipsis says a list opens.
        // The Theme door follows the View door: a list door captioned by
        // its own word (no fitting glyph in the bundled set).
        Cmd::View | Cmd::Theme => None,
        // Menu and keyboard only (#4484): no ribbon cell, no glyph.
        Cmd::Copy | Cmd::Paste | Cmd::Quit | Cmd::Projection => None,
    }
}
