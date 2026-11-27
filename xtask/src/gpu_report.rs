//! GPU coverage notes from libtest's captured output, not its successful exit code.
//! Provenance: the public libtest --show-output format and our GPU test skip messages (#3879).

use std::collections::BTreeSet;

/// Keep successful output grouped by test. Unlike --nocapture, this preserves identity in parallel runs.
pub fn output_args(args: &[String]) -> Vec<String> {
    let mut out = args.to_vec();
    if args.first().is_some_and(|a| a == "test") && !args.iter().any(|a| a == "--no-run") {
        if !out.iter().any(|a| a == "--") { out.push("--".into()); }
        if !out.iter().any(|a| a == "--show-output") { out.push("--show-output".into()); }
    }
    out
}

/// SKIP is a successful process with incomplete GPU coverage, never a tested GPU pass.
pub fn report_row(ok: bool, stall: bool, note: &str) -> &'static str {
    if ok && !stall && note.contains("GPU test(s) not exercised") { "SKIP" } else { crate::exec::row(ok, stall) }
}

/// Captured stdout headers retain the test identity even when libtest runs tests in parallel.
/// Only the explicit GPU skip diagnostic counts: an ordinary CPU test is not a GPU skip.
pub fn notes(text: &str) -> String {
    let mut test = None;
    let mut skips = BTreeSet::new();
    let mut adapters = BTreeSet::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("---- ").and_then(|s| s.strip_suffix(" stdout ----")) {
            test = Some(name);
        } else if line.starts_with("test result:") || line.starts_with("     Running ") {
            test = None;
        } else if line.starts_with("GPU adapter: ") {
            adapters.insert(line.to_string());
        } else if line.starts_with("*** ") && line.contains("SKIPPED") && line.contains("NOT a pass") {
            let name = test.unwrap_or("unknown test");
            let why = if line.contains("no adapter") || line.contains("no graphics adapter") { "no adapter" } else { "GPU unavailable" };
            skips.insert(format!("SKIPPED ({why}): {name}"));
        }
    }
    let mut notes = Vec::new();
    if !skips.is_empty() {
        notes.push(format!("{} GPU test(s) not exercised; {}", skips.len(), skips.into_iter().collect::<Vec<_>>().join("; ")));
    }
    notes.extend(adapters);
    notes.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_reporting_exposes_captured_parallel_output() {
        let exe = std::env::current_exe().unwrap();
        let args = output_args(&["test".into(), "--".into(), "--test-threads=2".into(), "exec::gpu_report::tests::gpu_reporting_fixture_".into()]);
        let output = std::process::Command::new(exe).env("CAMCRAFT_TEST_GPU_REPORT", "1").args(&args[2..]).output().unwrap();
        assert!(output.status.success());
        let note = notes(&String::from_utf8_lossy(&output.stdout));
        assert!(note.contains("SKIPPED (no adapter): exec::gpu_report::tests::gpu_reporting_fixture_skip"), "{note}");
        assert!(note.contains("1 GPU test(s) not exercised"), "{note}");
        assert!(!note.contains("fixture_pass"), "{note}");
        assert_eq!(report_row(true, false, &note), "SKIP");
        assert_eq!(report_row(false, false, &note), "FAIL");
        assert_eq!(report_row(false, true, &note), "STALL");
        assert_eq!(report_row(true, false, ""), "ok");
        assert_eq!(output_args(&["clippy".into()]), ["clippy"]);
        assert_eq!(output_args(&["test".into(), "--no-run".into()]), ["test", "--no-run"]);
    }

    #[test]
    fn gpu_reporting_fixture_pass() {
        println!("ordinary CPU test output");
    }

    #[test]
    fn gpu_reporting_fixture_skip() {
        if std::env::var_os("CAMCRAFT_TEST_GPU_REPORT").is_some() {
            eprintln!("*** suite: SKIPPED (no adapter) - NOT a pass ***");
        }
    }
}
