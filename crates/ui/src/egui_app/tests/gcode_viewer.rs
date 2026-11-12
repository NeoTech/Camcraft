//! #256 in the shell: the G-code viewer lists every program line from
//! the same left edge, the way a program reads, and a line wider than
//! the window stays on its one row so the scroll cursor stays exact.
//! Provenance: our own tests over our own viewer; no HSMWorks material
//! involved.

use super::*;

/// A program whose lines differ a lot in length, plus one comment far
/// wider than the viewer's default 560 px.
const PROGRAM: [&str; 5] = [
    "%",
    "G00 G90 G54 X12.5 Y-40. S8000 M3",
    "M30",
    "(A COMMENT FAR WIDER THAN THE VIEWER WINDOW - TOOL 1 IS A 6 MM FLAT END MILL, 3 FLUTES, 20 MM FLUTE LENGTH, 50 MM STICKOUT)",
    "%",
];

/// The viewer's label for program line `i` (0-based): the posted line
/// alone - the gutter number is its own, non-selectable label (#1089),
/// so a copy never carries it (`gcode_view::gcode_listing`).
fn row_label(i: usize) -> String {
    PROGRAM[i].to_owned()
}

/// The shell with the viewer open on [`PROGRAM`], laid out.
fn viewer() -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    harness.state_mut().state.post_view = Some(PostView {
        path: std::path::PathBuf::from("p.nc"),
        program: PROGRAM.join("\n"),
        line_pos: vec![None; PROGRAM.len()],
        open: true,
        line: 0,
        blocks: Vec::new(),
        trace: false,
        not_judged: None,
    });
    harness.run();
    harness
}

/// Where the frame painted program line `i`'s text: the left edge of
/// its galley, read off the frame's shapes. The accessibility rect is
/// the row's box, which spans the row whether the text inside it is
/// centred or not, so it cannot tell the two apart.
fn painted_left(h: &Harness<'static, ShellUi>, i: usize) -> f32 {
    fn find(shape: &egui::Shape, text: &str) -> Option<f32> {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => Some(t.pos.x + t.galley.rect.left()),
            egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, text)),
            _ => None,
        }
    }
    let text = row_label(i);
    h.output()
        .shapes
        .iter()
        .find_map(|c| find(&c.shape, &text))
        .unwrap_or_else(|| panic!("the frame painted no {text:?}"))
}

#[test]
fn every_gcode_line_starts_at_the_listings_left_edge() {
    let harness = viewer();
    let left = |i: usize| painted_left(&harness, i);
    let (short, long) = (left(2), left(1));
    assert!(
        (short - long).abs() < 0.5,
        "\"M30\" starts at x {short}, the longer motion line at x {long}: centred, not listed"
    );
    assert!(
        (left(0) - long).abs() < 0.5,
        "the '%' line starts there too"
    );
}

#[test]
fn a_line_wider_than_the_viewer_stays_on_one_row() {
    let harness = viewer();
    let rect = |i: usize| harness.get_by_label(&row_label(i)).rect();
    let (wide, short) = (rect(3), rect(2));
    assert!(
        (wide.height() - short.height()).abs() < 0.5,
        "the wide comment is {} tall, a one-line row {}: it wrapped",
        wide.height(),
        short.height()
    );
    assert!(
        (wide.left() - short.left()).abs() < 0.5,
        "and it starts at the same left edge"
    );
}

#[test]
fn gcode_words_are_coloured_by_kind_and_the_text_is_untouched() {
    let harness = viewer();
    fn job(shape: &egui::Shape, text: &str) -> Option<std::sync::Arc<egui::text::LayoutJob>> {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => Some(t.galley.job.clone()),
            egui::Shape::Vec(v) => v.iter().find_map(|s| job(s, text)),
            _ => None,
        }
    }
    let colours = |i: usize| -> Vec<egui::Color32> {
        let text = row_label(i);
        let j = harness
            .output()
            .shapes
            .iter()
            .find_map(|c| job(&c.shape, &text))
            .expect("row painted");
        assert_eq!(j.text, text, "display only: the text is the posted line");
        let mut v: Vec<_> = j.sections.iter().map(|s| s.format.color).collect();
        v.sort_by_key(|c| c.to_array());
        v.dedup();
        v
    };
    assert!(colours(1).len() >= 4, "motion line: G, axis, number, feed...");
    assert!(colours(3)[0] != colours(1)[0] || colours(3).len() == 1, "comment has its ink");
}

/// A long program with a distinct text per line, the viewer open.
fn long_viewer(lines: usize) -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    harness.state_mut().state.post_view = Some(PostView {
        path: std::path::PathBuf::from("p.nc"),
        program: (0..lines)
            .map(|i| format!("G01 X{i}.5 Y0. F200."))
            .collect::<Vec<_>>()
            .join("
"),
        line_pos: (0..lines).map(|i| Some([i as f64, 0.0, 0.0])).collect(),
        open: true,
        line: 0,
        blocks: Vec::new(),
        trace: false,
        not_judged: None,
    });
    harness.run();
    harness
}

fn cursor(h: &Harness<'static, ShellUi>) -> usize {
    h.state().state.post_view.as_ref().map_or(usize::MAX, |v| v.line)
}

fn window_rect(h: &Harness<'static, ShellUi>) -> egui::Rect {
    h.ctx
        .memory(|m| m.area_rect(egui::Id::new("gcode-viewer")))
        .expect("the viewer window is up")
}

#[test]
fn ticking_trace_keeps_the_window_its_size() {
    let mut harness = viewer();
    let before = window_rect(&harness);
    harness.get_by_label("trace").click();
    harness.run_steps(5);
    assert!(harness.state().state.post_view.as_ref().is_some_and(|v| v.trace));
    let after = window_rect(&harness);
    assert!(
        (after.width() - before.width()).abs() < 1.0 && (after.height() - before.height()).abs() < 1.0,
        "window went {before:?} -> {after:?} when trace was ticked"
    );
    harness.get_by_label("trace").click();
    harness.run_steps(5);
    let back = window_rect(&harness);
    assert!((back.width() - before.width()).abs() < 1.0, "and back: {back:?}");
}

#[test]
fn clicking_a_line_moves_the_cursor_there() {
    // First row, a row in the top half page, a middle row, the last
    // row: each click puts the cursor on exactly that row, the band
    // follows it, and the viewport's segment is its move (#1089).
    let mut harness = long_viewer(300);
    harness.get_by_label("G01 X150.5 Y0. F200.").scroll_to_me();
    harness.run_steps(20);
    let mid = cursor(&harness);
    assert!(mid > 100, "scrolled to the middle of the program: {mid}");
    for target in [mid + 3, 0usize, 4, 150, 299, 295, 1] {
        let label = format!("G01 X{target}.5 Y0. F200.");
        // Bring the row into view first (a click needs it on screen).
        harness.get_by_label(&label).scroll_to_me();
        harness.run_steps(10);
        harness.get_by_label(&label).click();
        harness.run_steps(30);
        assert_eq!(cursor(&harness), target, "the click became the cursor");
        assert!(
            super::super::shell::gcode_segment(&harness.state().state).is_some(),
            "the amber segment follows row {target}"
        );
    }
}

#[test]
fn the_cursor_line_is_highlighted() {
    let mut harness = long_viewer(300);
    harness.get_by_label("G01 X150.5 Y0. F200.").scroll_to_me();
    harness.run_steps(20);
    let cur = cursor(&harness);
    let fill = harness
        .ctx
        .global_style()
        .visuals
        .selection
        .bg_fill
        .linear_multiply(0.45);
    fn rects(shape: &egui::Shape, fill: egui::Color32, out: &mut Vec<egui::Rect>) {
        match shape {
            egui::Shape::Rect(r) if r.fill == fill => out.push(r.rect),
            egui::Shape::Vec(v) => v.iter().for_each(|s| rects(s, fill, out)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    for c in &harness.output().shapes {
        rects(&c.shape, fill, &mut found);
    }
    let row = harness.get_by_label(&format!("G01 X{cur}.5 Y0. F200.")).rect();
    assert!(
        found.iter().any(|r| r.contains(row.center())),
        "no highlight band behind the cursor line {cur} ({row:?}): {found:?}"
    );
    assert_eq!(found.len(), 1, "exactly one line is highlighted");
}

#[test]
fn dragging_across_lines_and_copying_takes_the_code_not_the_numbers() {
    let mut harness = viewer();
    let a = harness.get_by_label(&row_label(1)).rect();
    let b = harness.get_by_label(&row_label(2)).rect();
    harness.hover_at(a.left_center() + egui::vec2(1.0, 0.0));
    harness.event(egui::Event::PointerButton {
        pos: a.left_center() + egui::vec2(1.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    harness.event(egui::Event::PointerMoved(b.right_center() + egui::vec2(-1.0, 0.0)));
    harness.step();
    harness.event(egui::Event::PointerButton {
        pos: b.right_center() + egui::vec2(-1.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    harness.event(egui::Event::Copy);
    harness.step();
    let copied: Vec<String> = harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    let text = copied.join("
");
    assert!(text.contains("G00 G90 G54 X12.5"), "the line is copyable: {copied:?}");
    assert!(text.contains("M30"), "and so is the next: {copied:?}");
    assert!(!text.contains("   2  "), "without the gutter number: {copied:?}");
}
