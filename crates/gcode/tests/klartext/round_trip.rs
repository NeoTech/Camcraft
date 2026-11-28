//! Round trip: the recorded Klartext post golden reads back into the motion of the recorded CL stream that the post was given.

use camcraft_cl::cl::MoveKind;
use camcraft_gcode::{ProgramEnd, SpindleDir, WorkOffset};

use super::{assert_to, moves, near, ok};

/// The golden that `crates/post/tests/script_heidenhain_klartext.rs` writes from `contour.cnc`.
const GOLDEN: &str =
    include_str!("../../../post/tests/recorded/golden/heidenhain-klartext/picomax.nc");
/// The CL stream (machine frame, micrometres, feed in micrometres per minute) the post read.
const STREAM: &str = include_str!("../../../post/tests/klartext/contour.cnc");

#[derive(Debug)]
struct StreamMove {
    rapid: bool,
    to: [f64; 3],
    feed: Option<f64>,
    arc: Option<(bool, [f64; 2])>,
}

fn param(name: &str) -> f64 {
    STREAM
        .lines()
        .find_map(|l| l.strip_prefix(&format!("PARAM {name} ")))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| panic!("stream has no PARAM {name}"))
}

/// The section moves of the stream, moved into the work frame (minus the origin).
fn stream_moves() -> Vec<StreamMove> {
    let origin = [param("OriginX"), param("OriginY"), param("OriginZ")];
    let mut out = Vec::new();
    for l in STREAM.lines().skip_while(|l| *l != "SECTION").skip(1) {
        if l == "ENDSECTION" {
            break;
        }
        let toks: Vec<&str> = l.split_whitespace().collect();
        let word = |w: &str| {
            toks.iter()
                .position(|t| *t == w)
                .and_then(|i| toks.get(i + 1))
                .map(|v| v.parse::<f64>().expect("stream number") / 1000.0)
        };
        let xyz = |k: usize, w: &str| word(w).expect("stream axis") - origin[k];
        let to = [xyz(0, "X"), xyz(1, "Y"), xyz(2, "Z")];
        let arc = (toks[0] == "Arc").then(|| {
            let ccw = toks[1] == "CCW";
            (
                ccw,
                [
                    word("I").expect("I") - origin[0],
                    word("J").expect("J") - origin[1],
                ],
            )
        });
        out.push(StreamMove {
            rapid: toks[0] == "Rapid",
            to,
            feed: word("F"),
            arc,
        });
    }
    out
}

#[test]
fn klartext_post_golden_reads_back_into_the_stream_motion() {
    let p = ok(GOLDEN);
    let expected = stream_moves();
    assert_eq!(expected.len(), 6, "the stream fixture changed");
    let got = moves(&p);
    assert!(got.len() >= expected.len());
    let tail = &got[got.len() - expected.len()..];
    for (m, e) in tail.iter().zip(&expected) {
        assert_to(m, e.to[0], e.to[1], e.to[2]);
        assert_eq!(
            m.kind == MoveKind::Rapid,
            e.rapid,
            "rapid/feed kind of {e:?}"
        );
        match (m.feed, e.feed) {
            (Some(a), Some(b)) => assert!(near(a, b), "feed {a} vs {b}"),
            (None, None) => {}
            (a, b) => panic!("feed {a:?} vs {b:?} for {e:?}"),
        }
        match (&m.arc, e.arc) {
            (Some(a), Some((ccw, c))) => {
                assert_eq!(a.ccw, ccw);
                assert!(
                    near(a.center.x, c[0]) && near(a.center.y, c[1]),
                    "centre {:?} vs {c:?}",
                    a.center
                );
            }
            (None, None) => {}
            (a, b) => panic!("arc {a:?} vs {b:?}"),
        }
    }
    // The approach before the stream motion: M140 MB MAX, then XY, then Z, all rapid and above the first stream point.
    let approach = &got[..got.len() - expected.len()];
    assert_eq!(approach.len(), 2);
    assert!(
        approach
            .iter()
            .all(|m| m.kind == MoveKind::Rapid && m.to.z >= expected[0].to[2] - 1e-9)
    );
    assert_to(approach[0], 0.0, 0.0, 100.0);

    // Program facts: tool 2 at 3500 rpm on preset 2 (G55), three M0 stops, M2 end.
    let last = p.sections.last().expect("a section");
    assert_eq!(
        (last.tool_number, last.spindle_rpm, last.spindle_dir),
        (Some(2), Some(3500.0), SpindleDir::Cw)
    );
    assert_eq!(last.work_offset, WorkOffset::G55);
    assert_eq!(p.tools, vec![2]);
    assert_eq!(p.ended, Some(ProgramEnd::M2));
    assert_eq!(
        p.stops.iter().map(|s| s.line).collect::<Vec<_>>(),
        vec![12, 21, 26]
    );
    // Block 32 (the DR+ arc) is on physical line 34: line 14 is the Q339 continuation line.
    let arc_line = last.move_lines[last.move_lines.len() - 4];
    assert_eq!(arc_line, 34);
}

#[test]
fn klartext_post_golden_needs_the_traverse_end_for_m140() {
    let e = super::parse(GOLDEN, &camcraft_gcode::KlartextOptions::default())
        .expect_err("M140 MB MAX needs the option");
    assert!(
        matches!(
            e,
            camcraft_gcode::GcodeError::MissingReferencePosition { line: 23, .. }
        ),
        "{e}"
    );
    assert!(e.to_string().contains("block 21"), "{e}");
}
