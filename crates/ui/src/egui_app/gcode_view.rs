//! The G-code viewer window (M1.7d, #256, #1089): the posted program as
//! a listing with a highlighted cursor row, selectable text, click a
//! row to move the cursor there, and the optional trace pane. Display
//! and ephemeral view state only; specced from our own design notes, no
//! HSMWorks material involved.

use egui::{Align, Id, Label, Layout, ScrollArea, Sense, Ui, Vec2, Window};

use crate::state::{PostView, State};

/// Width of the trace pane's block list.
const TRACE_LIST_W: f32 = 300.0;

/// M1.7d: the G-code viewer. The program one successful post wrote,
/// line-numbered and monospaced, with the scroll cursor (the line at
/// the window's middle) published through `PostView::line` for the
/// viewport's amber segment (`shell::gcode_segment`). It shows the
/// retained text, not a re-read of the file: text and positions came
/// from the same engine run, so a highlighted line always highlights
/// the move that line posts, even if something edits the file after.
pub(crate) fn gcode_window(ctx: &egui::Context, state: &mut State) {
    let Some(view) = state.post_view.as_mut() else {
        return;
    };
    if !view.open {
        return; // closed by ✖ or a document switch: allocate nothing
    }
    let stem = view
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program")
        .to_owned();
    let mut open = view.open;
    let warn_ink = super::colorvision::palette_of(&state.world).text_on(super::theme::current().ink).warn;
    // Disjoint field borrows: the closure writes `line` and reads
    // `program` at once, and `open` is a copy resolved after the pass.
    let PostView {
        program,
        line,
        blocks,
        trace,
        not_judged,
        ..
    } = view;
    // The trace pane's selection is DERIVED (the block owning the
    // scroll cursor), never stored - the one cursor law the viewer
    // already obeys, applied to a second view of the same evidence.
    let owning = |ln: usize| -> Option<usize> {
        blocks
            .iter()
            .position(|b| ln >= b.first && ln < b.last.max(b.first + 1))
    };
    Window::new(format!("G-code - {stem}"))
        .id(Id::new("gcode-viewer"))
        .open(&mut open)
        .default_size([560.0, 400.0])
        .show(ctx, |ui| {
            let jump = std::cell::Cell::new(None::<usize>);
            ui.horizontal(|ui| {
                // Stage D: the debugging trace, beside the program it
                // explains. Off by default - the plain viewer draws
                // exactly as it always did.
                ui.checkbox(trace, "trace");
                // The header's not-judged line: same row as the
                // checkbox, so a judged program's viewer geometry is
                // untouched. Hover shows the whole reason.
                if let Some(line) = not_judged.as_deref() {
                    ui.add(
                        Label::new(
                            egui::RichText::new(format!("WARNING: {line}"))
                                .color(warn_ink),
                        )
                        .truncate(),
                    )
                    .on_hover_text(line);
                }
            });
            let trace_on = *trace;
            let sel = owning(*line);
            // Trace off: the listing is the window's direct child -
            // byte-for-byte the geometry every cursor-law test was
            // written against. Trace on: two panes laid into FIXED
            // rectangles carved from the window's own free rect
            // (#1089). egui 0.36's `Sides` was used here first, and
            // with an auto-sized window it fed the right pane an
            // unbounded width: ticking the box made the window grow to
            // thousands of pixels and cover the part. A rect we cut
            // ourselves never asks the window for more than it has, so
            // the window keeps the size the user gave it.
            if trace_on {
                let full = ui.available_rect_before_wrap();
                let gap = ui.spacing().item_spacing.x;
                let left_w = TRACE_LIST_W.min((full.width() * 0.5).max(0.0));
                let left = egui::Rect::from_min_size(full.min, Vec2::new(left_w, full.height()));
                let right = egui::Rect::from_min_max(
                    egui::pos2((left.right() + gap).min(full.right()), full.top()),
                    full.right_bottom(),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
                    ScrollArea::vertical()
                        .id_salt("gcode-trace-list")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (i, b) in blocks.iter().enumerate() {
                                let txt = format!("{:>4}  {}", b.first + 1, b.label);
                                let rt = egui::RichText::new(txt)
                                    .monospace()
                                    .size(11.0)
                                    .color(if sel == Some(i) {
                                        ui.visuals().strong_text_color()
                                    } else {
                                        ui.visuals().text_color()
                                    });
                                let r = ui.add(
                                    Label::new(rt).selectable(false).sense(Sense::click()),
                                );
                                if r.clicked() {
                                    jump.set(Some(b.first));
                                }
                            }
                        });
                });
                ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                    gcode_listing(ui, program.as_str(), line, blocks.as_slice(), true, &jump);
                });
                ui.allocate_rect(full, Sense::hover());
            } else {
                gcode_listing(ui, program.as_str(), line, blocks.as_slice(), false, &jump);
            }
        });
    view.open = open;
}

/// The program listing: every line, uniformly `row_h` tall, with the
/// scroll cursor law the viewer is tested against (the middle row of
/// the clip rect IS the published line). Drawn as the window's direct
/// child or to the right of the trace panel - one geometry, two
/// hosts. While the trace is open, lines outside the block under
/// study rest weak; a row of the clicked block centers itself, so the
/// cursor law reads the selection back next frame - the pane stores
/// no second truth about the scroll.
fn gcode_listing(
    ui: &mut Ui,
    program: &str,
    line: &mut usize,
    blocks: &[camcraft_app::post::TraceBlock],
    trace_on: bool,
    jump: &std::cell::Cell<Option<usize>>,
) {
    ScrollArea::vertical()
        .id_salt("gcode-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // The scroll offset is read, not stored: egui lays the
            // inner ui out at `-offset`, so the clip rect's center
            // against the content top names the middle row - that row
            // is the sync cursor. Uniform rows (one monospace label
            // per line, spacing fixed) are what make the division
            // exact.
            let n = program.lines().count();
            let row_h = ui.text_style_height(&egui::TextStyle::Monospace);
            ui.spacing_mut().item_spacing.y = 0.0;
            let top = ui.min_rect().min.y;
            let vis = ui.clip_rect();
            let cursor = (((vis.center().y - top) / row_h).max(0.0) as usize).min(n.saturating_sub(1));
            *line = cursor;
            let want = trace_on
                .then(|| {
                    blocks
                        .iter()
                        .position(|b| *line >= b.first && *line < b.last.max(b.first + 1))
                        .map(|bi| (blocks[bi].first, blocks[bi].last))
                })
                .flatten();
            let width = ui.available_width();
            let spans = super::gcode_highlight::spans_for(ui.ctx(), program);
            for (i, ln) in program.lines().enumerate() {
                let dim = want.is_some_and(|(a, b)| !(i >= a && i < b.max(a + 1)));
                let empty = Vec::new();
                let rt = super::gcode_highlight::row_job(
                    ui,
                    ln,
                    spans.get(i).unwrap_or(&empty),
                    dim,
                );
                // Every row is a left-to-right box exactly `width` by
                // `row_h`, so the text starts at the left edge (#256:
                // `add_sized` centred it) and the cursor arithmetic
                // above stays exact over a long program instead of
                // drifting a row. `truncate` keeps a line wider than
                // the window on its one row (egui shows the whole line
                // on hover) rather than wrapping it onto a second.
                // #1089: the gutter is its own non-selectable label and
                // the code is selectable (drag, Ctrl+C), so a copy is
                // the program without its numbers; a click anywhere on
                // the row moves the cursor there.
                let at_cursor = i == cursor;
                let resp = ui
                    .allocate_ui_with_layout(
                        Vec2::new(width, row_h),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_min_size(Vec2::new(width, row_h));
                            let rect = ui.max_rect();
                            let bg = ui.interact(rect, ui.id().with(("row", i)), Sense::click());
                            if at_cursor {
                                ui.painter().rect_filled(rect, 0.0, cursor_fill(ui));
                            }
                            ui.add(
                                Label::new(super::gcode_highlight::gutter_text(i + 1))
                                    .selectable(false),
                            );
                            let code = ui.add(
                                Label::new(rt)
                                    .selectable(true)
                                    .sense(Sense::click())
                                    .truncate(),
                            );
                            bg.clicked() || code.clicked()
                        },
                    );
                let (resp, clicked) = (resp.response, resp.inner);
                if clicked {
                    jump.set(Some(i));
                }
                if jump.get() == Some(i) {
                    resp.scroll_to_me(Some(Align::Center));
                    jump.set(None);
                }
            }
        });
}

/// The cursor row's background: the theme's selection fill, softened so
/// the coloured words stay readable on it (#1089).
pub(crate) fn cursor_fill(ui: &Ui) -> egui::Color32 {
    ui.visuals().selection.bg_fill.linear_multiply(0.45)
}
