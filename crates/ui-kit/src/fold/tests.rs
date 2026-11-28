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
fn a_clean_chip_paints_no_warn_bar() {
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
