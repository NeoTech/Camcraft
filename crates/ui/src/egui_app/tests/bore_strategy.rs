//! #2830: the Bore page's Strategy choice on the Passes tab - Rings (the
//! default) or Helical, with the Spring pass check only for the helix -
//! and the Heights tab names the stepdown "Helix pitch" for the helix.
//! Provenance: our own page law; asserted structurally (labels, the
//! combo's value; no pixels).

use super::*;
use camcraft_app::machining2d::BoreStrategy;
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{KindInfo, OpType};
use egui::accesskit::Role;

fn tab(name: &str) -> usize {
    super::super::op_tabs::SECTIONS.iter().position(|s| *s == name).unwrap_or(0)
}

/// The draft retyped to a Bore, on the named tab, drawn.
fn bore_on(name: &str) -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    let cur = harness.state().state.op_ui.params.clone();
    let kind = KindInfo::ALL.iter().position(|k| k.ty == OpType::Bore).unwrap_or(usize::MAX);
    harness.state_mut().state.op_ui.params = crate::operations::retype(&cur, kind);
    harness.state_mut().state.op_tab = tab(name);
    harness.run();
    harness
}

fn set(h: &mut Harness<'static, ShellUi>, s: BoreStrategy) {
    let OpParams::Bore { strategy, .. } = &mut h.state_mut().state.op_ui.params else {
        panic!("the draft is not a Bore");
    };
    *strategy = s;
    h.run();
}

fn set_entry(h: &mut Harness<'static, ShellUi>, e: Entry3D) {
    let OpParams::Bore { entry, .. } = &mut h.state_mut().state.op_ui.params else {
        panic!("the draft is not a Bore");
    };
    *entry = e;
    h.run();
}

fn combos_showing(h: &Harness<'static, ShellUi>, text: &str) -> usize {
    h.query_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some(text)).count()
}

#[test]
fn the_bore_passes_tab_offers_the_strategy_and_defaults_to_rings() {
    let mut h = bore_on("Passes");
    assert_eq!(h.query_all_by_label("Strategy").count(), 1, "the Strategy row");
    assert_eq!(combos_showing(&h, "Rings (level by level)"), 1, "a new Bore is the ring ladder");
    assert_eq!(h.query_all_by_label("Spring pass").count(), 0, "no spring pass for the rings");
    set(&mut h, BoreStrategy::Helical { spring_pass: false });
    assert_eq!(combos_showing(&h, "Helical"), 1, "the combo shows the draft's strategy");
    assert_eq!(h.query_all_by_role_and_label(Role::CheckBox, "Spring pass").count(), 1, "the helix offers the spring pass");
}

#[test]
fn the_heights_tab_names_the_stepdown_the_helix_pitch_for_the_helix() {
    let mut h = bore_on("Heights");
    assert_eq!(h.query_all_by_label("Stepdown").count(), 1);
    assert_eq!(h.query_all_by_label("Helix pitch").count(), 0);
    set(&mut h, BoreStrategy::Helical { spring_pass: false });
    assert_eq!(h.query_all_by_label("Stepdown").count(), 0);
    assert_eq!(h.query_all_by_label("Helix pitch").count(), 1, "the stepdown is the pitch of the helix");
}

#[test]
fn a_helical_bore_keeps_the_entry_group_and_says_the_entry_refusal() {
    // The helix hint is hover text on the Entry group (`layout::hint`);
    // the refusal line is always shown.
    use crate::egui_app::holes_page::HELICAL_ENTRY_REFUSED;
    let mut h = bore_on("Lead in / Lead out");
    set_entry(&mut h, Entry3D::Ramp);
    assert_eq!(h.query_all_by_label(HELICAL_ENTRY_REFUSED).count(), 0, "the rings ramp");
    set(&mut h, BoreStrategy::Helical { spring_pass: false });
    assert_eq!(h.query_all_by_label(HELICAL_ENTRY_REFUSED).count(), 1, "a Ramp entry on the helix is refused");
    assert_eq!(combos_showing(&h, "Ramp"), 1, "the Entry group stays, so the entry can be set back");
    set_entry(&mut h, Entry3D::Plunge);
    assert_eq!(h.query_all_by_label(HELICAL_ENTRY_REFUSED).count(), 0, "Plunge is the helix's entry word");
}
