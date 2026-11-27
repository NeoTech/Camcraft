//! #4055 pins of the one planner switch (`CAMCRAFT_PLANNER`): the default
//! is the old planner, and the output of a phase-1 kind does not move with
//! it; "new" refuses a kind that the path kernel does not plan yet, by name.
//! The golden is the #3750 baseline (recorded on origin/main 138a592a1).

use super::{PassStart, fig8_world, offset_op_entered};
use super::Entry3D;
use crate::common::{gen_to_cltext, write_golden};
use camcraft_app::pathkernel::{PLANNER_ENV, PlannerEngine, engine_from_switch};
use camcraft_app::operation::{KERNEL_KINDS, OpType, PHASE1_KINDS, check_engine, planner_engine};

/// The process of this test run has no switch set: the gate never sets it.
fn assert_default_process() {
    assert!(std::env::var_os(PLANNER_ENV).is_none(), "{PLANNER_ENV} is set: unset it to run the default pins");
    assert_eq!(planner_engine(), PlannerEngine::Old);
}

#[test]
fn switch_default_passes_each_phase1_kind_to_the_old_planner() {
    assert_eq!(engine_from_switch(None).0, PlannerEngine::Old);
    assert_default_process();
    assert_eq!(PHASE1_KINDS.len(), 16);
    for kind in PHASE1_KINDS {
        assert_eq!(check_engine(kind, PlannerEngine::Old), Ok(()), "{kind}");
    }
}

#[test]
fn switch_new_refuses_a_kind_it_does_not_plan_yet() {
    for kind in [OpType::Offset2D, OpType::Pocket3D] {
        let e = check_engine(kind, PlannerEngine::New).expect_err("refused by name, never a silent fallback");
        let text = e.to_string();
        assert!(text.contains(&format!("the new path planner does not plan {kind} yet")), "{text}");
        assert!(text.contains(PLANNER_ENV), "{text}");
    }
    // #3752: the 2D Pocket is the first kind that the kernel plans.
    assert_eq!(KERNEL_KINDS, [OpType::Pocket]);
    assert_eq!(check_engine(OpType::Pocket, PlannerEngine::New), Ok(()));
    // A kind outside phase 1 keeps its own planner under either value.
    assert_eq!(check_engine(OpType::Drill, PlannerEngine::New), Ok(()));
}

/// The golden of the #3750 baseline: the 2D Offset Inside with a Helix
/// entry on `fig8.step`, generated through the dispatch with the switch at
/// its default. The 2D Pocket rings are pinned against their own baseline
/// goldens in motion (`pass_start_levels::shared_planner`).
#[test]
#[ignore = "heavy lane (#4055 fig8 Inside): one generation and its verify run past the 60 s debug budget - run with --release -- --ignored"]
fn switch_default_is_the_old_planner() {
    assert_default_process();
    let (world, mut h, part, stock) = fig8_world();
    let (path, warnings) = offset_op_entered(&world, &mut h, &part, stock.0.z, PassStart::Inside, Entry3D::Helix);
    let mut text = gen_to_cltext(&path);
    for n in &path.notes {
        text.push_str(&format!("; note: {}\n", n.replace('\n', " ")));
    }
    for w in &warnings {
        text.push_str(&format!("; warning: {}\n", w.replace('\n', " ")));
    }
    write_golden("offset2d_fig8_inside.cl", &text);
}
