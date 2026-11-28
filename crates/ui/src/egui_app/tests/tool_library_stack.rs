//! #3898 follow-up (U7 of #3663): the component stack group by clicks -
//! the "Add component" combo box builds a stack and "Apply stack" lands
//! it in one undo step; the group shows the resolver's gauge length,
//! cutting diameter and corner radius for the shown stack, or its
//! refusal; "Swap" offers only the parts with which the stack still
//! resolves and lands the new stack in one undo step; a still frame
//! builds no stack view; and "Show compatible parts" on a built-in tool
//! lists the built-in ER collets of its shank.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::schema::AssemblyLink;
use camcraft_app::tool_assembly as ta;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};

use super::tool_library_components::{A, INSERT, done, lib_dir, open_library, pick, redo, shell, shop, shows, undo};
use super::tool_library_editor::{H, click_window_button, library_window, open_in_window, tool};
use crate::egui_app::tool_library_dialog::{components, stack};

fn assembly(h: &H, name: &str) -> Option<Vec<AssemblyLink>> {
    let w = &h.state().state.world;
    camcraft_app::model::find_by_name(w, name).and_then(|e| ta::assembly_of(w, &e)).map(|a| a.0)
}

fn ids(links: &[AssemblyLink]) -> Vec<&str> {
    links.iter().map(|l| l.component.as_str()).collect()
}

/// The draft links of the stack group.
fn draft(h: &H) -> Vec<AssemblyLink> {
    h.state().state.tool_library.as_ref().and_then(|v| v.stack.as_ref()).map(|d| d.links.clone()).unwrap_or_default()
}

/// Open the "Add component" combo box and pick the item `text`.
fn add_by_clicks(h: &mut H, text: &str) {
    library_window(h).get_by_role_and_label(Role::ComboBox, stack::ADD).click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, text).click();
    h.run_steps(3);
}

/// The text the group shows for the resolver's values.
fn value_texts(gauge: f64, diameter: f64, corner: f64) -> [String; 3] {
    [
        format!("{}: {gauge:.3} mm", stack::GAUGE),
        format!("{}: {diameter:.3} mm", stack::DIAMETER),
        format!("{}: {corner:.3} mm", stack::CORNER),
    ]
}

#[test]
fn the_add_component_combo_box_builds_the_stack_and_apply_lands_it() {
    let dir = lib_dir("stack-combo");
    let mut h = shell(&dir);
    shop(&mut h, false);
    open_in_window(&mut h, A);
    assert!(shows(&h, "No component stack: the tool is standalone."));
    for text in ["BT40 CYL 20", "fly-25 in BT40 CYL 20 CSW", "CCMT09T304 in fly-25 CSW1"] {
        add_by_clicks(&mut h, text);
    }
    assert_eq!(ids(&draft(&h)), vec!["bt40-cyl20", "fly-25", INSERT], "each pick adds its part to the draft");
    assert!(shows(&h, "(from the stack edits; not applied)"), "the values of a draft say so");
    let before = done(&h);
    click_window_button(&mut h, stack::APPLY);
    let stored = assembly(&h, A).unwrap_or_else(|| panic!("the stack is stored"));
    assert_eq!(ids(&stored), vec!["bt40-cyl20", "fly-25", INSERT]);
    assert_eq!(done(&h), before + 1, "Apply stack is one undo step");
    assert!((tool(&h, A).corner_r - 0.4).abs() < 1e-9, "the insert's corner lands on the tool");
    // The insert holds nothing: the combo offers no part after it but a twin insert.
    undo(&mut h);
    assert_eq!(assembly(&h, A), None, "undo takes the stack away");
    redo(&mut h);
    assert_eq!(assembly(&h, A).as_deref().map(ids), Some(vec!["bt40-cyl20", "fly-25", INSERT]));
}

#[test]
fn the_stack_shows_the_resolved_gauge_diameter_and_corner_or_the_refusal() {
    let dir = lib_dir("stack-values");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_in_window(&mut h, A);
    let (links, table, current) = {
        let st = &h.state().state;
        (assembly(&h, A).unwrap_or_default(), ta::components(&st.world), tool(&h, A))
    };
    let d = ta::derive(A, &current, &links, &table).unwrap_or_else(|e| panic!("the shop's stack resolves: {e}"));
    for text in value_texts(d.gauge_len, d.tool.diameter, d.tool.corner_r) {
        assert!(shows(&h, &text), "the group shows '{text}'");
    }
    assert!(!shows(&h, "(from the stack edits; not applied)"), "the applied stack needs no note");
    // Without its insert the stack has no cutter: the group says why.
    click_window_button(&mut h, stack::REMOVE);
    let e = ta::derive(A, &current, &links[..2], &table).err().unwrap_or_else(|| panic!("a stack without a cutter is refused"));
    assert!(shows(&h, &format!("The stack does not resolve: {e}")), "the refusal is shown by name: {e}");
}

#[test]
fn a_swap_offers_only_parts_that_resolve_and_is_one_undo_step() {
    let dir = lib_dir("stack-swap");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_in_window(&mut h, A);
    let (links, current) = (assembly(&h, A).unwrap_or_default(), tool(&h, A));
    let view = h.state_mut().state.component_stack_view(A, Some(&current), &links);
    let offered: Vec<Vec<&str>> = view.swaps.iter().map(|s| s.iter().map(|(t, _)| t.as_str()).collect()).collect();
    assert_eq!(offered[0], Vec::<&str>::new(), "no other part takes the spindle side with this stack");
    assert_eq!(offered[1], vec!["fly-32 in BT40 CYL 20 CSW"], "only the other cutter body fits in its place");
    assert_eq!(offered[2], Vec::<&str>::new(), "the document holds one insert");
    let root = stack::swap_label("BT40 CYL 20");
    assert!(library_window(&h).get_by_role_and_label(Role::Button, &root).accesskit_node().is_disabled(), "nothing to offer: the swap is off");

    let before = done(&h);
    let old_d = tool(&h, A).diameter;
    library_window(&h).get_by_role_and_label(Role::Button, &stack::swap_label("fly-25")).click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "fly-32 in BT40 CYL 20 CSW").click();
    h.run_steps(3);
    assert_eq!(assembly(&h, A).as_deref().map(ids), Some(vec!["bt40-cyl20", "fly-32", INSERT]), "the body is swapped in place");
    assert_eq!(done(&h), before + 1, "a swap is one undo step");
    assert!(tool(&h, A).diameter > old_d, "the tool follows the wider body");
    undo(&mut h);
    assert_eq!(assembly(&h, A), Some(links.clone()), "undo restores the stack");
    assert!((tool(&h, A).diameter - old_d).abs() < 1e-9, "and the tool");

    // Stack edits that are not applied hold the swap back.
    click_window_button(&mut h, stack::REMOVE);
    assert!(library_window(&h).get_by_role_and_label(Role::Button, &stack::swap_label("fly-25")).accesskit_node().is_disabled());
}

#[test]
fn a_still_frame_builds_no_stack_view() {
    let dir = lib_dir("stack-still");
    let mut h = shell(&dir);
    shop(&mut h, true);
    open_in_window(&mut h, A);
    let builds = |h: &H| h.state().state.component_views.builds().stack;
    let b0 = builds(&h);
    h.run_steps(3);
    assert_eq!(builds(&h), b0, "a still frame builds no stack view");
    click_window_button(&mut h, stack::REMOVE);
    assert!(builds(&h) > b0, "a changed stack builds the view again");
}

#[test]
fn show_compatible_parts_on_a_built_in_tool_lists_its_collets() {
    let dir = lib_dir("stack-collets");
    let mut h = shell(&dir);
    open_library(&mut h);
    pick(&mut h, "builtin:tool:EM-1200-2FL");
    click_window_button(&mut h, components::SHOW_COMPATIBLE);
    for series in ["ER20", "ER25", "ER32", "ER40"] {
        assert!(shows(&h, &format!("{series} collet 12 mm")), "{series}: the 12 mm collet takes the 12 mm shank");
    }
    assert!(!shows(&h, "ER32 collet 10 mm"), "a collet of another bore is hidden");
    assert!(!shows(&h, "BT40 ER32"), "the chuck takes the collet, not the tool");
}
