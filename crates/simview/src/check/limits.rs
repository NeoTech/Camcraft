//! The limit judgment of `camcraft-sim check` (#2586): each move against
//! the machine's stated strokes and feed ceilings, by the pure machine
//! check of #2585. A stroke of the machine file is in PROGRAM
//! coordinates (`assets/machines/AGENTS.md`, `travel`), so the moves are
//! judged as the program states them, with no work offset. A stroke that
//! the machine file does not state is named as not judged.
//! Provenance: general machine-tool practice; the judgment is
//! `camcraft_post::machine::MachineCheck`.

use camcraft_post::machine::{MachineCheck, SourceLocation};
use camcraft_sim::cl::Toolpath;

use super::inputs::MachineFile;
use super::program::Program;
use super::report::Report;

const CHECK: &str = "machine limits";

/// Judge each move that the run knows against `m`.
pub fn check(program: &Program, m: Option<&MachineFile>, report: &mut Report) {
    let Some(m) = m else {
        return report.not_judged(CHECK, "no machine file is given (--machine)");
    };
    let unstated: Vec<&str> = ["X", "Y", "Z"]
        .iter()
        .enumerate()
        .filter(|(k, _)| m.machine.axis(*k).and_then(|a| a.travel).is_none())
        .map(|(_, l)| *l)
        .collect();
    if !unstated.is_empty() {
        report.not_judged(CHECK, &format!("{} states no stroke for {}: travel is not judged on it", m.name, unstated.join(", ")));
    }
    let (mut judged, mut found) = (0usize, 0usize);
    for s in &program.sections {
        for ((mv, line), skip) in s.moves.iter().zip(&s.lines).zip(&s.skip) {
            if *skip {
                continue;
            }
            let mut c = MachineCheck::default();
            let path = Toolpath::new(vec![*mv]);
            // The NC moves are program coordinates already: the origin is zero.
            c.check_path(&m.machine, Some(SourceLocation::NcLine(*line)), &path, [0.0; 3], [false; 3], true);
            for p in &c.problems {
                found += 1;
                report.fail(CHECK, &p.replacen(": move 0 ", ": the move ", 1));
            }
            judged += 1;
        }
    }
    if judged > 0 && found == 0 {
        let what = if unstated.is_empty() { "each move is inside the strokes and feed ceilings" } else { "each move is inside the stated strokes and feed ceilings" };
        report.pass(CHECK, &format!("{judged} moves: {what} of {}", m.name));
    }
}
