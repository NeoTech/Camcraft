//! The tool editor's Profile section (#93): a milling tool's holder as a
//! spun (revolved) half-outline - Parametric (the cylinder its numbers
//! state) or Custom (a table of points with optional arcs), the live
//! refusal by name, and Apply profile, which queues the one command.
//! #2007: the tool has one preview - the to-scale one beside the fields
//! (`tool_preview`), which draws the draft of this tab through
//! [`preview_inputs`]. #93 slice 2: Draw cutter switches the tab to the
//! tool's own cutter, neck and shank (`tool_cutter_page`). Turning tools
//! are not spun profiles: for them the column stays the Body section
//! alone (owner, #93).
//! Provenance: our own pane over `app::tool_body_view` and
//! `app::spun`. No vendor material consulted.

use egui::{Color32, Ui};

use camcraft_app::cutter_profile;
use camcraft_app::spun::SpunPoint;
use camcraft_app::tool_body;
use camcraft_app::tool_body_view;
use camcraft_app::units::{self, Unit};

use crate::state::{State, ToolColumn, is_spun_kind};

pub(super) const BAD: Color32 = Color32::from_rgb(230, 90, 80);
/// #2007: the points table's height. It is also its least height, so a
/// table low on a scrolled page does not shrink to a few rows.
const TABLE_H: f32 = 150.0;

/// Holder bands `(above, len, r)` and a steel outline override.
pub(super) type PreviewInputs = (Option<Vec<(f64, f64, f64)>>, Option<Vec<(f64, f64)>>);

/// #2007: what the one to-scale preview draws. On the Profile tab a draft
/// that validates shows at once - the drawn holder as the bands the
/// collision check sweeps (#2017), the drawn cutter as the steel
/// outline. Otherwise the landed body, as before. A draft that does not
/// validate shows the landed state; its refusal stands below the table.
pub(super) fn preview_inputs(state: &mut State, exists: bool) -> PreviewInputs {
    if !exists {
        return (None, None);
    }
    // The Body tab's draft follows the landed body each frame, as before
    // #2007, also while a drawn holder draft shows.
    state.sync_tool_body_ui();
    let tool = state.tool_ui.to_tool();
    let profile_tab = is_spun_kind(state.tool_ui.tool_kind()) && state.tool_profile_ui.column == ToolColumn::Profile;
    let geom = tool.geom();
    let bands = |p: Option<&_>| tool_body::stack(&geom, p).bands.iter().map(|q| (q.above, q.len, q.r)).collect::<Vec<_>>();
    if profile_tab && state.tool_cutter_ui.shown {
        state.sync_tool_cutter_ui();
        let steel = cutter_profile::check(&tool, state.tool_cutter_ui.points.clone()).ok().map(|s| s.outline());
        let body = super::tool_body_page::envelope(state);
        // The drawn cutter stops at the shank top: the holder cylinder
        // goes on top of it as bands when no body is landed.
        let body = match (&steel, body) {
            (Some(_), None) => Some(bands(None)),
            (_, b) => b,
        };
        return (body, steel);
    }
    if profile_tab && state.tool_profile_ui.custom {
        state.sync_tool_profile_ui();
        if let Ok(s) = tool_body_view::spun_preview(&tool, &state.tool_profile_ui.points) {
            return (Some(bands(Some(&s.holder_profile()))), None);
        }
    }
    (super::tool_body_page::envelope(state), None)
}

/// The editor's third column: Body / Profile tabs for a milling tool,
/// the Body section alone for a turning tool.
pub(super) fn third_column(ui: &mut Ui, state: &mut State, exists: bool, u: Unit) {
    let milling = is_spun_kind(state.tool_ui.tool_kind());
    if milling {
        // #2007: the tab row wraps in a narrow pane.
        ui.horizontal_wrapped(|ui| {
            let c = state.tool_profile_ui.column;
            if ui.selectable_label(c == ToolColumn::Body, "Body").on_hover_text("import a holder or assembly mesh").clicked() {
                state.tool_profile_ui.column = ToolColumn::Body;
            }
            if ui
                .selectable_label(c == ToolColumn::Profile, "Profile")
                .on_hover_text("draw the holder as a revolved outline")
                .clicked()
            {
                state.tool_profile_ui.column = ToolColumn::Profile;
            }
            // #93 slice 2: which outline the Profile tab draws - on the
            // tab row, so the holder view keeps its height.
            if state.tool_profile_ui.column == ToolColumn::Profile {
                ui.separator();
                let cutter = state.tool_cutter_ui.shown;
                if ui.selectable_label(!cutter, "Draw holder").on_hover_text("the holder as a revolved outline").clicked() {
                    state.tool_cutter_ui.shown = false;
                }
                if ui
                    .selectable_label(cutter, "Draw cutter")
                    .on_hover_text("the tool's own cutter, neck and shank as a revolved outline")
                    .clicked()
                {
                    state.tool_cutter_ui.shown = true;
                }
            }
        });
    }
    if milling && state.tool_profile_ui.column == ToolColumn::Profile {
        profile_section(ui, state, exists, u);
    } else {
        super::tool_body_page::body_section(ui, state, exists, u);
    }
}

fn profile_section(ui: &mut Ui, state: &mut State, exists: bool, u: Unit) {
    if state.tool_cutter_ui.shown {
        super::tool_cutter_page::cutter_section(ui, state, exists, u);
        return;
    }
    ui.strong("Holder profile");
    if !exists {
        ui.label("Add the tool first, then draw its holder.");
        return;
    }
    state.sync_tool_profile_ui();
    let landed = state.editor_tool_body();
    // #2007: plain words - which holder the tool uses now.
    ui.label(match &landed {
        // #2016: a library holder is the entry, not a copy of it.
        Some(b) if b.library.is_some() => format!(
            "In use: library holder {}. Edit a point to make it a drawn outline.",
            b.library.as_ref().map_or("", |r| r.entry.as_str())
        ),
        Some(b) if b.spun.is_some() => "In use: a drawn holder outline".to_owned(),
        Some(b) => format!("In use: the imported holder {}. Apply profile replaces it.", b.source),
        None => "In use: the holder cylinder from the dimensions".to_owned(),
    });
    let custom = state.tool_profile_ui.custom;
    ui.horizontal(|ui| {
        if ui
            .selectable_label(!custom, "Parametric")
            .on_hover_text("the holder is a cylinder with the holder diameter and length, redrawn as they change")
            .clicked()
        {
            state.set_profile_custom(false);
        }
        if ui.selectable_label(custom, "Custom").on_hover_text("edit the outline point by point; changed dimensions leave it alone").clicked() {
            state.set_profile_custom(true);
        }
    });
    points_table(ui, state, u);
    let tool = state.tool_ui.to_tool();
    let points = state.tool_profile_ui.points.clone();
    let check = if points.is_empty() { Err("No holder: the tool has none.".to_owned()) } else { tool_body_view::spun_preview(&tool, &points) };
    match &check {
        Ok(s) => {
            state.retire_profile_refusal();
            let top = s.points().last().map_or(0.0, |p| p.z);
            let foot = s.points().first().map_or(0.0, |p| p.z);
            let wide = s.max_radius();
            let mm = u.suffix();
            ui.label(format!(
                "holder Ø {} × {} {mm}, from {} to {} {mm} above the tip",
                units::len(2.0 * wide, u, 1),
                units::len(top - foot, u, 1),
                units::len(foot, u, 1),
                units::len(top, u, 1),
            ));
        }
        Err(e) => {
            ui.colored_label(BAD, e);
        }
    }
    let state_custom = state.tool_profile_ui.custom;
    let can = state_custom || landed.is_some();
    if ui
        .add_enabled(can, egui::Button::new("Apply profile"))
        .on_hover_text(if state_custom {
            "use the drawn holder for this tool (one undo step)"
        } else {
            "remove the holder in use: the holder cylinder from the dimensions is checked again (one undo step)"
        })
        .clicked()
    {
        state.queue_apply_tool_profile();
    }
    ui.small("The holder is a turned shape: you edit one half of its outline. Parts that are not round are covered by a larger safe shape, never cut away.");
    let has_export = !state.tool_profile_ui.points.is_empty() || landed.is_some();
    // #2007: the three file buttons wrap in a narrow column.
    let (imp_dxf, exp_dxf, imp_step) = ui
        .horizontal_wrapped(|ui| {
            let imp_dxf = crate::browser_files::button(
                ui,
                state.in_browser(crate::browser_files::NEEDS_PICKER),
                "Import DXF…",
                "read an ASCII DXF with a CENTRELINE layer and one open half-profile into Custom. The centreline must be parallel to Y. DXF +Y is tool +Z; z = 0 is the lower end of the centreline, independent of draw direction",
            );
            let exp_dxf = if has_export {
                crate::browser_files::button(
                    ui,
                    state.in_browser(crate::browser_files::NO_EXPORT),
                    "Export DXF…",
                    "write the half-profile or holder body as ASCII DXF (mm, layers CENTRELINE and PROFILE)",
                )
            } else {
                ui.add_enabled(false, egui::Button::new("Export DXF…")).clicked()
            };
            let imp_step = crate::browser_files::button(
                ui,
                state.in_browser(crate::browser_files::NEEDS_PICKER),
                "Import STEP…",
                "convert an ISO 13399 aligned STEP holder into a conservative editable half-profile",
            );
            (imp_dxf, exp_dxf, imp_step)
        })
        .inner;
    if imp_dxf {
        state.import_profile_dxf_dialog();
    }
    if exp_dxf {
        state.export_profile_dxf_dialog();
    }
    if imp_step {
        state.import_step_profile_dialog();
    }
    super::tool_holder_picker::holder_picker(ui, state, u);
}

/// The points, tip-up: z, r, arc radius (0 = line), insert after,
/// remove. Read-only in Parametric.
fn points_table(ui: &mut Ui, state: &mut State, u: Unit) {
    let custom = state.tool_profile_ui.custom;
    let (insert, remove) = points_grid(ui, u, &mut state.tool_profile_ui.points, custom, "tool_profile_points");
    if let Some(i) = insert {
        state.insert_profile_point(i);
    }
    if let Some(i) = remove {
        state.remove_profile_point(i);
    }
}

/// The editable points grid both drawn outlines share: returns the row
/// to insert after and the row to remove, when clicked.
pub(super) fn points_grid(ui: &mut Ui, u: Unit, points: &mut [SpunPoint], editable: bool, id: &str) -> (Option<usize>, Option<usize>) {
    let n = points.len();
    let mut insert = None;
    let mut remove = None;
    // #2007: the least height keeps the rows in view when the table sits
    // low in the scrolled page - without it egui shrinks a nested scroll
    // area to the space left in the view, about two rows.
    egui::ScrollArea::vertical().max_height(TABLE_H).min_scrolled_height(TABLE_H).id_salt(id).show(ui, |ui| {
        ui.add_enabled_ui(editable, |ui| {
            // #2007: no 40 px least column width - with it the six columns
            // overran the 260 px column and pushed "×" out of a narrow pane.
            egui::Grid::new((id, "grid")).num_columns(6).min_col_width(0.0).spacing([4.0, 2.0]).show(ui, |ui| {
                let heads = [
                    ("#".to_owned(), "the point number, from the tip up"),
                    (format!("height {}", u.suffix()), "the height of the point above the tool tip"),
                    (format!("radius {}", u.suffix()), "the distance of the point from the tool axis"),
                    ("arc".to_owned(), "the arc radius of this point; 0 is a straight line"),
                    (String::new(), ""),
                    (String::new(), ""),
                ];
                for (h, tip) in heads {
                    if tip.is_empty() {
                        ui.label(h);
                    } else {
                        ui.label(h).on_hover_text(tip).on_disabled_hover_text(tip);
                    }
                }
                ui.end_row();
                for (i, p) in points.iter_mut().enumerate() {
                    ui.label(format!("{}", i + 1));
                    len_cell(ui, u, &mut p.z);
                    len_cell(ui, u, &mut p.r);
                    len_cell(ui, u, &mut p.arc);
                    if i + 1 < n && ui.small_button("+").on_hover_text("insert a point after this one").clicked() {
                        insert = Some(i);
                    } else if i + 1 >= n {
                        ui.label("");
                    }
                    if ui.small_button("×").on_hover_text("remove this point").clicked() {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
            });
        });
    });
    (insert, remove)
}

/// A compact unit-lens length cell (the `fields::num` law, narrower).
pub(super) fn len_cell(ui: &mut Ui, u: Unit, v: &mut f64) {
    let w = [54.0, ui.spacing().interact_size.y];
    if u == Unit::Millimeters {
        ui.add_sized(w, egui::DragValue::new(v).speed(0.1));
    } else {
        let mut d = units::to_display(*v, u);
        if ui.add_sized(w, egui::DragValue::new(&mut d).speed(0.1 / 25.4)).changed() {
            *v = units::from_display(d, u);
        }
    }
}

