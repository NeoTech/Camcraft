//! Fold chips (#4483, UI plan P of #4245): the narrow title chip that a
//! folded side pane shows, and the split arithmetic that keeps the pane at
//! the chip's width. The kit sees no dock: the shell passes rects and
//! widths in, and turns the chip's click into the unfold.
//! Provenance: our own code, specced from the egui 0.3x docs (`TextShape`
//! angle, `Response::widget_info`); no HSMWorks material involved.

use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::Tokens;

/// The width of a folded side pane, in points: room for the dock's fold
/// arrow (24 points) above the chip, and for one line of rotated title.
pub const CHIP_WIDTH: f32 = 28.0;

/// The space above and below the rotated title inside the chip.
const TITLE_PAD: f32 = 8.0;

/// Which child of a side-by-side split folds: the left one or the right one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldSide {
    Left,
    Right,
}

/// The split fraction (the LEFT child's share, as egui_dock keeps it) that
/// gives the folded child `chip` points of a split `span` points wide.
/// `None` when the split is not laid out yet, or is too narrow to hold the
/// chip and a pane beside it: the caller then leaves the fraction alone.
pub fn chip_fraction(side: FoldSide, chip: f32, span: f32) -> Option<f32> {
    if !(span.is_finite() && chip.is_finite()) || chip <= 0.0 || span <= 2.0 * chip {
        return None;
    }
    let share = (chip / span).clamp(0.0, 0.5);
    Some(match side {
        FoldSide::Left => share,
        FoldSide::Right => 1.0 - share,
    })
}

/// Whether `fraction` already gives the same width as `target` (to half a
/// point over `span`), so a still frame writes nothing.
pub fn holds(fraction: f32, target: f32, span: f32) -> bool {
    ((fraction - target) * span).abs() < 0.5
}

/// The share to put back on unfold: the stored one when it is a real
/// share (finite, inside 0..1), else `fallback`.
pub fn open_fraction(stored: f32, fallback: f32) -> f32 {
    if stored.is_finite() && stored > 0.0 && stored < 1.0 {
        stored
    } else {
        fallback
    }
}

/// The accessible name of the chip that unfolds the pane `title`.
pub fn chip_label(title: &str) -> String {
    format!("Unfold {title}")
}

/// How a chip's state line reads: all is well, or the user must act.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipTone {
    Ok,
    Attention,
}

/// The safety state that a folded pane keeps on its chip (#4245 E2): a
/// short text with a glyph, never a colour alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChipState {
    pub tone: ChipTone,
    /// The short text under the title, for example "2 stale".
    pub text: String,
    /// More words for the tooltip (a blocked reason); may be empty.
    pub detail: String,
}

impl ChipState {
    /// The glyph in front of the text: a check mark or a warning sign.
    pub fn glyph(&self) -> &'static str {
        match self.tone {
            ChipTone::Ok => "\u{2714}",
            ChipTone::Attention => "\u{26A0}",
        }
    }

    /// The state line as the chip paints it: glyph, space, text.
    pub fn line(&self) -> String {
        format!("{} {}", self.glyph(), self.text)
    }
}

/// The chip's frame: the border at rest, the accent when hovered. The
/// accent has a real step from the chip ground in both themes (the
/// hover fill alone is about 1.03:1 in the dark theme).
pub fn chip_edge(pal: &Tokens, hovered: bool) -> (f32, egui::Color32) {
    if hovered { (2.0, pal.accent) } else { (1.0, pal.border) }
}

/// The title's ink: dim at rest, full text when hovered.
pub fn chip_title_ink(pal: &Tokens, hovered: bool) -> egui::Color32 {
    if hovered { pal.text } else { pal.text_dim }
}

/// Paint `text` turned a quarter clockwise, from `top` down, centred on
/// the chip's middle. Returns the bottom of the text, or `None` when it
/// does not fit above `bottom`.
fn turned(painter: &egui::Painter, rect: Rect, top: f32, job: egui::text::LayoutJob, fallback: egui::Color32) -> Option<f32> {
    let galley = painter.layout_job(job);
    let size = galley.size();
    if top + size.x > rect.bottom() - TITLE_PAD {
        return None;
    }
    // Turned about its top-left corner, the text runs down and its box
    // spans `size.y` to the LEFT of `pos`.
    let pos = Pos2::new(rect.center().x + size.y / 2.0, top);
    painter.add(egui::epaint::TextShape::new(pos, galley, fallback).with_angle(std::f32::consts::FRAC_PI_2));
    Some(top + size.x)
}

fn job(sections: &[(&str, egui::Color32)], size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (text, color) in sections {
        job.append(text, 0.0, egui::TextFormat::simple(FontId::proportional(size), *color));
    }
    job
}

/// How much of a chip's state line fitted (#4503).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateFit {
    /// Glyph and text.
    Full,
    /// The pane was too short for the text: the glyph alone.
    GlyphOnly,
    /// Not even the glyph fitted, and the state needs attention: a warn
    /// bar at the chip's foot, so the chip never reads as clean.
    Mark,
    /// Not even the glyph fitted, and all is well: nothing is painted.
    Nothing,
}

/// The height of the warn bar at the foot of a chip (`StateFit::Mark`).
const MARK_BAR: f32 = 4.0;

/// Paint the state line from `top`. When the text does not fit, paint the
/// glyph alone. When not even the glyph fits and the state needs attention,
/// paint a warn bar at the chip's foot, so that a chip whose title fits does
/// not look clean by accident (the text is still in the tooltip).
pub fn paint_state(painter: &egui::Painter, rect: Rect, top: f32, st: &ChipState, pal: &Tokens) -> StateFit {
    let glyph_ink = match st.tone {
        ChipTone::Ok => pal.text_dim,
        ChipTone::Attention => pal.warn_ink,
    };
    let sections = [(st.glyph(), glyph_ink), (" ", pal.text), (st.text.as_str(), pal.text)];
    if turned(painter, rect, top, job(&sections, pal.font_row), pal.text).is_some() {
        return StateFit::Full;
    }
    let glyph = [(st.glyph(), glyph_ink)];
    if turned(painter, rect, top, job(&glyph, pal.font_row), pal.text).is_some() {
        StateFit::GlyphOnly
    } else if st.tone == ChipTone::Attention {
        // The title ends at least `TITLE_PAD` above the foot, so the bar
        // never covers it.
        let bar = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - MARK_BAR), rect.right_bottom());
        painter.rect_filled(bar, 0.0, glyph_ink);
        StateFit::Mark
    } else {
        StateFit::Nothing
    }
}

/// Draw one fold chip in `rect`: a full-height button with the pane's
/// title turned to read top to bottom and, under it, the pane's safety
/// state. The returned response is clicked when the user asks to unfold
/// the pane.
pub fn fold_chip(
    ui: &mut Ui,
    rect: Rect,
    id: egui::Id,
    title: &str,
    state: Option<&ChipState>,
    pal: &Tokens,
) -> Response {
    let resp = ui.interact(rect, id, Sense::click());
    let label = chip_label(title);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    let painter = ui.painter_at(rect);
    let hovered = resp.hovered();
    painter.rect_filled(rect, 0.0, pal.ink_alt);
    let (edge_w, edge) = chip_edge(pal, hovered);
    painter.rect_stroke(rect, 0.0, Stroke::new(edge_w, edge), StrokeKind::Inside);
    let ink = chip_title_ink(pal, hovered);
    let title_bottom = turned(&painter, rect, rect.top() + TITLE_PAD, job(&[(title, ink)], pal.font_row), ink);
    match (title_bottom, state) {
        (Some(bottom), Some(st)) => {
            paint_state(&painter, rect, bottom + 2.0 * TITLE_PAD, st, pal);
        }
        (Some(_), None) => {}
        (None, _) => {
            // Too short for the title: a mark that says the pane is folded.
            painter.text(rect.center(), Align2::CENTER_CENTER, "\u{25B8}", FontId::proportional(pal.font_row), pal.text_dim);
        }
    }
    let mut tip = label;
    if let Some(st) = state {
        tip.push('\n');
        tip.push_str(&st.line());
        if !st.detail.is_empty() {
            tip.push_str(" - ");
            tip.push_str(&st.detail);
        }
    }
    // The chip is as high as the pane: a tooltip under its rect lands far
    // from the pointer, so it follows the pointer.
    resp.on_hover_text_at_pointer(tip)
}

/// The chip's rect for a folded pane: the pane's own rect less its tab bar.
/// `None` when nothing is left to draw in.
pub fn chip_rect(pane: Rect, tab_bar: f32) -> Option<Rect> {
    let r = Rect::from_min_max(pane.min + Vec2::new(0.0, tab_bar.max(0.0)), pane.max);
    (r.is_finite() && r.width() > 0.0 && r.height() > 0.0).then_some(r)
}

#[cfg(test)]
mod tests;
