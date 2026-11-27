//! The face names of a `.sc` model in the document (#3915): the import
//! lands SolidCraft's name of each face beside the facets, in memory only;
//! STL, OBJ and STEP models have none; an update trades them on undo and
//! drops them when it does not state them; a repair drops them; a delete
//! brings them back on undo; and names that do not describe the facets
//! that stand are never handed out.
//! Provenance: our own tests. The `.sc` file is SolidCraft 0.46.2's
//! testdata/models/hole_on_pocket_wall.sc (format 10), which the kernel
//! pins read too.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use camcraft_app::commands::{DeleteEntity, History, ImportModel, RepairModel, UpdateModel};
use camcraft_app::geometry::source_faces::{FaceName, SourceFaces};
use camcraft_app::geometry::{Mesh, Point3, cube};
use camcraft_app::model;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/models")
        .join(name)
}

/// The names now handed out for model `name`.
fn names(world: &World, name: &str) -> Option<Arc<SourceFaces>> {
    model::find_by_name(world, name).and_then(|e| camcraft_app::source_faces::of(world, &e))
}

fn facet_count(world: &World, name: &str) -> usize {
    model::find_by_name(world, name)
        .and_then(|e| model::facets_of(world, &e))
        .map_or(0, |f| f.len())
}

/// The unit cube without its bottom: 10 facets and a 4-edge hole.
fn holed_cube() -> Mesh {
    Mesh::from_facets(cube(Point3::ZERO, Point3::new(1.0, 1.0, 1.0)).facets[2..].to_vec())
}

/// Names of our own over `facets` facets, two facets for each face.
fn record(facets: usize) -> Arc<SourceFaces> {
    let tri_face: Vec<u32> = (0..facets)
        .map(|i| u32::try_from(i / 2).unwrap_or(u32::MAX))
        .collect();
    let names = (0..facets.div_ceil(2))
        .map(|f| FaceName {
            name: format!("F1:side:l{f}"),
            verified: true,
            former: None,
        })
        .collect();
    match SourceFaces::from_tri_face(&tri_face, names) {
        Some(r) => Arc::new(r),
        None => panic!("every face is named"),
    }
}

/// A model `name` of `mesh` landed with `faces`.
fn land(world: &World, h: &mut History, name: &str, mesh: Mesh, faces: Arc<SourceFaces>) {
    let mut cmd = ImportModel::from_mesh(name, mesh);
    cmd.source_faces = Some(faces);
    h.execute(world, cmd);
}

#[test]
fn a_solidcraft_import_lands_its_face_names() {
    let cmd = ImportModel::new(&testdata("hole_on_pocket_wall.sc")).expect("the design imports");
    let held = match &cmd.source_faces {
        Some(r) => Arc::clone(r),
        None => panic!("a format-10 design carries its names: {:?}", cmd.notes),
    };
    assert_eq!(held.names.len(), 13);
    assert!(held.tiles(cmd.mesh.facets.len()));
    assert!(
        !cmd.notes.iter().any(|n| n.contains("face names")),
        "{:?}",
        cmd.notes
    );
    let name = cmd.name.clone();
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, cmd);
    let landed = names(&world, &name).expect("the names land with the facets");
    assert!(Arc::ptr_eq(&landed, &held));
    assert!(landed.tiles(facet_count(&world, &name)));
    assert!(h.undo(&world));
    assert!(model::find_by_name(&world, &name).is_none());
    assert!(h.redo(&world));
    assert!(
        names(&world, &name).is_some_and(|r| Arc::ptr_eq(&r, &held)),
        "redo lands them again"
    );
}

#[test]
fn stl_obj_and_step_models_have_no_face_names() {
    let world = new_world();
    let mut h = History::default();
    for file in ["cube.stl", "box.obj", "box.step"] {
        let cmd = ImportModel::new(&testdata(file)).expect("the model imports");
        assert!(cmd.source_faces.is_none(), "{file}");
        let name = cmd.name.clone();
        h.execute(&world, cmd);
        assert!(names(&world, &name).is_none(), "{file}");
    }
}

#[test]
fn an_update_trades_the_names_on_undo_and_drops_them_unstated() {
    let world = new_world();
    let mut h = History::default();
    let faces = record(10);
    land(&world, &mut h, "part", holed_cube(), Arc::clone(&faces));
    assert!(names(&world, "part").is_some());
    // An update from a file that is not `.sc` states no names: dropped.
    let other = cube(Point3::ZERO, Point3::new(2.0, 2.0, 2.0));
    h.execute(
        &world,
        UpdateModel::new(&world, "part", other).expect("update"),
    );
    assert!(
        names(&world, "part").is_none(),
        "an unstated update drops the names"
    );
    assert!(h.undo(&world));
    assert!(
        names(&world, "part").is_some_and(|r| Arc::ptr_eq(&r, &faces)),
        "undo puts them back"
    );
    assert!(h.redo(&world));
    assert!(names(&world, "part").is_none());
    // An update that states names of its own facets lands them.
    let again = record(10);
    h.execute(
        &world,
        UpdateModel::new(&world, "part", holed_cube())
            .expect("update")
            .with_source_faces(Some(Arc::clone(&again))),
    );
    assert!(names(&world, "part").is_some_and(|r| Arc::ptr_eq(&r, &again)));
    assert!(h.undo(&world));
    assert!(
        names(&world, "part").is_none(),
        "undo puts back the update's own state: none"
    );
}

#[test]
fn a_repair_drops_the_names_and_undo_brings_them_back() {
    let world = new_world();
    let mut h = History::default();
    let faces = record(10);
    land(&world, &mut h, "part", holed_cube(), Arc::clone(&faces));
    h.execute(
        &world,
        RepairModel::new(&world, "part", 0.001, 256).expect("repairs"),
    );
    assert_eq!(facet_count(&world, "part"), 12);
    assert!(
        names(&world, "part").is_none(),
        "the repaired facets have no names"
    );
    assert!(h.undo(&world));
    assert!(names(&world, "part").is_some_and(|r| Arc::ptr_eq(&r, &faces)));
    assert!(h.redo(&world));
    assert!(names(&world, "part").is_none());
}

#[test]
fn a_deleted_model_comes_back_with_its_names() {
    let world = new_world();
    let mut h = History::default();
    let faces = record(10);
    land(&world, &mut h, "part", holed_cube(), Arc::clone(&faces));
    h.execute(&world, DeleteEntity::new(&world, "part").expect("delete"));
    assert!(model::find_by_name(&world, "part").is_none());
    assert!(h.undo(&world));
    assert!(names(&world, "part").is_some_and(|r| Arc::ptr_eq(&r, &faces)));
}

#[test]
fn names_that_do_not_describe_the_facets_are_never_handed_out() {
    let world = new_world();
    let mut h = History::default();
    // Ranges for 10 facets on a 12-facet model: never landed.
    land(
        &world,
        &mut h,
        "short",
        cube(Point3::ZERO, Point3::new(1.0, 1.0, 1.0)),
        record(10),
    );
    assert!(names(&world, "short").is_none());
    // Names landed, then other facets landed outside the import commands:
    // the revision moved, so the names are stale.
    land(&world, &mut h, "part", holed_cube(), record(10));
    let e = model::find_by_name(&world, "part").expect("landed");
    assert!(names(&world, "part").is_some());
    model::set_facets(&world, &e, holed_cube().facets);
    assert_eq!(facet_count(&world, "part"), 10, "the same count");
    assert!(
        names(&world, "part").is_none(),
        "a new landing makes them stale"
    );
}
