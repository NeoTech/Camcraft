//! The machine editor's axis chain (#2756): an axis part is added to or
//! removed from `kinematics.default.parts` through `MachineDraft`, every
//! key the editor does not know is kept, and the parsed machine follows
//! the edit (rotary `parent`, `chain_order` and `turns_part`).
//!
//! Fixture law (clean-room): the shipped grblHAL mill plus our own
//! values in the `.mch` layout of `assets/machines/AGENTS.md`.
//! Provenance: our own tests, specced from the issue's "Done when".

use super::axis_support::*;

#[test]
fn an_a_about_minus_x_on_the_table_is_added_and_turns_the_work() {
    let mut m = grblhal();
    let at = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds");
    assert!(m.dirty);
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept(m.value());

    // On the table part, beside Z and before the head.
    assert_eq!(at, "/kinematics/default/parts/0/parts/1");
    let part = m.value().pointer(&at).expect("the pointer names the new part");
    assert_eq!(part.get("type"), Some(&json!("rotary")));
    assert_eq!(names(&m), ["X", "Y", "Z", "A"]);
    let a = axis(&m, "A");
    assert!(a.rotary);
    assert_eq!(a.axis, Some([-1.0, 0.0, 0.0]));
    assert_eq!(a.turns_part, Some(true), "a rotary beside the head turns the work");
    assert_eq!(a.chain_order, Some(0));
    assert_eq!(a.parent, None);
}

#[test]
fn a_b_that_carries_the_head_tilts_the_tool() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept(m.value());

    let b = axis(&m, "B");
    assert_eq!(b.turns_part, Some(false), "the head now rides on B");
    // The head and its coolants moved inside B, unchanged.
    let head = m.value().pointer("/kinematics/default/parts/0/parts/1/parts/0").expect("head under B");
    assert_eq!(head.get("type"), Some(&json!("head")));
    assert_eq!(head.pointer("/tool_station/coolants/1/mCode"), Some(&json!("M7")));
}

#[test]
fn a_rotary_added_after_one_on_the_table_is_its_child_when_it_carries_the_head() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::CarriesHead, "A", [1.0, 0.0, 0.0]).expect("adds A");
    m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    // The head rides on B, which rides on A: A comes first in the chain.
    let (a, b) = (axis(&m, "A"), axis(&m, "B"));
    assert_eq!((a.chain_order, a.parent), (Some(0), None));
    assert_eq!((b.chain_order, b.parent.as_deref()), (Some(1), Some("A")));
}

#[test]
fn removing_the_added_a_gives_back_the_file_as_it_was() {
    let mut m = grblhal();
    let before = m.value().clone();
    let at = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds");
    m.remove_axis(&at).expect("removes");
    assert_eq!(*m.value(), before);
}

#[test]
fn removing_a_rotary_that_carries_the_head_puts_the_head_back() {
    let mut m = grblhal();
    let before = m.value().clone();
    let at = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds");
    m.remove_axis(&at).expect("removes");
    assert_eq!(*m.value(), before);
}

#[test]
fn removing_z_keeps_the_axes_it_carried_in_its_place() {
    let mut m = grblhal();
    m.remove_axis("/kinematics/default/parts/0/parts/0").expect("removes Z");
    assert_eq!(names(&m), ["X", "Y"]);
    unknown_keys_kept_after_z_went(m.value());
    // Z's controller row goes with it; X and Y keep theirs.
    let ctrl = m.value().pointer("/controller/synced_configuration/parts").expect("controller");
    assert!(ctrl.get("Z").is_none());
    assert!(ctrl.get("X").is_some() && ctrl.get("Y").is_some());
}

fn unknown_keys_kept_after_z_went(v: &Value) {
    assert_eq!(v.pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = v.pointer("/kinematics/default/parts/0/parts/0/parts/0").expect("X under Y");
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

#[test]
fn an_add_that_the_file_cannot_hold_is_refused_by_name_and_changes_nothing() {
    let mut m = grblhal();
    let before = m.value().clone();
    assert_eq!(m.add_linear_axis(Place::OnTable, "X"), Err("axis X: the machine already has an axis X".to_owned()));
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "Q", [1.0, 0.0, 0.0]),
        Err("axis Q: a rotary is named A, B or C".to_owned())
    );
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "A", [0.0, 0.0, 0.0]),
        Err("axis A: the axis direction is zero".to_owned())
    );
    assert_eq!(
        m.add_rotary_axis(Place::OnTable, "A", [f64::NAN, 0.0, 0.0]),
        Err("axis A: the axis direction must be three finite numbers".to_owned())
    );
    assert_eq!(*m.value(), before);
    assert!(!m.dirty);
}

#[test]
fn a_remove_that_names_no_axis_is_refused_by_name() {
    let mut m = grblhal();
    let before = m.value().clone();
    for bad in ["/kinematics/default/parts/0", "/kinematics/default/parts/0/parts/9", "/general", "nonsense"] {
        assert_eq!(m.remove_axis(bad), Err(format!("{bad}: not an axis part")), "{bad}");
    }
    assert_eq!(*m.value(), before);
}

#[test]
fn moving_a_rotary_up_changes_its_parent_and_chain_order_and_keeps_its_keys() {
    let mut m = tilting_head();
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    let keys = trunnion_keys(&m, "B");
    assert_eq!((axis(&m, "B").chain_order, axis(&m, "B").parent.as_deref()), (Some(1), Some("A")));

    let at = m.move_axis(&pointer_of(&m, "B"), Step::Up).expect("moves");
    assert_eq!(at, pointer_of(&m, "B"), "the move names where B is now");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    assert_eq!(trunnion_keys(&m, "B"), keys, "B's own keys move with it");
    let (a, b) = (axis(&m, "A"), axis(&m, "B"));
    assert_eq!((b.chain_order, b.parent), (Some(0), None));
    assert_eq!((a.chain_order, a.parent.as_deref()), (Some(1), Some("B")));
    assert_eq!(b.pivot, Some([0.0, 0.0, 150.0]));
    // The head still rides on the last rotary of the chain.
    assert_eq!(a.turns_part, Some(false));
    assert_eq!(b.turns_part, Some(false));
    unknown_keys_kept(m.value());
}

#[test]
fn a_move_down_undoes_a_move_up() {
    let mut m = tilting_head();
    let before = m.value().clone();
    let at = m.move_axis(&pointer_of(&m, "B"), Step::Up).expect("up");
    m.move_axis(&at, Step::Down).expect("down");
    assert_eq!(*m.value(), before);
}

#[test]
fn an_a_on_the_table_is_added_moved_and_removed_and_the_file_validates_each_time() {
    let mut m = grblhal();
    m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds A");
    let b = m.add_rotary_axis(Place::OnTable, "B", [0.0, 1.0, 0.0]).expect("adds B");
    // Depth-first the rows read Z, Y, X, A, B: up trades B with A.
    let b = m.move_axis(&b, Step::Up).expect("moves");
    assert_eq!(b, "/kinematics/default/parts/0/parts/1");
    assert_eq!(names(&m), ["X", "Y", "Z", "B", "A"]);
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept_on_x_wherever_it_is(&m);
    assert_eq!(axis(&m, "B").turns_part, Some(true), "no head below B: it still turns the work");
    m.remove_axis(&pointer_of(&m, "A")).expect("removes A");
    m.remove_axis(&pointer_of(&m, "B")).expect("removes B");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    unknown_keys_kept_on_x_wherever_it_is(&m);
    assert_eq!(names(&m), ["X", "Y", "Z"]);
}

/// Q2 on #2756: a move trades only with a neighbour of the same kind.
/// A rotary in a linear's slot describes another machine (A riding on
/// Y); to put a rotary elsewhere the user removes it and adds it again.
#[test]
fn a_move_trades_only_with_an_axis_of_the_same_kind() {
    let mut m = grblhal();
    let a = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds A");
    m.dirty = false;
    let before = m.value().clone();
    let rule = "a rotary trades places only with a rotary; to put it elsewhere, remove it and add it again";
    assert_eq!(m.move_axis(&a, Step::Up), Err(format!("axis A: {rule}")));
    let rule = "a linear trades places only with a linear; to put it elsewhere, remove it and add it again";
    assert_eq!(m.move_axis(&pointer_of(&m, "X"), Step::Down), Err(format!("axis X: {rule}")));
    assert_eq!(*m.value(), before);
    assert!(!m.dirty, "a refused move changes nothing");
    // Linear with linear still trades: Y moves up past Z.
    m.move_axis(&pointer_of(&m, "Y"), Step::Up).expect("Y up");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
}

/// An A on the table and a B that carries the head are both rotaries,
/// but a trade would put the head on A: A would tilt the tool.
#[test]
fn a_rotary_that_turns_the_work_never_trades_with_one_that_tilts_the_tool() {
    let mut m = grblhal();
    let a = m.add_rotary_axis(Place::OnTable, "A", [-1.0, 0.0, 0.0]).expect("adds A");
    let b = m.add_rotary_axis(Place::CarriesHead, "B", [0.0, 1.0, 0.0]).expect("adds B");
    m.dirty = false;
    let before = m.value().clone();
    let tail = "trades places only with one that moves the";
    let again = "to put it elsewhere, remove it and add it again";
    assert_eq!(m.move_axis(&a, Step::Down), Err(format!("axis A: an axis that moves the work {tail} work; {again}")));
    assert_eq!(m.move_axis(&b, Step::Up), Err(format!("axis B: an axis that moves the tool {tail} tool; {again}")));
    assert_eq!(*m.value(), before);
    assert!(!m.dirty, "a refused move changes nothing");
}

fn unknown_keys_kept_on_x_wherever_it_is(m: &MachineDraft) {
    assert_eq!(m.value().pointer("/x_vendor_block/probe"), Some(&json!("OMP40")));
    let x = m.value().pointer(&pointer_of(m, "X")).expect("X");
    assert_eq!(x.get("note"), Some(&json!("gib adjusted 2026")));
}

#[test]
fn a_move_past_either_end_or_from_no_axis_is_refused_by_name() {
    let mut m = grblhal();
    let before = m.value().clone();
    let z = "/kinematics/default/parts/0/parts/0";
    let x = "/kinematics/default/parts/0/parts/0/parts/0/parts/0";
    assert_eq!(m.move_axis(z, Step::Up), Err("axis Z: already first in the chain".to_owned()));
    assert_eq!(m.move_axis(x, Step::Down), Err("axis X: already last in the chain".to_owned()));
    assert_eq!(m.move_axis("/general", Step::Up), Err("/general: not an axis part".to_owned()));
    assert_eq!(*m.value(), before);
    assert!(!m.dirty);
}

#[test]
fn a_linear_u_is_added_on_the_table() {
    let mut m = grblhal();
    m.add_linear_axis(Place::OnTable, "U").expect("adds");
    assert_eq!(validate_text(&m.text()), [], "{}", m.text());
    let u = axis(&m, "U");
    assert!(!u.rotary);
    assert_eq!(
        m.add_linear_axis(Place::OnTable, "A"),
        Err("axis A: a linear axis is named X, Y, Z, U, V or W".to_owned())
    );
}
