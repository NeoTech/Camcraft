//! #1444, #4004: UI pins for the undercut cutter tool editor (dovetail
//! flank angle and corner radius, undercut kind defaults, hidden corner
//! radius for disc/ball undercut kinds) and the Slot Undercut operation
//! page (`undercut_page.rs`).
//! Provenance: our own structural UI tests over the tool editor and
//! operation page; no vendor material consulted.

use camcraft_app::model;
use camcraft_app::operation::{KindInfo, OpType};
use camcraft_app::tools::ToolKind;
use camcraft_app::undercut;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::*;

fn open_new_tool_dialog() -> Harness<'static, ShellUi> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 960.0))
        .build_ui_state(frame, ShellUi::blank());
    h.run_steps(2);
    h.get_by_label("View").click();
    h.run_steps(2);
    h.get_by_label("Tool library…").click();
    h.run_steps(3);
    h.get_by_label("New tool…").click();
    h.run_steps(3);
    h
}

#[test]
fn new_tool_undercut_kinds_seed_sane_defaults_and_dovetail_corner_radius() {
    let cases = [
        ("T-slot cutter", ToolKind::TSlot, "NewTSlot"),
        ("Keyseat cutter", ToolKind::Keyseat, "NewKeyseat"),
        ("Dovetail cutter", ToolKind::Dovetail, "NewDovetail"),
        ("Lollipop cutter", ToolKind::Lollipop, "NewLollipop"),
    ];
    for (label, kind, name) in cases {
        let mut h = open_new_tool_dialog();
        h.query_all_by_role(Role::ComboBox)
            .find(|n| n.value().as_deref() == Some("End mill"))
            .expect("Type combo showing End mill")
            .click();
        h.run_steps(3);
        h.get_by_label(label).click_accesskit();
        h.run_steps(3);

        assert_eq!(h.state().state.tool_ui.tool_kind(), kind);
        assert_eq!(
            h.query_by_label("Corner radius").is_some(),
            kind == ToolKind::Dovetail,
            "{label} Corner radius visibility"
        );
        assert_eq!(
            h.query_by_label("Flank angle (deg)").is_some(),
            kind == ToolKind::Dovetail,
            "{label} flank angle visibility"
        );
        if kind == ToolKind::Dovetail {
            h.state_mut().state.tool_ui.corner_r = 0.254;
        }

        h.state_mut().state.tool_ui.name = name.into();
        h.run_steps(2);
        h.get_by_label("Add tool").click_accesskit();
        h.run_steps(3);

        assert_eq!(
            h.state().state.error_for(crate::state::ErrorScope::Tool),
            None,
            "{label} should add without error"
        );
        let saved = model::find_by_name(&h.state().state.world, name)
            .and_then(|e| model::tool_of(&h.state().state.world, &e))
            .unwrap_or_else(|| panic!("{name} added to world"));
        assert_eq!(saved.kind, kind);
        assert_eq!(saved.why_not_sane(), None, "{label}: {saved:?}");
        assert!(undercut::Undercut::of(&saved).is_some(), "{label}");
        let want_cr = if kind == ToolKind::Dovetail { 0.254 } else { 0.0 };
        assert_eq!(saved.corner_r, want_cr, "{label} corner_r preserved");
    }
}

#[test]
fn dovetail_properties_and_editor_edit_flank_angle_and_corner_radius() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 960.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.run_steps(2);
    let dove = camcraft_app::tools::Tool {
        corner_r: 0.254,
        ..undercut::cutter(ToolKind::Dovetail, 19.05, 9.525, 8.0, 30.0, 60.0)
    };
    h.state_mut().state.pending.push_back(CommandKind::AddTool {
        name: "DT-1905-60".into(),
        tool: dove,
    });
    h.run_steps(2);

    crate::actions::target(&mut h.state_mut().state, UiTarget::Tool("DT-1905-60".into()));
    h.run_steps(3);
    assert!(h.query_by_label("Corner radius").is_some(), "the Properties summary shows Corner radius for dovetail");
    assert!(h.query_by_label("0.254 mm").is_some(), "the Properties summary shows the dovetail corner radius value");

    // #3930: the settings are edited in the tool library window (Properties
    // shows a summary), so flank angle and corner radius are edited there.
    super::tool_library_editor::open_in_window(&mut h, "DT-1905-60");
    assert!(
        super::tool_library_editor::library_window(&h).query_by_label("Flank angle (deg)").is_some(),
        "the window shows Flank angle (deg)"
    );
    assert!(
        super::tool_library_editor::library_window(&h).query_by_label("Corner radius").is_some(),
        "the window shows Corner radius for dovetail"
    );

    let angle_drag = super::tool_library_editor::library_window(&h)
        .query_all_by_role(Role::SpinButton)
        .find(|n| n.accesskit_node().numeric_value() == Some(60.0))
        .expect("flank angle SpinButton")
        .rect()
        .center();
    h.drag_at(angle_drag);
    h.run();
    h.hover_at(angle_drag + Vec2::new(-50.0, 0.0));
    h.run();
    h.drop_at(angle_drag + Vec2::new(-50.0, 0.0));
    h.run();
    let staged_angle = super::tool_library_editor::draft(&mut h).tip_angle;
    assert!((staged_angle - 60.0).abs() > 0.5, "dragging flank angle stages tip_angle: {staged_angle}");

    let cr_row = super::tool_library_editor::library_window(&h)
        .query_all_by_label("Corner radius")
        .next()
        .expect("Corner radius label")
        .rect();
    let cr_drag = super::tool_library_editor::library_window(&h)
        .query_all_by_role(Role::SpinButton)
        .find(|n| (cr_row.top()..=cr_row.bottom()).contains(&n.rect().center().y))
        .expect("corner radius SpinButton")
        .rect()
        .center();
    h.drag_at(cr_drag);
    h.run();
    h.hover_at(cr_drag + Vec2::new(40.0, 0.0));
    h.run();
    h.drop_at(cr_drag + Vec2::new(40.0, 0.0));
    h.run();
    let staged_cr = super::tool_library_editor::draft(&mut h).corner_r;
    assert!((staged_cr - 0.254).abs() > 0.05, "dragging corner radius stages corner_r: {staged_cr}");

    super::tool_library_editor::click_window_button(&mut h, "Apply");
    let updated = model::find_by_name(&h.state().state.world, "DT-1905-60")
        .and_then(|e| model::tool_of(&h.state().state.world, &e))
        .expect("updated dovetail");
    assert!((updated.tip_angle - staged_angle).abs() < 1e-6, "Apply updates tip_angle in world");
    assert!((updated.corner_r - staged_cr).abs() < 1e-6, "Apply updates corner_r in world");

    // #2397: the body and profile have their own window; its cutter
    // parameters show the dovetail's flank angle and corner radius.
    crate::actions::open_tool_shape_editor(&mut h.state_mut().state, "DT-1905-60");
    h.run_steps(3);
    // The cutter parameters stand on the Profile tab's drawn cutter.
    h.get_by_label("Profile").click();
    h.run_steps(2);
    h.get_by_label("Draw cutter").click();
    h.run_steps(3);
    assert!(h.query_by_label("flank angle °").is_some(), "shape window shows the flank angle");
    assert!(h.query_by_label("corner R").is_some(), "shape window shows the corner radius");
}

#[test]
fn slot_undercut_page_shows_its_geometry_heights_passes_and_feeds_fields() {
    let mut h = shell_harness();
    let idx = KindInfo::ALL
        .iter()
        .position(|k| k.ty == OpType::SlotUndercut)
        .expect("SlotUndercut kind");
    let cur = h.state().state.op_ui.params.clone();
    h.state_mut().state.op_ui.params = crate::operations::retype(&cur, idx);

    let tab_pos = |name: &str| {
        super::super::op_tabs::SECTIONS
            .iter()
            .position(|s| *s == name)
            .unwrap_or_else(|| panic!("tab {name}"))
    };

    h.state_mut().state.op_tab = tab_pos("Geometry");
    h.run_steps(2);
    for label in ["A X", "A Y", "B X", "B Y", "Bulge", "Undercut width", "Entry run-in", "Neck slot width"] {
        assert!(h.query_by_label(label).is_some(), "Geometry shows {label}");
    }

    h.state_mut().state.op_tab = tab_pos("Heights");
    h.run_steps(2);
    for label in ["Clearance Z", "Top Z", "Bottom Z"] {
        assert!(h.query_by_label(label).is_some(), "Heights shows {label}");
    }

    h.state_mut().state.op_tab = tab_pos("Passes");
    h.run_steps(2);
    assert!(h.query_by_label("Stepover").is_some(), "Passes shows Stepover");

    h.state_mut().state.op_tab = tab_pos("Feeds");
    h.run_steps(2);
    for label in ["Cutting feed", "Plunge feed", "Climb milling"] {
        assert!(h.query_by_label(label).is_some(), "Feeds shows {label}");
    }
}
