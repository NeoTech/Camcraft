//! #1119 slice 3: every input on every operation page tab has an
//! accessible name - the visible row label (`labelled_by`) or a
//! plain-English name. The walk retypes the draft through each kind of
//! the registry, as `every_registry_kind_gets_its_tab_bar` does, and
//! opens each tab the kind has. An input that a check box or a choice
//! hides is not reached here; its row names it through the same seam.
//! Provenance: our own accessibility law over the AccessKit tree.

use super::*;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

fn is_input(role: Role) -> bool {
    matches!(role, Role::SpinButton | Role::TextInput | Role::MultilineTextInput | Role::ComboBox | Role::Slider)
}

/// The inputs of the Properties pane with no name, or with one label
/// named twice (a screen reader reads it twice), as "role on row", and
/// the number of inputs seen. The row is the nearest text left of the
/// input.
fn unnamed_in_properties(h: &Harness<'static, ShellUi>) -> (Vec<String>, usize) {
    let pane = h.get_by_role_and_label(Role::Pane, "Properties");
    let texts: Vec<(egui::Rect, String)> = pane
        .children_recursive()
        .filter(|n| n.accesskit_node().role() == Role::Label)
        .filter_map(|n| {
            let node = n.accesskit_node();
            let b = node.bounding_box()?;
            let r = egui::Rect::from_min_max(egui::pos2(b.x0 as f32, b.y0 as f32), egui::pos2(b.x1 as f32, b.y1 as f32));
            node.value().map(|v| (r, v.to_string()))
        })
        .collect();
    let (mut bad, mut seen) = (Vec::new(), 0);
    for n in pane.children_recursive() {
        let node = n.accesskit_node();
        if !is_input(node.role()) {
            continue;
        }
        seen += 1;
        let ids: Vec<_> = node.labelled_by().map(|l| l.id()).collect();
        let mut once = ids.clone();
        once.sort();
        once.dedup();
        let named = !ids.is_empty() || node.label().is_some_and(|l| !l.is_empty());
        if named && once.len() == ids.len() {
            continue;
        }
        let row = node.bounding_box().and_then(|b| {
            let (y, left) = ((b.y0 + b.y1) as f32 / 2.0, b.x0 as f32);
            texts
                .iter()
                .filter(|(r, _)| (r.center().y - y).abs() < 9.0 && r.right() <= left + 1.0)
                .min_by(|a, b| (left - a.0.right()).total_cmp(&(left - b.0.right())))
                .map(|(_, t)| t.clone())
        });
        let why = if named { "named twice by one label" } else { "no name" };
        bad.push(format!("{:?} on {:?}: {why}", node.role(), row.unwrap_or_default()));
    }
    (bad, seen)
}

#[test]
fn every_operation_page_tab_names_every_input() {
    let mut harness = shell_harness();
    harness.run();
    let secs = super::super::op_tabs::SECTIONS;
    let mut bad: Vec<String> = Vec::new();
    let mut seen = 0;
    for (ki, k) in camcraft_app::operation::KindInfo::ALL.iter().enumerate() {
        let cur = harness.state().state.op_ui.params.clone();
        let retyped = crate::operations::retype(&cur, ki);
        let kind = retyped.kind();
        harness.state_mut().state.op_ui.params = retyped;
        for (si, sname) in secs.iter().enumerate() {
            if !super::super::op_tabs::has_section(kind, si) {
                continue;
            }
            harness.state_mut().state.op_tab = si;
            harness.run();
            let (b, s) = unnamed_in_properties(&harness);
            seen += s;
            bad.extend(b.into_iter().map(|b| format!("{} / {sname}: {b}", k.label)));
        }
    }
    assert!(seen > 500, "the walk reached only {seen} inputs");
    bad.sort();
    bad.dedup();
    assert!(bad.is_empty(), "{} operation page inputs have no single name:\n{}", bad.len(), bad.join("\n"));
}
