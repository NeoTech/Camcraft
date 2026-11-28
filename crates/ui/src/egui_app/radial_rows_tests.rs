//! The rotary kinds' radial rows on the Heights tab (#2774), driven
//! through the real page code: each stated slot draws its check row, an
//! enabled slot adds the From combo and echoes the radius the app's one
//! resolver gives about the fitted axis, and an unstated slot stays a
//! plain unchecked row. Provenance: our own page law; no vendor material
//! consulted.

use egui::accesskit::Role;
use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use camcraft_app::commands::{AddTool, CreateOperation, ImportModel};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::operation::{
    Height, OpParams, RadialHeight, RadialSource, SrcRef,
};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;

use crate::state::{CommandKind, State};

/// A standing turned bar: radius `r`, z from 0 to 16, swept at `n` turns.
fn turned_bar(n: usize, r: f64) -> Vec<Triangle> {
    let v = |j: usize, rho: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / n as f64;
        Point3::new(rho * th.cos(), rho * th.sin(), z)
    };
    let mut t = Vec::new();
    for (seg, next) in [((0.0, 0.0), (r, 0.0)), ((r, 0.0), (r, 16.0)), ((r, 16.0), (0.0, 16.0))] {
        for j in 0..n {
            let k = (j + 1) % n;
            if seg.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            } else if next.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
            } else {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            }
        }
    }
    t
}

/// Stock cylinder d 6.4 z 0..16, a turned bar r 3 (the same bar), a tool,
/// and one RotaryParallel sourced from the bar - opened in the pane the
/// way a tree click opens it. Clearance reads the stock OD, Bottom an
/// absolute radius; Retract and Top stay the Z planes.
fn opened() -> State {
    let mut state = State::new();
    state.pending.push_back(CommandKind::SetStock {
        shape: StockShape::Cylinder { diameter: 6.4, height: 16.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    state.drain_actions();
    let bn = catalog()
        .into_iter()
        .find(|(n, _)| *n == "BN-0600")
        .map(|(_, t)| t)
        .expect("catalog entry");
    state.history.execute(&state.world, AddTool::new(&state.world, "BN-0600", bn).expect("tool"));
    state.history.execute(&state.world, ImportModel::from_mesh("bar", Mesh::from_facets(turned_bar(16, 3.0))));
    let params = OpParams::RotaryParallel {
        tolerance: 0.02,
        stepover: Some(0.5),
        clearance_z: Height::fixed(4.0),
        retract_z: None,
        top_z: Height::fixed(3.0),
        bottom_z: Height::fixed(-3.0),
        clearance_r: Some(RadialHeight::of(RadialSource::StockOd, 0.0)),
        retract_r: None,
        top_r: None,
        bottom_r: Some(RadialHeight::of(RadialSource::Absolute, 4.0)),
        feed: 200.0,
        plunge_feed: 50.0,
        climb: true,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        stated_axis: None,
        picks: Default::default(),
    };
    state.history.execute(
        &state.world,
        CreateOperation::new(&state.world, "W1", "BN-0600", SrcRef::Entity("bar".into()), params)
            .expect("rotary operation"),
    );
    state.pending.push_back(CommandKind::Select(Some("W1".into())));
    state.drain_actions();
    state.sync_ops_ui();
    state
}

fn heights_tab(state: State) -> Harness<'static, State> {
    Harness::builder()
        .with_size(Vec2::new(600.0, 900.0))
        .build_ui_state(
            |ui, st: &mut State| super::params_page::params_fields(ui, st, super::op_tabs::SEC_HEIGHTS),
            state,
        )
}

fn combo_showing(h: &Harness<'static, State>, text: &str) -> usize {
    h.query_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some(text)).count()
}

fn row_caption_drawn(h: &Harness<'static, State>, text: &str) -> bool {
    h.query_all_by(|n| n.role() == Role::CheckBox && n.label().as_deref() == Some(text)).count() > 0
}

fn label_drawn(h: &Harness<'static, State>, text: &str) -> bool {
    h.query_all_by(|n| n.role() == Role::Label && n.value().as_deref() == Some(text)).count() > 0
}

fn checkbox_on(h: &Harness<'static, State>, text: &str) -> bool {
    h.query_all_by(|n| {
        n.role() == Role::CheckBox && n.label().as_deref() == Some(text)
            && n.toggled() == Some(egui::accesskit::Toggled::True)
    })
    .count()
        > 0
}

#[test]
fn the_rotary_heights_tab_draws_one_check_row_for_every_radial_slot() {
    let h = heights_tab(opened());
    for caption in ["Clearance radius", "Retract radius", "Top radius", "Bottom radius"] {
        assert!(row_caption_drawn(&h, caption), "the row {caption} is missing: {caption}");
    }
    assert!(checkbox_on(&h, "Clearance radius") && checkbox_on(&h, "Bottom radius"), "the stated slots are checked");
    for caption in ["Retract radius", "Top radius"] {
        assert!(!checkbox_on(&h, caption), "the unstated slot {caption} stays unchecked");
    }
}

#[test]
fn an_enabled_slot_adds_its_from_combo_and_echoes_the_resolved_radius() {
    let mut h = heights_tab(opened());
    h.run();
    // Clearance: stock OD + 0 -> the stock's radius 3.2; Bottom: the stated
    // radius itself. The two unstated slots draw no combo - they stay the
    // kind's Z planes, and the viewport law says their Z rows own them.
    // Each enabled slot keeps its own combo: the row scoping keeps the
    // four slots' field ids apart (one node with the last row's selection
    // is the signature of a collision).
    assert_eq!(combo_showing(&h, "stock OD"), 1, "the enabled Clearance slot shows its source");
    assert_eq!(combo_showing(&h, "absolute radius"), 1, "the enabled Bottom slot shows its source");
    assert_eq!(combo_showing(&h, "model max radius"), 0, "an unstated slot draws no combo");
    assert!(label_drawn(&h, "(= R 3.200)"), "the clearance echo reads the stock OD radius");
    assert!(label_drawn(&h, "(= R 4.000)"), "the absolute radius echoes the stated number");
}

/// The interactive control that shares a row with `label`'s text node:
/// a checkbox draws its caption as its own node, so the row is found by
/// the caption's band, the way `dialog_tests` reads its rows.
fn on_row<'a>(h: &'a Harness<'static, State>, label: &str, role: Role) -> egui_kittest::Node<'a> {
    let lab = h.get_by_label(label).rect();
    h.query_all_by(move |n| n.role() == role)
        .find(|n| {
            let c = n.rect().center().y;
            c >= lab.top() - 2.0 && c <= lab.bottom() + 2.0
        })
        .expect("a control on the label's row")
}

#[test]
fn the_default_bottom_radius_echoes_its_valid_zero() {
    // #2774 review: the checkbox default for the Bottom radius is R0 - the
    // band floor at the axis, a valid radius - and the echo must display
    // the resolved zero, never report it unresolved as not positive.
    // Driven through the full properties pane (the shell's own page
    // wrapper), the way a user's click lands.
    let mut state = opened();
    state.op_tab = super::op_tabs::SEC_HEIGHTS;
    if let OpParams::RotaryParallel { bottom_r, .. } = &mut state.op_ui.params {
        *bottom_r = None;
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(620.0, 1400.0))
        .build_ui_state(
            |ui, st: &mut State| super::pages::properties(ui, st, &super::theme::current()),
            state,
        );
    h.run();
    on_row(&h, "Bottom radius", Role::CheckBox).click();
    h.run();
    assert!(checkbox_on(&h, "Bottom radius"), "the click states the slot");
    assert!(label_drawn(&h, "(= R 0.000)"), "the default Bottom radius echoes the resolved zero");
    assert!(
        !label_drawn(&h, "(unresolved: the radius is not positive)"),
        "a valid zero is never reported unresolved"
    );
}
