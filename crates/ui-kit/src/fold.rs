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
            let glyph_ink = match st.tone {
                ChipTone::Ok => pal.text_dim,
                ChipTone::Attention => pal.warn_ink,
            };
            let sections = [(st.glyph(), glyph_ink), (" ", pal.text), (st.text.as_str(), pal.text)];
            let _ = turned(&painter, rect, bottom + 2.0 * TITLE_PAD, job(&sections, pal.font_row), pal.text);
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
mod tests {
    use super::*;

    #[test]
    fn the_chip_fraction_holds_the_chip_width_on_either_side() {
        let left = chip_fraction(FoldSide::Left, CHIP_WIDTH, 1000.0).expect("a wide split folds");
        assert!((left * 1000.0 - CHIP_WIDTH).abs() < 1e-3);
        let right = chip_fraction(FoldSide::Right, CHIP_WIDTH, 1000.0).expect("a wide split folds");
        assert!(((1.0 - right) * 1000.0 - CHIP_WIDTH).abs() < 1e-3);
    }

    #[test]
    fn an_unlaid_or_narrow_split_keeps_its_fraction() {
        assert_eq!(chip_fraction(FoldSide::Left, CHIP_WIDTH, f32::INFINITY), None);
        assert_eq!(chip_fraction(FoldSide::Left, CHIP_WIDTH, f32::NAN), None);
        assert_eq!(chip_fraction(FoldSide::Right, CHIP_WIDTH, 2.0 * CHIP_WIDTH), None);
        assert_eq!(chip_fraction(FoldSide::Right, 0.0, 500.0), None);
    }

    #[test]
    fn holds_reads_half_a_point_and_open_fraction_refuses_a_bad_share() {
        assert!(holds(0.1, 0.1004, 1000.0));
        assert!(!holds(0.1, 0.101, 1000.0));
        assert_eq!(open_fraction(0.3, 0.2), 0.3);
        for bad in [f32::NAN, f32::INFINITY, 0.0, 1.0, -0.5, 1.5] {
            assert_eq!(open_fraction(bad, 0.2), 0.2, "{bad} is no share");
        }
    }

    #[test]
    fn the_chip_rect_drops_the_tab_bar_and_refuses_an_empty_pane() {
        let pane = Rect::from_min_size(Pos2::new(0.0, 100.0), Vec2::new(CHIP_WIDTH, 400.0));
        let r = chip_rect(pane, 24.0).expect("a tall pane has a chip");
        assert_eq!(r.top(), 124.0);
        assert_eq!(r.width(), CHIP_WIDTH);
        assert_eq!(chip_rect(pane, 400.0), None);
        assert_eq!(chip_rect(Rect::NOTHING, 24.0), None);
    }

    #[test]
    fn the_chip_is_a_button_named_for_its_pane_and_takes_a_click() {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(Pos2::new(10.0, 40.0), Vec2::new(CHIP_WIDTH, 300.0));
        let mut clicked = false;
        let mut run = |events: Vec<egui::Event>| {
            let input = egui::RawInput { events, ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| {
                let st = ChipState { tone: ChipTone::Attention, text: "2 stale".into(), detail: String::new() };
                let resp = fold_chip(ui, rect, egui::Id::new("chip"), "Browser", Some(&st), &Tokens::DARK);
                clicked |= resp.clicked();
            });
            // The font atlas delta must be taken, or epaint refuses the drop.
            out.textures_delta.clear();
        };
        let at = rect.center();
        run(vec![egui::Event::PointerMoved(at)]);
        let press = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        run(vec![press(true)]);
        run(vec![press(false)]);
        assert!(clicked, "a click on the chip is the unfold ask");
        assert_eq!(chip_label("Browser"), "Unfold Browser");
    }

    #[test]
    fn the_hover_step_is_visible_in_both_themes() {
        for (name, pal) in [("dark", Tokens::DARK), ("light", Tokens::LIGHT)] {
            let (rest_w, rest) = chip_edge(&pal, false);
            let (hover_w, hover) = chip_edge(&pal, true);
            assert!(hover_w > rest_w, "{name}: the hovered frame is wider");
            let step = crate::contrast::ratio(hover, pal.ink_alt);
            assert!(step >= 2.0, "{name}: the hover frame reads on the chip ({step:.2}:1)");
            assert!(crate::contrast::ratio(hover, rest) > 1.2, "{name}: hover and rest frames differ");
            let title = crate::contrast::ratio(chip_title_ink(&pal, true), pal.ink_alt);
            assert!(title >= 4.5, "{name}: the hovered title is readable ({title:.2}:1)");
        }
    }

    #[test]
    fn a_state_line_carries_a_glyph_and_words() {
        let ok = ChipState { tone: ChipTone::Ok, text: "Up to date".into(), detail: String::new() };
        let act = ChipState { tone: ChipTone::Attention, text: "Needs regenerate".into(), detail: String::new() };
        assert_eq!(ok.line(), "\u{2714} Up to date");
        assert_eq!(act.line(), "\u{26A0} Needs regenerate");
        assert_ne!(ok.glyph(), act.glyph(), "the glyph alone tells the two apart");
    }
}
