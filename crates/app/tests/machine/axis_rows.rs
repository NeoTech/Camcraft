//! The machine editor's axis rows (#2756): what each row says and edits
//! (the letter RS-274 gives a rotary's axis, a linear's direction
//! (#373), a rotary's axis, its work lengths (#804, #1107)) and the
//! notes a row shows.
//!
//! Provenance: our own tests, specced from the issue's "Done when".

use super::axis_support::*;

#[test]
fn the_rows_follow_the_chain_and_say_what_carries_the_head() {
    let mut d = grblhal();
    d.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("A added");
    d.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("B added");
    let rows: Vec<(String, bool, bool)> =
        d.axis_rows().into_iter().map(|r| (r.name, r.rotary, r.carries_head)).collect();
    let row = |n: &str, rotary, head| (n.to_owned(), rotary, head);
    assert_eq!(rows, [row("Z", false, false), row("Y", false, false), row("X", false, false), row("A", true, false), row("B", true, true)]);
}

#[test]
fn a_linear_is_reversed_and_set_back_and_its_other_keys_stay() {
    use camcraft_app::machine::AxisDirection;
    let mut m = grblhal();
    let x = pointer_of(&m, "X");
    m.set_axis_reversed(&x, true).expect("X reversed");
    assert_eq!(m.value().pointer(&format!("{x}/direction")), Some(&json!(-1)));
    assert_eq!(axis(&m, "X").direction, AxisDirection::Inverted);
    assert!(m.axis_rows().iter().any(|r| r.name == "X" && r.reversed));
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());

    // Back to standard: unstated is the file's way of saying standard.
    m.set_axis_reversed(&x, false).expect("X standard");
    assert_eq!(m.value().pointer(&format!("{x}/direction")), None);
    assert_eq!(axis(&m, "X").direction, AxisDirection::Standard);
    unknown_keys_kept(m.value());
}

#[test]
fn a_rotary_cannot_be_reversed_and_a_linear_has_no_axis() {
    let mut m = grblhal();
    let a = m.add_rotary_axis(Place::OnTable, "A", [1.0, 0.0, 0.0]).expect("A added");
    m.dirty = false;
    assert_eq!(m.set_axis_reversed(&a, true), Err("axis A: a rotary is not reversed; turn its axis instead".to_owned()));
    let x = pointer_of(&m, "X");
    assert_eq!(m.set_rotary_axis(&x, [0.0, 1.0, 0.0]), Err("axis X: a linear axis has no rotary axis".to_owned()));
    assert!(!m.dirty, "a refused edit changes nothing");
}

#[test]
fn a_rotary_turns_about_a_new_axis_and_keeps_its_trunnion_keys() {
    let mut m = tilting_head();
    let before = trunnion_keys(&m, "B");
    let b = pointer_of(&m, "B");
    m.set_rotary_axis(&b, [0.0, 0.0, -1.0]).expect("B about -Z");
    assert_eq!(axis(&m, "B").axis, Some([0.0, 0.0, -1.0]));
    assert_eq!(trunnion_keys(&m, "B"), before);
    assert_eq!(
        m.set_rotary_axis(&b, [0.0, 0.0, 0.0]),
        Err("axis B: the axis direction is zero".to_owned())
    );
    assert_eq!(
        m.set_rotary_axis(&b, [f64::NAN, 0.0, 1.0]),
        Err("axis B: the axis direction must be three finite numbers".to_owned())
    );
    assert_eq!(axis(&m, "B").axis, Some([0.0, 0.0, -1.0]), "a refused axis changes nothing");
}

#[test]
fn the_letter_follows_the_axis_by_rs274() {
    use camcraft_app::machine::chain::suggested_letter;
    assert_eq!(suggested_letter([1.0, 0.0, 0.0]), Some("A"));
    assert_eq!(suggested_letter([-1.0, 0.0, 0.0]), Some("A"));
    assert_eq!(suggested_letter([0.0, -2.0, 0.0]), Some("B"));
    assert_eq!(suggested_letter([0.0, 0.0, 1.0]), Some("C"));
    assert_eq!(suggested_letter([1.0, 1.0, 0.0]), None, "no letter for a tilted axis");
    assert_eq!(suggested_letter([0.0, 0.0, 0.0]), None);
}

#[test]
fn a_row_whose_letter_and_axis_disagree_says_so() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::OnTable, "A", [0.0, 1.0, 0.0]).expect("an A about Y is allowed");
    let a = m.axis_rows().into_iter().find(|r| r.name == "A").expect("A row");
    assert_eq!(a.axis, Some([0.0, 1.0, 0.0]));
    assert_eq!(a.letter_note().as_deref(), Some("A turns about Y: by RS-274 an axis about Y is B"));
    let a_ptr = a.pointer.clone();
    m.set_rotary_axis(&a_ptr, [-1.0, 0.0, 0.0]).expect("A about -X");
    let a = m.axis_rows().into_iter().find(|r| r.name == "A").expect("A row");
    assert_eq!(a.letter_note(), None);
    let x = m.axis_rows().into_iter().find(|r| r.name == "X").expect("X row");
    assert_eq!((x.axis, x.letter_note()), (None, None));
}
/// #804, #1107: a rotary that turns the work states its two work lengths
/// in "Rotaries and limits"; one that carries the head has neither row.
#[test]
fn a_table_rotary_states_its_work_lengths_and_a_head_rotary_has_none() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds A");
    m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    let labels: Vec<String> = m.groups().into_iter().flat_map(|g| g.fields).map(|f| f.label).collect();
    assert!(!labels.iter().any(|x| x.starts_with("B nose") || x.starts_with("B axis to")), "{labels:?}");
    for (label, text) in [("A nose to axis", "98.08"), ("A axis to table", "40")] {
        let f = field(&m, label);
        m.set(&f.pointer, f.kind, text).expect("the edit applies");
    }
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    let a = axis(&m, "A");
    assert_eq!((a.nose_to_axis, a.axis_to_table), (Some(98.08), Some(40.0)));
    unknown_keys_kept(m.value());
}

/// A work length is above zero and only on a rotary that turns the work;
/// each refusal sits at its row, and a stated key shows its row to clear.
#[test]
fn a_bad_work_length_is_refused_at_its_row() {
    let mut m = grblhal();
    let a = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds A");
    let b = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    m.value_mut().pointer_mut(&a).expect("A")["nose_to_axis"] = json!(-1.0);
    m.value_mut().pointer_mut(&b).expect("B")["axis_to_table"] = json!(5.0);
    let r = m.refusals();
    let on = |label: &str| -> Vec<String> { field(&m, label).refusals(&r).iter().map(|r| r.key.clone()).collect() };
    assert_eq!(on("A nose to axis"), ["A nose_to_axis"]);
    assert_eq!(on("B axis to table"), ["B axis_to_table"]);
}

/// Q1 on #2756: `offset` is measured from a parent rotary (#1962). A
/// rotary that states one with no rotary carrying it shows a note at its
/// row. The file still validates: a refusal is upstream's to decide.
#[test]
fn an_offset_with_no_parent_rotary_shows_a_note_at_its_row() {
    let mut m = tilting_head();
    let row = |m: &MachineDraft, n: &str| m.axis_rows().into_iter().find(|r| r.name == n).expect("row");
    assert_eq!(row(&m, "B").parent.as_deref(), Some("A"));
    assert_eq!(row(&m, "B").offset_note(), None, "B rides on A: its offset has a parent");
    m.move_axis(&pointer_of(&m, "B"), Step::Up).expect("B up");
    assert_eq!(row(&m, "B").parent, None);
    let note = "B states offset, but no rotary carries it: offset is measured from a parent rotary";
    assert_eq!(row(&m, "B").offset_note().as_deref(), Some(note));
    assert_eq!(row(&m, "A").parent.as_deref(), Some("B"));
    assert_eq!(row(&m, "A").offset_note(), None, "A states no offset");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
}
