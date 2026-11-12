//! #821 in the shell: a setup whose machine turns the work on a rotary
//! shows its Machine page's "Rotary centerline" rows - the machine file's
//! nose_to_axis and how the stack raises it - and its own measured figure
//! is one undoable command (`SetupNoseToAxis`), stated from the checkbox.
//! A machine with no work-turning rotary draws none of it.
//! Provenance: our own tests over our own shell; no vendor material.

use super::*;
use crate::state::SetupPage;
use camcraft_app::rotary_stack;

const ARC4: &str = "Tormach 770MX + microARC 4";
const NOSE: f64 = 10.6 * 25.4;

fn run(state: &mut State, k: CommandKind) {
    state.pending.push_back(k);
    state.drain_actions();
}

#[test]
fn the_stated_nose_is_one_undoable_command() {
    let mut state = State::new();
    run(&mut state, CommandKind::CreateSetup { name: None, datum_from: None });
    run(&mut state, CommandKind::SetupNoseToAxis { setup: "Setup1".into(), value: Some(250.0) });
    assert_eq!(rotary_stack::stated(&state.world, Some("Setup1")), Some(250.0));
    run(&mut state, CommandKind::SetupNoseToAxis { setup: "Setup1".into(), value: Some(-1.0) });
    assert_eq!(rotary_stack::stated(&state.world, Some("Setup1")), Some(250.0), "a nonsense figure is refused");
    run(&mut state, CommandKind::Undo);
    assert_eq!(rotary_stack::stated(&state.world, Some("Setup1")), None);
}

#[test]
fn the_machine_page_shows_the_rotary_rows_and_states_the_figure() {
    let mut harness = shell_harness();
    harness.state_mut().state.pending.push_back(CommandKind::LoadBuiltinMachine(ARC4.into()));
    harness.state_mut().state.pending.push_back(CommandKind::CreateSetup { name: None, datum_from: None });
    harness.run_steps(2);
    crate::actions::target(&mut harness.state_mut().state, UiTarget::SetupNode("Setup1".into(), SetupPage::Machine));
    harness.run_steps(2);
    assert!(harness.query_by_label(crate::egui_app::setup_rotary_page::HEADING).is_some(), "the section is drawn");
    assert!(harness.query_by_label("No rotary subplate in this setup: judged on the machine's figure.").is_some());
    harness.get_by_label("This setup states its own nose_to_axis (measured on its stack)").click();
    harness.run_steps(2);
    let stated = rotary_stack::stated(&harness.state().state.world, Some("Setup1"));
    assert!(stated.is_some_and(|v| (v - NOSE).abs() < 1e-9), "stated from the machine's figure: {stated:?}");
}

#[test]
fn a_machine_without_a_work_turning_rotary_draws_no_rotary_rows() {
    let mut harness = shell_harness();
    harness.state_mut().state.pending.push_back(CommandKind::LoadBuiltinMachine("VMC 3-axis".into()));
    harness.state_mut().state.pending.push_back(CommandKind::CreateSetup { name: None, datum_from: None });
    harness.run_steps(2);
    crate::actions::target(&mut harness.state_mut().state, UiTarget::SetupNode("Setup1".into(), SetupPage::Machine));
    harness.run_steps(2);
    assert!(harness.query_by_label(crate::egui_app::setup_rotary_page::HEADING).is_none());
}
