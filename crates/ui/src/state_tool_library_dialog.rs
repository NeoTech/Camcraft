//! #3892 (U1 of #3663): the tool library window's view state. The window
//! is open while `State::tool_library` is `Some`. Like the tool shape
//! window, it is ephemeral view state: never a command, never saved,
//! never undoable. The edits it starts ("Add", Import, Apply) are commands.
//! The Tool type and Name filters stay in `State::tool_catalog_kind` and
//! `State::tool_catalog_name`, so they hold between two openings.
//! #3893 (U2): a tool open in the window has its OWN draft
//! ([`ToolEditorView`]). Properties keeps `State::tool_ui`, so the two can
//! show different tools and never write into each other's draft.
//! #3894 (U3): the editor's preview - the 3D model view (the default) or
//! the to-scale side view - and the 3D view's orbit are view state of the
//! window too ([`EditorFace`]).
//! #3895 (U4): the window's holder pick, gauge length and projection are
//! its own draft too ([`ToolEditorView::holder`]), and a change of the
//! stored tool's length re-seeds only that draft's projection (#808
//! item 2). The stored `HolderRef` changes only through Apply holder.
//! #3896 (U5): the editor's Recipes tab ([`ToolEditorTab::Recipes`]).
//! Provenance: our own view state, after the tool shape window's
//! `Option` pattern; no HSMWorks material involved.

use super::{Orbit, State, ToolProfileUi, ToolUi, UiTarget};
use camcraft_app::model;

/// Which tools the list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolSource {
    /// The tools of the open document.
    Document,
    /// The built-in catalog; each row can be added to the document.
    #[default]
    Catalog,
}

/// The tabs of the tool editor in the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolEditorTab {
    /// Type, geometry and the kind's own rows.
    #[default]
    Tool,
    /// #3896 (U5): the cutting data per workpiece material, with the
    /// surface speed and the feed per tooth of each row.
    Recipes,
    /// Material group, the catalog facts (#898) and the tool number.
    Details,
}

/// #3894: which view the editor's preview area shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewMode {
    /// The orbitable 3D model: the imported body, or the revolved profile.
    #[default]
    Model,
    /// The to-scale side view with dimension lines.
    Side,
}

/// The editor's view state that stays when another tool opens: the tab,
/// the preview view and the 3D view's orbit.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EditorFace {
    pub tab: ToolEditorTab,
    pub preview: PreviewMode,
    pub orbit: Orbit,
}

/// #3893: the tool open in the window, with its own draft. `synced` is the
/// (name, `doc_version`) the draft was read at, the same key that
/// `State::sync_tool_ui` uses for Properties. Typing does not change
/// `doc_version`, so a draft is read again only after a command or when
/// another tool opens.
#[derive(Clone, Debug)]
pub struct ToolEditorView {
    pub draft: ToolUi,
    pub(super) synced: (String, u64),
    pub face: EditorFace,
    /// #3895: the window's holder draft (pick, gauge length, projection).
    pub holder: ToolProfileUi,
}

/// The open tool library window.
#[derive(Clone, Debug, Default)]
pub struct ToolLibraryView {
    pub source: ToolSource,
    /// #3893: the tool open in the editor. `None` shows the full list;
    /// `Some` folds the list into a rail beside the editor.
    pub editor: Option<ToolEditorView>,
}

impl ToolLibraryView {
    /// The name of the tool open in the editor, if any.
    #[must_use]
    pub fn open_tool(&self) -> Option<&str> {
        self.editor.as_ref().map(|e| e.draft.name.as_str())
    }
}

impl State {
    /// Open the tool library window. An open window keeps its view; a new
    /// one starts on the document's tools, or on the catalog when the
    /// document has no tool yet. #3893: when Properties shows a tool, the
    /// new window opens that tool in its editor.
    pub(crate) fn open_tool_library(&mut self) {
        if self.tool_library.is_some() {
            return;
        }
        let source = if self.tool_names().is_empty() {
            ToolSource::Catalog
        } else {
            ToolSource::Document
        };
        self.tool_library = Some(ToolLibraryView { source, editor: None });
        if let UiTarget::Tool(name) = &self.ui_target
            && !name.is_empty()
            && model::find_by_name(&self.world, name).is_some_and(|e| model::is_tool(&self.world, &e))
        {
            let name = name.clone();
            self.open_tool_in_library(&name);
        }
    }

    pub(crate) fn close_tool_library(&mut self) {
        self.tool_library = None;
    }

    /// #3893: open `name` in the window's editor, with a fresh draft that
    /// the next draw reads from the document. The document selection and
    /// the Properties subject do not change.
    pub(crate) fn open_tool_in_library(&mut self, name: &str) {
        self.open_tool_library();
        let Some(view) = self.tool_library.as_mut() else {
            return;
        };
        let face = view.editor.as_ref().map(|e| e.face).unwrap_or_default();
        view.source = ToolSource::Document;
        view.editor = Some(ToolEditorView {
            draft: ToolUi { name: name.to_owned(), ..ToolUi::default() },
            // An empty name never matches, so the first draw reads the tool.
            synced: (String::new(), self.doc_version),
            face,
            holder: ToolProfileUi::default(),
        });
    }

    /// #3893: close the editor; the window shows the full list again.
    pub(crate) fn close_tool_in_library(&mut self) {
        if let Some(view) = self.tool_library.as_mut() {
            view.editor = None;
        }
    }

    /// #3893: run `f` with the window's draft in `tool_ui` (and its sync
    /// key in `tool_synced`; #3895: its holder draft in `tool_profile_ui`),
    /// then put Properties' drafts back. The shared
    /// row and Apply helpers read `tool_ui`; this lets them serve the
    /// window without a copy, while the two drafts stay apart. `f` also
    /// gets the editor's view state (tab, preview, orbit). `None` when no
    /// tool is open in the window.
    pub(crate) fn with_library_draft<R>(
        &mut self,
        f: impl FnOnce(&mut State, &mut EditorFace) -> R,
    ) -> Option<R> {
        let mut editor = self.tool_library.as_mut().and_then(|v| v.editor.take())?;
        std::mem::swap(&mut self.tool_ui, &mut editor.draft);
        std::mem::swap(&mut self.tool_synced, &mut editor.synced);
        std::mem::swap(&mut self.tool_profile_ui, &mut editor.holder);
        self.sync_tool_ui();
        let out = f(self, &mut editor.face);
        std::mem::swap(&mut self.tool_ui, &mut editor.draft);
        std::mem::swap(&mut self.tool_synced, &mut editor.synced);
        std::mem::swap(&mut self.tool_profile_ui, &mut editor.holder);
        if let Some(view) = self.tool_library.as_mut()
            && view.editor.is_none()
        {
            view.editor = Some(editor);
        }
        Some(out)
    }
}

impl State {
    /// #3895 (#808 item 2): when the stored tool's stickout (flute plus
    /// shank length) changes under a holder pick, the projection draft
    /// takes the new stickout and is marked as re-seeded, so the user sees
    /// it before Apply holder. Only the draft changes: the stored
    /// `HolderRef` keeps its projection until Apply holder lands a new one.
    pub(crate) fn reseed_holder_projection(&mut self) {
        if self.tool_profile_ui.library.is_none() {
            return;
        }
        let Some(now) = self.editor_stored_tool().map(|t| t.flute_len + t.shank_len) else {
            return;
        };
        let d = &mut self.tool_profile_ui;
        match d.seeded {
            Some(was) if (was - now).abs() > 1e-9 => {
                d.projection = now;
                d.seeded = Some(now);
                d.reseeded = true;
            }
            Some(_) => {}
            None => d.seeded = Some(now),
        }
    }
}
