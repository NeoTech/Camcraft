//! Provenance: issue #3303 - a gate step whose process tree writes nothing for the stall time is killed
//! as a tree and named; output of the child or of a descendant keeps it alive.
//!
//! The children are this test executable itself, started on `watchdog_helper` with a mode in `HELPER`:
//! it exists on every platform that xtask builds on. The stall time goes in as a parameter (one or two
//! seconds), not through `CAMCRAFT_GATE_STALL_MIN`: that value is in whole minutes, and `set_var` is
//! unsafe in edition 2024 and races with the tests that run in parallel.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::watchdog::{self, End};

const HELPER: &str = "XTASK_WATCHDOG_HELPER";

/// The child side. Without `HELPER` it returns at once (an ordinary test run).
#[test]
fn watchdog_helper() {
    let Ok(mode) = std::env::var(HELPER) else { return };
    let me = std::env::current_exe().expect("exe");
    let spawn = |m: &str| helper(&me, m, Stdio::inherit(), Stdio::inherit()).expect("grandchild");
    match mode.as_str() {
        "silent" => std::thread::sleep(Duration::from_secs(120)),
        "tree" => {
            let mut g = spawn("silent");
            println!("grandchild pid {}", g.id());
            std::thread::sleep(Duration::from_secs(120));
            let _ = g.wait();
        }
        "tick" => {
            for i in 0..30 {
                println!("tick {i}");
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        "quiet-parent" => {
            let mut g = spawn("tick");
            g.wait().expect("wait");
        }
        _ => {}
    }
}

fn helper(me: &PathBuf, mode: &str, out: Stdio, err: Stdio) -> std::io::Result<Child> {
    Command::new(me)
        .args(["--exact", "watchdog_tests::watchdog_helper", "--nocapture", "--test-threads=1"])
        .env(HELPER, mode)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .spawn()
}

/// Starts the helper in `mode` with its output in a fresh log, and waits under the watchdog.
fn watched(mode: &str, limit: Duration) -> (End, String, Duration) {
    let dir = std::env::temp_dir().join(format!("xtask-watchdog-{mode}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("dir");
    let path = dir.join("step.log");
    let mut log = File::create(&path).expect("log");
    writeln!(log, "     Running helper {mode}").expect("write");
    let me = std::env::current_exe().expect("exe");
    let mut child = helper(&me, mode, log.try_clone().expect("clone").into(), log.try_clone().expect("clone").into()).expect("spawn");
    let t0 = Instant::now();
    let end = watchdog::wait(&mut child, &log, &|| 0, limit).expect("wait");
    let took = t0.elapsed();
    let text = fs::read_to_string(&path).expect("read");
    let _ = fs::remove_dir_all(&dir);
    (end, text, took)
}

#[test]
fn a_silent_step_is_killed_as_a_tree_and_named() {
    let (end, text, took) = watched("tree", Duration::from_secs(2));
    assert!(matches!(end, End::Stalled), "a silent tree must stall: {end:?}\n{text}");
    assert!(took < Duration::from_secs(60), "the watchdog must end the step soon after the stall time: {took:?}");
    let line = text.lines().find(|l| l.starts_with(watchdog::LINE)).expect("the log has the STALL line");
    let alive = line.split_once("alive: ").map(|(_, a)| a.split(", ").count()).unwrap_or(0);
    assert!(alive >= 2, "the child and the grandchild were alive: {line}");
    assert!(crate::select::log_failed(&text), "--failed must select a stalled step");
    let pid: u32 = text
        .lines()
        .find_map(|l| l.split_once("grandchild pid ").and_then(|(_, p)| p.trim().parse().ok()))
        .unwrap_or_else(|| panic!("the helper names its grandchild:
{text}"));
    // taskkill and kill return before the process is gone: give it a moment.
    let gone = (0..50).any(|_| {
        std::thread::sleep(Duration::from_millis(100));
        !watchdog::processes().iter().any(|p| p.0 == pid)
    });
    assert!(gone, "the grandchild {pid} must die with the tree");
    let note = watchdog::report("post suites: lib", &text);
    assert!(note.starts_with("stall: no output for 1min"), "{note}");
}

#[test]
fn output_of_the_child_or_a_descendant_keeps_the_step_alive() {
    for mode in ["tick", "quiet-parent"] {
        let (end, text, _) = watched(mode, Duration::from_secs(2));
        assert!(matches!(end, End::Exited(st) if st.success()), "{mode}: the step writes output, so it must run to its end: {end:?}\n{text}");
        assert!(!text.contains(watchdog::LINE), "{mode}: no STALL line:\n{text}");
    }
}

#[test]
fn the_stall_time_reads_as_the_job_budget_does() {
    assert_eq!(watchdog::from_value(None).0, watchdog::DEFAULT_MIN);
    assert_eq!(watchdog::from_value(Some(" ")).0, watchdog::DEFAULT_MIN);
    let (n, line) = watchdog::from_value(Some("30"));
    assert!(n == 30 && line.contains("CAMCRAFT_GATE_STALL_MIN=30"), "{line}");
    for bad in ["0", "-3", "ten", "1.5"] {
        let (n, line) = watchdog::from_value(Some(bad));
        assert_eq!(n, watchdog::DEFAULT_MIN, "{bad}");
        assert!(line.starts_with("note: ") && line.contains(watchdog::ENV) && line.contains(bad), "{line}");
    }
}

#[test]
fn release_and_ignored_steps_get_the_longer_stall_time() {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(watchdog::limit_for(&args(&["test", "-p", "x", "--lib"]), 15), Duration::from_secs(15 * 60));
    assert_eq!(watchdog::limit_for(&args(&["test", "--release", "-p", "x"]), 15), Duration::from_secs(60 * 60));
    assert_eq!(watchdog::limit_for(&args(&["test", "-p", "x", "--", "--ignored"]), 15), Duration::from_secs(60 * 60));
    assert_eq!(watchdog::limit_for(&args(&["--release"]), u64::MAX), Duration::from_secs(u64::MAX), "no overflow");
}

#[test]
fn a_stall_has_its_own_row_and_exit_code() {
    assert_eq!(crate::exec::row(true, false), "ok");
    assert_eq!(crate::exec::row(false, false), "FAIL");
    assert_eq!(crate::exec::row(false, true), "STALL");
    assert_eq!(watchdog::exit_code(false, 0, 0), 0);
    assert_eq!(watchdog::exit_code(false, 0, 2), watchdog::EXIT);
    assert_eq!(watchdog::exit_code(false, 1, 2), 1, "a FAIL row wins: main is red");
    assert_eq!(watchdog::exit_code(true, 0, 1), 1, "a void run stays 1");
    assert_ne!(watchdog::EXIT, 1);
}

#[test]
fn a_stall_report_without_the_watchdog_line_still_names_the_step() {
    let note = watchdog::report("post suites: lib", "running 1 test
");
    assert!(note.starts_with("stall: no output for ?min"), "{note}");
}

#[test]
fn the_process_tree_follows_the_parent_links() {
    let all = watchdog::parse_processes("  1 0 init\n 10 1 cargo\n 11 10 rustc\n 12 11 my test exe\n 13 1 other\nbad line\n");
    let tree = watchdog::descendants(10, &all);
    assert_eq!(tree, vec![(10, "cargo".into()), (11, "rustc".into()), (12, "my test exe".into())]);
}
