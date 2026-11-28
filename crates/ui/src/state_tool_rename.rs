//! #4370: the tool library editor's Name field and the views that follow
//! a renamed tool. The typed name is a draft of the window
//! ([`RenameDraft`]); the footer's Apply lands it as one `RenameTool`.
//! A rename keeps the tool's entity, so each view that shows a tool by
//! name (the library editor, the Properties subject and its draft, the
//! tool shape window) follows that entity to its new name - after the
//! editor's Apply, an undo, a redo or an agent's `rename_tool` alike.
//! Provenance: our own view state over the `RenameTool` command; no
//! HSMWorks material involved.

use camcraft_app::model;
use flecs_ecs::prelude::Entity;

use super::{CommandKind, ErrorScope, State, UiTarget};

/// The new name typed for `tool` in the editor's Name field. It is not
/// applied until the footer's Apply.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenameDraft {
    pub tool: String,
    pub text: String,
}

/// The entities of the tools that the views show before a command runs.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ToolViews {
    library: Option<Entity>,
    target: Option<Entity>,
    properties: Option<Entity>,
    shape: Option<Entity>,
}

impl State {
    /// The Name field's text for the open tool `name`: the typed draft,
    /// or the name itself.
    #[must_use]
    pub(crate) fn library_name_text(&self, name: &str) -> String {
        self.tool_library
            .as_ref()
            .and_then(|v| v.rename.as_ref())
            .filter(|r| r.tool == name)
            .map_or_else(|| name.to_owned(), |r| r.text.clone())
    }

    /// #4340: the editor's draft has edits that are not applied - its
    /// fields (`library_fields_changed`) or, #4370, a typed new name.
    /// Duplicate copies the applied tool, so the footer disables it while
    /// this holds (no edit is dropped silently).
    #[must_use]
    pub(crate) fn library_draft_changed(&self) -> bool {
        self.library_fields_changed() || self.library_rename().is_some()
    }

    /// Keep `text` as the typed new name of the open tool `name`.
    pub(crate) fn set_library_name_text(&mut self, name: &str, text: String) {
        if let Some(view) = self.tool_library.as_mut() {
            view.rename = Some(RenameDraft { tool: name.to_owned(), text });
        }
    }

    /// The rename that the editor's Apply lands: `(old, new)` when the
    /// typed name of the open tool differs from its name. An empty name is
    /// a rename too: `RenameTool` refuses it by name.
    #[must_use]
    pub(crate) fn library_rename(&self) -> Option<(String, String)> {
        let view = self.tool_library.as_ref()?;
        let open = view.open_tool()?;
        let draft = view.rename.as_ref().filter(|r| r.tool == open)?;
        let to = draft.text.trim();
        (to != open).then(|| (open.to_owned(), to.to_owned()))
    }

    /// The command handler: validate now; a refusal is said on the Tool
    /// pane by name and the typed name stays for correction.
    pub(crate) fn run_rename_tool(&mut self, from: &str, to: &str) {
        match camcraft_app::commands::RenameTool::new(&self.world, from, to) {
            Ok(command) => {
                let (to, ops) = (command.to().to_owned(), command.operations().len());
                self.history.execute(&self.world, command);
                self.doc_version += 1;
                self.status = format!("tool {from} renamed to {to} ({ops} operation(s) follow)");
                if let Some(view) = self.tool_library.as_mut() {
                    view.rename = None;
                }
            }
            Err(err) => self.refuse(ErrorScope::Tool, format!("tool not renamed: {err}")),
        }
    }

    /// Queue the editor's rename after its other Apply commands.
    pub(crate) fn push_library_rename(&mut self, rename: Option<(String, String)>) {
        if let Some((from, to)) = rename {
            self.pending.push_back(CommandKind::RenameTool { from, to });
        }
    }

    /// The tool entities the views show now (only views that show a tool
    /// cost a lookup).
    pub(crate) fn tool_views(&self) -> ToolViews {
        let tool = |name: &str| {
            (!name.is_empty())
                .then(|| model::find_by_name(&self.world, name))
                .flatten()
                .filter(|e| model::is_tool(&self.world, e))
        };
        let target = match &self.ui_target {
            UiTarget::Tool(name) => tool(name),
            _ => None,
        };
        ToolViews {
            library: self.tool_library.as_ref().and_then(|v| v.open_tool()).and_then(tool),
            target,
            properties: tool(&self.tool_ui.name),
            shape: self.tool_shape_editor.as_deref().and_then(tool),
        }
    }

    /// Each view whose tool lost its name while its entity stayed a tool
    /// follows the entity's new name. A deleted tool is not followed: its
    /// view closes as before.
    pub(crate) fn follow_renamed_tools(&mut self, before: ToolViews) {
        let world = &self.world;
        let renamed = |e: Option<Entity>, old: &str| -> Option<String> {
            let e = e?;
            if !world.is_alive(e) || !model::is_tool(world, &e) {
                return None;
            }
            model::name_of(world, &e).filter(|now| now != old)
        };
        let open = self.tool_library.as_ref().and_then(|v| v.open_tool()).map(str::to_owned);
        if let Some(new) = open.and_then(|old| renamed(before.library, &old)) {
            // The editor keeps its face (tab, preview); the draft reads
            // the tool again under its new name.
            self.open_tool_in_library(&new);
        }
        if let UiTarget::Tool(old) = &self.ui_target
            && let Some(new) = renamed(before.target, old)
        {
            self.ui_target = UiTarget::Tool(new);
        }
        if let Some(new) = renamed(before.properties, &self.tool_ui.name) {
            self.tool_ui.name = new;
        }
        if let Some(old) = self.tool_shape_editor.clone()
            && let Some(new) = renamed(before.shape, &old)
        {
            self.tool_shape_editor = Some(new);
        }
    }
}
