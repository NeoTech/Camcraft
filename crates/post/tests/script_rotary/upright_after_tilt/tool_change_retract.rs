//! #4341: the Z retract before a later preamble's G49 and T M6 is never a
//! reference return under TCPM. A tilted section leaves G43.4 armed; the
//! post cancels it (G49) first, as its Go home does, and lifts Z once.
//! Provenance: the posts' own Go-home order; our synthetic streams.

use super::program;

fn words(l: &str) -> Vec<&str> {
    l.split_whitespace().filter(|w| !w.starts_with('N')).collect()
}

#[test]
fn fanuc_and_doosan_cancel_tcpm_before_the_tool_change_retract() {
    for stem in ["fanuc", "doosan-dnm"] {
        let out = program(stem, 2);
        let prog = &out.program;
        assert!(out.warnings.is_empty(), "{stem}: {:?}\n{prog}", out.warnings);
        let lines: Vec<Vec<&str>> = prog.lines().map(words).collect();
        let arm = lines.iter().position(|w| w.contains(&"G43.4")).unwrap_or_else(|| panic!("{stem}: fixture arms TCPM:\n{prog}"));
        let m6 = lines.iter().position(|w| w.contains(&"T02") && w.contains(&"M6")).unwrap_or_else(|| panic!("{stem}: tool change:\n{prog}"));
        let stop = lines[..m6].iter().rposition(|w| w.contains(&"M5")).unwrap_or_else(|| panic!("{stem}: spindle stop:\n{prog}"));
        assert!(arm < stop, "{stem}: TCPM armed in the first section:\n{prog}");
        let gap = &lines[stop..m6];
        let lifts: Vec<usize> = gap.iter().enumerate()
            .filter(|(_, w)| w.contains(&"G28") && w.contains(&"Z0")).map(|(i, _)| i).collect();
        assert_eq!(lifts.len(), 1, "{stem}: one Z retract before the tool change: {gap:?}");
        let cancel = gap.iter().position(|w| w.contains(&"G49")).unwrap_or_else(|| panic!("{stem}: G49: {gap:?}"));
        assert!(cancel < lifts[0], "{stem}: TCPM off before the reference return: {gap:?}");
    }
}
