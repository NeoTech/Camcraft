//! The Klartext integration suite of `camcraft-gcode`: HEIDENHAIN conversational text in, `cl::Toolpath` sections or a refusal that names the block out.
//! Provenance: HEIDENHAIN iTNC 530 conversational manual 533 190-22 and the TNC cycle manuals (behaviour only, via docs/research/heidenhain/), and CamCraft's recorded Klartext post golden.
//! Gate map: covers camcraft_gcode::{klartext, types, section, arc, cycle_expand, domain, error}.

mod cycles;
mod degenerate;
mod paths;
mod refusals;
mod round_trip;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use camcraft_cl::cl::Move;
use camcraft_gcode::{GcodeError, KlartextOptions, ParsedProgram, parse_klartext_with};

/// Run `f` on a worker thread. Fail on a panic, or when no answer arrives in
/// `secs` seconds (a hang is a failure, not a slow pass).
pub fn in_time<T: Send + 'static>(
    secs: u64,
    what: &str,
    f: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(v) => {
            assert!(worker.join().is_ok(), "{what}: panicked");
            v
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!("{what}: panicked"),
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("{what}: no answer in {secs} s"),
    }
}

/// Options with a tool-axis traverse end at Z+100 (for `M140 MB MAX`).
pub fn opts() -> KlartextOptions {
    KlartextOptions {
        tool_axis_max_z: Some(100.0),
        ..KlartextOptions::default()
    }
}

pub fn parse(nc: &str, o: &KlartextOptions) -> Result<ParsedProgram, GcodeError> {
    let (nc, o) = (nc.to_string(), o.clone());
    in_time(5, "parse_klartext", move || parse_klartext_with(&nc, &o))
}

pub fn ok(nc: &str) -> ParsedProgram {
    match parse(nc, &opts()) {
        Ok(p) => p,
        Err(e) => panic!("expected a parse, got {e} for\n{nc}"),
    }
}

pub fn refusal(nc: &str) -> GcodeError {
    match parse(nc, &opts()) {
        Ok(p) => panic!(
            "expected a refusal, got {} sections for\n{nc}",
            p.sections.len()
        ),
        Err(e) => e,
    }
}

/// Wrap body lines in `BEGIN PGM T MM` / `END PGM T MM` and number them.
/// A body line that starts with a space is a continuation line (no number).
/// With no continuation lines, block n is on physical line n + 1.
pub fn pgm(body: &[&str]) -> String {
    let mut out = String::from("0 BEGIN PGM T MM\n");
    let mut n = 1;
    for l in body {
        if l.starts_with(' ') {
            out.push_str(l);
        } else {
            out.push_str(&format!("{n} {l}"));
            n += 1;
        }
        out.push('\n');
    }
    out.push_str(&format!("{n} END PGM T MM\n"));
    out
}

pub fn moves(p: &ParsedProgram) -> Vec<&Move> {
    p.sections
        .iter()
        .flat_map(|s| s.toolpath.moves.iter())
        .collect()
}

pub fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6
}

/// Assert a move ends at (x, y, z).
pub fn assert_to(m: &Move, x: f64, y: f64, z: f64) {
    assert!(
        near(m.to.x, x) && near(m.to.y, y) && near(m.to.z, z),
        "move ends at {:?}, expected ({x}, {y}, {z})",
        m.to
    );
}
