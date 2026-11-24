//! Issue #3679: the verifier names a feed plunge that reaches deeper
//! below the floor the levels above left under the full tool disc than
//! one stepdown. The catalog case `2d/contour-tabs` (the reporter's
//! tabbed contour of #3453) carries such dives at its last level's tab
//! windows: after the law, its operation names them, advisory by the
//! #3525 pattern - nothing gouges and nothing collides, the risk is the
//! tool load, so the post is not held. #3678 owns the generator fix
//! (each window dive pecks or enters in the kerf); when it lands, these
//! findings go and this test changes on purpose with it.
//!
//! Gate map (#3679): covers app::operation (generate, cutting_rows), app::model,
//! app::project, sim::verify.
//! Provenance: clean-room test over the catalog case of #3453.

use camcraft_app::commands::{GenerateOperation, History};
use camcraft_app::operation::OpStatus;
use camcraft_app::verify::Warning;
use camcraft_app::{model, operation, project, setup, world};

fn repo(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

#[test]
fn the_tabbed_contour_catalog_case_names_its_tab_window_dives() {
    let w = world::new_world();
    let proj = repo("examples/catalog/2d/contour-tabs/contour-tabs.camproj");
    let notes = project::load_noted(&w, &proj).unwrap_or_else(|e| panic!("load {}: {e}", proj.display()));
    assert!(notes.is_empty(), "load notes: {notes:?}");
    let ops = setup::operation_order(&w);
    assert_eq!(ops.len(), 1, "the case holds one contour: {ops:?}");
    let mut history = History::default();
    for (op_name, _) in &ops {
        let cmd = GenerateOperation::new(&w, op_name).unwrap_or_else(|e| panic!("generate {op_name}: {e}"));
        history.execute(&w, cmd);
    }
    let (_, entity) = ops.into_iter().next().expect("one op");
    assert_eq!(operation::status_of(&w, &entity), OpStatus::Ok, "status after regenerate");
    let tp = model::toolpath_of(&w, &entity).expect("cached path");

    // The last level's three later tab windows each dive at a lead-in
    // start that no level above cut: 6 mm below the Z 6 stock top
    // against the op's 2 mm stepdown. Window 0 enters where the levels
    // above cut (one stepdown below their floor) and stays clean.
    let dives: Vec<(usize, f64, f64)> = tp
        .warnings
        .iter()
        .filter_map(|warn| match warn {
            Warning::PlungeTooDeep { at, mm, limit_mm } => Some((*at, *mm, *limit_mm)),
            _ => None,
        })
        .collect();
    assert_eq!(dives.len(), 3, "three tab window dives named: {:?}", tp.warnings);
    for (at, mm, limit_mm) in &dives {
        let m = tp.path.moves.get(*at)
            .unwrap_or_else(|| panic!("finding at move {at} names no move of a {}-move path", tp.path.moves.len()));
        assert_eq!(m.to.z, 0.0, "the dive at move {at} must land on the last level");
        assert!((mm - 6.0).abs() < 1e-9, "the dive at move {at} reads 6 mm below the stock top, not {mm}");
        assert!((limit_mm - 2.0).abs() < 1e-12, "the op's stepdown is 2 mm, not {limit_mm}");
        assert!(
            !Warning::PlungeTooDeep { at: *at, mm: *mm, limit_mm: *limit_mm }.blocks_post(),
            "a tool-load note does not hold the post"
        );
    }
    let ats: Vec<usize> = dives.iter().map(|d| d.0).collect();
    assert_eq!(ats.len(), 3, "one finding per dive move, not {ats:?}");
    assert!(ats[0] != ats[1] && ats[0] != ats[2] && ats[1] != ats[2], "each dive is named once: {ats:?}");
}
