//! Help tips (#3030), through the real shell's accessible controls.
//! Gate map: covers ui::egui_app::tips, ui::egui_app::home, ui::state_prefs.
//! Provenance: our own structural tests over egui_kittest.

use super::*;
use egui::accesskit::Role;

#[test]
fn tips_use_the_screens_os_for_shortcuts() {
    for os in [
        egui::os::OperatingSystem::Mac,
        egui::os::OperatingSystem::Windows,
        egui::os::OperatingSystem::Nix,
    ] {
        let mut h = shell_harness();
        h.ctx.set_os(os);
        h.state_mut().state.tips_open = true;
        h.run_steps(3);
        let name = if os.is_mac() { "macos" } else { "linux" };
        let tip = camcraft_app::mcp::tips::all().unwrap()[1].for_os(name);
        assert!(
            h.query_by_label(&tip.text).is_some(),
            "{os:?} uses its own shortcut"
        );
    }
}

#[test]
fn show_me_opens_the_shared_guide_in_a_browser_tab() {
    let mut h = shell_harness();
    h.state_mut().state.browser_build = true;
    h.state_mut().state.tips_open = true;
    h.run_steps(3);
    let tip = &camcraft_app::mcp::tips::all().unwrap()[0];
    let before = h.state().state.history.done_len();
    h.get_by_role_and_label(Role::Button, &format!("Show me: {}", tip.title))
        .click();
    h.step();
    let urls: Vec<_> = h
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::OpenUrl(url) => Some(url),
            _ => None,
        })
        .collect();
    assert_eq!(urls.len(), 1);
    assert_eq!(Some(&urls[0].url), tip.guide_link.as_ref());
    assert!(urls[0].new_tab);
    assert_eq!(h.state().state.history.done_len(), before);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_guide_launcher_refusal_is_named_and_a_success_clears_it() {
    let mut h = shell_harness();
    h.state_mut().state.tips_open = true;
    h.run_steps(3);
    let id = egui::Id::new(("tip-guide", "mcp-setup"));
    let before = h.state().state.doc_version;
    crate::egui_app::browser_open::record_result(&h.ctx, id, Err("test launcher failure".into()));
    h.run_steps(2);
    assert!(
        h.query_by_label("Guide not opened: test launcher failure")
            .is_some()
    );
    crate::egui_app::browser_open::record_result(&h.ctx, id, Ok(()));
    h.run_steps(2);
    assert!(
        h.query_by_label("Guide not opened: test launcher failure")
            .is_none()
    );
    assert_eq!(h.state().state.doc_version, before);
}

#[test]
fn start_tip_is_nonblocking_and_opt_out_survives_restart() {
    let mut h = shell_harness();
    let dir = std::env::temp_dir().join(format!("camcraft-tip-prefs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let store = dir.join("prefs.json");
    let _ = std::fs::remove_file(&store);
    h.state_mut().state.load_prefs_from(store.clone());
    super::super::super::home::at_start(&mut h.state_mut().state, &[]);
    h.run_steps(3);
    assert!(h.query_by_label("Tip of the day").is_some());
    assert!(
        h.query_by_role_and_label(Role::Button, "New project")
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Window, "Tips").is_none());
    let version = h.state().state.doc_version;
    let history = h.state().state.history.done_len();
    h.get_by_role_and_label(Role::CheckBox, "Don't show again")
        .click();
    h.run_steps(3);
    assert!(h.query_by_label("Tip of the day").is_none());
    assert_eq!(h.state().state.doc_version, version);
    assert_eq!(h.state().state.history.done_len(), history);
    let mut next = shell_harness();
    next.state_mut().state.load_prefs_from(store.clone());
    super::super::super::home::at_start(&mut next.state_mut().state, &[]);
    next.run_steps(3);
    assert!(next.query_by_label("Tip of the day").is_none());
    // The manual door still works after opting out.
    next.get_by_role_and_label(Role::Button, "Help").click();
    next.run_steps(2);
    next.get_by_role_and_label(Role::Button, "Tips").click();
    next.run_steps(3);
    assert!(next.query_by_role_and_label(Role::Window, "Tips").is_some());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn start_tip_is_hidden_during_generation_and_returns_afterwards() {
    let mut h = shell_harness();
    super::super::super::home::at_start(&mut h.state_mut().state, &[]);
    h.run_steps(3);
    assert!(h.query_by_label("Tip of the day").is_some());
    let (tx, rx) = std::sync::mpsc::channel();
    h.state_mut().state.gen_jobs.run_with("Contour1", rx);
    h.run_steps(3);
    assert!(h.query_by_label("Tip of the day").is_none());
    assert!(
        h.query_by_role_and_label(Role::Button, "New project")
            .is_some()
    );
    tx.send(Err(camcraft_app::commands::CommandError::Machining(
        "test refusal".into(),
    )))
    .unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Tip of the day").is_some());
}

#[test]
fn help_opens_the_tips_window_without_changing_the_document() {
    let mut h = shell_harness();
    h.run();
    let version = h.state().state.doc_version;
    let history = h.state().state.history.done_len();
    h.get_by_role_and_label(Role::Button, "Help").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Tips").click();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Window, "Tips").is_some());
    for tip in camcraft_app::mcp::tips::all().unwrap() {
        let tip = tip.for_os(std::env::consts::OS);
        assert!(
            h.query_by_label(&tip.title).is_some(),
            "missing title {}",
            tip.id
        );
        assert!(
            h.query_by_label(&tip.text).is_some(),
            "missing shared text {}",
            tip.id
        );
        assert!(
            h.query_by_role_and_label(Role::Button, &format!("Show me: {}", tip.title))
                .is_some(),
            "missing guide action {}",
            tip.id
        );
    }
    assert_eq!(h.state().state.doc_version, version);
    assert_eq!(h.state().state.history.done_len(), history);
}
