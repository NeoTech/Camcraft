//! #2257: a generation split for a background worker. Staging then
//! finishing gives the same command as `GenerateOperation::new`, and a
//! finished result knows when its operation changed since it was staged
//! (an edit, an undo of a generation, a delete), so the UI never writes
//! old parameters over newer ones.
//!
//! Gate map: covers app::commands::operation (generate).
//! Provenance: our own tests over our own commands; no HSMWorks material.

mod area_support;
mod common;

use area_support::*;
use camcraft_app::commands::{DeleteOperation, GenerateOperation, StagedGeneration};

const OP: &str = "Face1";

fn world_with_face() -> (World, History) {
    let world = block_world();
    let mut h = History::default();
    h.execute(&world, CreateOperation::new(&world, OP, TOOL, SrcRef::Stock, face(9.0)).expect("create"));
    (world, h)
}

fn path_of(world: &World) -> Option<camcraft_app::schema::Toolpath> {
    model::find_by_name(world, OP).and_then(|e| model::toolpath_of(world, &e))
}

#[test]
fn staged_then_finished_lands_what_the_one_shot_command_lands() {
    let (world, mut h) = world_with_face();
    let staged = StagedGeneration::new(&world, OP).expect("stages");
    assert_eq!(staged.name(), OP);
    let done = staged.finish().expect("finishes");
    assert_eq!(done.name(), OP);
    assert!(done.still_current(&world));
    h.execute(&world, done);
    let a = path_of(&world).expect("a path");
    assert!(h.undo(&world));
    h.execute(&world, GenerateOperation::new(&world, OP).expect("one shot"));
    assert_eq!(path_of(&world).expect("a path"), a, "the same path either way");
}

#[test]
fn a_result_is_stale_after_an_edit_a_generation_or_a_delete() {
    let (world, mut h) = world_with_face();
    let done = StagedGeneration::new(&world, OP).and_then(StagedGeneration::finish).expect("generates");
    // An edit of the parameters.
    let p = face(8.0);
    h.execute(&world, EditOperationParams::new(&world, OP, p).expect("edit"));
    assert!(!done.still_current(&world), "an edit makes it stale");
    assert!(h.undo(&world));
    assert!(done.still_current(&world), "undoing the edit restores the staged state");
    // Another generation landed meanwhile: the cache moved.
    h.execute(&world, GenerateOperation::new(&world, OP).expect("generate"));
    assert!(!done.still_current(&world), "a newer cache makes it stale");
    assert!(h.undo(&world));
    // The operation is gone.
    h.execute(&world, DeleteOperation::new(&world, OP).expect("delete"));
    assert!(!done.still_current(&world), "a deleted operation takes nothing");
}

/// #3035: staging a 3D Pocket after a cut reads the world only. The
/// stock simulation of the program ahead runs in `finish`, on the
/// worker's thread, so an MCP session or the GUI stays free while it
/// runs. The finished path is the path the one-shot command lands.
#[test]
fn a_rest_reading_pocket_simulates_the_program_ahead_in_its_job_not_at_staging() {
    let (world, mut h) = world_with_face();
    h.execute(&world, GenerateOperation::new(&world, OP).expect("the face cuts"));
    let p3d = operation::retype_params(&pocket(10.0, 5.0), OpType::Pocket3D as usize);
    let src = SrcRef::Entity("part".into());
    h.execute(&world, CreateOperation::new(&world, "Rough3D", TOOL, src, p3d).expect("create"));

    let before = camcraft_app::rest::prior_simulations();
    let staged = StagedGeneration::new(&world, "Rough3D").expect("stages");
    assert_eq!(camcraft_app::rest::prior_simulations(), before, "staging simulates nothing");
    let (done, simulated) = std::thread::spawn(move || {
        let n = camcraft_app::rest::prior_simulations();
        let done = staged.finish();
        (done, camcraft_app::rest::prior_simulations() - n)
    })
    .join()
    .expect("the worker thread");
    assert_eq!(simulated, 1, "the job simulates the face ahead once");
    let done = done.expect("finishes");
    assert!(done.still_current(&world));
    h.execute(&world, done);
    let staged_path = model::find_by_name(&world, "Rough3D").and_then(|e| model::toolpath_of(&world, &e)).expect("a path");
    assert!(h.undo(&world));
    h.execute(&world, GenerateOperation::new(&world, "Rough3D").expect("one shot"));
    let one_shot = model::find_by_name(&world, "Rough3D").and_then(|e| model::toolpath_of(&world, &e)).expect("a path");
    assert_eq!(staged_path, one_shot, "the same path either way");
}
