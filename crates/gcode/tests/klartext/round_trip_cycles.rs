//! Round trip of the cycle golden: every block that the Klartext post writes for drilling, pecking, the cycle 9 dwell and tapping (`picomax_cycles.nc`) reads back into the motion and the cycle records of the CL stream that the post was given (`cycles.cnc`). Every `.nc` golden of the Klartext post must parse.

use camcraft_cl::cl::{CycleKind, MoveKind};
use camcraft_gcode::{ProgramEnd, SpindleDir};

use super::{moves, near, ok};

/// The golden that `crates/post/tests/klartext/cycle_tests.rs` writes from `cycles.cnc`.
const GOLDEN: &str =
    include_str!("../../../post/tests/recorded/golden/heidenhain-klartext/picomax_cycles.nc");
/// The CL stream (machine frame, micrometres, feed in micrometres per minute) the post read.
const STREAM: &str = include_str!("../../../post/tests/klartext/cycles.cnc");

/// The folder of the recorded Klartext post goldens.
const GOLDEN_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../post/tests/recorded/golden/heidenhain-klartext"
);

fn param(name: &str) -> f64 {
    STREAM
        .lines()
        .find_map(|l| l.strip_prefix(&format!("PARAM {name} ")))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| panic!("stream has no PARAM {name}"))
}

/// The value after word `w` in a stream line, in mm (or mm/min, or s for `P`).
fn word(toks: &[&str], w: &str) -> Option<f64> {
    toks.iter()
        .position(|t| *t == w)
        .and_then(|i| toks.get(i + 1))
        .map(|v| v.parse::<f64>().expect("stream number") / 1000.0)
}

/// `(rapid, to)` and the feed of each stream move, per section, in the work frame.
type StreamMove = (bool, [f64; 3], Option<f64>);

fn stream_sections(origin: [f64; 3]) -> (Vec<Vec<StreamMove>>, Vec<Vec<String>>) {
    let (mut sections, mut cycles) = (Vec::new(), Vec::new());
    let (mut cur, mut cyc): (Vec<StreamMove>, Vec<String>) = (Vec::new(), Vec::new());
    let mut inside = false;
    for l in STREAM.lines() {
        match l {
            "SECTION" => inside = true,
            "ENDSECTION" => {
                inside = false;
                sections.push(std::mem::take(&mut cur));
                cycles.push(std::mem::take(&mut cyc));
            }
            _ if inside && l.starts_with("CYCLE ") => cyc.push(l.to_string()),
            _ if inside => {
                let toks: Vec<&str> = l.split_whitespace().collect();
                let axis = |k: usize, w: &str| word(&toks, w).expect("stream axis") - origin[k];
                let to = [axis(0, "X"), axis(1, "Y"), axis(2, "Z")];
                cur.push((toks[0] == "Rapid", to, word(&toks, "F")));
            }
            _ => {}
        }
    }
    (sections, cycles)
}

#[test]
fn klartext_cycle_golden_reads_back_into_the_stream_motion_and_cycles() {
    let p = ok(GOLDEN);
    let origin = [param("OriginX"), param("OriginY"), param("OriginZ")];
    let (sections, stream_cycles) = stream_sections(origin);
    assert_eq!(sections.len(), 5, "the stream fixture changed");
    let got = moves(&p);
    let same = |k: usize, e: &StreamMove| {
        let m = got[k];
        (m.kind == MoveKind::Rapid) == e.0
            && near(m.to.x, e.1[0])
            && near(m.to.y, e.1[1])
            && near(m.to.z, e.1[2])
            && match (m.feed, e.2) {
                (Some(a), Some(b)) => near(a, b),
                (None, None) => true,
                _ => false,
            }
    };
    // Each stream section is a contiguous run of the read moves, in order.
    // The moves between the runs are the post's approach: rapids at or
    // above the first point of the next section (M140 MB MAX, then XY).
    let mut at = 0usize;
    for (i, sec) in sections.iter().enumerate() {
        let start = (at..=got.len().saturating_sub(sec.len()))
            .find(|&k| sec.iter().enumerate().all(|(j, e)| same(k + j, e)))
            .unwrap_or_else(|| panic!("stream section {i} not found after move {at}: {sec:?}"));
        for m in &got[at..start] {
            assert!(
                m.kind == MoveKind::Rapid && m.to.z >= sec[0].1[2] - 1e-9,
                "section {i}: approach move {m:?} is not a rapid above the section start"
            );
        }
        at = start + sec.len();
    }
    assert_eq!(at, got.len(), "moves after the last stream section");

    // The cycle records: the post writes DRILL as cycle 200, TAP as 207 and
    // FLOATINGTAP as 206; PECK and DEEP are written as explicit L blocks.
    let read: Vec<_> = p
        .sections
        .iter()
        .flat_map(|s| s.toolpath.cycles.iter().copied())
        .collect();
    let expected: Vec<String> = stream_cycles
        .iter()
        .flatten()
        .filter(|l| !l.starts_with("CYCLE PECK") && !l.starts_with("CYCLE DEEP"))
        .cloned()
        .collect();
    assert_eq!(read.len(), expected.len(), "{read:#?}");
    for (c, l) in read.iter().zip(&expected) {
        let toks: Vec<&str> = l.split_whitespace().collect();
        let kind = match toks[1] {
            "DRILL" => CycleKind::Drill,
            "TAP" => CycleKind::Tap,
            "FLOATINGTAP" => CycleKind::FloatingTap,
            other => panic!("cycle kind {other}"),
        };
        let w = |k: &str| word(&toks, k).expect("cycle word");
        assert_eq!(c.kind, kind, "{l}");
        assert!(near(c.xy.x, w("X") - origin[0]) && near(c.xy.y, w("Y") - origin[1]), "{l}");
        assert!(near(c.r_z, w("R") - origin[2]), "{l}: r_z {}", c.r_z);
        assert!(near(c.bottom_z, w("Z") - origin[2]), "{l}: bottom {}", c.bottom_z);
        assert!(near(c.entry_z, w("ENTRY") - origin[2]), "{l}: entry {}", c.entry_z);
        assert!(near(c.dwell, w("P")), "{l}: dwell {}", c.dwell);
    }

    // The deep-hole dwell (stream P 300 ms) is cycle 9, after the last infeed.
    let deep = stream_cycles
        .iter()
        .flatten()
        .find(|l| l.starts_with("CYCLE DEEP"))
        .expect("a deep cycle in the stream");
    let toks: Vec<&str> = deep.split_whitespace().collect();
    assert_eq!(p.dwells.len(), 1, "{:?}", p.dwells);
    let d = p.dwells[0];
    assert!(near(d.seconds, word(&toks, "P").expect("P")), "{d:?}");
    let after = p.sections[d.section].toolpath.moves[d.after_move.expect("after a move")];
    assert!(near(after.to.z, word(&toks, "Z").expect("Z") - origin[2]), "{after:?}");

    // Program facts: tools 3, 4 and 5; one stop for each M0 block; M2 end.
    assert_eq!(p.tools, vec![3, 4, 5]);
    let m0 = GOLDEN.lines().filter(|l| l.ends_with(" M0")).count();
    assert_eq!(p.stops.len(), m0);
    assert_eq!(p.ended, Some(ProgramEnd::M2));
    let tap = p
        .sections
        .iter()
        .find(|s| !s.toolpath.cycles.is_empty() && s.toolpath.cycles[0].kind == CycleKind::Tap)
        .expect("the rigid tap section");
    assert_eq!(
        (tap.tool_number, tap.spindle_rpm, tap.spindle_dir),
        (Some(4), Some(500.0), SpindleDir::Cw)
    );
}

#[test]
fn klartext_every_post_golden_reads_back_without_a_refused_block() {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(GOLDEN_DIR).expect("the Klartext golden folder") {
        let path = entry.expect("a folder entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("nc") {
            continue; // .refusal files hold the post's refusal text, not a program
        }
        let nc = std::fs::read_to_string(&path).expect("a golden");
        let p = ok(&nc);
        assert!(!p.sections.is_empty(), "{}", path.display());
        names.push(path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string());
    }
    names.sort();
    for known in ["picomax.nc", "picomax_cycles.nc"] {
        assert!(names.iter().any(|n| n == known), "{known} missing from {names:?}");
    }
}
