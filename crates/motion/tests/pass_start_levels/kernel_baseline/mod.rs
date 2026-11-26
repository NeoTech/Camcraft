//! #4153: the old-planner baseline of each phase-1 kind of the path kernel
//! (#4023, `docs/decisions/4024-path-kernel-design.md`). Each pin records
//! the output of the planner that is on `side/path-kernel` before the
//! kernel takes the kind over: the CL text at 1 um and one `; note:` line
//! for each note, the form of the #3750 baseline. The switch-over proof
//! compares the new kernel with these files. A golden must not move
//! unless the output change is intended. Regenerate with
//! `WRITE_GOLDEN=1 cargo test -p camcraft-motion --test pass_start_levels kernel_baseline`,
//! then read the diff in crates/motion/tests/golden/ (files `kernel_*`).
//!
//! Gate map (#4153): covers camcraft_motion::machining2d (face, contour,
//! pocket, adaptive_troch, trace, trace_avoid, v_carve, chamfer,
//! circular, slot, trochoid_slot, undercut, thread_mill, bore),
//! camcraft_motion::machining3d (pocket3d).
//! Provenance: our own fixtures and recordings of our own planners; no
//! vendor material consulted.

mod adaptive;
mod bore;
mod chamfer;
mod circular;
mod contour;
mod engrave;
mod face;
mod pocket3d;
mod slot;
mod thread;
mod trace;
mod trochoid_slot;
mod undercut;
mod v_carve;

use camcraft_motion::cl::{Toolpath, text::to_text};
use camcraft_motion::geo2d::{Loop, Point2, Region};

/// Compares `text` with `tests/golden/<name>`, or writes it under
/// `WRITE_GOLDEN`.
pub(super) fn golden(name: &str, text: &str) {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(&file, text).expect("cannot write the golden file");
        return;
    }
    let want = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("cannot read golden {}: {e}", file.display()));
    assert!(want.replace("\r\n", "\n") == text, "golden {} moved: the old planner's output changed", file.display());
}

/// The CL text of `path` with its notes, one `; note:` line each.
pub(super) fn dump(path: &Toolpath) -> String {
    let mut s = to_text(path).expect("the path has a CL text form");
    for n in &path.notes {
        s.push_str(&format!("; note: {}\n", n.replace('\n', " ")));
    }
    s
}

/// Records `path` as `kernel_<kind>_<case>.cl`.
pub(super) fn pin(kind: &str, case: &str, path: &Toolpath) {
    golden(&format!("kernel_{kind}_{case}.cl"), &dump(path));
}

pub(super) fn poly(pts: &[(f64, f64)]) -> Loop {
    Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()).expect("fixture loop")
}

pub(super) fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    poly(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
}

/// A closed circle of 128 points round (`cx`, `cy`).
pub(super) fn disc(cx: f64, cy: f64, r: f64) -> Loop {
    let pts: Vec<(f64, f64)> = (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    poly(&pts)
}

pub(super) fn region(loops: Vec<Loop>) -> Region {
    Region::new(loops).expect("fixture region")
}
