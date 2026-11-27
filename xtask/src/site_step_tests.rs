//! Pins of the site tests step (#4133): the tiers that run it, its input rule against the paths that the
//! site tests really open, the missing-tool rule and the dry-run lines.
//! Provenance: our own build law (AGENTS.md "Three gate tiers"); issue #4133.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::exec::gpu_report::report_row;
use crate::plan::{self, Step};
use crate::quick::{self, Diff, Quick};
use crate::site_step::{self, Missing, Tools};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_default()
}

fn site_steps(steps: &[Step]) -> Vec<&Step> {
    steps.iter().filter(|s| s.site.is_some()).collect()
}

/// Every repository path that a site test names: the string literals, and the `NAME + "rest"` joins of
/// a constant `NAME = "dir/"` of the same file (`UI + "panes/cmd.rs"`).
fn named_paths(text: &str) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = site_step::literal_paths(text).expect("regex").into_iter().collect();
    let def = crate::scan::re(r#"(?m)^([A-Z_]+) = "([^"]+/)"$"#).expect("regex");
    for d in def.captures_iter(text) {
        let join = crate::scan::re(&format!(r#"\b{} \+ "([^"]+)""#, &d[1])).expect("regex");
        out.extend(join.captures_iter(text).map(|j| format!("{}{}", &d[2], &j[1])));
    }
    out
}

#[test]
fn each_path_that_a_site_test_names_is_an_input() {
    let dir = repo().join("site").join("tests");
    let mut named = BTreeSet::new();
    for e in std::fs::read_dir(&dir).expect("site/tests") {
        let p = e.expect("entry").path();
        if p.extension().is_some_and(|x| x == "py") {
            named.extend(named_paths(&std::fs::read_to_string(&p).expect("site test")));
        }
    }
    // The reader is not vacuous: it finds the inputs of the four kinds of site test.
    for known in [
        "assets/ui",
        "crates/ui/src/egui_app/icons.rs",
        "crates/ui/src/egui_app/panes/cmd.rs",
        "crates/app/src/operation/types/sel.rs",
        "assets/posts/AGENTS.md",
        "examples/first-4axis-part.camproj",
        "site/content/guide",
    ] {
        assert!(named.contains(known), "the site test reader lost {known}: {named:#?}");
    }
    let lost: Vec<&String> = named.iter().filter(|p| !site_step::is_input(p) && !site_step::is_input(&format!("{p}/"))).collect();
    assert!(lost.is_empty(), "add each path to site_step::INPUTS: {lost:#?}");
}

#[test]
fn input_rule_takes_directories_by_prefix_and_files_exactly() {
    assert!(site_step::is_input("site/static/icons/op-rotary-flow.png"));
    assert!(site_step::is_input("assets/ui/tool-form-mill.png"));
    assert!(site_step::is_input("crates/ui/src/egui_app/panes/cmd.rs"));
    assert!(site_step::is_input("crates\\app\\src\\operation\\types\\sel.rs"));
    assert!(!site_step::is_input("crates/app/src/operation/types/sel.rs.bak"));
    assert!(!site_step::is_input("crates/motion/src/finish3d.rs"));
    assert!(!site_step::is_input("xtask/src/plan.rs"));
    assert!(!site_step::is_input("sites/x.md"));
}

#[test]
fn batch_and_release_run_the_site_step_with_their_missing_tool_rule() {
    let root = repo();
    let b = plan::batch(&root);
    let r = plan::release(&root);
    let r_cut = plan::release_with(&root, false);
    assert_eq!(site_steps(&b.steps).iter().map(|s| s.site).collect::<Vec<_>>(), vec![Some(Missing::Skip)]);
    assert_eq!(site_steps(&r.steps).iter().map(|s| s.site).collect::<Vec<_>>(), vec![Some(Missing::Fail)]);
    assert_eq!(site_steps(&r_cut.steps).iter().map(|s| s.site).collect::<Vec<_>>(), vec![Some(Missing::Fail)], "cut.yml runs --no-snapshots");
    let s = site_step::step(Missing::Fail);
    assert_eq!(s.name, site_step::NAME);
    assert!(s.args.is_empty(), "the site step starts no cargo");
    // It shares the lanes (not a build, not alone), and `gate batch --lane N/M` gives it to one lane.
    assert!(!crate::lanes::is_build(&s) && !crate::lanes::alone(&s));
    let lanes = crate::lane_split::assign(&b.steps, 3);
    let i = b.steps.iter().position(|x| x.site.is_some()).expect("site step");
    assert!(lanes[i].is_some());
    assert!(b.notes.iter().any(|n| n.contains("site tests")), "{:#?}", b.notes);
}

fn quick_steps(root: &Path, changed: &[&str]) -> Vec<Step> {
    let changed: Vec<String> = changed.iter().map(|s| (*s).to_string()).collect();
    let none = |_: &str| -> Vec<String> { Vec::new() };
    let before = |f: &str| std::fs::read_to_string(root.join(f)).ok();
    let diff = Diff { changed: &changed, untracked: &[], hunks: &none, before: &before };
    match quick::plan(root, &diff, false).expect("quick plan") {
        Quick::Plan(p) => p.steps,
        Quick::Batch(f) => panic!("unexpected batch fallback on {f}"),
    }
}

#[test]
fn quick_runs_the_site_step_only_when_an_input_changed() {
    let root = repo();
    for f in ["site/content/guide/reference/post-processors.md", "site/tests/test_guide_icons.py", "assets/ui/op-rotary-flow.png", "crates/app/src/operation/types/sel.rs"] {
        let steps = quick_steps(&root, &[f]);
        assert_eq!(site_steps(&steps).iter().map(|s| s.site).collect::<Vec<_>>(), vec![Some(Missing::Skip)], "{f}");
    }
    for f in ["xtask/src/plan.rs", "docs/decisions/README.md", "crates/motion/src/finish3d.rs"] {
        assert!(site_steps(&quick_steps(&root, &[f])).is_empty(), "{f} is not a site test input");
    }
}

fn gone(python: bool, zola: bool, node: bool) -> Vec<String> {
    let t = Tools {
        python: python.then(|| "python3".to_string()),
        zola: zola.then(|| ("zola".to_string(), format!("zola {}", site_step::ZOLA_VERSION))),
        node: node.then(|| "node".to_string()),
    };
    site_step::missing(&t)
}

#[test]
fn a_missing_tool_is_a_skip_row_in_quick_and_batch_and_a_fail_row_in_release() {
    assert!(gone(true, true, true).is_empty());
    let all = gone(false, false, false);
    assert_eq!(all.len(), 3);
    for (tool, item) in ["python", "zola", "node"].iter().zip(&all) {
        assert!(item.starts_with(tool) && item.contains("install"), "{item}");
    }
    let zola = gone(true, false, true);
    let (ok, note) = site_step::verdict(&zola, Missing::Skip);
    assert!(ok && note.starts_with("SKIPPED (tool missing: zola)"), "{note}");
    assert!(note.contains(site_step::ZOLA_VERSION) && note.contains("ZOLA="), "{note}");
    assert_eq!(report_row(ok, false, &note), "SKIP", "a skipped site step is never an ok row");
    let (ok, note) = site_step::verdict(&zola, Missing::Fail);
    assert!(!ok && note.contains("tool missing: zola"), "{note}");
    assert_eq!(report_row(ok, false, &note), "FAIL");
    let (_, note) = site_step::verdict(&gone(true, true, false), Missing::Skip);
    assert!(note.starts_with("SKIPPED (tool missing: node)"), "{note}");
}

#[test]
fn dry_run_prints_the_command_and_the_rule_and_runs_nothing() {
    let lines = site_step::describe("wt", "python3", "/opt/zola", Missing::Fail, &gone(true, false, true));
    assert!(lines[0].contains("python3 -m unittest discover -s site/tests") && lines[0].contains("ZOLA=/opt/zola"), "{lines:#?}");
    assert!(lines[1].contains("FAIL row") && lines[1].contains("missing now: zola"), "{lines:#?}");
    let lines = site_step::describe("wt", "python", "zola", Missing::Skip, &[]);
    assert!(lines[1].contains("SKIP row") && lines[1].contains("all tools found"), "{lines:#?}");
    let logs = std::env::temp_dir().join(format!("xtask-site-dry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&logs);
    std::fs::create_dir_all(&logs).expect("log dir");
    let d = site_step::execute(&repo(), &logs, true, 1, &site_step::step(Missing::Fail), Missing::Fail).expect("dry");
    assert!(d.ok && d.note == "dry run" && d.name == site_step::NAME);
    assert_eq!(std::fs::read_dir(&logs).expect("logs").count(), 0, "a dry run writes no log");
    let _ = std::fs::remove_dir_all(&logs);
}

#[test]
fn the_note_reads_the_unittest_summary() {
    let red = "FAIL: test_x (test_a.A.test_x)\nERROR: test_y (test_b.B.test_y)\n---\nRan 112 tests in 4.9s\n\nFAILED (failures=1, errors=1)\n";
    assert_eq!(site_step::result_line(red), "Ran 112 tests in 4.9s, FAILED (failures=1, errors=1) [tests: test_x, test_y]");
    assert_eq!(site_step::result_line("Ran 3 tests in 0.1s\n\nOK\n"), "Ran 3 tests in 0.1s, OK");
    assert_eq!(site_step::result_line(""), "no unittest summary");
    assert!(site_step::zola_note(&format!("zola {}", site_step::ZOLA_VERSION)).is_none());
    assert!(site_step::zola_note("zola 0.19.1").is_some_and(|n| n.contains("pins zola")));
}
