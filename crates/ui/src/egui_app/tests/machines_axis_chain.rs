//! The file editor's "Axis chain" group through the real shell (issue
//! #2756): a blank mill gets an A on the table, a B trades places with
//! it (never with a linear), a refused letter changes nothing, and the saved file
//! reads back with the A. None of it touches the command queue.
//! Provenance: our own tests, specced from issue #2756.
//! Structural only: no golden PNG.

use super::machines_edit::{draft_at, pick_kind, press, prompt, type_into, user_dir};
use super::machines_pane::{assert_no_commands, harness_with_user_dir, open_machines};
use super::*;
use crate::egui_app::machine_file_editor::Prompt;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

const TABLE: &str = "/kinematics/default/parts/0/parts";

fn name_at(h: &Harness<'static, ShellUi>, slot: usize) -> Option<String> {
    draft_at(h, &format!("{TABLE}/{slot}/name")).and_then(|v| v.as_str().map(str::to_owned))
}

fn names(h: &Harness<'static, ShellUi>) -> Vec<String> {
    (0..5).filter_map(|i| name_at(h, i)).filter(|n| n != "head").collect()
}

fn status(h: &Harness<'static, ShellUi>) -> Option<String> {
    h.state().state.machines_pane.editor.as_ref().and_then(|e| e.status.as_ref().map(|s| s.1.clone()))
}

/// Pick `words` in the dropdown labelled `combo`.
fn about(h: &mut Harness<'static, ShellUi>, combo: &str, words: &str) {
    h.get_by_role_and_label(Role::ComboBox, combo).scroll_to_me();
    h.run_steps(4);
    h.get_by_role_and_label(Role::ComboBox, combo).click();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, words).click();
    h.run_steps(3);
}

/// Tick or untick the checkbox labelled `label`.
fn tick(h: &mut Harness<'static, ShellUi>, label: &str) {
    h.get_by_role_and_label(Role::CheckBox, label).click();
    h.run_steps(3);
}

fn blank_mill(tag: &str) -> (Harness<'static, ShellUi>, std::path::PathBuf) {
    let dir = user_dir(tag);
    let mut h = harness_with_user_dir(Some(dir.clone()));
    h.run();
    open_machines(&mut h);
    pick_kind(&mut h, "All", "mill");
    press(&mut h, "New blank");
    (h, dir)
}

#[test]
fn a_blank_mill_gets_an_a_on_the_table_moves_it_and_saves_it() {
    let (mut h, dir) = blank_mill("chain");
    h.get_by_label("Z - linear");
    type_into(&mut h, "New axis", "a");
    about(&mut h, "Rotary turns about", "-X");
    press(&mut h, "Add axis");
    assert_eq!(names(&h), ["X", "Y", "Z", "A"], "A goes on the table, before the head");
    let axis = draft_at(&h, &format!("{TABLE}/3/axis"));
    assert_eq!(axis, Some(serde_json::json!([-1.0, 0.0, 0.0])));
    h.get_by_label("A - rotary");
    h.get_by_label("Validation: passes");

    // A move trades only with an axis of its kind: Z is a linear.
    assert!(h.get_by_label("Move A up").accesskit_node().is_disabled());
    type_into(&mut h, "New axis", "B");
    about(&mut h, "Rotary turns about", "+Y");
    press(&mut h, "Add axis");
    press(&mut h, "Move B up");
    assert_eq!(names(&h), ["X", "Y", "Z", "B", "A"]);
    press(&mut h, "Move B down");
    assert_eq!(names(&h), ["X", "Y", "Z", "A", "B"], "up then down is no change");
    press(&mut h, "Remove B");
    // Nor with one that moves the tool: a B that carries the head.
    type_into(&mut h, "New axis", "B");
    about(&mut h, "Place", "Carries the head");
    press(&mut h, "Add axis");
    assert!(h.get_by_label("Move A down").accesskit_node().is_disabled());
    press(&mut h, "Remove B");

    // The travel field of the moved chain edits the right part.
    type_into(&mut h, "Z travel min", "-100");
    assert_eq!(draft_at(&h, &format!("{TABLE}/2/travel/min")).and_then(|v| v.as_f64()), Some(-100.0));
    type_into(&mut h, "Z travel max", "0");
    h.get_by_label("Validation: passes");

    press(&mut h, "Save machine");
    assert_eq!(prompt(&h), Some(Prompt::SaveAs));
    press(&mut h, "Write file");
    let text = std::fs::read_to_string(dir.join("new-mill.mch")).expect("the machine was written");
    let m = camcraft_app::machine::parse_mch(&text).expect("the saved machine reads back");
    assert!(m.axes.iter().any(|a| a.name == "A"), "{text}");
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_refused_letter_adds_nothing_and_a_removed_axis_leaves_the_rest() {
    let (mut h, dir) = blank_mill("chain_refused");
    type_into(&mut h, "New axis", "Q");
    press(&mut h, "Add axis");
    assert_eq!(status(&h).as_deref(), Some("axis Q: a linear axis is named X, Y, Z, U, V or W"));
    assert_eq!(names(&h), ["X", "Y", "Z"], "nothing added");

    type_into(&mut h, "New axis", "X");
    press(&mut h, "Add axis");
    assert_eq!(status(&h).as_deref(), Some("axis X: the machine already has an axis X"));

    press(&mut h, "Remove Y");
    assert_eq!(names(&h), ["X", "Z"]);
    assert!(h.query_by_label("Y - linear").is_none());
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn x_is_reversed_and_set_back_from_its_row() {
    let (mut h, dir) = blank_mill("chain_reversed");
    tick(&mut h, "X reversed");
    assert_eq!(draft_at(&h, &format!("{TABLE}/0/direction")), Some(serde_json::json!(-1)));
    h.get_by_label("Validation: passes");
    tick(&mut h, "X reversed");
    assert_eq!(draft_at(&h, &format!("{TABLE}/0/direction")), None, "standard is unstated");
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn turning_about_y_suggests_b_and_an_a_about_y_shows_a_note_at_its_row() {
    let (mut h, dir) = blank_mill("chain_letter");
    about(&mut h, "Rotary turns about", "+Y");
    let typed = h.state().state.machines_pane.editor.as_ref().map(|e| e.axis_form.name.clone());
    assert_eq!(typed.as_deref(), Some("B"), "+Y suggests B");

    type_into(&mut h, "New axis", "A");
    press(&mut h, "Add axis");
    assert_eq!(draft_at(&h, &format!("{TABLE}/3/axis")), Some(serde_json::json!([0.0, 1.0, 0.0])));
    h.get_by_label("Note: A turns about Y: by RS-274 an axis about Y is B");

    about(&mut h, "A turns about", "-X");
    assert_eq!(draft_at(&h, &format!("{TABLE}/3/axis")), Some(serde_json::json!([-1.0, 0.0, 0.0])));
    assert!(h.query_by_label("Note: A turns about Y: by RS-274 an axis about Y is B").is_none());
    h.get_by_label("Validation: passes");
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The FlexiHAL router from sc-cam's `examples/cam/flexihal.json`
/// (travel and A; its `pivot` waits for the Kinematics group): X -672..0,
/// Y -1023..0, Z -160..0 and a continuous A about -X on the table, its
/// centre 40 mm above the table and (from the pivot's Z -98.08, an
/// estimate) 98.08 mm below the nose at Z 0.
#[test]
fn the_flexihal_router_is_built_from_a_blank_mill_and_saved() {
    let (mut h, dir) = blank_mill("chain_flexihal");
    for (name, min) in [("X", "-672"), ("Y", "-1023"), ("Z", "-160")] {
        type_into(&mut h, &format!("{name} travel min"), min);
        type_into(&mut h, &format!("{name} travel max"), "0");
    }
    about(&mut h, "Rotary turns about", "-X");
    press(&mut h, "Add axis");
    h.get_by_label("A - rotary");
    type_into(&mut h, "A nose to axis", "98.08");
    type_into(&mut h, "A axis to table", "40");
    h.get_by_label("Validation: passes");

    press(&mut h, "Save machine");
    assert_eq!(prompt(&h), Some(Prompt::SaveAs));
    press(&mut h, "Write file");
    let text = std::fs::read_to_string(dir.join("new-mill.mch")).expect("the machine was written");
    let m = camcraft_app::machine::parse_mch(&text).expect("the saved machine reads back");
    let axis = |n: &str| m.axes.iter().find(|a| a.name == n).unwrap_or_else(|| panic!("no {n} in {text}"));
    assert_eq!(axis("X").travel, Some((-672.0, 0.0)));
    assert_eq!(axis("Y").travel, Some((-1023.0, 0.0)));
    assert_eq!(axis("Z").travel, Some((-160.0, 0.0)));
    let a = axis("A");
    assert!(a.rotary);
    assert_eq!(a.axis, Some([-1.0, 0.0, 0.0]));
    assert_eq!(a.travel, None, "continuous");
    assert_eq!(a.turns_part, Some(true), "on the table, A turns the work");
    assert_eq!((a.nose_to_axis, a.axis_to_table), (Some(98.08), Some(40.0)));
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Q1 on #2756: an A on the table that states `offset` has no rotary to
/// measure it from, and its row says so; the file still validates.
#[test]
fn an_offset_with_no_parent_rotary_shows_a_note_at_its_row() {
    let (mut h, dir) = blank_mill("chain_offset");
    type_into(&mut h, "New axis", "A");
    press(&mut h, "Add axis");
    let note = "Note: A states offset, but no rotary carries it: offset is measured from a parent rotary";
    assert!(h.query_by_label(note).is_none());
    {
        let editor = h.state_mut().state.machines_pane.editor.as_mut().expect("the editor is open");
        let a = editor.draft.value_mut().pointer_mut(&format!("{TABLE}/3")).expect("A on the table");
        a["offset"] = serde_json::json!([0.0, 0.0, -40.0]);
    }
    h.run();
    h.get_by_label(note);
    h.get_by_label("Validation: passes");
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A field keeps its focus while a refusal above or below it comes and
/// goes: typing 1, 3, 0 into Z travel max gives 130, though the refusal
/// under Z travel min goes away at 13. (A field drawn by its place in the
/// frame got a new id when a row above it went, and lost the 0.)
#[test]
fn a_field_keeps_its_focus_when_a_refusal_goes_away() {
    let (mut h, dir) = blank_mill("chain_focus");
    type_into(&mut h, "Z travel min", "10");
    type_into(&mut h, "Z travel max", "1");
    assert_eq!(h.query_all_by_label_contains("min 10 must be below max 1").count(), 2, "under min and max");
    for key in ["3", "0"] {
        h.input_mut().events.push(egui::Event::Text(key.to_owned()));
        h.run_steps(3);
    }
    assert_eq!(draft_at(&h, &format!("{TABLE}/2/travel/max")).and_then(|v| v.as_f64()), Some(130.0));
    h.get_by_label("Validation: passes");
    let _ = std::fs::remove_dir_all(&dir);
}
