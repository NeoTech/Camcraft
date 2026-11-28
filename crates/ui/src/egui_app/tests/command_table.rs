//! #4484: the one command table. The ribbon and the menu bar list the same
//! command ids, each command has a label, a tip and an enabled rule, no two
//! commands share a shortcut, and the menu rows run the actions layer. The
//! menu bar's named layout test is here too (#1953).
//! Gate map: covers ui::egui_app::panes, ui::egui_app::shell.
//! Provenance: our own tests over our own command table; no HSMWorks
//! material involved.

use std::collections::BTreeSet;

use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::*;

type H = Harness<'static, ShellUi>;

fn kinds() -> Vec<camcraft_app::machine::Kind> {
    crate::machine_modes::MACHINE_MODES.iter().map(|(k, _, _)| *k).collect()
}

fn ids(cmds: &[Cmd]) -> BTreeSet<String> {
    cmds.iter().map(|c| panes::id_of(*c)).collect()
}

/// The menu bar shows every ribbon command, on every machine, and with no
/// pins the two lists differ only by the table's menu-only rows.
#[test]
fn ribbon_and_menu_bar_list_the_same_command_ids() {
    let menu_only: BTreeSet<String> = panes::TABLE
        .iter()
        .filter(|r| r.ribbon.is_none() && r.menu.is_some())
        .map(|r| r.id.to_owned())
        .collect();
    assert!(!menu_only.is_empty());
    for machine in kinds() {
        let menu = ids(&panes::menu_cmds(machine));
        let ribbon = ids(&panes::ribbon_cmds(machine, &[]));
        let extra: BTreeSet<String> = menu.difference(&ribbon).cloned().collect();
        assert_eq!(extra, menu_only, "{}: menu-only commands", machine.word());
        let missing: Vec<&String> = ribbon.difference(&menu).collect();
        assert!(missing.is_empty(), "{}: ribbon commands not in the menu: {missing:?}", machine.word());
        // Pins move a kind onto the bar; the menu still lists it.
        let pinned = ids(&panes::ribbon_cmds(machine, &[0, 2, 5]));
        let lost: Vec<&String> = pinned.difference(&menu).collect();
        assert!(lost.is_empty(), "{}: pinned commands not in the menu: {lost:?}", machine.word());
    }
}

/// Each command has a unique id, a label, a tip and an enabled rule; each
/// ribbon command is also on a menu.
#[test]
fn every_command_has_a_label_a_tip_and_an_enabled_rule() {
    let h = shell_harness();
    let state = &h.state().state;
    let mut seen = BTreeSet::new();
    for r in panes::TABLE {
        assert!(seen.insert(r.id), "two rows have the id {}", r.id);
        assert_eq!(panes::id_of(r.cmd), r.id);
        assert!(!r.label.is_empty(), "{}: no label", r.id);
        assert_eq!(panes::label_of(r.cmd), r.label, "{}: one label", r.id);
        assert!(!panes::tip_of(r.cmd, state).trim().is_empty(), "{}: no tip", r.id);
        let _ = panes::enabled(r.cmd, state);
        if r.ribbon.is_some() {
            assert!(r.menu.is_some(), "{}: on the ribbon but on no menu", r.id);
        }
    }
    let mut all = Vec::new();
    for machine in kinds() {
        all.extend(panes::menu_cmds(machine));
    }
    for c in all {
        assert!(!panes::text_of(c, state).is_empty(), "{c:?}: no label");
        assert!(!panes::tip_of(c, state).trim().is_empty(), "{c:?}: no tip");
        let _ = panes::enabled(c, state);
    }
}

/// One key press never fires two commands, and each tip names its key.
#[test]
fn no_two_commands_share_a_shortcut() {
    let h = shell_harness();
    let state = &h.state().state;
    let keyed: Vec<_> = panes::TABLE.iter().filter_map(|r| r.key.map(|k| (r, k))).collect();
    assert!(keyed.len() >= 10, "the keyboard layer reads the table");
    for (i, (a, ka)) in keyed.iter().enumerate() {
        for (b, kb) in &keyed[i + 1..] {
            assert!(!ka.overlaps(kb), "{} and {} share the shortcut {}", a.id, b.id, ka.hint());
        }
        assert!(
            panes::tip_of(a.cmd, state).ends_with(&format!("\t{}", ka.hint())),
            "{}: the tip names the key {}",
            a.id,
            ka.hint()
        );
    }
}

/// A disabled menu row says why (the gates without a refusal of their own).
#[test]
fn disabled_menu_rows_give_the_reason() {
    let h = shell_harness();
    let state = &h.state().state;
    assert!(!panes::enabled(Cmd::Undo, state));
    assert_eq!(panes::disabled_reason(Cmd::Undo, state), Some("Nothing to undo"));
    assert!(!panes::enabled(Cmd::Paste, state));
    assert_eq!(panes::disabled_reason(Cmd::Paste, state), Some("Copy an operation first"));
    assert_eq!(panes::disabled_reason(Cmd::Post, state), None, "a live command has no reason");
}

fn open_menu(h: &mut H, title: &str) {
    h.get_by_role_and_label(Role::Button, &format!("{title} menu")).click();
    h.run_steps(2);
}

/// A menu row runs the actions layer: File > Quit queues the same Quit
/// command as the window close button and Ctrl+Q.
#[test]
fn file_menu_quit_runs_the_actions_layer() {
    let mut h = shell_harness();
    h.run();
    open_menu(&mut h, "File");
    h.get_by_role_and_label(Role::Button, "Quit").click();
    h.run_steps(1);
    let st = &h.state().state;
    // The Quit command either waits in the queue, has run, or has asked
    // the unsaved-changes prompt first (#107).
    assert!(
        st.pending.iter().any(|c| matches!(c, CommandKind::Quit)) || st.request_quit || st.unsaved_prompt.is_some(),
        "File > Quit did not reach the Quit command: {:?}",
        st.pending
    );
}

/// View > Toggle projection flips the camera, as the bare P key does; the
/// Edit menu's Paste row is disabled with nothing copied.
#[test]
fn view_menu_projection_and_disabled_paste() {
    let mut h = shell_harness();
    h.run();
    let before = h.state().state.cam.projection;
    open_menu(&mut h, "View");
    h.get_by_role_and_label(Role::Button, "Toggle projection").click();
    h.run_steps(2);
    assert_ne!(h.state().state.cam.projection, before, "View > Toggle projection flips the camera");
    open_menu(&mut h, "Edit");
    assert!(h.get_by_role_and_label(Role::Button, "Paste").accesskit_node().is_disabled());
}

/// The Operation menu reaches a kind through its section submenu and
/// creates it through the actions layer, like the ribbon's door row.
#[test]
fn operation_menu_creates_a_kind() {
    let mut h = shell_harness();
    h.run();
    open_menu(&mut h, "Operation");
    h.get_by_role_and_label(Role::Button, "Milling 2D").hover();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Pocket").click();
    h.run_steps(3);
    let st = &h.state().state;
    assert!(
        model::find_by_name(&st.world, "2D Pocket1").is_some(),
        "Operation > Milling 2D > Pocket did not create (status: {})",
        st.status
    );
}

/// #1953: the menu bar's coarse layout. The six menus sit in one row at
/// the top of the window, in bar order, and the ribbon starts below them.
#[test]
fn layout_window_menu_bar() {
    let mut h = shell_harness();
    h.run_steps(3);
    let bar_h = panes::menu_bar_h(&theme::current());
    let mut last_x = f32::NEG_INFINITY;
    for m in panes::Menu::ALL {
        let r = h.get_by_role_and_label(Role::Button, &format!("{} menu", m.title())).rect();
        assert!(r.top() >= -1.0 && r.bottom() <= bar_h + 1.0, "{} menu {r:?} leaves the bar", m.title());
        assert!(r.left() > last_x, "{} menu is out of bar order", m.title());
        last_x = r.left();
    }
    for cell in ["New", "Save", "Undo", "Simulate"] {
        let r = h.get_by_role_and_label(Role::Button, cell).rect();
        assert!(r.top() >= bar_h - 1.0, "ribbon cell {cell} {r:?} must sit under the menu bar");
    }
}
