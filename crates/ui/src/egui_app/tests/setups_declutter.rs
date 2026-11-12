//! #1104: the Setups page lists setups only (rows with Delete, New setup,
//! Export setup sheets) - a setup's alignment, work offset, workholding
//! and tool list live on its own Setup<n> pages, and a document with no
//! setups keeps its implicit-context sections.
//! Provenance: our own tests over our own shell; no HSMWorks material.
//!
//! Gate map (M4.4): covers ui::egui_app::pages (setups_page).

use super::*;
use crate::state::SetupPage;

const MOVED: [&str; 4] = ["Part alignment", "Lay on a plane", "Axes in the plane", "Work offset"];

fn cube_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/cube.stl")
}

/// Whether any painted label contains `text` (the page may repeat one).
fn shows(h: &Harness<'static, ShellUi>, text: &str) -> bool {
    h.query_all_by_label_contains(text).next().is_some()
}

fn open(h: &mut Harness<'static, ShellUi>, t: UiTarget) {
    crate::actions::target(&mut h.state_mut().state, t);
    h.run_steps(3);
}

fn two_setups_and_a_plate() -> Harness<'static, ShellUi> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1280.0, 720.0))
        .build_ui_state(frame, ShellUi::blank());
    {
        let st = &mut h.state_mut().state;
        for _ in 0..2 {
            st.pending.push_back(CommandKind::CreateSetup { name: None, datum_from: None });
            st.drain_actions();
        }
        st.pending.push_back(CommandKind::ImportFile(cube_path()));
        st.settle();
        st.pending.push_back(CommandKind::TagFixture { name: "cube".into(), setup: Some("Setup1".into()) });
        st.drain_actions();
    }
    h
}

#[test]
fn the_setups_page_lists_setups_only_and_each_setup_page_keeps_its_settings() {
    let mut h = two_setups_and_a_plate();
    open(&mut h, UiTarget::Setups);
    assert_eq!(h.get_all_by_label("Delete").count(), 2, "a Delete per setup row");
    assert_eq!(h.get_all_by_label("Open page").count(), 2, "a way to each setup's page");
    h.get_by_label("New setup");
    h.get_by_label("Carry stock and WCS over");
    h.get_by_label_contains("Export setup sheets");
    for text in MOVED.into_iter().chain(["orkholding", "Tool list"]) {
        assert!(!shows(&h, text), "{text:?} left the Setups page");
    }

    h.state_mut().state.pending.push_back(CommandKind::ActivateSetup(Some("Setup1".into())));
    h.run_steps(3);
    // The way in: Open page lands on that setup's own page.
    h.get_all_by_label("Open page").nth(1).expect("second row").click();
    h.run_steps(3);
    assert!(
        matches!(&h.state().state.ui_target, UiTarget::SetupNode(n, SetupPage::Settings) if n == "Setup2"),
        "Open page targets Setup2: {:?}",
        h.state().state.ui_target
    );

    // Open page on Setup2 also made it the active setup.
    assert_eq!(camcraft_app::setup::active(&h.state().state.world).as_deref(), Some("Setup2"));
    for name in ["Setup1", "Setup2"] {
        h.state_mut().state.pending.push_back(CommandKind::ActivateSetup(Some(name.into())));
        h.state_mut().state.drain_actions();
        open(&mut h, UiTarget::SetupNode(name.into(), SetupPage::Settings));
        for text in MOVED {
            assert!(shows(&h, text), "{name} page shows {text:?}");
        }
        open(&mut h, UiTarget::SetupNode(name.into(), SetupPage::Fixtures));
        assert!(shows(&h, "orkholding"), "{name} fixtures page shows Workholding");
        open(&mut h, UiTarget::SetupNode(name.into(), SetupPage::Tools));
        assert!(shows(&h, "Tool list"), "{name} tools page shows the tool list");
    }
}

#[test]
fn a_document_without_setups_keeps_its_implicit_sections_on_the_setups_page() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1280.0, 720.0))
        .build_ui_state(frame, ShellUi::blank());
    open(&mut h, UiTarget::Setups);
    h.get_by_label("Create first setup");
    assert!(shows(&h, "orkholding"), "the implicit workholding stays");
    assert!(shows(&h, "Tool list"), "the implicit tool list stays");
    h.get_by_label_contains("Export setup sheets");
}
