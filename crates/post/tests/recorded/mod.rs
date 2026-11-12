//! The shared runner of the per-post script suites (#926): the `post`
//! binary on each recorded intermediate stream, its result compared with
//! the golden beside the suites.
//!
//! Provenance: our own test harness. The maintainer's direction on #926
//! ("cps files should be tested against the post binary with a couple of
//! edge case intermediary files to parse") and the golden workflow of
//! `crates/app/tests/golden/CLAUDE.md` (WRITE_GOLDEN=1 regenerates).
//!
//! A case's result is one of two goldens under `recorded/golden/<post>/`:
//! - `<case>.nc` - the program the post released (exit 0, no warning);
//! - `<case>.refusal` - the post's named refusal: either the counted
//!   warnings (the program is written but the in-app gate never releases
//!   it), or the error the binary stopped on (nothing written).
//!
//! The inputs live in `recorded/streams/`; `recorded_streams.rs` (cl-built
//! cases) and app's `recorded_post_streams.rs` (motion-built cases) keep
//! each of them equal to what its generator writes today.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The recorded cases every post suite runs, in file-stem form.
pub const CASES: [&str; 9] = [
    "arcs",
    "full_circle",
    "full_circle_raw",
    "drill_retract",
    "tool_change",
    "rotary",
    "probe",
    "empty_section",
    "extreme_coords",
];

/// Post bugs these suites found, by (script, case, issue): wrong output
/// is never baked into a golden. Such a case still runs through the binary
/// (no crash, no hang) but compares nothing and has no golden; the fix
/// removes its line and records the golden (`WRITE_GOLDEN=1`), read first.
/// `recorded_streams.rs` holds that no listed case has a golden.
pub const KNOWN_BUGS: [(&str, &str, u32); 7] = [
    // The cycle traverses between holes at the R plane, not the hop.
    ("fanuc.cps", "drill_retract", 994),
    ("haas.cps", "drill_retract", 994),
    ("mazak.cps", "drill_retract", 994),
    ("sinumerik.cps", "drill_retract", 994),
    ("tormach.cps", "drill_retract", 994),
    ("doosan-dnm.cps", "drill_retract", 994),
    // Flood coolant (M8) in a probing section.
    ("doosan-dnm.cps", "probe", 995),
];

pub fn known_bug(script: &str, case: &str) -> Option<u32> {
    KNOWN_BUGS.iter().find(|(s, c, _)| *s == script && *c == case).map(|(_, _, n)| *n)
}

/// The machine a case posts on, by file name under `assets/machines/`:
/// the rotary case needs one stating its rotaries (a 5-axis head with A
/// and B), every other case posts machine-less, as the binary does with
/// no `--machine`. The file is handed to the binary in the grammar the
/// app hands the engine (`post::machine::mirror` of the parsed vendor
/// file): the binary's `--machine` door reads only that static grammar,
/// so a vendor tree passed raw would reach the script with no rotaries.
pub fn machine_of(case: &str) -> Option<&'static str> {
    (case == "rotary").then_some("tilthead.mch")
}

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded")
}

pub fn stream_path(case: &str) -> PathBuf {
    here().join("streams").join(format!("{case}.cnc"))
}

fn tmp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("camcraft_recorded_{tag}_{}", std::process::id()))
}

/// What the binary did with one stream: the golden's file extension and
/// its text.
pub fn verdict(script: &str, case: &str) -> (&'static str, String) {
    let cps = repo().join("assets/posts").join(script);
    let input = stream_path(case);
    assert!(cps.is_file(), "no post script {}", cps.display());
    assert!(input.is_file(), "no recorded stream {}", input.display());
    let tag = format!("{}_{case}", script.trim_end_matches(".cps"));
    let out = tmp(&format!("{tag}.nc"));
    let log = tmp(&format!("{tag}.log"));
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&log);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_post"));
    let mch = tmp(&format!("{tag}.mch"));
    if let Some(m) = machine_of(case) {
        let path = repo().join("assets/machines").join(m);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let parsed = camcraft_post::machine::parse_mch(&text).unwrap_or_else(|e| panic!("{m}: {e}"));
        let mirrored = camcraft_post::post::machine::mirror(&parsed);
        std::fs::write(&mch, mirrored.to_string()).expect("write the mirrored machine");
        cmd.arg("--machine").arg(&mch);
    }
    let r = cmd.arg("--log").arg(&log).arg(&cps).arg(&input).arg(&out).output().expect("run the post binary");
    let program = std::fs::read_to_string(&out).ok();
    let warnings = std::fs::read_to_string(&log).ok();
    for f in [&out, &log, &mch] {
        let _ = std::fs::remove_file(f);
    }
    if !r.status.success() {
        assert!(program.is_none(), "{script} {case}: a failed post wrote a program");
        let err = String::from_utf8_lossy(&r.stderr).replace("\r\n", "\n");
        return ("refusal", format!("; {script} refused {case}: the post stopped, nothing written\n{err}"));
    }
    let program = program.expect("a successful post writes its program");
    match warnings {
        Some(w) if !w.trim().is_empty() => {
            let n = w.lines().count();
            ("refusal", format!("; {script} refused {case}: {n} warning(s), the program is never released\n{w}"))
        }
        _ => ("nc", program),
    }
}

/// Post `case` through `script` and compare with its golden. With
/// WRITE_GOLDEN set, write the golden instead (and drop the other form).
pub fn check(script: &str, case: &str) {
    let (ext, text) = verdict(script, case);
    if let Some(issue) = known_bug(script, case) {
        eprintln!("{script} {case}: known post bug #{issue} - posted ({ext}), not compared");
        return;
    }
    let dir = here().join("golden").join(script.trim_end_matches(".cps"));
    let path = dir.join(format!("{case}.{ext}"));
    let other = dir.join(format!("{case}.{}", if ext == "nc" { "refusal" } else { "nc" }));
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::create_dir_all(&dir).expect("create the golden dir");
        std::fs::write(&path, &text).expect("write the golden");
        let _ = std::fs::remove_file(&other);
        return;
    }
    assert!(
        !other.exists(),
        "{script} {case}: the golden says {} but the post gave {ext}:\n{text}",
        other.display()
    );
    let want = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("cannot read golden {}: {e}; create it with WRITE_GOLDEN=1 and read it before committing", path.display())
    });
    assert_eq!(
        want.replace("\r\n", "\n"),
        text,
        "golden {} is stale; regenerate with WRITE_GOLDEN=1 and read the diff",
        path.display()
    );
}

/// One `#[test]` per recorded case, named after it, for one script.
#[allow(unused_macros)] // the staleness guard includes this module without a suite
macro_rules! post_suite {
    ($script:literal) => {
        #[test]
        fn arcs() {
            recorded::check($script, "arcs");
        }
        #[test]
        fn full_circle() {
            recorded::check($script, "full_circle");
        }
        #[test]
        fn full_circle_raw() {
            recorded::check($script, "full_circle_raw");
        }
        #[test]
        fn drill_retract() {
            recorded::check($script, "drill_retract");
        }
        #[test]
        fn tool_change() {
            recorded::check($script, "tool_change");
        }
        #[test]
        fn rotary() {
            recorded::check($script, "rotary");
        }
        #[test]
        fn probe() {
            recorded::check($script, "probe");
        }
        #[test]
        fn empty_section() {
            recorded::check($script, "empty_section");
        }
        #[test]
        fn extreme_coords() {
            recorded::check($script, "extreme_coords");
        }
    };
}
