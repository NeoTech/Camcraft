//! #4483: docked panes fold to title chips. A side pane (Browser at the
//! left, Properties at the right) folds to a chip with the dock's fold
//! arrow and unfolds with one click on its chip; a stacked pane folds
//! natively to its tab bar. A fold survives a separator drag and a window
//! resize, rides the saved panel setups, and Reset layout clears it. A
//! setup from an older build loads with every pane unfolded. None of it
//! touches the command queue (layout is view state).
//! Gate map: covers ui::egui_app::layout::fold, ui::egui_app::layout::setups
//! and camcraft_ui_kit::fold (the chip).
//! Provenance: our own tests, specced from the egui_dock 0.21.1 and
//! egui_kittest vendored sources; no HSMWorks material involved.

use egui_dock::{Node, NodePath, Split, SurfaceIndex};

use camcraft_ui_kit::fold::CHIP_WIDTH;

use super::super::layout::{self, fold};
use super::layout::leaf_rect;
use super::*;
use crate::state::{LayoutRequest, PanelSetupAsk};

type H = Harness<'static, ShellUi>;

/// The plan's target screen (#4245): 1366 x 768.
fn harness() -> H {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1366.0, 768.0))
        .build_ui_state(frame, ShellUi::blank());
    h.run_steps(3);
    h
}

fn width(h: &H, tab: &Tab) -> f32 {
    leaf_rect(&h.state().dock, tab).map_or(f32::NAN, |r| r.width())
}

fn click(h: &mut H, at: egui::Pos2) {
    set_pos(h, at);
    button_at(h, egui::PointerButton::Primary, at, true);
    button_at(h, egui::PointerButton::Primary, at, false);
    h.run_steps(3);
}

/// The dock's own fold arrow sits at the top-left corner of each leaf.
fn click_fold_arrow(h: &mut H, tab: &Tab) {
    let r = leaf_rect(&h.state().dock, tab).expect("the pane is laid out");
    click(h, r.left_top() + Vec2::new(12.0, 10.0));
}

fn chip(h: &H, title: &str) -> Option<egui::Rect> {
    let label = camcraft_ui_kit::fold::chip_label(title);
    h.query_by_role_and_label(egui::accesskit::Role::Button, &label).map(|n| n.rect())
}

fn assert_no_commands(h: &H) {
    let st = &h.state().state;
    assert!(st.pending.is_empty(), "a fold queued a command: {:?}", st.pending);
    assert!(st.history.is_empty(), "a fold landed in the history");
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 2.0
}

#[test]
fn the_browser_folds_to_a_chip_and_its_chip_unfolds_it() {
    let mut h = harness();
    let open = width(&h, &Tab::Browser);
    assert!(open > 4.0 * CHIP_WIDTH, "the browser starts open: {open}");
    assert!(chip(&h, "Browser").is_none(), "an open pane draws no chip");
    click_fold_arrow(&mut h, &Tab::Browser);
    assert!(fold::is_folded(&h.state().dock, &Tab::Browser), "the arrow folds the browser");
    assert!(near(width(&h, &Tab::Browser), CHIP_WIDTH), "folded to chip width: {}", width(&h, &Tab::Browser));
    let Some(c) = chip(&h, "Browser") else { panic!("the folded browser draws its chip") };
    let leaf = leaf_rect(&h.state().dock, &Tab::Browser).expect("still docked");
    assert!(leaf.contains(c.center()), "the chip {c:?} is in the folded leaf {leaf:?}");
    assert!(h.state().dock.find_tab(&Tab::Browser).is_some(), "the folded pane stays docked");
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Unfold Browser").click();
    h.run_steps(3);
    assert!(!fold::is_folded(&h.state().dock, &Tab::Browser), "one click on the chip unfolds");
    assert!(near(width(&h, &Tab::Browser), open), "back at its open width {open}: {}", width(&h, &Tab::Browser));
    assert!(chip(&h, "Browser").is_none(), "the chip goes with the fold");
    assert_no_commands(&h);
}

#[test]
fn the_properties_pane_folds_at_the_right_and_the_arrow_unfolds_it() {
    let mut h = harness();
    let open = width(&h, &Tab::Properties);
    let vp_open = width(&h, &Tab::Viewport);
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Properties));
    h.run_steps(3);
    assert!(near(width(&h, &Tab::Properties), CHIP_WIDTH), "folded: {}", width(&h, &Tab::Properties));
    assert!(width(&h, &Tab::Viewport) > vp_open + open - 2.0 * CHIP_WIDTH, "the viewport takes the room");
    let props = leaf_rect(&h.state().dock, &Tab::Properties).expect("still docked");
    let vp = leaf_rect(&h.state().dock, &Tab::Viewport).expect("the viewport");
    assert!(props.left() >= vp.right() - 1.0, "the chip stays at the right side");
    assert!(chip(&h, "Properties").is_some(), "the folded pane draws its chip");
    click_fold_arrow(&mut h, &Tab::Properties);
    assert!(!fold::is_folded(&h.state().dock, &Tab::Properties), "the arrow unfolds too");
    assert!(near(width(&h, &Tab::Properties), open), "back at {open}: {}", width(&h, &Tab::Properties));
    assert_no_commands(&h);
}

#[test]
fn a_stacked_pane_folds_natively_to_its_tab_bar() {
    let mut h = harness();
    {
        let dock = &mut h.state_mut().dock;
        let props = dock.find_tab(&Tab::Properties).expect("the default dock has Properties");
        dock.split(
            NodePath::new(SurfaceIndex::main(), props.node),
            Split::Below,
            0.5,
            Node::leaf(Tab::Machines),
        );
    }
    h.run_steps(3);
    let open = leaf_rect(&h.state().dock, &Tab::Machines).expect("Machines is docked");
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Machines));
    h.run_steps(3);
    let folded = leaf_rect(&h.state().dock, &Tab::Machines).expect("still docked");
    assert!(folded.height() < open.height() / 4.0, "folded to its tab bar: {folded:?} from {open:?}");
    assert!(near(folded.width(), open.width()), "a stacked fold keeps the width");
    assert!(chip(&h, "Machines").is_none(), "a stacked fold shows its own tab bar, no chip");
    let mut folds = std::mem::take(&mut h.state_mut().state.pane_folds);
    fold::unfold(&mut h.state_mut().dock, &mut folds, &Tab::Machines);
    h.state_mut().state.pane_folds = folds;
    h.run_steps(3);
    let back = leaf_rect(&h.state().dock, &Tab::Machines).expect("still docked");
    assert!(near(back.height(), open.height()), "unfolded: {back:?} vs {open:?}");
}

#[test]
fn a_separator_drag_does_not_unfold_a_folded_pane() {
    let mut h = harness();
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Browser));
    h.run_steps(3);
    let leaf = leaf_rect(&h.state().dock, &Tab::Browser).expect("docked");
    let from = egui::pos2(leaf.right() + 1.0, leaf.center().y);
    let to = from + Vec2::new(240.0, 0.0);
    set_pos(&mut h, from);
    button_at(&mut h, egui::PointerButton::Primary, from, true);
    set_pos(&mut h, from + Vec2::new(60.0, 0.0));
    set_pos(&mut h, to);
    button_at(&mut h, egui::PointerButton::Primary, to, false);
    h.run_steps(3);
    assert!(fold::is_folded(&h.state().dock, &Tab::Browser), "the drag left the fold on");
    assert!(near(width(&h, &Tab::Browser), CHIP_WIDTH), "still at chip width: {}", width(&h, &Tab::Browser));
}

#[test]
fn a_window_resize_keeps_the_fold_at_chip_width() {
    let mut h = harness();
    let share = width(&h, &Tab::Browser) / 1366.0;
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Browser));
    h.run_steps(3);
    h.set_size(Vec2::new(1920.0, 1080.0));
    h.run_steps(4);
    assert!(near(width(&h, &Tab::Browser), CHIP_WIDTH), "chip width after the resize: {}", width(&h, &Tab::Browser));
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Unfold Browser").click();
    h.run_steps(3);
    let back = width(&h, &Tab::Browser) / 1920.0;
    assert!((back - share).abs() < 0.01, "the open share comes back: {back} vs {share}");
}

#[test]
fn the_viewport_never_folds() {
    let mut h = harness();
    assert!(!fold::fold(&mut h.state_mut().dock, &Tab::Viewport), "the fold refuses the viewport");
    // The dock's own arrow sets the flag: the fold law clears it.
    let open = width(&h, &Tab::Viewport);
    click_fold_arrow(&mut h, &Tab::Viewport);
    assert!(!fold::is_folded(&h.state().dock, &Tab::Viewport), "the viewport stays open");
    assert!(near(width(&h, &Tab::Viewport), open), "and keeps its width");
}

#[test]
fn reset_layout_unfolds_every_pane() {
    let mut h = harness();
    let open = width(&h, &Tab::Browser);
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Browser));
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Properties));
    h.run_steps(3);
    h.state_mut().state.layout_request = Some(LayoutRequest::Reset);
    h.run_steps(3);
    for tab in [Tab::Browser, Tab::Properties] {
        assert!(!fold::is_folded(&h.state().dock, &tab), "{tab:?} is open after the reset");
    }
    assert!(h.state().state.pane_folds.is_empty(), "no stored share outlives the reset");
    assert!(near(width(&h, &Tab::Browser), open), "the default width: {}", width(&h, &Tab::Browser));
    assert_no_commands(&h);
}

#[test]
fn a_saved_setup_keeps_its_folds_and_their_open_width() {
    let mut h = harness();
    let open = width(&h, &Tab::Browser);
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Browser));
    h.run_steps(3);
    h.state_mut().state.panel_setup_ask = Some(PanelSetupAsk::Save(String::from("Folded")));
    h.run_steps(2);
    assert_eq!(h.state().state.status, "panel setup 'Folded' saved");
    h.state_mut().state.layout_request = Some(LayoutRequest::Reset);
    h.run_steps(3);
    assert!(!fold::is_folded(&h.state().dock, &Tab::Browser), "the reset unfolded it");
    h.state_mut().state.panel_setup_ask = Some(PanelSetupAsk::Load(String::from("Folded")));
    h.run_steps(4);
    assert!(fold::is_folded(&h.state().dock, &Tab::Browser), "the load folds it again");
    assert!(near(width(&h, &Tab::Browser), CHIP_WIDTH), "at chip width: {}", width(&h, &Tab::Browser));
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Unfold Browser").click();
    h.run_steps(3);
    assert!(near(width(&h, &Tab::Browser), open), "the saved open width {open}: {}", width(&h, &Tab::Browser));
    assert_no_commands(&h);
}

#[test]
fn an_older_setup_loads_with_every_pane_unfolded() {
    let mut dock = build_dock();
    assert!(fold::fold(&mut dock, &Tab::Browser));
    let mut json = layout::setups::encode(&dock).expect("a dock is writable");
    assert!(
        layout::setups::decode(&json).is_ok_and(|d| fold::is_folded(&d, &Tab::Browser)),
        "a setup of this build keeps its fold"
    );
    // An older build wrote no fold mark: its collapsed flags did nothing.
    json.as_object_mut().map(|m| m.remove(fold::FOLD_MARK));
    let loaded = layout::setups::decode(&json).expect("an older setup loads");
    for tab in [Tab::Browser, Tab::Viewport, Tab::Properties] {
        assert!(!fold::is_folded(&loaded, &tab), "{tab:?} loads open");
    }
}

// ------------------------------------------------ the state on the chips

/// The seeded shell (Contour1, generated) at the plan's screen size.
fn seeded() -> H {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1366.0, 768.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.run_steps(3);
    h
}

/// Make the seed's Contour1 stale through the command queue (an edit
/// that changes what generation read).
fn make_stale(h: &mut H) {
    let edited = {
        let st = &h.state().state;
        let e = model::find_by_name(&st.world, "Contour1").expect("the seed's op");
        let mut p = model::operation_of(&st.world, &e).expect("op").params;
        if let OpParams::Contour { plunge_feed, .. } = &mut p {
            *plunge_feed = 150.0;
        }
        p
    };
    h.state_mut()
        .state
        .pending
        .push_back(CommandKind::EditOperationParams { name: "Contour1".into(), params: edited });
    h.run_steps(3);
}

/// The warnings of the seed's one cached toolpath, as the tree row counts them.
fn seed_warnings(h: &H) -> usize {
    let w = &h.state().state.world;
    let e = model::find_by_name(w, "Contour1").expect("the seed's op");
    let n = model::with_toolpath(w, &e, |tp| tp.warnings.len()).unwrap_or(0);
    // A literal, so that a miscount in the chip's counter fails here.
    assert_eq!(n, 89, "the seed's cached toolpath warnings");
    n
}

/// The texts painted inside the leaf of `tab`, under its tab bar.
fn chip_texts(h: &H, tab: &Tab) -> Vec<String> {
    let leaf = leaf_rect(&h.state().dock, tab).expect("docked");
    super::structural::painted(h)
        .into_iter()
        .filter(|(_, p)| p.x >= leaf.left() - 1.0 && p.x <= leaf.right() + 1.0 && p.y > leaf.top() + 20.0)
        .map(|(t, _)| t)
        .collect()
}

#[test]
fn a_stale_operation_shows_on_the_folded_browser_chip() {
    let mut h = seeded();
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Browser));
    h.run_steps(3);
    let texts = chip_texts(&h, &Tab::Browser);
    // The seed's toolpath has warnings, which the chip counts (#4503), so
    // it is not "Up to date" and it is not stale either.
    let want = format!("\u{26A0} {} warnings", seed_warnings(&h));
    assert!(texts.contains(&want), "a current job: {texts:?}");
    make_stale(&mut h);
    let texts = chip_texts(&h, &Tab::Browser);
    let want = format!("\u{26A0} 1 stale \u{b7} {} warnings", seed_warnings(&h));
    assert!(texts.contains(&want), "the stale count stays on screen: {texts:?}");
}

#[test]
fn the_folded_properties_chip_names_the_operation_state() {
    let mut h = seeded();
    crate::actions::target(&mut h.state_mut().state, UiTarget::Operation("Contour1".into()));
    h.run_steps(2);
    assert!(fold::fold(&mut h.state_mut().dock, &Tab::Properties));
    h.run_steps(3);
    let texts = chip_texts(&h, &Tab::Properties);
    assert!(texts.iter().any(|t| t == "\u{2714} Up to date"), "the open operation is current: {texts:?}");
    make_stale(&mut h);
    let texts = chip_texts(&h, &Tab::Properties);
    assert!(texts.iter().any(|t| t == "\u{26A0} Needs regenerate"), "the open operation needs work: {texts:?}");
}

#[test]
fn a_fold_does_not_move_the_camera() {
    let mut h = seeded();
    let before = h.state().state.cam;
    for tab in [Tab::Browser, Tab::Properties] {
        assert!(fold::fold(&mut h.state_mut().dock, &tab));
    }
    h.run_steps(3);
    let folded = h.state().state.cam;
    h.state_mut().state.layout_request = Some(LayoutRequest::Reset);
    h.run_steps(3);
    let after = h.state().state.cam;
    for cam in [folded, after] {
        assert_eq!(cam.target, before.target, "the orbit target stays");
        assert_eq!((cam.yaw, cam.pitch, cam.dist), (before.yaw, before.pitch, before.dist), "the view stays");
    }
}

/// Add a Bore operation through the command queue, with Contour1's tool.
fn add_op(h: &mut H, name: &str) {
    let tool = contour_tool(h);
    add_op_with(h, name, &tool);
}

fn add_op_with(h: &mut H, name: &str, tool: &str) {
    let index = camcraft_app::operation::ALL_LABELS.iter().position(|s| *s == "Bore").expect("a Bore kind");
    h.state_mut().state.pending.push_back(CommandKind::CreateOperation {
        name: name.into(),
        tool: tool.into(),
        src: SrcRef::Stock,
        params: crate::operations::quick_params(index),
    });
    h.run_steps(3);
}

fn contour_tool(h: &H) -> String {
    let w = &h.state().state.world;
    let e = model::find_by_name(w, "Contour1").expect("the seed's op");
    model::operation_of(w, &e).expect("op").tool
}

#[test]
fn a_never_generated_operation_shows_on_both_folded_chips() {
    let mut h = seeded();
    add_op(&mut h, "Bore9");
    crate::actions::target(&mut h.state_mut().state, UiTarget::Operation("Bore9".into()));
    for tab in [Tab::Browser, Tab::Properties] {
        assert!(fold::fold(&mut h.state_mut().dock, &tab));
    }
    h.run_steps(3);
    let browser = chip_texts(&h, &Tab::Browser);
    let want = format!("\u{26A0} 1 not generated \u{b7} {} warnings", seed_warnings(&h));
    assert!(browser.contains(&want), "Browser: {browser:?}");
    let props = chip_texts(&h, &Tab::Properties);
    assert!(props.iter().any(|t| t == "\u{26A0} Not generated"), "Properties: {props:?}");
}

#[test]
fn a_blocked_operation_shows_on_both_folded_chips() {
    let mut h = seeded();
    // Deleting a tool entity by name leaves its operations without it.
    let tool = contour_tool(&h);
    h.state_mut().state.pending.push_back(CommandKind::DeleteByName(tool));
    h.run_steps(3);
    crate::actions::target(&mut h.state_mut().state, UiTarget::Operation("Contour1".into()));
    for tab in [Tab::Browser, Tab::Properties] {
        assert!(fold::fold(&mut h.state_mut().dock, &tab));
    }
    h.run_steps(3);
    let browser = chip_texts(&h, &Tab::Browser);
    let want = format!("\u{26A0} 1 blocked \u{b7} {} warnings", seed_warnings(&h));
    assert!(browser.contains(&want), "Browser: {browser:?} ({})", h.state().state.status);
    let props = chip_texts(&h, &Tab::Properties);
    assert!(props.iter().any(|t| t == "\u{26A0} Blocked"), "Properties: {props:?}");
}
