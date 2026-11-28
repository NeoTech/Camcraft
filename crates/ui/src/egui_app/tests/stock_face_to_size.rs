//! #3196: the Stock page's "Face to size" group adds a Face on the stock
//! from numbers through one undoable `FaceToSize`; with "Set the stock to
//! this box" ticked the same undo step takes the box back too.
//! Provenance: our own page law.

use super::*;

use crate::egui_app::stock_face_to_size::{APPLY, GROUP, SET_STOCK};
use camcraft_app::commands::FaceToSizeSpec;
use camcraft_app::operation::SrcRef;

fn stock_page() -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    harness.state_mut().state.ui_target = UiTarget::Stock;
    harness.run();
    harness
}

fn face_src(st: &State, name: &str) -> Option<SrcRef> {
    let e = camcraft_app::model::find_by_name(&st.world, name)?;
    camcraft_app::model::operation_of(&st.world, &e).map(|o| o.src)
}

#[test]
fn the_stock_page_adds_a_face_to_size_as_one_undo_step() {
    let mut h = stock_page();
    for label in [GROUP, APPLY, SET_STOCK, "Top Z", "Final Z", "Depth per pass", "Stepover", "Climb cut"] {
        assert!(h.query_all_by_label(label).next().is_some(), "the group shows {label}");
    }
    let before = h.state().state.history.done_len();
    h.get_by_label(APPLY).scroll_to_me();
    h.run();
    h.get_by_label(APPLY).click();
    h.run_steps(3);
    let st = &mut h.state_mut().state;
    assert_eq!(face_src(st, "Face to size"), Some(SrcRef::Stock), "a Face on the stock: {}", st.status);
    assert_eq!(st.history.done_len(), before + 1, "one history step");
    assert!(st.history.undo(&st.world), "one undo step");
    assert_eq!(face_src(st, "Face to size"), None, "the undo takes the Face back");
}

#[test]
fn face_to_size_with_a_ticked_box_sets_the_stock_in_the_same_undo_step() {
    let mut h = stock_page();
    let st = &mut h.state_mut().state;
    let held = camcraft_app::model::stock_world_aabb(&st.world);
    let tool = camcraft_app::model::named_entities(&st.world)
        .into_iter()
        .find(|(_, e)| camcraft_app::model::is_tool(&st.world, e))
        .map(|(n, _)| n)
        .expect("the seed has a tool");
    st.pending.push_back(CommandKind::FaceToSize(FaceToSizeSpec {
        name: Some("FTS".into()),
        tool,
        stock: Some([90.0, 70.0, 30.0]),
        top_z: 40.0,
        final_z: 39.0,
        stepdown: 0.5,
        stepover: 1.0,
        climb: true,
        feed: 1000.0,
        plunge_feed: 200.0,
    }));
    st.drain_actions();
    let b = camcraft_app::model::stock_world_aabb(&st.world).expect("stock");
    assert!((b.max.z - 40.0).abs() < 1e-9 && (b.max.x - b.min.x - 90.0).abs() < 1e-9, "the box, top at Z40: {b:?} {}", st.status);
    assert_eq!(face_src(st, "FTS"), Some(SrcRef::Stock));
    assert!(st.history.undo(&st.world));
    assert_eq!(face_src(st, "FTS"), None);
    assert_eq!(camcraft_app::model::stock_world_aabb(&st.world), held, "the same undo puts the old stock back");
}

#[test]
fn a_refused_face_to_size_names_the_number_and_adds_nothing() {
    let mut h = stock_page();
    let st = &mut h.state_mut().state;
    let before = st.history.done_len();
    st.pending.push_back(CommandKind::FaceToSize(FaceToSizeSpec {
        name: None,
        tool: String::new(),
        stock: Some([90.0, 70.0, 30.0]),
        top_z: 0.0,
        final_z: 0.0,
        stepdown: 0.5,
        stepover: 1.0,
        climb: true,
        feed: 1000.0,
        plunge_feed: 200.0,
    }));
    st.drain_actions();
    assert!(st.status.contains("below the top Z"), "{}", st.status);
    assert_eq!(st.history.done_len(), before, "nothing entered the history");
}
