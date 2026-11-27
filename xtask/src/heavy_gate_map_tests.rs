//! Pins the Gate map part of the heavy reach (#3560): a library module reaches the unit and doc tests
//! of its crate and the heavy binaries whose Gate map names it; with no such map the crate reach stays.
//! Provenance: our own build law (AGENTS.md "Heavy lanes at review", "A test lives with the code it tests").

use std::path::{Path, PathBuf};

use crate::{heavy, quick};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn plan(changed: &[&str]) -> crate::plan::Plan {
    let changed: Vec<String> = changed.iter().map(|s| (*s).to_string()).collect();
    heavy::plan(&root(), &changed).expect("heavy plan")
}

fn names(changed: &[&str]) -> Vec<String> {
    plan(changed).steps.into_iter().map(|s| s.name).collect()
}

/// The integration binaries of the steps (`<crate>:<binary>`): no lib, bins, doc or known-fail step.
fn binaries(names: &[String]) -> Vec<String> {
    names
        .iter()
        .skip(1)
        .filter_map(|n| n.split_once(" heavy: "))
        .filter(|(_, l)| !matches!(*l, "lib" | "bins" | "doc") && !l.starts_with("known fail #"))
        .map(|(d, l)| format!("{d}:{l}"))
        .collect()
}

#[test]
fn a_library_module_reaches_its_crate_targets_and_the_binaries_whose_gate_map_names_it() {
    let p = plan(&["crates/motion/src/drilling/boss.rs"]);
    let n: Vec<String> = p.steps.iter().map(|s| s.name.clone()).collect();
    for want in ["motion heavy: lib", "motion heavy: bins", "motion heavy: doc"] {
        assert!(n.contains(&want.to_string()), "{want} missing in {n:?}");
    }
    // The lib of app links motion, but no Gate map of it names motion::drilling: not reached.
    assert!(!n.contains(&"app heavy: lib".to_string()), "{n:?}");
    let maps = quick::gate_maps(&root()).expect("gate maps");
    let (named, _) = quick::suites_naming(&maps, "motion", "drilling", "boss").expect("naming");
    let reached = binaries(&n);
    assert!(!reached.is_empty(), "{n:?}");
    for b in &reached {
        assert!(named.contains(b), "{b} is reached, but its Gate map does not name motion::drilling: {named:?}");
    }
    let all = crate::plan::heavy_steps(&root()).len();
    assert!(n.len() * 4 < all, "{} of {all} steps: not narrow", n.len());
    let why = "motion heavy: cross_invariance - crates/motion/src/drilling/boss.rs changed: the gate map names motion::drilling";
    assert!(p.notes.iter().any(|l| l.trim_start().starts_with(why)), "{:#?}", p.notes);
}

#[test]
fn a_module_that_no_gate_map_names_and_a_shared_crate_keep_the_crate_reach() {
    let maps = quick::gate_maps(&root()).expect("gate maps");
    let (named, _) = quick::suites_naming(&maps, "motion", "break_through", "").expect("naming");
    assert!(named.is_empty(), "the probe needs a module that no Gate map names: {named:?}");
    let wide = names(&["crates/motion/src/lib.rs"]);
    // With a test-only file beside it, the unnamed library file still decides.
    assert_eq!(names(&["crates/motion/src/ring_zig_tests.rs", "crates/motion/src/break_through.rs"]), wide);
    // geometry is shared (quick runs batch for it): every lane of motion and app stays.
    let shared = names(&["crates/geometry/src/nesting.rs"]);
    assert!(wide.iter().all(|s| shared.contains(s)), "{shared:?}");
    assert!(shared.contains(&"app heavy: lib".to_string()), "{shared:?}");
}

#[test]
fn a_path_included_file_reaches_through_the_gate_map_of_its_includer() {
    // `posting.rs` holds `#[path = "posting_units.rs"] mod program_unit;`: no map names `posting_units`.
    let maps = quick::gate_maps(&root()).expect("gate maps");
    assert!(quick::suites_naming(&maps, "app", "posting_units", "").expect("naming").0.is_empty());
    let mods = quick::src_modules(&root(), "app", "src/posting_units.rs").expect("modules");
    assert!(mods.contains(&("app".into(), "posting".into(), "program_unit".into())), "{mods:?}");
    let (named, _) = quick::suites_naming(&maps, "app", "posting", "program_unit").expect("naming");
    let n = names(&["crates/app/src/posting_units.rs"]);
    let reached = binaries(&n);
    assert!(!reached.is_empty() && reached.iter().all(|b| named.contains(b)), "{reached:?} not in {named:?}");
    assert!(n.contains(&"app heavy: lib".to_string()), "{n:?}");
}
