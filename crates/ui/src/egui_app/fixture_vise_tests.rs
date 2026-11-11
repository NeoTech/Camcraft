//! #788 in the shell, structurally: the Fixtures node's menu and the
//! Fixtures page offer "Add built-in vise…"; the dialog lists the size
//! table, and its Place queues one `InsertVise` (fitted to the stock by
//! default; undo takes it away); a refusal is named in the dialog; a
//! placed vise's row edits its jaw opening through `SetVise` (Set and
//! "Open to stock"). Driven through the real dialog and page code in a
//! kittest harness - no pixels (the snapshot lane is release).
//! Provenance: our own tests over our own shell.

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use camcraft_app::commands::SetStock;
use camcraft_app::fixture;
use camcraft_app::model;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;

use crate::state::{CommandKind, FixtureLibDialog, State, TreeMenuTarget};

use super::fixture_library_dialog::window;
use super::fixture_menu::menu_rows;

fn with_stock() -> State {
    let mut state = State::new();
    let cmd = SetStock::new(&state.world, StockShape::Box { dx: 60.0, dy: 40.0, dz: 20.0 }, Location::default());
    state.history.execute(&state.world, cmd.expect("stock"));
    state.doc_version += 1;
    state
}

fn dialog(state: State) -> Harness<'static, State> {
    Harness::builder().with_size(Vec2::new(900.0, 1000.0)).build_ui_state(
        |ui, st: &mut State| {
            st.drain_actions();
            window(ui.ctx(), st);
        },
        state,
    )
}

fn opening(state: &State, name: &str) -> Option<f64> {
    model::find_by_name(&state.world, name)
        .and_then(|e| fixture::def_of(&state.world, &e))
        .and_then(|d| d.vise)
        .map(|v| v.opening)
}

#[test]
fn the_menu_and_the_page_offer_add_built_in_vise() {
    let state = State::new();
    let rows = menu_rows(&state, &TreeMenuTarget::Fixtures("Setup2".into()));
    let row = rows.iter().find(|(l, _)| l == "Add built-in vise…").expect("menu row");
    assert!(matches!(&row.1, Some(CommandKind::BuiltinViseDialog(s)) if s == "Setup2"));

    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(|ui, st: &mut State| super::pages::fixtures_section(ui, st, None), State::new());
    h.run();
    h.get_by_label("Add built-in vise…").click();
    h.run();
    assert!(matches!(h.state().pending.back(), Some(CommandKind::BuiltinViseDialog(s)) if s.is_empty()));
}

#[test]
fn the_dialog_places_a_fitted_vise_in_one_undoable_step() {
    let mut state = with_stock();
    state.pending.push_back(CommandKind::BuiltinViseDialog(String::new()));
    let mut h = dialog(state);
    h.run();
    for label in ["Add built-in vise", "3 in", "8 in", "100 mm", "200 mm", "Jaw height", "Max opening", "Place"] {
        assert!(h.query_by_label(label).is_some(), "the dialog shows {label}");
    }
    let Some(FixtureLibDialog::Vise(d)) = &h.state().fixture_lib else { panic!("vise dialog open") };
    assert_eq!(d.params.size, "150 mm", "a mm document starts on 150 mm");
    assert!(d.fit_stock, "fitting to the stock is the default");
    h.get_by_label("6 in").click();
    h.run();
    let Some(FixtureLibDialog::Vise(d)) = &h.state().fixture_lib else { panic!("still open") };
    assert_eq!(d.params.size, "6 in");
    h.get_by_label("Place").click();
    h.run();
    h.run();
    assert!(h.state().fixture_lib.is_none(), "placed and closed: {:?}", h.state().fixture_lib);
    assert_eq!(opening(h.state(), "Vise 6 in"), Some(40.0), "jaws open to the 40 mm stock");
    let st = h.state_mut();
    assert!(st.history.undo(&st.world));
    assert!(model::find_by_name(&st.world, "Vise 6 in").is_none(), "one undo takes the vise");
}

#[test]
fn a_refused_vise_is_named_in_the_dialog() {
    let mut state = State::new(); // no stock to fit to
    state.pending.push_back(CommandKind::BuiltinViseDialog(String::new()));
    let mut h = dialog(state);
    h.run();
    h.get_by_label("Place").click();
    h.run();
    h.run();
    let Some(FixtureLibDialog::Vise(d)) = &h.state().fixture_lib else { panic!("stays open") };
    assert!(d.error.as_deref().is_some_and(|e| e.contains("no stock")), "{:?}", d.error);
    assert!(h.query_by_label_contains("no stock").is_some(), "said in the dialog");
}

#[test]
fn a_vise_row_sets_its_opening_and_opens_to_the_stock() {
    let mut state = with_stock();
    state.pending.push_back(CommandKind::BuiltinViseDialog(String::new()));
    state.drain_actions();
    let Some(FixtureLibDialog::Vise(d)) = &state.fixture_lib else { panic!("open") };
    let cmd = super::fixture_vise_dialog::place_command(d);
    state.pending.push_back(cmd);
    state.drain_actions();
    let name = "Vise 150 mm";
    assert_eq!(opening(&state, name), Some(40.0));

    let mut h = Harness::builder().with_size(Vec2::new(600.0, 600.0)).build_ui_state(
        |ui, st: &mut State| {
            st.drain_actions();
            super::pages::fixtures_section(ui, st, None);
        },
        state,
    );
    h.run();
    assert!(h.query_by_label("Jaw opening").is_some(), "the row offers the opening");
    h.state_mut().vise_opening = Some((name.to_owned(), 90.0));
    h.get_by_label("Set opening").click();
    h.run();
    h.run();
    assert_eq!(opening(h.state(), name), Some(90.0), "Set lands through SetVise");
    h.get_by_label("Open to stock").click();
    h.run();
    h.run();
    assert_eq!(opening(h.state(), name), Some(40.0), "back against the stock");
    let st = h.state_mut();
    assert!(st.history.undo(&st.world));
    assert_eq!(opening(st, name), Some(90.0), "each edit is one undo");
}
