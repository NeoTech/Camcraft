//! The `check` suite of `camcraft-simview` (#2586): the `camcraft-sim`
//! binary run as a user runs it, each exit code and finding pinned.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory
//! suite"); each module keeps its own provenance and Gate map.

mod catalog;
mod cl_cmd;
mod exits;
mod findings;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The repository root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A path under the repository root, as a string.
pub fn at(rel: &str) -> String {
    root().join(rel).to_string_lossy().into_owned()
}

/// The test machine with a part tree, strokes and a spindle body.
pub fn tree_mch() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vmc-tree.mch").to_string_lossy().into_owned()
}

/// Write `body` to a temporary file named for this process and `name`.
pub fn tmp(name: &str, body: &str) -> String {
    let p = std::env::temp_dir().join(format!("cc-simview-{}-{name}", std::process::id()));
    std::fs::write(&p, body).expect("write fixture");
    p.to_string_lossy().into_owned()
}

/// Run `camcraft-sim check` with `args`.
pub fn check(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_camcraft-sim")).arg("check").args(args).output().expect("spawn")
}

/// The stdout and stderr of `o`, for a failure message.
pub fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// A fanuc-style 3-axis program: one 10 mm tool cuts a 2 mm slot
/// from X-30 to X30 at Y0. `extra` goes after the slot (line 12 on).
pub fn program(extra: &str) -> String {
    format!(
        "%\nO0100 (TEST)\nG21\nG90 G17 G94 G54 G40 G49 G80\nT1 M6\nG43 H1\nS8000 M3\n\
G0 X-30. Y0. Z10.\nG0 Z2.\nG1 Z-2. F300.\nG1 X30. F600.\n{extra}G0 Z10.\nM5\nM30\n%\n"
    )
}

/// The stock and tool arguments that fit [`program`].
pub const STOCK: &str = "-40,-20,-10,40,20,0";
pub const TOOLS: &str = "T1=10:flute=20:holder=40x50";
