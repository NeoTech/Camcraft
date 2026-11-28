//! #1122: the seam helpers that the common operation tabs added keep the
//! behaviour of the raw calls that they replace: a wrapped row and a
//! read-out keep their text, a read-out takes no selection, and a choice
//! button shows its state and takes a click only while it is enabled.
//! Provenance: our own tests over our own wrappers; no vendor material.

#[derive(Default)]
struct OpsProbe {
    on_hits: usize,
    off_hits: usize,
}

#[test]
fn the_operation_tab_seam_helpers_keep_clicks_text_and_state() {
    use egui::accesskit::{Role, Toggled};
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};

    let mut h = Harness::builder().with_size(egui::Vec2::new(480.0, 240.0)).build_ui_state(
        |ui, p: &mut OpsProbe| {
            super::list_row_wrapped(ui, "Boundary settings do not apply to it: the kind cuts holes.");
            super::readout(ui, "Setup 1 offset: X 0.000 Y 0.000 Z 0.000");
            if super::choice_button_enabled(ui, true, true, "Include").clicked() {
                p.on_hits += 1;
            }
            if super::choice_button_enabled(ui, false, false, "Exclude").clicked() {
                p.off_hits += 1;
            }
        },
        OpsProbe::default(),
    );
    h.run();

    for text in ["Boundary settings do not apply to it: the kind cuts holes.", "Setup 1 offset: X 0.000 Y 0.000 Z 0.000"] {
        assert!(h.query_by_label(text).is_some(), "{text} is on the page");
    }
    let toggled = |label: &str| {
        h.root()
            .children_recursive()
            .find(|n| n.accesskit_node().role() == Role::Button && n.accesskit_node().label().as_deref() == Some(label))
            .and_then(|n| n.accesskit_node().toggled())
    };
    assert_eq!(toggled("Include"), Some(Toggled::True), "the selected choice says so");
    assert_eq!(toggled("Exclude"), Some(Toggled::False), "the other choice says so");

    h.get_by_label("Include").click();
    h.get_by_label("Exclude").click();
    h.run();
    assert_eq!(h.state().on_hits, 1, "an enabled choice takes a click");
    assert_eq!(h.state().off_hits, 0, "a greyed-out choice takes no click");
}
