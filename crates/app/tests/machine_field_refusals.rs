//! Each machine-editor refusal sits only on the fields it is about
//! (issue #3246): the spindle min/max speed refusal shows under the two
//! speed fields of "Spindle range", not under Spindle power.
//!
//! Fixture law (clean-room): the blank mill is the editor's own draft.
//! Provenance: our own tests, specced from the issue's "Done means".
//!
//! Gate map (#3246): covers app::machine, post::machine::fields, app::machine, post::machine::draft.

use camcraft_app::machine::draft::MachineDraft;
use camcraft_app::machine::fields::{self, FieldGroup};
use camcraft_app::machine::Kind;

fn field<'a>(groups: &'a [FieldGroup], label: &str) -> &'a fields::Field {
    groups
        .iter()
        .flat_map(|g| &g.fields)
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no field {label}"))
}

fn edit(d: &mut MachineDraft, label: &str, text: &str) {
    let f = field(&d.groups(), label).clone();
    d.set(&f.pointer, f.kind, text).expect("the edit applies");
}

#[test]
fn the_spindle_speed_refusal_sits_on_the_speed_fields_and_not_on_spindle_power() {
    // The reporter's steps: a blank mill, min speed 1000, max speed 24.
    let mut m = MachineDraft::blank(Kind::Mill);
    edit(&mut m, "Spindle min speed", "1000");
    edit(&mut m, "Spindle max speed", "24");
    let refusals = m.refusals();
    let g = m.groups();
    let on = |label: &str| field(&g, label).refusals(&refusals).iter().map(|r| r.to_string()).collect::<Vec<_>>();
    let want = ["spindle: min_speed 1000 must not exceed max_speed 24"];
    assert_eq!(on("Spindle min speed"), want);
    assert_eq!(on("Spindle max speed"), want);
    assert!(on("Spindle power").is_empty(), "Spindle power carries no speed refusal: {:?}", on("Spindle power"));
    assert!(fields::unplaced(&g, &refusals).is_empty(), "the refusal still has its fields");
    // A refusal of the power value itself still sits on Spindle power.
    edit(&mut m, "Spindle power", "-3");
    let refusals = m.refusals();
    let g = m.groups();
    let power: Vec<String> = field(&g, "Spindle power").refusals(&refusals).iter().map(|r| r.to_string()).collect();
    assert!(power.iter().all(|r| r.starts_with("spindle power:")) && power.len() == 1, "{power:?}");
}
