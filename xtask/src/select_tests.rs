//! Pins `--only` and `--failed` step selection and the kernel suites in batch (#1702).
//! Provenance: our own build law (AGENTS.md "Three gate tiers").

use crate::parse_gate;
use crate::plan::{self, Plan, Step};
use crate::select::{apply, failed_in_dir, failed_stems, log_failed, name_matches};

fn a(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

fn plan3() -> Plan {
    Plan {
        notes: vec![],
        steps: vec![
            Step::new("geometry suites", &["test"], ""),
            Step::new("kernel suites", &["test"], ""),
            Step::new("clippy -D warnings", &["clippy"], ""),
        ],
    }
}

#[test]
fn batch_runs_kernel_and_release_and_slow_list_are_unchanged() {
    assert!(plan::BATCH_CRATES.contains(&"kernel"));
    assert!(plan::ALL_CRATES.contains(&"kernel"));
    assert!(plan::BATCH_CRATES.contains(&"pathkernel") && plan::ALL_CRATES.contains(&"pathkernel"));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let b = plan::batch(&root);
    assert!(b.steps.iter().any(|s| s.name.starts_with("kernel suites")));
    let r = plan::release(&root);
    assert!(r.steps.iter().any(|s| s.name.starts_with("kernel suites: ")));
    assert!(plan::SLOW_BINARIES.iter().all(|k| !k.starts_with("kernel:")));
}

#[test]
fn only_matches_a_whole_word_prefix() {
    assert!(name_matches("kernel suites", "kernel"));
    assert!(name_matches("Clippy -D warnings", "clippy"));
    assert!(!name_matches("application suites", "app"));
    assert!(!name_matches("kernel suites", ""));
}

#[test]
fn only_keeps_named_steps_and_refuses_an_empty_selection() {
    let p = apply(plan3(), &a("kernel"), None).expect("kernel");
    assert_eq!(p.steps.len(), 1);
    assert_eq!(p.steps[0].name, "kernel suites");
    let p = apply(plan3(), &a("kernel clippy"), None).expect("two");
    assert_eq!(p.steps.len(), 2);
    assert!(apply(plan3(), &a("nothing"), None).is_err());
    assert_eq!(apply(plan3(), &[], None).expect("all").steps.len(), 3);
}

#[test]
fn failed_reads_the_logs_of_an_earlier_run() {
    assert!(log_failed("running\ntest result: FAILED. 1 passed\n"));
    assert!(log_failed("error: could not compile `x`\n"));
    assert!(!log_failed("test result: ok. 3 passed\n"));
    let logs = vec![
        ("01-geometry_suites.log".to_string(), "test result: ok.\n".to_string()),
        ("02-kernel_suites.log".to_string(), "test result: FAILED. 0 passed\n".to_string()),
        ("notes.log".to_string(), "error: x\n".to_string()),
    ];
    let stems = failed_stems(&logs);
    assert_eq!(stems, vec!["kernel_suites".to_string()]);
    let p = apply(plan3(), &[], Some(&stems)).expect("failed");
    assert_eq!(p.steps.len(), 1);
    assert_eq!(p.steps[0].name, "kernel suites");
    // --only and --failed together keep only the steps both name.
    assert!(apply(plan3(), &a("clippy"), Some(&stems)).is_err());
}

#[test]
fn failed_dir_that_is_missing_is_an_error() {
    assert!(failed_in_dir(std::path::Path::new("no/such/gate-logs/dir")).is_err());
}

#[test]
fn gate_options_parse() {
    let g = parse_gate(&a("batch --only kernel --only clippy --dry-run")).expect("only");
    assert_eq!(g.only, vec!["kernel".to_string(), "clippy".to_string()]);
    let g = parse_gate(&a("release --failed gate-logs/release-1")).expect("failed");
    assert_eq!(g.failed.as_deref(), Some("gate-logs/release-1"));
    assert!(parse_gate(&a("batch --only")).is_err());
    assert!(parse_gate(&a("test op_area --only kernel")).is_err());
}

/// #4055: the quick gate reads `ALL_CRATES` (quick.rs), so a workspace crate that no list names would be
/// missed. Each `crates/<d>` member of the root manifest is in `ALL_CRATES`, batch runs each but ui,
/// and clippy covers each but kernel (as before).
#[test]
fn the_gate_lists_name_every_workspace_crate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let text = std::fs::read_to_string(root.join("Cargo.toml")).expect("root manifest");
    let start = text.find("members").expect("members list");
    let end = start + text[start..].find(']').expect("end of the members list");
    let members: std::collections::BTreeSet<&str> = text[start..end]
        .split('"')
        .filter_map(|m| m.strip_prefix("crates/"))
        .collect();
    let all: std::collections::BTreeSet<&str> = plan::ALL_CRATES.iter().copied().collect();
    assert_eq!(members, all, "the root members and ALL_CRATES differ");
    let batch: Vec<&str> = plan::ALL_CRATES.iter().copied().filter(|d| *d != "ui").collect();
    assert_eq!(plan::BATCH_CRATES, batch.as_slice());
    let mut clippy: Vec<&str> = plan::CLIPPY.to_vec();
    clippy.sort_unstable();
    let mut want: Vec<&str> = plan::ALL_CRATES.iter().copied().filter(|d| *d != "kernel").collect();
    want.sort_unstable();
    assert_eq!(clippy, want);
}
