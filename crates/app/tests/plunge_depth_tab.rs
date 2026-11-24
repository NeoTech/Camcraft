//! Issue #3679: the verifier names a feed plunge that reaches deeper
//! below the floor the levels above left under the full tool disc than
//! one stepdown. The catalog case `2d/contour-tabs` (the reporter's
//! tabbed contour of #3453) carried such dives at its last level's tab
//! windows. #3678 (PR #3866) made each of those dives peck one stepdown
//! at a time, so the verifier names none now (#3920). This test pins
//! both halves: no finding, and the pecks in the path that make it so.
//!
//! Gate map (#3679): covers app::operation (generate, cutting_rows), app::model,
//! app::project, sim::verify.
//! Provenance: clean-room test over the catalog case of #3453.

use camcraft_app::commands::{GenerateOperation, History};
use camcraft_app::operation::OpStatus;
use camcraft_app::verify::Warning;
use camcraft_app::cl::MoveKind;
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

    // The verifier names no plunge that is too deep: #3678 pecks the dives.
    let dives: Vec<&Warning> =
        tp.warnings.iter().filter(|warn| matches!(warn, Warning::PlungeTooDeep { .. })).collect();
    assert!(dives.is_empty(), "no tab window dive is named after the #3678 peck: {dives:?}");

    // The path itself: a dive is the feed plunges at one XY with only
    // rapids between them. Tab windows 1 to 3 start where no level above
    // cut, so each dive pecks Z4, Z2, Z0 from the Z6 stock top, never more
    // than the op's 2 mm stepdown below the depth before it. Window 0
    // enters where the levels above cut and needs one plunge per level.
    let mut pecked: Vec<Vec<f64>> = Vec::new();
    let mut dive: Vec<f64> = Vec::new();
    let mut at: Option<(f64, f64)> = None;
    for m in &tp.path.moves {
        let still = m.from.x == m.to.x && m.from.y == m.to.y;
        let here = at == Some((m.to.x, m.to.y));
        if m.kind == MoveKind::Rapid && still && here {
            continue;
        }
        if m.kind != MoveKind::Rapid && still && m.to.z < m.from.z {
            if !here {
                if dive.len() > 1 {
                    pecked.push(std::mem::take(&mut dive));
                }
                dive.clear();
                at = Some((m.to.x, m.to.y));
            }
            dive.push(m.to.z);
            continue;
        }
        if dive.len() > 1 {
            pecked.push(std::mem::take(&mut dive));
        }
        dive.clear();
        at = None;
    }
    assert_eq!(pecked.len(), 3, "tab windows 1 to 3 each peck their dive: {pecked:?}");
    for depths in &pecked {
        assert_eq!(depths, &[4.0, 2.0, 0.0], "a pecked dive steps Z4, Z2, Z0: {depths:?}");
        let mut floor = 6.0_f64;
        for &z in depths {
            assert!(floor - z <= 2.0 + 1e-9, "a peck feeds {} mm below the depth before it: {depths:?}", floor - z);
            floor = floor.min(z);
        }
    }
}
