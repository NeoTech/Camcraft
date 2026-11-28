//! #3896 (U5 of #3663): the Recipes tab of the tool library window.
//! "Suggest" and "Copy from tool" fill the window's draft and queue no
//! command; the one Apply lands the rows as one `ChangeTool`; the row of
//! the document's material is marked; Properties' draft never changes.
//! #4337 (U11): the star follows the stock material in force; the derived
//! lines follow the rows' rpm and feed and a still frame does not compute
//! them; "Add material…" lands as one undo step.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::egui_app::tool_cutting_rows, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use super::tool_library_editor::{H, click_window_button, draft, library_window, open_in_window, tool};
use super::*;
use crate::egui_app::tool_library_dialog::recipes_actions::{ADD, COPY, SUGGEST};
use crate::egui_app::tool_library_dialog::recipes_tab::{STAR, TAB};
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;
use crate::state::CommandKind;
use camcraft_app::tool_cutting_rows::CuttingRow;
use camcraft_app::tools::{Tool, ToolMaterial, WorkMaterial};

/// A document tool that the cutting table speaks for (an end, ball, bull
/// nose or chamfer mill), with a carbide edge, three flutes and the
/// given material rows. Lands through `ChangeTool`.
pub(super) fn mill(h: &mut H, rows: Vec<CuttingRow>) -> String {
    let st = &h.state().state;
    let found = st.tool_names().into_iter().find(|n| {
        model::find_by_name(&st.world, n)
            .and_then(|e| model::tool_of(&st.world, &e))
            .is_some_and(|t| camcraft_app::cutting::tabulated(t.kind))
    });
    let name = match found {
        Some(n) => n,
        None => {
            let Some((n, t)) = camcraft_app::tools::catalog().into_iter().find(|(n, t)| {
                model::find_by_name(&st.world, n).is_none()
                    && camcraft_app::cutting::tabulated(t.kind)
                    && crate::machine_modes::tool_kind_offered(&st.world, t.kind)
            }) else {
                panic!("the catalog has a mill for this machine");
            };
            h.state_mut().state.pending.push_back(CommandKind::AddTool { name: n.to_owned(), tool: t });
            h.run_steps(2);
            n.to_owned()
        }
    };
    let t = Tool { tool_material: Some(ToolMaterial::Carbide), flutes: Some(3), cutting: rows, ..tool(h, &name) };
    h.state_mut().state.pending.push_back(CommandKind::ChangeTool { name: name.clone(), tool: t });
    h.run_steps(2);
    assert_eq!(tool(h, &name).flutes, Some(3), "the mill lands: {}", h.state().state.status);
    name
}

/// A second mill from the catalog, added (`AddTool`) with `rows`.
fn other_mill(h: &mut H, rows: Vec<CuttingRow>) -> String {
    let st = &h.state().state;
    let Some((n, t)) = camcraft_app::tools::catalog().into_iter().find(|(n, t)| {
        model::find_by_name(&st.world, n).is_none()
            && camcraft_app::cutting::tabulated(t.kind)
            && crate::machine_modes::tool_kind_offered(&st.world, t.kind)
    }) else {
        panic!("the catalog has a second mill for this machine");
    };
    let tool = Tool { cutting: rows, ..t };
    h.state_mut().state.pending.push_back(CommandKind::AddTool { name: n.to_owned(), tool });
    h.run_steps(2);
    assert!(model::find_by_name(&h.state().state.world, n).is_some(), "the second mill lands: {}", h.state().state.status);
    n.to_owned()
}

fn set_material(h: &mut H, m: Option<WorkMaterial>) {
    h.state_mut().state.pending.push_back(CommandKind::SetWorkMaterial(m));
    h.run_steps(3);
}

fn row_of(rows: &[CuttingRow], m: WorkMaterial) -> Option<CuttingRow> {
    rows.iter().find(|r| r.material == Some(m) && !r.is_recipe()).copied()
}

/// Suggest fills the window's draft from the cutting table and queues no
/// command; Properties' draft of the same tool does not change. Apply is
/// one `ChangeTool` undo step.
#[test]
fn suggest_fills_the_draft_and_apply_is_one_undo_step() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = mill(&mut h, Vec::new());
    set_material(&mut h, Some(WorkMaterial::Aluminium6061));
    crate::actions::target(&mut h.state_mut().state, UiTarget::Tool(name.clone()));
    h.run_steps(3);
    open_in_window(&mut h, &name);
    click_window_button(&mut h, TAB);
    let before = tool(&h, &name);
    let props = h.state().state.tool_ui.cutting.clone();
    let done = h.state().state.history.done_len();
    pick_from_menu(&mut h, SUGGEST, &format!("{STAR} {}", WorkMaterial::Aluminium6061.label()));
    let st = &h.state().state;
    let machine = camcraft_app::machine::of_setup(&st.world, None).1;
    let want = camcraft_app::cutting::compute(
        &name, &before, 3, ToolMaterial::Carbide, WorkMaterial::Aluminium6061, None, machine.as_ref(),
    );
    let Some(row) = row_of(&draft(&mut h).cutting, WorkMaterial::Aluminium6061) else {
        panic!("Suggest adds the document material's row: {}", h.state().state.status);
    };
    assert_eq!((row.rpm, row.feed), (want.rpm, want.feed), "the row holds the table's numbers");
    assert_eq!(h.state().state.history.done_len(), done, "Suggest queues no command");
    assert_eq!(tool(&h, &name), before, "Suggest writes the draft, not the document");
    assert_eq!(h.state().state.tool_ui.cutting, props, "Properties' draft of the same tool does not change");
    click_window_button(&mut h, "Apply");
    assert_eq!(h.state().state.history.done_len(), done + 1, "one Apply is one undo step");
    let landed = row_of(&tool(&h, &name).cutting, WorkMaterial::Aluminium6061);
    assert_eq!(landed.map(|r| (r.rpm, r.feed)), Some((want.rpm, want.feed)), "Apply lands the row: {}", h.state().state.status);
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    assert_eq!(tool(&h, &name), before, "one undo restores the tool");
}

/// Copy from tool puts the other tool's material rows into the draft: a
/// row of the same material is replaced, the others join, the draft's own
/// rows stay. Nothing lands before Apply.
#[test]
fn copy_from_tool_copies_the_rows() {
    let mut h = shell_harness();
    h.run_steps(2);
    let steel = CuttingRow::plain(Some(WorkMaterial::MildSteel), 3000.0, 500.0);
    let brass = CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0);
    let plastics = CuttingRow::plain(Some(WorkMaterial::Plastics), 12000.0, 2000.0);
    let a = mill(&mut h, vec![CuttingRow::plain(Some(WorkMaterial::MildSteel), 1000.0, 100.0), plastics]);
    let b = other_mill(&mut h, vec![steel, brass]);
    open_in_window(&mut h, &a);
    click_window_button(&mut h, TAB);
    let before = tool(&h, &a);
    let done = h.state().state.history.done_len();
    pick_from_menu(&mut h, COPY, &b);
    let rows = draft(&mut h).cutting.clone();
    assert_eq!(row_of(&rows, WorkMaterial::MildSteel), Some(steel), "the same material's row is replaced");
    assert_eq!(row_of(&rows, WorkMaterial::Brass), Some(brass), "a new material's row joins");
    assert_eq!(row_of(&rows, WorkMaterial::Plastics), Some(plastics), "the draft's own row stays");
    assert_eq!(h.state().state.history.done_len(), done, "Copy queues no command");
    assert_eq!(tool(&h, &a), before, "Copy writes the draft, not the document");
    click_window_button(&mut h, "Apply");
    assert_eq!(h.state().state.history.done_len(), done + 1, "one Apply is one undo step");
    let landed = tool(&h, &a).cutting;
    assert_eq!(row_of(&landed, WorkMaterial::Brass), Some(brass), "Apply lands the copied rows: {}", h.state().state.status);
}

/// The row of the document's material carries the star; with no row for
/// it, the default row does; with no material stated, no row does. The
/// derived line under the table shows the starred row's surface speed.
#[test]
fn the_document_material_row_is_marked() {
    let mut h = shell_harness();
    h.run_steps(2);
    let rows = vec![
        CuttingRow::plain(Some(WorkMaterial::MildSteel), 3000.0, 500.0),
        CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0),
    ];
    let name = mill(&mut h, rows);
    set_material(&mut h, Some(WorkMaterial::Brass));
    open_in_window(&mut h, &name);
    click_window_button(&mut h, TAB);
    let plain: Vec<CuttingRow> = draft(&mut h).cutting.iter().filter(|r| r.material.is_some() && !r.is_recipe()).copied().collect();
    let Some(brass) = plain.iter().position(|r| r.material == Some(WorkMaterial::Brass)) else {
        panic!("the draft has the Brass row");
    };
    let marks = |h: &H| -> Vec<egui::Rect> { library_window(h).query_all_by_label(STAR).map(|n| n.rect()).collect() };
    let removes = |h: &H| -> Vec<egui::Rect> { library_window(h).query_all_by_label("Remove").map(|n| n.rect()).collect() };
    let on_row = |m: egui::Rect, r: egui::Rect| m.center().y >= r.top() - 4.0 && m.center().y <= r.bottom() + 4.0;
    let (m, r) = (marks(&h), removes(&h));
    assert_eq!(m.len(), 1, "one row is marked");
    assert_eq!(r.len(), plain.len(), "each material row has its Remove button");
    assert!(on_row(m[0], r[brass]), "the mark {:?} stands on the Brass row {:?}", m[0], r[brass]);
    if h.state().state.units() == camcraft_app::units::Unit::Millimeters {
        let d = tool(&h, &name).diameter;
        let vc = std::f64::consts::PI * d * 8000.0 / 1000.0;
        let label = format!("{} ({})", crate::egui_app::tool_library_dialog::recipes_speeds::VC, WorkMaterial::Brass.label());
        let shown = library_window(&h).get_by_role_and_label(Role::SpinButton, &label).accesskit_node().numeric_value();
        assert!(shown.is_some_and(|v| (v - vc).abs() < 1e-6), "the Brass row's surface speed field shows Vc {vc}: {shown:?}");
    }
    set_material(&mut h, Some(WorkMaterial::Hdpe));
    let m = marks(&h);
    assert_eq!(m.len(), 1, "with no HDPE row, the default row is marked");
    assert!(removes(&h).iter().all(|r| !on_row(m[0], *r)), "the default row has no Remove button");
    set_material(&mut h, None);
    assert!(marks(&h).is_empty(), "no material stated, no mark");
}

/// Click the menu button `menu` in the window, then its entry `entry`
/// (the menu stands under its button; the nearest twin is the entry).
fn pick_from_menu(h: &mut H, menu: &str, entry: &str) {
    let at = library_window(h).get_by_role_and_label(Role::Button, menu).rect();
    library_window(h).get_by_role_and_label(Role::Button, menu).click();
    h.run_steps(2);
    let near = |r: egui::Rect| (r.top() - at.bottom()).abs() + (r.left() - at.left()).abs();
    let Some(node) = h
        .query_all_by_role_and_label(Role::Button, entry)
        .filter(|n| n.rect().top() >= at.bottom() - 2.0)
        .min_by(|a, b| near(a.rect()).total_cmp(&near(b.rect())))
    else {
        panic!("the {menu} menu has no entry {entry}");
    };
    node.click();
    h.run_steps(3);
}

/// The feed field of each drawn table row, top down.
fn feed_fields(h: &H) -> Vec<egui::Rect> {
    let mut v: Vec<egui::Rect> = library_window(h).query_all_by_label("Feed").map(|n| n.rect()).collect();
    v.sort_by(|a, b| a.top().total_cmp(&b.top()));
    v
}

/// The material of each drawn speed-and-feed row, top down: the table
/// shows the derived default row first when the draft stores none.
fn drawn(h: &mut H) -> Vec<Option<WorkMaterial>> {
    let rows = draft(h).cutting.clone();
    let lead = (!rows.iter().any(CuttingRow::is_unknown)).then_some(None);
    lead.into_iter().chain(rows.iter().filter(|r| !r.is_recipe()).map(|r| r.material)).collect()
}

/// #4337: the star stands on the row of the stock material in force - the
/// setup stock's material wins over the document's - and the first line
/// names that material.
#[test]
fn the_star_follows_the_stock_material() {
    let mut h = shell_harness();
    h.run_steps(2);
    let rows = vec![
        CuttingRow::plain(Some(WorkMaterial::MildSteel), 3000.0, 500.0),
        CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0),
    ];
    let name = mill(&mut h, rows);
    set_material(&mut h, Some(WorkMaterial::Brass));
    open_in_window(&mut h, &name);
    click_window_button(&mut h, TAB);
    let star_row = |h: &H| -> usize {
        let star = library_window(h).get_by_label(STAR).rect();
        feed_fields(h).iter().position(|r| (r.center().y - star.center().y).abs() < 8.0).unwrap_or(usize::MAX)
    };
    let at = |h: &mut H, m: WorkMaterial| drawn(h).iter().position(|k| *k == Some(m));
    let brass = at(&mut h, WorkMaterial::Brass);
    assert_eq!(Some(star_row(&h)), brass, "the document material's row (Brass) has the star: {:?}", drawn(&mut h));
    let first = format!("The {STAR} marks the stock material of this document ({})", WorkMaterial::Brass.label());
    assert!(library_window(&h).query_by_label_contains(&first).is_some(), "the first line names Brass");
    if camcraft_app::setup::stock_entity(&h.state().state.world, None).is_some() {
        h.state_mut().state.pending.push_back(CommandKind::SetStockMaterial(Some(WorkMaterial::MildSteel)));
        h.run_steps(3);
        let steel = at(&mut h, WorkMaterial::MildSteel);
        assert_eq!(Some(star_row(&h)), steel, "the stock's own material (Mild steel) wins: {:?}", drawn(&mut h));
        let first = format!("({})", WorkMaterial::MildSteel.label());
        assert!(library_window(&h).query_all_by_label_contains(&first).next().is_some(), "the first line names Mild steel");
    }
}

/// #4337: each row's derived line follows its rpm and feed; a still frame
/// computes the lines never.
#[test]
fn the_derived_line_follows_rpm_and_feed() {
    let mut h = shell_harness();
    h.run_steps(2);
    let rows = vec![
        CuttingRow::plain(Some(WorkMaterial::MildSteel), 3000.0, 500.0),
        CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0),
    ];
    let name = mill(&mut h, rows);
    set_material(&mut h, Some(WorkMaterial::Brass));
    open_in_window(&mut h, &name);
    click_window_button(&mut h, TAB);
    let computes = |h: &H| h.state().state.tool_library.as_ref().map_or(0, |v| v.recipes.computes);
    // #4453: the line is the hover text of the row's fields; read it from
    // the tab's cache.
    let line = |h: &H, m: WorkMaterial| -> String {
        h.state().state.tool_library.as_ref()
            .and_then(|v| v.recipes.derived.as_ref())
            .and_then(|(_, lines)| lines.iter().find(|l| l.ends_with(&format!("({})", m.label()))).cloned())
            .unwrap_or_default()
    };
    let (c0, l0) = (computes(&h), line(&h, WorkMaterial::Brass));
    assert!(c0 >= 1 && !l0.is_empty(), "the Brass row has its line ({c0} computes)");
    assert!(!line(&h, WorkMaterial::MildSteel).is_empty(), "the Mild steel row has its line");
    h.run_steps(5);
    assert_eq!(computes(&h), c0, "a still frame computes nothing");
    let Some(i) = draft(&mut h).cutting.iter().position(|r| r.material == Some(WorkMaterial::Brass)) else {
        panic!("the draft has the Brass row");
    };
    let steel = line(&h, WorkMaterial::MildSteel);
    draft(&mut h).cutting[i].rpm = 4000.0;
    h.run_steps(2);
    let l1 = line(&h, WorkMaterial::Brass);
    assert_ne!(l1, l0, "a new rpm changes the line");
    assert_eq!(line(&h, WorkMaterial::MildSteel), steel, "the other row's line stays");
    assert_eq!(computes(&h), c0 + 1, "one change, one computation");
    draft(&mut h).cutting[i].feed = 600.0;
    h.run_steps(2);
    assert_ne!(line(&h, WorkMaterial::Brass), l1, "a new feed changes the line");
}

/// #4337: "Add material…" adds the picked material's row to the draft and
/// queues nothing; Apply lands it as one undo step; one undo removes it.
#[test]
fn add_material_is_one_undo_step() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = mill(&mut h, vec![CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0)]);
    open_in_window(&mut h, &name);
    click_window_button(&mut h, TAB);
    let before = tool(&h, &name);
    let done = h.state().state.history.done_len();
    pick_from_menu(&mut h, ADD, WorkMaterial::Copper.label());
    assert!(row_of(&draft(&mut h).cutting, WorkMaterial::Copper).is_some(), "the draft has the Copper row");
    assert_eq!(h.state().state.history.done_len(), done, "Add queues no command");
    assert_eq!(tool(&h, &name), before, "Add writes the draft, not the document");
    click_window_button(&mut h, "Apply");
    assert_eq!(h.state().state.history.done_len(), done + 1, "one Apply is one undo step");
    assert!(row_of(&tool(&h, &name).cutting, WorkMaterial::Copper).is_some(), "Apply lands the row: {}", h.state().state.status);
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    assert_eq!(tool(&h, &name), before, "one undo restores the tool");
}
