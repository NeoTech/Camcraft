//! #4334 (U8 of #3663): the frame of the tool library window. One header -
//! the way back to the list while an editor is open, the title (the tool,
//! holder or component name in an editor), the Cutting tools | Holders tabs
//! and the close button - and one footer that all pages share: a hint on
//! the left; Close on a list page; Duplicate, Cancel and Apply on the tool
//! editor. The pages draw between the two and do not change here.
//! Provenance: our own dialog frame after the #3663 mock-ups and the egui
//! panel docs; no HSMWorks material involved.

use egui::{Align, Button, Layout, RichText, Ui, WidgetInfo, WidgetType};

use super::super::{colorvision, contrast, error_text, theme};
use crate::state::{ErrorScope, LibraryPage, State, ToolSource};
use camcraft_app::tools::ToolKind;

/// The way back from an editor to its list.
pub(crate) const BACK: &str = "‹ List";
/// The footer's button on a list page.
pub(crate) const CLOSE: &str = "Close";
/// The header's close button (its AccessKit name; it shows "×").
pub(crate) const CLOSE_WINDOW: &str = "Close the tool library";
pub(crate) const DUPLICATE: &str = "Duplicate";
pub(crate) const CANCEL: &str = "Cancel";
pub(crate) const APPLY: &str = "Apply";
/// The footer hints.
pub(crate) const HINT_TOOLS: &str = "Click a row to edit the tool.";
pub(crate) const HINT_CATALOG: &str = "Add puts a catalog tool into the document.";
pub(crate) const HINT_LIBRARY: &str = "Click a document component to edit it.";
pub(crate) const HINT_HOLDERS: &str = "Click a holder to see its facts and the tools that use it.";
pub(crate) const HINT_HOLDER: &str = "Copy as user holder to make a holder that you can edit.";
pub(crate) const HINT_APPLY: &str = "Apply is one undo step.";
pub(crate) const HINT_COMPONENT: &str = "Apply component changes each tool built from it in one undo step.";
/// The `frame_cost` names of the header and the footer (crates/ui/AGENTS.md "Frame cost").
pub(crate) const HEADER_SPAN: &str = "ui/tool_library.header";
pub(crate) const FOOTER_SPAN: &str = "ui/tool_library.footer";
/// Why Duplicate is not offered yet (#4340).
pub(crate) const NO_DUPLICATE: &str =
    "a copy must keep the holder and the component stack of the tool; until it can, add a tool with New tool…";

/// What the window shows; the header and the footer follow it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Face {
    /// A list page: the tools (from a source) or the holders.
    List(LibraryPage, ToolSource),
    /// A built-in holder's read-only view.
    Holder(String),
    /// The tool editor: the name, the kind and the tool number (0: none).
    Tool { name: String, kind: ToolKind, number: u32 },
    /// The component editor, by the draft's name.
    Component(String),
}

impl Face {
    /// An editor (or a holder view) is open: the header offers the way back.
    pub(crate) fn has_back(&self) -> bool {
        !matches!(self, Face::List(..))
    }
}

/// What the user asked of the frame this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ask {
    Page(LibraryPage),
    Back,
    Close,
    Cancel,
    Apply,
}

/// The face of the open window, or `None` when it is closed.
#[must_use]
pub(crate) fn face(state: &State) -> Option<Face> {
    let v = state.tool_library.as_ref()?;
    if let Some(d) = &v.component_edit {
        return Some(Face::Component(d.name.clone()));
    }
    Some(match v.page {
        LibraryPage::Tools => match &v.editor {
            Some(e) => Face::Tool { name: e.draft.name.clone(), kind: e.draft.tool_kind(), number: e.draft.number },
            None => Face::List(LibraryPage::Tools, v.source),
        },
        LibraryPage::Holders => match &v.holders.open {
            Some(n) => Face::Holder(n.clone()),
            None => Face::List(LibraryPage::Holders, v.source),
        },
    })
}

/// The header row: back, title, tabs, then the close button at the right.
pub(crate) fn header(ui: &mut Ui, state: &State, face: &Face) -> Option<Ask> {
    let _s = crate::frame_cost::span(HEADER_SPAN);
    let page = state.tool_library.as_ref().map_or(LibraryPage::Tools, |v| v.page);
    let mut ask = None;
    ui.horizontal(|ui| {
        if face.has_back() && ui.button(BACK).on_hover_text("back to the list; an unapplied draft is dropped").clicked() {
            ask = Some(Ask::Back);
        }
        match face {
            Face::List(..) => {
                ui.label(title(ui, super::TITLE));
            }
            Face::Tool { name, kind, number } => {
                ui.label(title(ui, name));
                let t = if *number > 0 { format!("T{number}") } else { "T?".to_owned() };
                ui.weak(format!("{} · {t}", super::super::browser_tree::kind_name(*kind)));
            }
            Face::Holder(name) => {
                ui.label(title(ui, name));
                ui.weak("holder");
            }
            Face::Component(name) => {
                ui.label(title(ui, name));
                ui.weak("component");
            }
        }
        ui.add_space(12.0);
        let mut picked = page;
        ui.selectable_value(&mut picked, LibraryPage::Tools, super::TOOLS_PAGE)
            .on_hover_text("the tools: the document's and the catalog's");
        ui.selectable_value(&mut picked, LibraryPage::Holders, super::HOLDERS_PAGE)
            .on_hover_text("the built-in holders by spindle taper, with the tools that use each one");
        if picked != page {
            ask = Some(Ask::Page(picked));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let r = ui.add(Button::new("×").frame(false));
            r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, CLOSE_WINDOW));
            if r.on_hover_text("close the tool library; an unapplied draft is dropped").clicked() {
                ask = Some(Ask::Close);
            }
        });
    });
    ask
}

/// The footer: the tool editor's refusal (if any), then the hint on the
/// left and the face's buttons on the right.
pub(crate) fn footer(ui: &mut Ui, state: &State, face: &Face, component_changed: bool) -> Option<Ask> {
    let _s = crate::frame_cost::span(FOOTER_SPAN);
    if matches!(face, Face::Tool { .. })
        && let Some(msg) = state.error_for(ErrorScope::Tool)
    {
        error_text::error_text(ui, colorvision::text_inks(ui, &state.world).error, msg);
    }
    let mut ask = None;
    ui.horizontal(|ui| {
        ui.weak(hint(face));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| match face {
            Face::List(..) | Face::Holder(_) => {
                if ui.button(CLOSE).on_hover_text("close the tool library").clicked() {
                    ask = Some(Ask::Close);
                }
            }
            Face::Tool { .. } => {
                let refusal = tool_refusal(state);
                if primary(ui, APPLY, refusal.is_none())
                    .on_hover_text("write this draft to the tool (one undo step)")
                    .on_disabled_hover_text(refusal.as_deref().unwrap_or_default())
                    .clicked()
                {
                    ask = Some(Ask::Apply);
                }
                if ui.button(CANCEL).on_hover_text("drop the unapplied draft and go back to the list").clicked() {
                    ask = Some(Ask::Cancel);
                }
                let _ = ui.add_enabled(false, Button::new(DUPLICATE)).on_disabled_hover_text(NO_DUPLICATE);
            }
            Face::Component(_) => {
                if primary(ui, super::component_edit::APPLY, component_changed)
                    .on_hover_text("write this draft to the component and each tool built from it (one undo step)")
                    .on_disabled_hover_text("the draft does not differ from the component")
                    .clicked()
                {
                    ask = Some(Ask::Apply);
                }
                if ui.button(CANCEL).on_hover_text("drop the unapplied draft and go back to the list").clicked() {
                    ask = Some(Ask::Cancel);
                }
            }
        });
    });
    ask
}

/// The footer's hint for `face`.
#[must_use]
pub(crate) fn hint(face: &Face) -> &'static str {
    match face {
        Face::List(LibraryPage::Tools, ToolSource::Document) => HINT_TOOLS,
        Face::List(LibraryPage::Tools, ToolSource::Catalog) => HINT_CATALOG,
        Face::List(LibraryPage::Tools, ToolSource::Library) => HINT_LIBRARY,
        Face::List(LibraryPage::Holders, _) => HINT_HOLDERS,
        Face::Holder(_) => HINT_HOLDER,
        Face::Tool { .. } => HINT_APPLY,
        Face::Component(_) => HINT_COMPONENT,
    }
}

/// The header's title: a little larger than the body text, in the body
/// ink (the theme's strong ink is not readable on every window fill).
fn title(ui: &Ui, text: &str) -> RichText {
    RichText::new(text).size(15.0).color(ui.visuals().text_color())
}

/// The primary button: filled with the theme's accent while enabled.
fn primary(ui: &mut Ui, label: &str, enabled: bool) -> egui::Response {
    let btn = if enabled {
        let accent = theme::current().accent;
        Button::new(RichText::new(label).strong().color(contrast::on_fill(accent))).fill(accent)
    } else {
        Button::new(label)
    };
    ui.add_enabled(enabled, btn)
}

/// Why the tool editor's draft cannot be applied (a torch that states a
/// spindle speed), read from the window's draft.
fn tool_refusal(state: &State) -> Option<String> {
    let d = &state.tool_library.as_ref()?.editor.as_ref()?.draft;
    (d.tool_kind() == ToolKind::Waterjet).then(|| d.to_tool().torch_refusal()).flatten()
}
