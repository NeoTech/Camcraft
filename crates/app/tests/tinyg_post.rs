//! The Synthetos TinyG post: the program shape a TinyG reads (no % fence,
//! no O word, no N numbers, M30; only the word letters and G/M codes its
//! wiki lists), no tool length word, `T<n> M6` then `M0` after a Z lift
//! (M6 moves nothing on TinyG, so the stop is the operator's change),
//! drilling posted as the plain-move chain with `G4 P<s>` at the bottom
//! (no canned cycles), and the words TinyG does not have - G84, G41/G42,
//! tools over 31 - refused by name. The Z lift never trusts machine Z0
//! unless the machine file states `safe_retract` (grblhal.cps's law: an
//! unhomed board's machine zero is its power-on height). The generic
//! machine file mounts and posts clean.
//! Provenance: our own tests over our own post and machine file; the
//! dialect facts from TinyG's public wiki ("Gcode Support").
//!
//! Gate map: covers app::post (tinyg.cps), post::machine (safeRetractZ,
//! units), app::posting (registry, machine library).

use camcraft_app::machine::parse_mch;
use camcraft_app::post::{engine, machine};
use camcraft_app::posting::{BUILTINS, MACHINES};
use serde_json::{Value, json};
use std::path::Path;

const STREAM: &str = "PARAM ClearancePlane 15\nPARAM StockZTop 0\nPARAM SpindleSpeed 8000\nTOOL 3 EM6\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 10000 Y 0 Z -1000 F 500000\nENDSECTION\n\
                      PARAM SpindleSpeed 6000\nTOOL 4 DRILL5\nSECTION\nCYCLE DRILL X 20000 Y 5000 Z -3000 R 2000 P 1500 F 100000\nRapid X 20000 Y 5000 Z 2000\nPlunge X 20000 Y 5000 Z -3000 F 100000\nRapid X 20000 Y 5000 Z 2000\nENDSECTION\n";

/// The G and M codes TinyG's wiki lists (the dotted ones spelled as the
/// board spells them), and the word letters a block may carry.
const CODES: &[&str] = &[
    "G0", "G1", "G2", "G3", "G4", "G10", "G17", "G18", "G19", "G20", "G21", "G28", "G28.1", "G28.2", "G28.3",
    "G28.4", "G30", "G30.1", "G38.2", "G40", "G43.1", "G49", "G53", "G54", "G55", "G56", "G57", "G58", "G59",
    "G61", "G61.1", "G64", "G80", "G90", "G91", "G90.1", "G91.1", "G92", "G92.1", "G92.2", "G92.3", "G93",
    "G94", "M0", "M1", "M2", "M30", "M60", "M3", "M4", "M5", "M6", "M7", "M8", "M9", "M105", "M114", "M115",
    "M201.3", "M400",
];
const LETTERS: &str = "GMTFPSXYZABCIJKLRN";

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn cps() -> String {
    read("assets/posts/tinyg.cps")
}

/// The shipped generic machine, mounted the way the app hands it on.
fn generic() -> Value {
    let text = MACHINES
        .iter()
        .find(|(n, _)| *n == "TinyG 3-axis")
        .map(|(_, t)| *t)
        .expect("the library ships the TinyG machine");
    machine::mirror(&parse_mch(text).unwrap_or_else(|e| panic!("parses: {e}")))
}

fn post(m: &Value, stream: &str) -> String {
    let out = engine::run(&cps(), m, stream).unwrap_or_else(|e| panic!("posts: {e}"));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.program
}

/// Every word of every non-comment line is a letter TinyG accepts, and
/// every G or M code is one its wiki lists.
fn assert_tinyg_words(nc: &str) {
    for l in nc.lines() {
        if l.starts_with('(') {
            assert!(l.ends_with(')') && l.matches('(').count() == 1, "one closed comment: {l:?}");
            assert!(!l.contains('!') && !l.contains(';') && !l.contains('%'), "a control character in a comment: {l:?}");
            continue;
        }
        assert!(l.len() <= 191, "over TinyG's line length: {l:?}");
        for w in l.split_whitespace() {
            let letter = w.chars().next().expect("a word");
            assert!(LETTERS.contains(letter), "{letter} is not a TinyG word letter: {l:?}\n{nc}");
            if letter == 'G' || letter == 'M' {
                assert!(CODES.contains(&w), "{w} is not in TinyG's table: {l:?}\n{nc}");
            }
        }
    }
}

#[test]
fn the_registry_and_the_library_ship_it() {
    assert!(BUILTINS.iter().any(|(n, s)| *n == "TinyG" && *s == cps()), "the post is built in");
    let m = generic();
    assert!(m.get("safeRetractZ").is_none_or(Value::is_null), "the generic file assumes no homing: {m}");
}

#[test]
fn the_program_speaks_only_what_tinyg_lists() {
    let nc = post(&generic(), STREAM);
    let lines: Vec<&str> = nc.lines().collect();
    assert_eq!(lines.first(), Some(&"(TinyG 3-axis)"), "{nc}");
    assert_eq!(lines.get(1), Some(&"G21 G17 G90 G91.1 G94 G64"), "{nc}");
    assert_eq!(lines.last(), Some(&"M30"), "{nc}");
    assert_tinyg_words(&nc);
    for l in &lines {
        let first = l.split_whitespace().next().unwrap_or("");
        assert!(!first.starts_with('%') && !first.starts_with('O') && !first.starts_with('N'), "{l:?}");
        for w in l.split_whitespace() {
            // No length offset (the operator's), no predefined positions, no cycle words.
            assert!(!["G43", "G43.1", "G49", "G28", "G30", "G80", "G81", "G82", "G83", "G98", "G84"].contains(&w), "{w} in {l:?}\n{nc}");
        }
    }
    for golden in ["contour_disc_hole", "drill_peck", "pocket_zigzag_plate_with_island", "arc_planes"] {
        let nc = post(&json!({"safeRetractZ": 0.0}), &read(&format!("crates/app/tests/golden/{golden}.cl")));
        assert_tinyg_words(&nc);
    }
}

#[test]
fn each_tool_change_lifts_z_then_selects_the_tool_and_stops() {
    let nc = post(&generic(), STREAM);
    for call in ["T3 M6", "T4 M6"] {
        // Z up in the work offset (no homing assumed), the tool word alone,
        // then the stop that is the operator's change; the spindle starts
        // only after it.
        assert!(nc.contains(&format!("G0 Z15\n{call}\nM0\nS")), "{call}: lift, call, stop, spindle\n{nc}");
    }
    assert_eq!(nc.matches("\nM0\n").count(), 2, "one stop per tool change and no other\n{nc}");
    assert_eq!(nc.matches("G0 Z15\n").count(), 3, "two changes and the end\n{nc}");
    assert!(!nc.contains("G53"), "no machine-coordinate move without homing\n{nc}");
    // X Y move first after a change, at the safe height; Z comes down after.
    let lines: Vec<&str> = nc.lines().collect();
    let t4 = lines.iter().position(|l| *l == "T4 M6").expect("T4");
    let first_motion = lines[t4 + 1..].iter().find(|l| l.starts_with("G0 ") || l.starts_with("G1 ")).expect("a move");
    assert_eq!(*first_motion, "G0 X20 Y5", "{nc}");
}

#[test]
fn drilling_posts_as_plain_moves_with_the_dwell_at_the_bottom() {
    let nc = post(&generic(), STREAM);
    // The cycle record itself writes nothing; its chain is the motion.
    assert!(nc.contains("G0 X20 Y5 Z2\nG1 X20 Y5 Z-3 F100\nG4 P1.5\nG0 X20 Y5 Z2\n"), "{nc}");
    // The committed drilling goldens post their chains, never a cycle word.
    let m = json!({"safeRetractZ": 0.0});
    for golden in ["drill_plate", "drill_peck"] {
        let nc = post(&m, &read(&format!("crates/app/tests/golden/{golden}.cl")));
        assert!(nc.lines().filter(|l| l.starts_with("G1 ")).count() >= 2, "{golden}: the plunges post\n{nc}");
        assert!(!nc.contains("G8") && !nc.contains("G98"), "{golden}: no cycle word\n{nc}");
    }
    // A dwell whose chain never reaches the bottom is refused, not dropped.
    let short = STREAM.replacen("Plunge X 20000 Y 5000 Z -3000 F 100000", "Plunge X 20000 Y 5000 Z -2000 F 100000", 1);
    let out = engine::run(&cps(), &generic(), &short).expect("runs");
    assert!(out.warnings.iter().any(|w| w.contains("dwell") && w.contains("refused")), "{:?}", out.warnings);
    assert!(!out.program.contains("G4"), "{}", out.program);
}

#[test]
fn a_tap_is_refused_by_name() {
    let stream = read("crates/app/tests/golden/tap_plate.cl");
    let out = engine::run(&cps(), &generic(), &stream).expect("the script runs");
    let taps: Vec<&String> = out.warnings.iter().filter(|w| w.contains("tapping")).collect();
    assert_eq!(taps.len(), 1, "one named refusal per program: {:?}", out.warnings);
}

#[test]
fn a_tool_number_over_thirty_one_is_refused_by_name() {
    let run = |tool: u32| {
        let stream = STREAM.replacen("TOOL 3 EM6", &format!("TOOL {tool} EM6"), 1);
        engine::run(&cps(), &generic(), &stream).expect("runs")
    };
    assert!(run(31).warnings.is_empty(), "{:?}", run(31).warnings);
    assert!(run(31).program.contains("T31 M6\n"));
    let stray = run(32);
    assert!(stray.warnings.iter().any(|w| w.contains("1-31")), "{:?}", stray.warnings);
}

#[test]
fn a_homed_machine_lifts_in_machine_coordinates() {
    for z in [0.0, -5.0] {
        let mut m = generic();
        m["safeRetractZ"] = json!(z);
        let nc = post(&m, STREAM);
        let want = format!("G53 G0 Z{z}\nT3 M6\nM0\n");
        assert!(nc.contains(&want), "{want:?}\n{nc}");
        assert_eq!(nc.matches("G53 G0 Z").count(), 3, "{nc}");
    }
}

#[test]
fn without_homing_an_unknown_or_buried_lift_height_is_refused_by_name() {
    // No machine at all assumes no homing either.
    let nc = post(&Value::Null, STREAM);
    assert!(nc.contains("G0 Z15\nT3 M6\n") && !nc.contains("G53"), "{nc}");
    let refused = |stream: &str| {
        let out = engine::run(&cps(), &generic(), stream).expect("the script runs");
        assert!(!out.program.contains("G53"), "nothing guessed:\n{}", out.program);
        out.warnings
    };
    let w = refused(&STREAM.replacen("PARAM ClearancePlane 15\n", "", 1));
    assert!(w.iter().any(|w| w.contains("no clearance height is known")), "{w:?}");
    let w = refused(&STREAM.replacen("PARAM StockZTop 0", "PARAM StockZTop 20", 1));
    assert!(w.iter().any(|w| w.contains("is not above the stock top")), "{w:?}");
}

#[test]
fn the_after_operation_stops_are_the_boards_own_words_and_home_is_refused() {
    let with = |ask: &str| STREAM.replacen("PARAM SpindleSpeed 8000\n", &format!("PARAM AfterOperation {ask}\nPARAM SpindleSpeed 8000\n"), 1);
    // Program stop: one more M0, right after the first section's M5 - the
    // tool-change stops are not after-operation stops.
    let nc = post(&generic(), &with("stop"));
    assert!(nc.contains("M5\nM0\n"), "{nc}");
    assert_eq!(nc.matches("\nM0\n").count(), 3, "two tool changes plus the asked stop\n{nc}");
    let nc = post(&generic(), &with("optional-stop"));
    assert!(nc.contains("M5\nM1\n"), "{nc}");
    assert_eq!(nc.matches("\nM0\n").count(), 2, "{nc}");
    let out = engine::run(&cps(), &generic(), &with("home")).expect("runs");
    assert!(out.warnings.iter().any(|w| w.contains("TinyG") && w.contains("home")), "{:?}", out.warnings);
}

#[test]
fn hostile_names_never_carry_a_control_character_into_the_program() {
    // ! is a feed hold and % a queue flush wherever they stand, ( ) and ;
    // end a comment, and a msg prefix is echoed to the operator.
    let stream = STREAM
        .replacen("TOOL 3 EM6", "TOOL 3 msg (stop!) 50% ; now", 1)
        .replacen("PARAM SpindleSpeed 8000\n", "PARAM OperationName msg hello\nPARAM SpindleSpeed 8000\n", 1);
    let nc = post(&generic(), &stream);
    assert_tinyg_words(&nc);
    // Tool and operation notes carry their own prefix, so a msg inside stays text.
    assert!(nc.contains("(T3 msg stop 50 now)"), "{nc}");
    assert!(nc.contains("(OPERATION: msg hello - G54)"), "{nc}");
    // The machine name opens its comment: one that reads as a message is led by "- ".
    let mut m = generic();
    m["name"] = json!("MSG router");
    assert!(post(&m, STREAM).starts_with("(- MSG router)\n"), "{}", post(&m, STREAM));
}

#[test]
fn an_inch_machine_posts_g20_and_inches() {
    let mut m = generic();
    m["units"] = json!("inch");
    let nc = post(&m, STREAM);
    assert!(nc.lines().nth(1).is_some_and(|l| l.starts_with("G20 ")), "{nc}");
    assert!(nc.contains("G1 X0.3937 Y0 Z-0.0394 F19.69"), "{nc}");
    assert!(nc.contains("G4 P1.5"), "a dwell is seconds in either unit\n{nc}");
}
