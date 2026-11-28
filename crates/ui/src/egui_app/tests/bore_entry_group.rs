//! #3828: the Bore page's Entry group (Leads tab). Structural pins: the
//! type offers plunge, ramp and helix and no arc; each type shows only its
//! own fields; Apply writes one command that undo and redo invert.
//! Labels, values and commands only - no position, size or colour.
use super::*;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use camcraft_app::machining3d::Entry3D;

const OP: &str = "Bore";

fn page() -> Harness<'static, ShellUi> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1600.0, 1100.0))
        .build_ui_state(frame, ShellUi::seeded());
    let st = &mut h.state_mut().state;
    rack(st, &["EM-0600-4FL"]);
    let index = camcraft_app::operation::ALL_LABELS.iter().position(|s| *s == OP).unwrap();
    st.pending.push_back(CommandKind::CreateOperation {
        name: OP.into(),
        tool: "EM-0600-4FL".into(),
        src: SrcRef::Stock,
        params: crate::operations::quick_params(index),
    });
    st.drain_actions();
    crate::actions::target(st, UiTarget::Operation(OP.into()));
    st.op_tab = super::super::op_tabs::SEC_LEADS;
    h.run_steps(3);
    h
}

fn staged(h: &Harness<'static, ShellUi>) -> Entry3D {
    match &h.state().state.op_ui.params {
        OpParams::Bore { entry, .. } => *entry,
        _ => panic!("a bore page"),
    }
}

fn stored(h: &Harness<'static, ShellUi>) -> Entry3D {
    let st = &h.state().state;
    let e = model::find_by_name(&st.world, OP).expect("the operation");
    match model::operation_of(&st.world, &e).expect("operation").params {
        OpParams::Bore { entry, .. } => entry,
        _ => panic!("a bore operation"),
    }
}

fn stage(h: &mut Harness<'static, ShellUi>, e: Entry3D) {
    if let OpParams::Bore { entry, .. } = &mut h.state_mut().state.op_ui.params {
        *entry = e;
    }
    h.run_steps(3);
}

fn shown(h: &Harness<'static, ShellUi>, label: &str) -> bool {
    h.query_all_by_label(label).count() > 0
}

#[test]
fn a_bore_defaults_to_plunge_and_shows_no_arm_fields() {
    let _g = snapshot_guard();
    let h = page();
    assert_eq!(staged(&h), Entry3D::Plunge);
    for label in ["Ramp angle", "Max ramp leg", "Minimum radius", "Helix pitch", "Helix angle", "Arc radius"] {
        assert!(!shown(&h, label), "{label} stays hidden under Plunge; text {:?}", structural::painted(&h));
    }
}

#[test]
fn each_bore_entry_type_shows_only_its_own_fields() {
    let _g = snapshot_guard();
    let mut h = page();
    stage(&mut h, Entry3D::Ramp);
    assert!(shown(&h, "Ramp angle") && shown(&h, "Max ramp leg"), "text {:?}", structural::painted(&h));
    assert!(!shown(&h, "Minimum radius") && !shown(&h, "Helix angle"), "no helix field under Ramp");
    stage(&mut h, Entry3D::Helix);
    assert!(shown(&h, "Minimum radius") && shown(&h, "Helix angle") && shown(&h, "Helix pitch"), "text {:?}", structural::painted(&h));
    assert!(!shown(&h, "Ramp angle") && !shown(&h, "Max ramp leg"), "no ramp field under Helix");
    stage(&mut h, Entry3D::Plunge);
    assert!(!shown(&h, "Ramp angle") && !shown(&h, "Helix angle"), "the fields leave with the type");
}

#[test]
fn the_bore_entry_type_offers_plunge_ramp_and_helix_but_no_arc() {
    let _g = snapshot_guard();
    let mut h = page();
    let combo = h
        .query_all_by_role(Role::ComboBox)
        .find(|n| n.accesskit_node().value().as_deref() == Some("Plunge"))
        .expect("the entry type control shows Plunge");
    combo.click();
    h.run_steps(3);
    for word in ["Ramp", "Helix"] {
        assert!(h.query_all_by_label(word).count() > 0, "{word} is offered; text {:?}", structural::painted(&h));
    }
    assert_eq!(h.query_all_by_label("Arc").count(), 0, "no arc on a ring ladder");
}

#[test]
fn applying_a_bore_entry_is_one_undoable_command() {
    let _g = snapshot_guard();
    let mut h = page();
    stage(&mut h, Entry3D::Helix);
    assert_eq!(stored(&h), Entry3D::Plunge, "a staged edit does not write the document");
    let st = &mut h.state_mut().state;
    st.apply_operation_params(OP);
    st.drain_actions();
    assert_eq!(stored(&h), Entry3D::Helix, "Apply writes the entry");
    let st = &mut h.state_mut().state;
    assert!(st.history.undo(&st.world), "Apply is undoable");
    assert_eq!(stored(&h), Entry3D::Plunge, "one undo restores the default");
    let st = &mut h.state_mut().state;
    assert!(st.history.redo(&st.world), "and redoable");
    assert_eq!(stored(&h), Entry3D::Helix, "redo restores the helix");
}
