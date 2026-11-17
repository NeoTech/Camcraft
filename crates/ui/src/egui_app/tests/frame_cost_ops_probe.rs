//! #2201: the frame-cost probe over a growing operation list. The same
//! still frames as `frame_cost_probe`, on fig8.step with one and then
//! eight generated facing passes, one selected, then "Show all
//! toolpaths": the pieces that grow with the operation count, and the
//! pieces a selected toolpath adds, print by name (`--nocapture`).
//! Debug asserts counts, never times.
//! Provenance: our own frame-cost law over our own shell; no vendor
//! material consulted.

use camcraft_app::operation::{self, OpType};

use super::frame_cost_probe::{N, calls, create, fig8, report, run, still};
use super::*;

fn face(k: usize) -> String {
    format!("Probe face {k}")
}

/// Create and generate facing passes `from..to`.
fn faces(h: &mut Harness<'static, ShellUi>, from: usize, to: usize) {
    for k in from..to {
        create(h, &face(k), OpType::Face, Vec::new());
        run(h, |st| st.pending.push_back(CommandKind::GenerateOperation(face(k))));
        let has = model::find_by_name(&h.state().state.world, &face(k))
            .is_some_and(|e| model::has_toolpath(&h.state().state.world, &e));
        assert!(has, "{} generates: {}", face(k), h.state().state.status);
    }
}

/// Select `name` in the world and open its page, as a browser click does.
fn select(h: &mut Harness<'static, ShellUi>, name: &str) {
    run(h, |st| {
        st.pending.push_back(CommandKind::Select(Some(name.to_owned())));
        crate::actions::target(st, UiTarget::Operation(name.to_owned()));
    });
}

/// #2201 pins: still frames (the UI pass and the scene) build no toolpath
/// lines, no solid box and no operation status after the first.
fn builds_nothing(h: &mut Harness<'static, ShellUi>, name: &str) {
    let lines = crate::path_line_memo::path_line_builds();
    let boxes = crate::solid_aabb_memo::solid_aabb_builds();
    let statuses = operation::status_builds();
    for _ in 0..3 {
        std::hint::black_box(world::collect(&h.state().state));
        h.step();
    }
    assert_eq!(crate::path_line_memo::path_line_builds() - lines, 0, "{name}: no toolpath line build on a still frame");
    assert_eq!(crate::solid_aabb_memo::solid_aabb_builds() - boxes, 0, "{name}: no solid box build on a still frame");
    assert_eq!(operation::status_builds() - statuses, 0, "{name}: no operation status computed on a still frame");
}

#[test]
fn still_frames_with_many_toolpaths_name_their_costs() {
    let mut h = fig8();

    // (f1) One generated facing pass, selected.
    faces(&mut h, 0, 1);
    select(&mut h, &face(0));
    let f1 = still(&mut h);
    builds_nothing(&mut h, "f1");
    report("f1 one toolpath, selected", &f1);

    // (f8) Eight generated facing passes, the first selected.
    faces(&mut h, 1, 8);
    select(&mut h, &face(0));
    let f8 = still(&mut h);
    builds_nothing(&mut h, "f8");
    report("f8 eight toolpaths, one selected", &f8);

    // (g8) The same, with "Show all toolpaths" on.
    run(&mut h, |st| st.show_all_paths = true);
    let g8 = still(&mut h);
    builds_nothing(&mut h, "g8");
    report("g8 eight toolpaths, all shown", &g8);

    // (n8) Nothing selected: no toolpath draws.
    run(&mut h, |st| {
        st.show_all_paths = false;
        st.pending.push_back(CommandKind::Select(None));
        crate::actions::target(st, UiTarget::None);
    });
    let n8 = still(&mut h);
    builds_nothing(&mut h, "n8");
    report("n8 eight toolpaths, none selected", &n8);

    for (name, p) in [("f1", &f1), ("f8", &f8), ("g8", &g8), ("n8", &n8)] {
        assert_eq!(calls(p, "scene/collect"), N, "{name}");
        assert_eq!((p.delta.walks, p.delta.digests), (0, 0), "{name}: no chain walk, no digest on a still frame");
        assert_eq!(p.delta.statuses, 0, "{name}: no operation status computed on a still frame");
    }
}
