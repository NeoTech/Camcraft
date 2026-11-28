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

    fn fit_at(ctx: &egui::Context, st: &ChipState, height: f32) -> StateFit {
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(CHIP_WIDTH, height));
        let mut seen = StateFit::Nothing;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            seen = paint_state(&ui.painter_at(rect), rect, 4.0, st, &Tokens::DARK);
        });
        out.textures_delta.clear();
        seen
    }

    #[test]
    fn a_short_pane_paints_the_glyph_alone_not_nothing() {
        let ctx = egui::Context::default();
        let st = ChipState { tone: ChipTone::Attention, text: "1 blocked \u{b7} 3 stale \u{b7} 2 not generated".into(), detail: String::new() };
        assert_eq!(fit_at(&ctx, &st, 600.0), StateFit::Full, "a tall pane has room for the words");
        assert_eq!(fit_at(&ctx, &st, 90.0), StateFit::GlyphOnly, "a short pane keeps the glyph");
        assert_eq!(fit_at(&ctx, &st, 20.0), StateFit::Mark, "a tiny pane keeps a warn bar");
        let ok = ChipState { tone: ChipTone::Ok, text: "Up to date".into(), detail: String::new() };
        assert_eq!(fit_at(&ctx, &ok, 20.0), StateFit::Nothing, "a clean chip may stay bare");
    }

    /// The shapes that `fold_chip` itself paints for `st` in a chip of `height`.
    fn chip_shapes(ctx: &egui::Context, st: &ChipState, height: f32) -> (Rect, Vec<egui::Shape>) {
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(CHIP_WIDTH, height));
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            fold_chip(ui, rect, egui::Id::new("chip"), "Browser", Some(st), &Tokens::DARK);
        });
        out.textures_delta.clear();
        (rect, out.shapes.into_iter().map(|c| c.shape).collect())
    }

    /// The turned texts (title, state) as `(top, bottom)` pairs, in paint order.
    fn turned_spans(shapes: &[egui::Shape]) -> Vec<(f32, f32)> {
        shapes
            .iter()
            .filter_map(|s| match s {
                egui::Shape::Text(t) if t.angle != 0.0 => Some((t.pos.y, t.pos.y + t.galley.size().x)),
                _ => None,
            })
            .collect()
    }

    /// The warn bars: filled rects of the mark's height in the warn ink.
    fn bars(shapes: &[egui::Shape]) -> Vec<Rect> {
        let ink = Tokens::DARK.warn_ink;
        shapes
            .iter()
            .filter_map(|s| match s {
                egui::Shape::Rect(r) if r.fill == ink && (r.rect.height() - MARK_BAR).abs() < 1e-3 => Some(r.rect),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn an_attention_chip_paints_a_state_below_its_title_at_every_height() {
        let ctx = egui::Context::default();
        let states = [
            ChipState { tone: ChipTone::Attention, text: "1 stale".into(), detail: String::new() },
            ChipState { tone: ChipTone::Attention, text: "1 blocked \u{b7} 3 stale \u{b7} 2 not generated".into(), detail: String::new() },
        ];
        let (mut full, mut glyph, mut bar) = (0, 0, 0);
        for st in &states {
            for height in 20..700 {
                let (rect, shapes) = chip_shapes(&ctx, st, height as f32);
                let spans = turned_spans(&shapes);
                let Some(&(_, title_bottom)) = spans.first() else { continue };
                let below = title_bottom + TITLE_PAD;
                if let Some(&(top, _)) = spans.get(1) {
                    assert!(top >= below, "{}: height {height}: the state starts under the title", st.text);
                    if spans.len() == 2 && bars(&shapes).is_empty() {
                        // Full text or glyph alone: told apart by its length.
                        if spans[1].1 - spans[1].0 > 30.0 { full += 1 } else { glyph += 1 }
                    }
                } else {
                    let found = bars(&shapes);
                    assert_eq!(found.len(), 1, "{}: height {height}: no state and no warn bar", st.text);
                    // The title ends at least TITLE_PAD above the foot, the bar is
                    // MARK_BAR high: a gap of TITLE_PAD - MARK_BAR stays at least.
                    assert!(found[0].min.y >= title_bottom + TITLE_PAD - MARK_BAR, "{}: height {height}: the bar clears the title", st.text);
                    assert_eq!(found[0].max.y, rect.bottom(), "{}: height {height}: the bar sits at the foot", st.text);
                    bar += 1;
                }
            }
        }
        assert!(full > 0 && glyph > 0 && bar > 0, "the sweep met every regime ({full} full, {glyph} glyph, {bar} bar)");
    }

    #[test]
    fn a_clean_chip_paints_no_warn_bar_and_no_warn_ink() {
        let ctx = egui::Context::default();
        let ok = ChipState { tone: ChipTone::Ok, text: "Up to date".into(), detail: String::new() };
        for height in 20..700 {
            let (_, shapes) = chip_shapes(&ctx, &ok, height as f32);
            assert!(bars(&shapes).is_empty(), "height {height}: a clean chip has no warn bar");
            assert!(turned_spans(&shapes).len() <= 2, "height {height}: title and state at most");
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
