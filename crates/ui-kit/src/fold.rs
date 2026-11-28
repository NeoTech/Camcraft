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

/// Draw one fold chip in `rect`: a full-height button with the pane's
/// title turned to read top to bottom. The returned response is clicked
/// when the user asks to unfold the pane.
pub fn fold_chip(ui: &mut Ui, rect: Rect, id: egui::Id, title: &str, pal: &Tokens) -> Response {
    let resp = ui.interact(rect, id, Sense::click());
    let label = chip_label(title);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    let painter = ui.painter_at(rect);
    let fill = if resp.hovered() { pal.field_hover } else { pal.ink_alt };
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, pal.border), StrokeKind::Inside);
    let galley = painter.layout_no_wrap(title.to_owned(), FontId::proportional(pal.font_row), pal.text);
    let size = galley.size();
    if size.x + 2.0 * TITLE_PAD <= rect.height() {
        // Turned a quarter clockwise about its top-left corner, the text
        // runs down and its box spans `size.y` to the LEFT of `pos`.
        let pos = Pos2::new(rect.center().x + size.y / 2.0, rect.top() + TITLE_PAD);
        painter.add(egui::epaint::TextShape::new(pos, galley, pal.text).with_angle(std::f32::consts::FRAC_PI_2));
    } else {
        // Too short for the title: a mark that says the pane is folded.
        painter.text(rect.center(), Align2::CENTER_CENTER, "\u{25B8}", FontId::proportional(pal.font_row), pal.text_dim);
    }
    resp.on_hover_text(label)
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
            let _ = ctx.run_ui(input, |ui| {
                let resp = fold_chip(ui, rect, egui::Id::new("chip"), "Browser", &Tokens::DARK);
                clicked |= resp.clicked();
            });
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
}
