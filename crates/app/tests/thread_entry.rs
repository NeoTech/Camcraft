//! A thread mill's way in (#992): a thread mill is not centre-cutting
//! and the ride feeds it down the hole's axis, so a Thread operation
//! whose hole the program ahead has not opened - nothing drilled, a
//! drill too narrow or too shallow, or a Drill moved after it - is
//! refused by name at generate; the same thread after a tap drill
//! generates, verifies clean and posts.
//!
//! Provenance: public handbook thread milling (tap-drill first, to the
//! minor diameter, deeper than the thread); our own fixtures.
//!
//! Gate map (M4.4): covers app::operation (thread_entry, thread_op,
//! drill_prior, stamp), app::rest (prior_plan, prior_digest),
//! app::commands (GenerateOperation, ReorderOperation), app::posting.
//! Lane shape (M4.4): all tests here run in seconds.

mod common;
mod thread_support;

use flecs_ecs::prelude::World;

use camcraft_app::cl::ThreadHand;
use camcraft_app::commands::{CreateOperation, GenerateOperation, History, ReorderOperation};
use camcraft_app::model;
use camcraft_app::operation::{self, OpStatus, SrcRef};
use camcraft_app::posting;

use thread_support::*;

fn multi_world() -> (World, History) {
    bare_world(R_M8, &[(MULTI, catalog_tool(MULTI))])
}

/// Create "Thread1" (M8 x 1.25, multi-form) and [`generate`] it.
fn try_thread(world: &World, history: &mut History) -> String {
    let params = thread(R_M8, None, ThreadHand::Right, true);
    let op = CreateOperation::new(world, "Thread1", MULTI, SrcRef::Entity("disc".into()), params);
    history.execute(world, op.expect("an undrilled thread is a valid operation to make"));
    generate(world, history)
}

/// Generate "Thread1": the refusal's words, or empty when it generated.
fn generate(world: &World, history: &mut History) -> String {
    match GenerateOperation::new(world, "Thread1") {
        Ok(cmd) => {
            history.execute(world, cmd);
            String::new()
        }
        Err(e) => e.to_string(),
    }
}

fn status(world: &World, name: &str) -> OpStatus {
    operation::status_of(world, &model::find_by_name(world, name).expect("op"))
}

#[test]
fn a_thread_with_nothing_drilled_ahead_is_refused_by_name() {
    let (world, mut history) = multi_world();
    let msg = try_thread(&world, &mut history);
    assert!(msg.contains("the hole at (0.000, 0.000) is not drilled yet"), "{msg}");
    assert!(msg.contains("add a drilling operation of at least the 6.647 mm minor diameter"), "{msg}");
    assert_eq!(status(&world, "Thread1"), OpStatus::Never, "no path is cached for a refused thread");
    assert!(!posting::verification_gate(&world).is_empty(), "nothing posts");
}

#[test]
fn a_tap_drill_ahead_opens_the_hole_and_the_thread_verifies_clean() {
    let (world, mut history) = multi_world();
    drill_ahead(&world, &mut history, tap_drill_for(R_M8), DRILL_FLOOR);
    assert_eq!(try_thread(&world, &mut history), "", "the drilled thread generates");
    assert_eq!(status(&world, "Thread1"), OpStatus::Ok);
    assert_clean(&world);
}

#[test]
fn a_drill_of_the_mills_own_diameter_opens_the_hole_too() {
    // The issue's lower bound: a Ø6 drill under a Ø6 mill in an M8 hole - the grid reads the mill's own
    // cylinder (the minor circle less one cell is narrower here), and it is air.
    let (world, mut history) = multi_world();
    drill_ahead(&world, &mut history, 6.0, DRILL_FLOOR);
    assert_eq!(try_thread(&world, &mut history), "");
}

#[test]
fn a_drill_too_narrow_or_too_shallow_is_refused() {
    // Ø5 under a Ø6 mill: the mill's rim lands on the ring the drill left.
    let (world, mut history) = multi_world();
    drill_ahead(&world, &mut history, 5.0, DRILL_FLOOR);
    let msg = try_thread(&world, &mut history);
    assert!(msg.contains("not drilled yet") && msg.contains("stands at z 20.000"), "{msg}");
    // The tap drill's point ends above the thread's floor: its cone stands in the mill's way.
    let (world, mut history) = multi_world();
    drill_ahead(&world, &mut history, tap_drill_for(R_M8), FLOOR + 1.0);
    let msg = try_thread(&world, &mut history);
    assert!(msg.contains("not drilled yet") && msg.contains("above the thread mill's lowest point at z 8.000"), "{msg}");
}

#[test]
fn moving_the_drill_after_the_thread_marks_it_stale_and_refuses_it() {
    let (world, mut history) = multi_world();
    drill_ahead(&world, &mut history, tap_drill_for(R_M8), DRILL_FLOOR);
    assert_eq!(try_thread(&world, &mut history), "");
    assert_eq!(status(&world, "Thread1"), OpStatus::Ok);
    history.execute(&world, ReorderOperation::new(&world, "Drill1", "Thread1", true).expect("reorder"));
    assert_eq!(status(&world, "Thread1"), OpStatus::Stale, "the program ahead of the thread changed");
    assert!(!posting::verification_gate(&world).is_empty(), "a stale thread does not post");
    let msg = generate(&world, &mut history);
    assert!(msg.contains("not drilled yet"), "{msg}");
    // Undo puts the drill back ahead: the cached thread is current again.
    assert!(history.undo(&world));
    assert_eq!(status(&world, "Thread1"), OpStatus::Ok);
}
