//! The shell's one command table (#4484): each fixed command's id, label,
//! ribbon group, menu group and shortcut, read by the ribbon, the menu bar
//! and the keyboard layer alike; our own design notes (desktop menu-bar
//! convention), no third-party text.

use egui::{InputState, Key};

use crate::shortcut_keys::chord;

use super::cmd::Cmd;

/// The ribbon's anchor group captions. The Create groups between them come
/// from the operation registry (`ribbon_plan`), not from this table.
pub(in crate::egui_app) const FILE: &str = "File";
pub(in crate::egui_app) const SETUP: &str = "Setup";
pub(in crate::egui_app) const MODIFY: &str = "Modify";
pub(in crate::egui_app) const INSPECT: &str = "Inspect";
pub(in crate::egui_app) const VIEW: &str = "View";

/// The menu bar's top-level menus, in bar order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::egui_app) enum Menu {
    File,
    Edit,
    Setup,
    /// The operation kinds, one submenu for each catalog section, and Run.
    Operation,
    Inspect,
    View,
}

impl Menu {
    pub(in crate::egui_app) const ALL: [Menu; 6] =
        [Menu::File, Menu::Edit, Menu::Setup, Menu::Operation, Menu::Inspect, Menu::View];

    pub(in crate::egui_app) const fn title(self) -> &'static str {
        match self {
            Menu::File => "File",
            Menu::Edit => "Edit",
            Menu::Setup => "Setup",
            Menu::Operation => "Operation",
            Menu::Inspect => "Inspect",
            Menu::View => "View",
        }
    }
}

/// One keyboard shortcut. `None` in a modifier slot means "either state";
/// `typing` lets the key fire while a text field has focus (only the
/// history keys do, the imgui `WantTextInput` law).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::egui_app) struct Shortcut {
    pub key: Key,
    pub command: Option<bool>,
    pub shift: Option<bool>,
    pub alt: Option<bool>,
    pub typing: bool,
    /// The hint after the command key (`chord`), or the whole hint for a
    /// key without the command modifier.
    pub text: &'static str,
}

const fn slot(want: Option<bool>, have: bool) -> bool {
    match want {
        Some(w) => w == have,
        None => true,
    }
}

#[cfg(test)]
const fn meet(a: Option<bool>, b: Option<bool>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

impl Shortcut {
    /// The key in this frame's input, with its modifiers and focus rule.
    pub(in crate::egui_app) fn fires(&self, i: &InputState, typing: bool) -> bool {
        i.key_pressed(self.key)
            && (self.typing || !typing)
            && slot(self.command, i.modifiers.command)
            && slot(self.shift, i.modifiers.shift)
            && slot(self.alt, i.modifiers.alt)
    }

    /// Can one key press fire both shortcuts? The duplicate test's word.
    #[cfg(test)]
    pub(in crate::egui_app) fn overlaps(&self, other: &Shortcut) -> bool {
        self.key == other.key
            && meet(self.command, other.command)
            && meet(self.shift, other.shift)
            && meet(self.alt, other.alt)
    }

    /// The platform hint: `Cmd+S` on macOS, `Ctrl+S` elsewhere, `Del`.
    pub(in crate::egui_app) fn hint(&self) -> String {
        if self.command == Some(true) { chord(self.text) } else { self.text.to_owned() }
    }
}

/// One command of the shell.
#[derive(Clone, Copy, Debug)]
pub(in crate::egui_app) struct Row {
    pub cmd: Cmd,
    /// Stable id (tests, a later command search).
    #[cfg_attr(not(test), allow(dead_code))]
    pub id: &'static str,
    pub label: &'static str,
    /// The ribbon anchor group that shows the command, if any.
    pub ribbon: Option<&'static str>,
    /// The menu that lists the command, if any.
    pub menu: Option<Menu>,
    pub key: Option<Shortcut>,
}

const fn ctrl(key: Key, shift: Option<bool>, text: &'static str) -> Option<Shortcut> {
    Some(Shortcut { key, command: Some(true), shift, alt: None, typing: false, text })
}

const fn bare(key: Key, text: &'static str) -> Option<Shortcut> {
    Some(Shortcut { key, command: Some(false), shift: None, alt: Some(false), typing: false, text })
}

const fn row(
    cmd: Cmd,
    id: &'static str,
    label: &'static str,
    ribbon: Option<&'static str>,
    menu: Option<Menu>,
    key: Option<Shortcut>,
) -> Row {
    Row { cmd, id, label, ribbon, menu, key }
}

/// The table. Row order is ribbon order inside a group and menu order
/// inside a menu. `Cmd::Create` and `Cmd::More` are the registry's
/// (`camcraft_app::operation`), so they have no row here.
pub(in crate::egui_app) const TABLE: &[Row] = &[
    row(Cmd::New, "file.new", "New", Some(FILE), Some(Menu::File), ctrl(Key::N, None, "N")),
    row(Cmd::Model, "file.model", "Model", Some(FILE), Some(Menu::File), ctrl(Key::O, Some(false), "O")),
    row(Cmd::Project, "file.project", "Open", Some(FILE), Some(Menu::File), ctrl(Key::O, Some(true), "Shift+O")),
    row(Cmd::Save, "file.save", "Save", Some(FILE), Some(Menu::File), ctrl(Key::S, Some(false), "S")),
    row(Cmd::SaveAs, "file.save_as", "Save As", Some(FILE), Some(Menu::File), ctrl(Key::S, Some(true), "Shift+S")),
    row(Cmd::Post, "file.post", "Post", Some(FILE), Some(Menu::File), ctrl(Key::P, None, "P")),
    row(Cmd::Quit, "file.quit", "Quit", None, Some(Menu::File), ctrl(Key::Q, None, "Q")),
    row(Cmd::Setups, "setup.setups", "Setups", Some(SETUP), Some(Menu::Setup), None),
    row(Cmd::Stock, "setup.stock", "Stock", Some(SETUP), Some(Menu::Setup), None),
    row(Cmd::Origin, "setup.origin", "Origin", Some(SETUP), Some(Menu::Setup), None),
    row(Cmd::Tools, "setup.tools", "Tools", Some(SETUP), Some(Menu::Setup), None),
    // The workspace tabs carry the machine switch; the row keeps its label.
    row(Cmd::Machine, "setup.machine", "Machine", None, None, None),
    // Undo and redo fire while typing too (the history keys).
    row(
        Cmd::Undo,
        "edit.undo",
        "Undo",
        Some(MODIFY),
        Some(Menu::Edit),
        Some(Shortcut { key: Key::Z, command: Some(true), shift: Some(false), alt: Some(false), typing: true, text: "Z" }),
    ),
    // Ctrl+Y and Ctrl+Shift+Y both redo (imgui law).
    row(
        Cmd::Redo,
        "edit.redo",
        "Redo",
        Some(MODIFY),
        Some(Menu::Edit),
        Some(Shortcut { key: Key::Y, command: Some(true), shift: None, alt: None, typing: true, text: "Y" }),
    ),
    row(Cmd::Run, "operation.run", "Run", Some(MODIFY), Some(Menu::Operation), None),
    row(Cmd::Copy, "edit.copy", "Copy", None, Some(Menu::Edit), ctrl(Key::C, None, "C")),
    row(Cmd::Paste, "edit.paste", "Paste", None, Some(Menu::Edit), ctrl(Key::V, None, "V")),
    row(
        Cmd::Delete,
        "edit.delete",
        "Delete",
        Some(MODIFY),
        Some(Menu::Edit),
        Some(Shortcut { key: Key::Delete, command: None, shift: None, alt: None, typing: false, text: "Del" }),
    ),
    row(Cmd::Frame, "inspect.frame", "Frame", Some(INSPECT), Some(Menu::Inspect), bare(Key::F, "F")),
    row(Cmd::Paths, "inspect.paths", "Paths", Some(INSPECT), Some(Menu::Inspect), None),
    row(Cmd::Simulate, "inspect.simulate", "Simulate", Some(INSPECT), Some(Menu::Inspect), None),
    row(Cmd::Deviate, "inspect.deviate", "Deviate", Some(INSPECT), Some(Menu::Inspect), None),
    row(Cmd::View, "view.panels", "Panels", Some(VIEW), Some(Menu::View), None),
    row(Cmd::Theme, "view.theme", "Theme", Some(VIEW), Some(Menu::View), None),
    // #434: bare P flips perspective/ortho (Ctrl+P is the post lane).
    row(Cmd::Projection, "view.projection", "Toggle projection", None, Some(Menu::View), bare(Key::P, "P")),
];

/// The table row of a fixed command; `None` for the registry's commands.
pub(in crate::egui_app) fn row_of(c: Cmd) -> Option<&'static Row> {
    TABLE.iter().find(|r| r.cmd == c)
}

/// The commands of one ribbon anchor group, in table order.
pub(in crate::egui_app) fn ribbon_group(title: &str) -> Vec<Cmd> {
    TABLE.iter().filter(|r| r.ribbon == Some(title)).map(|r| r.cmd).collect()
}

/// The shortcut hint of a command, if it has a key.
pub(in crate::egui_app) fn key_hint(c: Cmd) -> Option<String> {
    row_of(c).and_then(|r| r.key).map(|k| k.hint())
}

/// A stable id for any command, the registry's included.
#[cfg(test)]
pub(in crate::egui_app) fn id_of(c: Cmd) -> String {
    match c {
        Cmd::Create(i) => match camcraft_app::operation::KindInfo::ALL.get(i) {
            Some(k) => format!("operation.create.{}", k.label),
            None => format!("operation.create.{i}"),
        },
        Cmd::More(s) => format!("operation.section.{}", s.label()),
        other => row_of(other).map_or_else(|| format!("{other:?}"), |r| r.id.to_owned()),
    }
}
