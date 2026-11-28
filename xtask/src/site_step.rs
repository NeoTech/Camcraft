//! The site tests as a gate step (#4133): `python -m unittest discover -s site/tests` with zola and
//! node, the one step kind that does not start cargo, and the input paths whose change selects it in quick.
//! Provenance: our own build law (AGENTS.md "Three gate tiers"); the Python `unittest` CLI; the pinned
//! zola of release.yml's `site` job.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::exec::Done;
use crate::plan::Step;

/// The step name; `--only "site tests"` selects it.
pub const NAME: &str = "site tests (python unittest, zola, node)";
/// The zola that release.yml installs ("Install pinned Zola"); another version is a visible note.
pub const ZOLA_VERSION: &str = "0.23.6";
/// The prefix of the note of a step whose tool is missing: the summary row is `SKIP`, never `ok`.
pub const SKIPPED: &str = "SKIPPED (tool missing: ";

/// The repository paths that the site tests read, outside `site/` too: a change under one of them can
/// change a site test result, so quick runs the step. A directory ends with `/`. The pin
/// `site_step_tests::each_path_that_a_site_test_names_is_an_input` reads every path literal of
/// `site/tests/*.py` and fails when one is not covered here, so a new input cannot be forgotten.
pub const INPUTS: &[&str] = &[
    "site/",
    "assets/ui/",
    "assets/posts/",
    "assets/machines/",
    "examples/",
    "crates/ui/src/egui_app/",
    "crates/ui/src/operations.rs",
    "crates/app/src/operation/types/sel.rs",
    "crates/app/src/operation/types/height.rs",
    "crates/app/src/operation/kinds.rs",
    "crates/app/src/operation/job.rs",
    "crates/app/src/cli.rs",
    "crates/app/src/posting_files.rs",
    "crates/app/src/posting_sim.rs",
    "crates/app/src/stock_sim_rotary.rs",
    "crates/geometry/src/geometry/units.rs",
    "crates/sim/src/stock_sim/cut.rs",
    "crates/motion/src/multiaxis/rotary.rs",
    "crates/post/src/post/js_host.rs",
];

/// What a missing tool does to the step: a visible `SKIP` row (quick, batch) or a `FAIL` row (release:
/// the cut gate is the last check before the tag, #4133 decision).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    Skip,
    Fail,
}

/// The site step of a tier.
pub fn step(missing: Missing) -> Step {
    let mut s = Step::new(NAME, &[], "");
    s.site = Some(missing);
    s
}

/// True when `path` (`/`-separated, relative to the root) is an input of the site tests.
pub fn is_input(path: &str) -> bool {
    let p = path.replace('\\', "/");
    INPUTS.iter().any(|i| if i.ends_with('/') { p.starts_with(i) } else { p == *i })
}

/// The first changed file that is a site test input.
pub fn touched(changed: &[String]) -> Option<&String> {
    changed.iter().find(|f| is_input(f))
}

#[cfg(test)]
/// The repository paths that a site test names as string literals (`"crates/..."`, `"assets/..."`, ...).
pub fn literal_paths(text: &str) -> Result<Vec<String>, String> {
    let lit = crate::scan::re(r#""((?:site|assets|examples|crates|docs|scripts|xtask)/[^"\s{}]*)""#)?;
    Ok(lit.captures_iter(text).map(|c| c[1].to_string()).collect())
}

/// The tools of the step, found on this host: python (`python3`, else `python`), zola (`ZOLA`, else
/// `zola`) and node. A tool is found when `<tool> --version` exits 0.
pub struct Tools {
    pub python: Option<String>,
    pub zola: Option<(String, String)>,
    pub node: Option<String>,
}

fn version(cmd: &str) -> Option<String> {
    let out = Command::new(cmd).arg("--version").stdin(Stdio::null()).output().ok()?;
    out.status.success().then(|| {
        let t = String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
        t.trim().to_string()
    })
}

pub fn find_tools() -> Tools {
    let zola = std::env::var("ZOLA").ok().filter(|z| !z.is_empty()).unwrap_or_else(|| "zola".into());
    Tools {
        python: ["python3", "python"].into_iter().find(|p| version(p).is_some_and(|v| v.contains("Python 3"))).map(String::from),
        zola: version(&zola).map(|v| (zola, v)),
        node: version("node").map(|_| "node".to_string()),
    }
}

/// The missing tools, each with its install step.
pub fn missing(t: &Tools) -> Vec<String> {
    let mut out = Vec::new();
    if t.python.is_none() {
        out.push("python (install Python 3 on PATH as python3 or python)".into());
    }
    if t.zola.is_none() {
        out.push(format!(
            "zola (install zola {ZOLA_VERSION} from https://github.com/getzola/zola/releases/tag/v{ZOLA_VERSION}, sha256 in release.yml \"Install pinned Zola\"; put it on PATH or set ZOLA=<path>)"
        ));
    }
    if t.node.is_none() {
        out.push("node (install Node.js from https://nodejs.org on PATH)".into());
    }
    out
}

/// The note and the verdict of a step whose tools are `gone` (not empty): `(ok, note)`.
pub fn verdict(gone: &[String], mode: Missing) -> (bool, String) {
    let names: Vec<&str> = gone.iter().map(|g| g.split(' ').next().unwrap_or(g)).collect();
    match mode {
        Missing::Skip => (true, format!("{SKIPPED}{}) - not a pass; install: {}", names.join(", "), gone.join("; "))),
        Missing::Fail => (false, format!("tool missing: {} - the release tier needs it; install: {}", names.join(", "), gone.join("; "))),
    }
}

/// A note when the found zola is not the pinned one.
pub fn zola_note(v: &str) -> Option<String> {
    (!v.contains(ZOLA_VERSION)).then(|| format!("note: {v}, release.yml pins zola {ZOLA_VERSION}"))
}

/// The lines that the gate prints under the step's number: the command, and the missing-tool rule of
/// the tier with the tools that are missing on this host now.
pub fn describe(cwd: &str, python: &str, zola: &str, mode: Missing, gone: &[String]) -> Vec<String> {
    let rule = match mode { Missing::Skip => "SKIP row", Missing::Fail => "FAIL row" };
    let found = if gone.is_empty() { "all tools found".to_string() } else { format!("missing now: {}", gone.join("; ")) };
    vec![
        format!("     (cwd {cwd}) {python} -m unittest discover -s site/tests (ZOLA={zola}, PYTHONUTF8=1; node on PATH)"),
        format!("     (not cargo; a missing tool gives a {rule}; {found})"),
    ]
}

/// Runs the site step `s` (number `n`) with its output in its own log under `log_dir`.
pub(crate) fn execute(root: &Path, log_dir: &Path, dry: bool, n: usize, s: &Step, mode: Missing) -> Result<Done, String> {
    let done = |ok: bool, secs: u64, note: String| Done { name: s.name.clone(), ok, stall: false, secs, note, built: None };
    let tools = find_tools();
    let python = tools.python.clone().unwrap_or_else(|| "python3".into());
    let zola = tools.zola.clone().map(|z| z.0).unwrap_or_else(|| "zola".into());
    let gone = missing(&tools);
    println!("[{n:02}] {}", s.name);
    describe(&leaf(root), &python, &zola, mode, &gone).iter().for_each(|l| println!("{l}"));
    if dry {
        return Ok(done(true, 0, "dry run".into()));
    }
    let log = log_dir.join(crate::exec::log_name(n, &s.name));
    let mut file = File::create(&log).map_err(|e| format!("create {}: {e}", log.display()))?;
    let io = |e: std::io::Error| format!("site step log: {e}");
    if !gone.is_empty() {
        let (ok, note) = verdict(&gone, mode);
        writeln!(file, "{note}").map_err(io)?;
        println!("     {} {note}", if ok { "SKIP" } else { "FAIL" });
        return Ok(done(ok, 0, note));
    }
    let t0 = Instant::now();
    let child = Command::new(&python)
        .args(["-m", "unittest", "discover", "-s", "site/tests"])
        .env("ZOLA", &zola)
        .env("PYTHONUTF8", "1")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(file.try_clone().map_err(io)?)
        .stderr(file.try_clone().map_err(io)?)
        .spawn();
    let limit = crate::watchdog::limit_for(&[], crate::watchdog::from_env().0);
    let (code, stall) = match child.and_then(|mut c| crate::watchdog::wait(&mut c, &file, &|| 0, limit)) {
        Ok(crate::watchdog::End::Stalled) => (-1, true),
        Ok(crate::watchdog::End::Exited(st)) => (crate::exit_status::read(st).0, false),
        Err(e) => {
            writeln!(file, "error: could not start {python}: {e}").map_err(io)?;
            (-1, false)
        }
    };
    let secs = t0.elapsed().as_secs();
    let text = std::fs::read_to_string(&log).unwrap_or_default();
    let mut note = result_line(&text);
    if let Some(z) = tools.zola.as_ref().and_then(|z| zola_note(&z.1)) {
        note = format!("{note}; {z}");
    }
    let ok = code == 0 && !stall;
    println!("     {} in {secs}s {note}", crate::exec::row(ok, stall));
    Ok(Done { name: s.name.clone(), ok, stall, secs, note, built: None })
}

/// The unittest count and verdict lines (`Ran 112 tests ...`, `OK` / `FAILED (failures=..)`), and the
/// first failing test ids.
pub fn result_line(text: &str) -> String {
    let ran = text.lines().find(|l| l.starts_with("Ran ")).unwrap_or("no unittest summary");
    let end = text.lines().rev().find(|l| l.starts_with("OK") || l.starts_with("FAILED")).unwrap_or("");
    let bad: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("FAIL: ").or_else(|| l.strip_prefix("ERROR: ")))
        .filter_map(|l| l.split(' ').next())
        .collect();
    let mut note = format!("{ran}, {end}").trim_end_matches(", ").to_string();
    if !bad.is_empty() {
        let mut uniq = bad.clone();
        uniq.dedup();
        note.push_str(&format!(" [tests: {}]", uniq.into_iter().take(8).collect::<Vec<_>>().join(", ")));
    }
    note
}

fn leaf(d: &Path) -> String {
    d.file_name().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default()
}
