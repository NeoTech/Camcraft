//! Degenerate input: truncated programs, bad numbers, huge files and seeded random mutations give a result or a refusal, never a panic or a hang.

use camcraft_gcode::{GcodeError, parse_klartext_with};

use super::{in_time, opts, pgm, refusal};

const GOLDEN: &str =
    include_str!("../../../post/tests/recorded/golden/heidenhain-klartext/picomax.nc");

#[test]
fn klartext_every_truncation_of_the_golden_is_refused_without_a_panic() {
    let n = in_time(20, "truncations", || {
        let o = opts();
        let mut refused = 0usize;
        let end = GOLDEN.rfind("END PGM").expect("END PGM in the golden");
        for cut in (0..end).filter(|i| GOLDEN.is_char_boundary(*i)) {
            if parse_klartext_with(&GOLDEN[..cut], &o).is_err() {
                refused += 1;
            }
        }
        (refused, end)
    });
    assert_eq!(n.0, n.1, "a program cut before END PGM must be refused");
}

#[test]
fn klartext_bad_numbers_are_refused() {
    let lines = [
        "L X+1.2.3 FMAX",
        "L X++1 FMAX",
        "L X+1E5 FMAX",
        "L X+INF FMAX",
        "L XNAN FMAX",
        "L X+ FMAX",
        "L X+. FMAX",
        "L X+99999999999 FMAX",
        "L X+1 X+2 FMAX",
        "L X+1 IX+2 FMAX",
        "L X+1 F0",
        "L X+1 F-5",
        "L X+1 F+999999999",
        "TOOL CALL 1 Z S-1",
        "Q2000 = 1",
        "L X+Q7 FMAX",
        "L X+\u{ff11}\u{ff10} FMAX",
        "TOOL CALL \"unterminated Z",
        "TOOL CALL 99999999999 Z",
        "CYCL DEF 200 X Q200=2 Q200=3",
        "CYCL DEF 99999999999",
    ];
    let long = format!("L X+1{} FMAX", "0".repeat(400));
    for line in lines.iter().copied().chain(std::iter::once(long.as_str())) {
        let e = refusal(&pgm(&[line]));
        assert_eq!(e.line(), 2, "{line}: {e}");
    }
    let e = refusal("0 BEGIN PGM T MM\n99999999999999999999999 L X+1 FMAX\n");
    assert!(matches!(e, GcodeError::Syntax { line: 2, .. }), "{e}");
    let e = refusal("0 BEGIN PGM T MM\n1 CYCL DEF 247 ~\n");
    assert!(matches!(e, GcodeError::Syntax { line: 2, .. }), "{e}");
}

#[test]
fn klartext_huge_programs_parse_or_refuse_in_time() {
    let mut body = String::from("0 BEGIN PGM BIG MM\n1 L X+0 Y+0 Z+1 R0 F1000\n");
    let n = 200_000;
    for i in 0..n {
        let x = if i % 2 == 0 { "+1" } else { "+0" };
        body.push_str(&format!("{} L X{x} Y+{}\n", i + 2, i % 7));
    }
    body.push_str(&format!("{} END PGM BIG MM\n", n + 2));
    let p = in_time(30, "200k blocks", move || {
        parse_klartext_with(&body, &opts())
    });
    let p = p.expect("a long program parses");
    assert_eq!(p.sections[0].toolpath.moves.len(), n + 1);

    // One huge block and a long chain of continuation lines that ends truncated.
    let wide = format!("0 BEGIN PGM T MM\n1 L {} FMAX\n", "X+1 ".repeat(250_000));
    let e = in_time(30, "wide block", move || {
        parse_klartext_with(&wide, &opts())
    });
    assert!(e.is_err());
    let chain = format!(
        "0 BEGIN PGM T MM\n1 CYCL DEF 247 ~\n{}",
        "  Q339=+1 ~\n".repeat(100_000)
    );
    let e = in_time(30, "continuation chain", move || {
        parse_klartext_with(&chain, &opts())
    });
    assert!(matches!(e, Err(GcodeError::Syntax { line: 2, .. })));
}

#[test]
fn klartext_seeded_mutations_of_the_golden_never_panic() {
    in_time(60, "mutations", || {
        let o = opts();
        let alphabet: &[u8] = b"0123456789+-.~;\"QXYZFMRCLDB =\n \t";
        let mut seed: u64 = 0x2227_4b4c_4152_0001;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) as usize
        };
        for _ in 0..3000 {
            let mut bytes = GOLDEN.as_bytes().to_vec();
            for _ in 0..(1 + next() % 4) {
                let at = next() % bytes.len();
                match next() % 3 {
                    0 => bytes[at] = alphabet[next() % alphabet.len()],
                    1 => {
                        let len = (next() % 12).min(bytes.len() - at);
                        bytes.drain(at..at + len);
                    }
                    _ => bytes.insert(at, alphabet[next() % alphabet.len()]),
                }
                if bytes.is_empty() {
                    bytes.push(b'0');
                }
            }
            let text = String::from_utf8_lossy(&bytes);
            let _ = parse_klartext_with(&text, &o);
        }
    });
}

#[test]
fn klartext_empty_and_comment_only_input_is_refused() {
    for nc in ["", "\n\n", "; only a comment\n", "0 BEGIN PGM T MM\n"] {
        let e = refusal(nc);
        assert!(matches!(e, GcodeError::Syntax { .. }), "{nc:?}: {e}");
    }
}
