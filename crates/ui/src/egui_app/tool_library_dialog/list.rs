//! #3892 (U1 of #3663): the tools list of the tool library window. Moved
//! here from the retired dock tab (#2002, #2029).
//! #3897 (U6): the tool categories stand on the left of the rows
//! (`categories`), and each Document row starts with a small drawing of
//! the tool (`thumbs`, cached) and has a Gauge column: the holder's gauge
//! length plus the stick-out, for a tool in a library holder.
//! #3898 (U7): a third source, "My library", shows the component library
//! (`components`).
//! #4335 (U9): one header row (`list_header`) with the source tabs, the
//! one search box and the file buttons. The search matches the name, the
//! type, the diameters, the holder, the tool number, the part number and
//! the vendor (`search`); State keeps the rows and the hits
//! (`state_tool_list`), so a still frame builds nothing. "N of M tools"
//! heads the rows; the rail lists every category with its count. Document
//! and Catalog rows have the same columns, with the Recipes count and, on
//! a Catalog row, the "in document" mark. A click on a Document row's name
//! opens the tool in the editor.
//! Provenance: our own catalog and library controls; no HSMWorks material
//! involved.

use super::super::browser_tree::kind_name;
use super::super::tool_icons;
use crate::state::{CommandKind, State, ToolRow, ToolSource};
use camcraft_app::tools::Tool;
use camcraft_app::units::{self, Unit};
use camcraft_app::model;
use egui::{RichText, Ui};

/// The text when no row matches the search and the category.
pub(crate) const NO_MATCH: &str = "No tools match the filters.";
/// The Recipes column's head.
pub(crate) const RECIPES: &str = "Recipes";
/// The mark of a Catalog row whose name the document has.
pub(crate) const IN_DOCUMENT: &str = "in document";
/// The button of a Catalog row that the document does not have.
pub(crate) const ADD: &str = "Add";
/// The width of the category column.
const CATEGORY_W: f32 = 170.0;

/// The Gauge column's head in the document's unit.
#[must_use]
pub(crate) fn gauge_head(u: Unit) -> String {
    format!("Gauge {}", u.suffix())
}

/// The column heads in the document's unit (#3663 section 1): the
/// drawing, T, Name, Type, Ø, Flute, Flutes, Holder, Gauge, Recipes and
/// the mark.
#[must_use]
pub(crate) fn heads(u: Unit) -> [String; 11] {
    let s = u.suffix();
    [
        String::new(),
        "T".into(),
        "Name".into(),
        "Type".into(),
        format!("Ø {s}"),
        format!("Flute {s}"),
        "Flutes".into(),
        "Holder".into(),
        gauge_head(u),
        RECIPES.into(),
        String::new(),
    ]
}

/// "N of M tools".
#[must_use]
pub(crate) fn count_text(shown: usize, total: usize) -> String {
    format!("{shown} of {total} tools")
}

/// A length in the document's unit, without the zeros at the end.
#[must_use]
pub(crate) fn compact(v_mm: f64, u: Unit) -> String {
    let prec = if u == Unit::Inches { 4 } else { 3 };
    let s = format!("{:.prec$}", units::to_display(v_mm, u));
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" { "0".to_owned() } else { s.to_owned() }
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    super::list_header::draw(ui, state);
    ui.separator();
    let Some(source) = state.tool_library.as_ref().map(|v| v.source) else {
        return;
    };
    if source == ToolSource::Library {
        super::components::draw(ui, state);
        return;
    }
    let query = state.tool_search.clone();
    let hits = state.tool_hits(source, &query);
    let category = state.tool_library.as_ref().and_then(|v| v.category);
    ui.horizontal_top(|ui| {
        let pick = ui
            .vertical(|ui| {
                ui.set_width(CATEGORY_W);
                egui::ScrollArea::vertical()
                    .id_salt("tool_library_categories")
                    .auto_shrink([false, false])
                    .show(ui, |ui| super::categories::draw(ui, &hits.counts, hits.hits.len(), category))
                    .inner
            })
            .inner;
        if let Some(pick) = pick
            && let Some(view) = state.tool_library.as_mut()
        {
            view.category = pick;
        }
        ui.separator();
        ui.vertical(|ui| {
            let shown: Vec<&ToolRow> = hits
                .hits
                .iter()
                .filter_map(|i| hits.rows.get(*i))
                .filter(|r| super::categories::passes(category, &r.tool))
                .collect();
            ui.weak(count_text(shown.len(), hits.rows.len()));
            egui::ScrollArea::both()
                .id_salt("tool_library_rows")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if shown.is_empty() {
                        ui.label(NO_MATCH);
                    } else {
                        rows(ui, state, source, &shown);
                    }
                });
        });
    });
}

/// The document's tools, by name, in document order (the editor's rail).
pub(super) fn document_tools(state: &State) -> Vec<(String, Tool)> {
    state
        .tool_names()
        .into_iter()
        .filter_map(|name| {
            let tool = model::find_by_name(&state.world, &name).and_then(|e| model::tool_of(&state.world, &e))?;
            Some((name, tool))
        })
        .collect()
}

/// The rows. A Document row's name opens the tool in this window's editor
/// (#3893: view state; the Properties subject and the selection do not
/// change). A Catalog row ends with the "in document" mark or Add (the
/// `AddTool` command).
fn rows(ui: &mut Ui, state: &mut State, source: ToolSource, rows: &[&ToolRow]) {
    let u = state.units();
    let mut edit: Option<String> = None;
    let mut add: Option<(String, Tool)> = None;
    egui::Grid::new("tool_library_rows").spacing([14.0, 6.0]).show(ui, |ui| {
        for h in heads(u) {
            ui.label(RichText::new(h).weak());
        }
        ui.end_row();
        for r in rows {
            let t = &r.tool;
            if source == ToolSource::Document {
                super::thumbs::draw(ui, &r.name, t);
            } else {
                tool_icons::glyph(ui, tool_icons::catalog_row_icon(t));
            }
            ui.label(r.number.map_or_else(|| "-".to_owned(), |n| n.to_string()));
            if source == ToolSource::Document {
                if ui.selectable_label(false, r.name.as_str()).on_hover_text("edit this tool in this window").clicked() {
                    edit = Some(r.name.clone());
                }
            } else {
                ui.label(r.name.as_str());
            }
            ui.label(kind_name(t.kind));
            ui.label(compact(t.diameter, u));
            ui.label(compact(t.flute_len, u));
            ui.label(t.flutes.map_or_else(|| "-".to_owned(), |z| z.to_string()));
            ui.label(r.holder.as_deref().unwrap_or("-"));
            ui.label(r.gauge.map_or_else(|| "-".to_owned(), |g| compact(g, u)))
                .on_hover_text("the holder's gauge length plus the stick-out (a tool in a library holder only)");
            ui.label(t.cutting.len().to_string()).on_hover_text("the cutting data rows: one for each workpiece material");
            match source {
                ToolSource::Catalog if r.in_document => {
                    ui.weak(IN_DOCUMENT).on_hover_text("the document has a tool of this name");
                }
                ToolSource::Catalog => {
                    if ui.small_button(ADD).on_hover_text("put this catalog tool into the document (one undo step)").clicked() {
                        add = Some((r.name.clone(), t.clone()));
                    }
                }
                _ => {
                    ui.label("");
                }
            }
            ui.end_row();
        }
    });
    if let Some(name) = edit {
        state.open_tool_in_library(&name);
    }
    if let Some((name, tool)) = add {
        state.pending.push_back(CommandKind::AddTool { name, tool });
    }
}
