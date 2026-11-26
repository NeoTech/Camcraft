//! The registry of the K7 suite (#4027): the 16 phase-1 kinds (record
//! "Phase 1, exactly") and their state on the kernel. It is the progress
//! list of the migration. A kind that moves to the kernel changes its row
//! to `Moved` and registers its recipe (its planners and cases, with the
//! record fixtures and the recorded old output); `each_moved_kind_obeys_every_law`
//! then runs every law check on it. No new law test is written for a kind.

use camcraft_pathkernel::order::StartSide;
use camcraft_pathkernel::{Refusal, Stage};

use super::kit::{Case, Law, LawVerdict, NO_OUTPUT, Planners, plan_cuts, refused_held, run};
use super::l1_guide::check_l1;
use super::l2_sides::check_l2;
use super::l3_order::check_l3;
use super::l4_fallback::check_l4;
use super::l5_entry::check_l5;
use super::l6_load::check_l6;
use super::l7_move::check_l7;
use super::link_rule::check_link;
use super::standins::{good, island_case, square_case};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The old planner of `camcraft-motion` plans the kind.
    NotYet,
    /// The kernel plans the kind; the law suite checks it.
    Moved,
}

/// The planners of a kind and its cases.
pub struct KindRecipe {
    pub planners: Planners,
    pub cases: Vec<Case>,
}

pub struct Kind {
    pub name: &'static str,
    pub state: State,
    pub recipe: Option<fn() -> KindRecipe>,
}

const fn not_yet(name: &'static str) -> Kind {
    Kind { name, state: State::NotYet, recipe: None }
}

/// The 16 phase-1 kinds. A row that moves: `Kind { name, state: State::Moved, recipe: Some(<fn>) }`.
pub const KINDS: [Kind; 16] = [
    not_yet("Face"),
    not_yet("2D Contour"),
    not_yet("2D Pocket"),
    not_yet("2D Adaptive"),
    not_yet("2D Offset"),
    not_yet("Engrave"),
    not_yet("V-Carve"),
    not_yet("2D Chamfer"),
    not_yet("Circular"),
    not_yet("Slot"),
    not_yet("Trochoidal slot"),
    not_yet("Trace"),
    not_yet("Slot Undercut"),
    not_yet("Thread"),
    not_yet("Bore"),
    not_yet("3D Pocket"),
];

/// The rows whose state and recipe do not agree, each with its fix.
pub fn gaps(kinds: &[Kind]) -> Vec<String> {
    kinds
        .iter()
        .filter_map(|k| match (k.state, k.recipe.is_some()) {
            (State::Moved, false) => Some(format!(
                "{} is marked as moved to the kernel but registers no recipe: add its planners and cases to its row in KINDS",
                k.name
            )),
            (State::NotYet, true) => Some(format!("{} registers a recipe but is not marked as moved: set its state to Moved", k.name)),
            _ => None,
        })
        .collect()
}

/// Every law check on one case of a kind.
pub fn check_all(p: &Planners, c: &Case) -> Vec<LawVerdict> {
    let mut v = Vec::new();
    match (c.region)(c.material) {
        Ok(r) => v.push(check_l1(p.guide.as_ref(), &r, &c.recipe.guide(), &[])),
        Err(e) => v.push(refused_held(Law::L1, &e)),
    }
    let auto = run(p, c, StartSide::Auto);
    match &auto {
        Ok(r) => match r.guide.out() {
            Some(g) => v.push(check_l2(c.region, p.order.as_ref(), g, c.recipe.order())),
            None => v.push(refused_held(Law::L2, &Refusal::new(Stage::Guide, NO_OUTPUT))),
        },
        Err(e) => v.push(refused_held(Law::L2, e)),
    }
    v.push(check_l3(p, c));
    match &auto {
        Ok(r) => v.push(check_l4(r, &c.recipe)),
        Err(e) => v.push(refused_held(Law::L4, e)),
    }
    v.push(check_l5(&auto, c));
    v.push(check_l6(&auto, c));
    v.push(check_l7(c.recorded.as_deref(), &auto.as_ref().map(|r| r.plan.clone()).map_err(Clone::clone)));
    v.push(check_link(&auto, c));
    v
}

fn assert_all_held(kind: &str, c: &Case, verdicts: &[LawVerdict]) {
    assert_eq!(verdicts.len(), 8, "each law and the link rule");
    for v in verdicts {
        assert!(v.held, "{kind} / {}: {v}", c.name);
    }
}

#[test]
fn the_table_lists_the_16_phase_1_kinds_once() {
    let mut names: Vec<&str> = KINDS.iter().map(|k| k.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 16, "{names:?}");
}

#[test]
fn a_kind_marked_as_moved_registers_a_recipe() {
    let g = gaps(&KINDS);
    assert!(g.is_empty(), "the registry of the law suite:\n{}", g.join("\n"));
}

#[test]
fn the_registry_guard_names_a_moved_kind_without_a_recipe() {
    let rows = [
        Kind { name: "2D Offset", state: State::Moved, recipe: None },
        Kind { name: "Face", state: State::NotYet, recipe: Some(stand_in_kind) },
        not_yet("Bore"),
    ];
    let g = gaps(&rows);
    assert_eq!(g.len(), 2, "{g:?}");
    assert!(g[0].starts_with("2D Offset is marked as moved") && g[0].contains("registers no recipe"), "{}", g[0]);
    assert!(g[1].starts_with("Face registers a recipe"), "{}", g[1]);
}

#[test]
fn each_moved_kind_obeys_every_law() {
    for k in KINDS.iter().filter(|k| k.state == State::Moved) {
        let Some(make) = k.recipe else { continue };
        let kr = make();
        for c in &kr.cases {
            let v = check_all(&kr.planners, c);
            for x in &v {
                assert!(x.held, "{} / {}: {x}", k.name, c.name);
            }
        }
    }
}

/// The obeying stand-ins as one registered kind, with the old output
/// recorded from their own run: the path that a moved kind takes.
fn stand_in_kind() -> KindRecipe {
    let mut cases = vec![square_case().expect("fixture"), island_case().expect("fixture")];
    for c in &mut cases {
        let r = run(&good(), c, StartSide::Auto).expect("the stand-ins plan the fixture");
        c.recorded = Some(plan_cuts(&r.plan).into_iter().cloned().collect());
    }
    KindRecipe { planners: good(), cases }
}

#[test]
fn the_law_runner_passes_a_registered_kind_that_obeys_every_law() {
    let kr = stand_in_kind();
    for c in &kr.cases {
        assert_all_held("stand-in", c, &check_all(&kr.planners, c));
    }
}
