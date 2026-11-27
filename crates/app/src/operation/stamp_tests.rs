//! The generator-output rule at the salt (#3704): a change of generator
//! output that fixes a defect bumps `GENERATOR_OUTPUT_VERSION`, and every
//! cache a project saved before the bump reads Stale instead of posting the
//! old move. The tabbed-contour case is the one PR #3684 changed: its
//! first tab window used to dive 18 mm into uncut sheet.
//!
//! Provenance: our own staleness design (see `stamp`).

use super::*;

use crate::commands::{AddTool, GenerateOperation, History, SetStock};
use crate::schema::Location;
use crate::stock::StockShape;
use crate::tools::{Tool, catalog};

fn catalog_tool(name: &str) -> Tool {
    catalog()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| t)
        .expect("catalog entry")
}

fn repo(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

/// The stamp at the version before the written one: what a document saved
/// before a bump carries.
fn stamp_before(
    world: &World,
    tool_name: &str,
    src: &SrcRef,
    params: &OpParams,
    op_name: &str,
    setup: Option<&str>,
) -> GenStamp {
    stamp_at(
        world,
        tool_name,
        src,
        params,
        op_name,
        setup,
        GENERATOR_OUTPUT_VERSION.checked_sub(1).expect("the version starts at 1"),
    )
    .expect("the old version digests")
}

/// The catalog tabbed contour of #3453, loaded and generated: the case
/// whose first window PR #3684 moved.
fn generated_tabbed_case() -> (World, String, flecs_ecs::prelude::Entity) {
    let world = crate::world::new_world();
    let proj = repo("examples/catalog/2d/contour-tabs/contour-tabs.camproj");
    let notes =
        crate::project::load_noted(&world, &proj).unwrap_or_else(|e| panic!("load {}: {e}", proj.display()));
    assert!(notes.is_empty(), "load notes: {notes:?}");
    let ops = crate::setup::operation_order(&world);
    assert_eq!(ops.len(), 1, "the case holds one contour: {ops:?}");
    let (name, entity) = ops.into_iter().next().expect("one op");
    let mut h = History::default();
    h.execute(&world, GenerateOperation::new(&world, &name).expect("generate"));
    assert_eq!(status_of(&world, &entity), OpStatus::Ok, "the regenerated cache is current");
    (world, name, entity)
}

#[test]
fn a_bump_of_the_generator_output_version_moves_the_stamp() {
    let world = crate::world::new_world();
    let mut h = History::default();
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default())
            .expect("stock"),
    );
    h.execute(&world, AddTool::new(&world, "EM-0800-4FL", catalog_tool("EM-0800-4FL")).expect("tool"));
    let params = crate::operation::seed_params(crate::operation::OpType::Contour.ordinal());
    let now = GENERATOR_OUTPUT_VERSION;
    let before = now.checked_sub(1).expect("the version starts at 1");
    let at = |v: u64| {
        stamp_at(&world, "EM-0800-4FL", &SrcRef::Stock, &params, "X", None, v)
            .expect("the world digests a stamp")
    };
    assert_eq!(at(now), at(now), "an unchanged version digests the same stamp");
    assert_ne!(at(now), at(before), "a bump moves every stamp made under the old number");
    assert_eq!(
        current_stamp(&world, "EM-0800-4FL", &SrcRef::Stock, &params, "X", None),
        Some(at(now)),
        "current_stamp stamps at the written version"
    );
}

#[test]
fn an_unchanged_generator_output_version_keeps_a_saved_cache_current() {
    let (world, name, _entity) = generated_tabbed_case();
    let path =
        std::env::temp_dir().join(format!("camcraft-3704-unchanged-{}.camproj", std::process::id()));
    let _ = std::fs::remove_file(&path);
    crate::project::save(&world, &path).expect("save");
    let back = crate::world::new_world();
    crate::project::load(&back, &path).expect("load");
    let _ = std::fs::remove_file(&path);
    let e = crate::model::find_by_name(&back, &name).expect("the op loads");
    assert_eq!(
        status_of(&back, &e),
        OpStatus::Ok,
        "no churn: the cache saved at the written version is current, nothing regenerates"
    );
    let op = crate::model::operation_of(&back, &e).expect("op");
    assert_eq!(
        op.stamp,
        current_stamp(&back, &op.tool, &op.src, &op.params, &name, op.setup.as_deref()),
        "the saved stamp is the stamp the unchanged version digests"
    );
}

#[test]
fn a_tabbed_contour_saved_before_the_bump_reads_stale() {
    let (world, name, entity) = generated_tabbed_case();
    let op = crate::model::operation_of(&world, &entity).expect("op");
    let current = op.stamp.expect("generated").0;
    let old = stamp_before(&world, &op.tool, &op.src, &op.params, &name, op.setup.as_deref()).0;
    assert_ne!(current, old, "the bump separates a saved cache from a fresh one");

    let path =
        std::env::temp_dir().join(format!("camcraft-3704-pre-bump-{}.camproj", std::process::id()));
    let _ = std::fs::remove_file(&path);
    crate::project::save(&world, &path).expect("save");
    let text = std::fs::read_to_string(&path).expect("read");
    let from = format!("\"stamp\": {current}");
    assert_eq!(text.matches(&from).count(), 1, "one stamp on the contour: {text}");
    std::fs::write(&path, text.replace(&from, &format!("\"stamp\": {old}"))).expect("rewrite");

    let back = crate::world::new_world();
    crate::project::load(&back, &path).expect("load the pre-bump file");
    let _ = std::fs::remove_file(&path);
    let e = crate::model::find_by_name(&back, &name).expect("the op loads");
    assert_eq!(
        status_of(&back, &e),
        OpStatus::Stale,
        "the cache saved before the bump reads stale, not current - the old 18 mm plunge can never post"
    );
    assert_eq!(
        stale_reason(&back, &e).as_deref(),
        Some("the geometry or tool inputs changed since it was generated"),
        "the general reason: no own input moved"
    );
}
