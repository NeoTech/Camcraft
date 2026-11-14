//! #1616 (the #1614 hazard in the other mill posts): grblHAL, TinyG and
//! the Carvera state the height of the first X/Y traverse after a tool change
//! (and at the program's start) - a Z-only `G0 Z` to the section's
//! clearance plane, else its retract plane - instead of rapiding X/Y at
//! whatever height the tool change left the tip. A long tool on a high
//! work offset can stand below the part top there. A stream stating
//! neither plane is refused by name.
//! The Carvera alone keeps X/Y first right after its `M6` (the changer's
//! exit height, as before), then states the clearance before any descent:
//! nothing we have read says where its firmware parks X/Y after a tool
//! change, and a Z-only rapid down at the rack's X/Y could meet the stored
//! tools (M3's option (c), docs/decisions/1616-first-traverse-clearance.md).
//! That X/Y traverse at the unverified exit height is the #1614 hazard,
//! which stays open on the Carvera until its M6 exit position is confirmed.
//! At a program start with no tool change it states Z first like the rest.
//! Provenance: our own tests over our own posts and stream format.
//!
//! Gate map (M4.4): covers assets/posts/grblhal.cps, assets/posts/tinyg.cps,
//! assets/posts/carvera.cps, camcraft_post::post::engine.

use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};
use std::path::{Path, PathBuf};

/// Each post with a shipped machine it serves. The AltMill states its Z
/// `safe_retract`, so grblHAL lifts with `G53 G0 Z0` before T M6 (the
/// #1614 shape) whatever planes the stream states; the TinyG machine
/// states none, so `post` gives it one (a homed machine), for the same
/// lift - otherwise its lift needs the ClearancePlane and the
/// retract-plane-only stream would be refused for the lift, not the
/// traverse. The flag: X/Y first right after a tool change (the Carvera).
const POSTS: &[(&str, &str, bool)] = &[
    ("assets/posts/grblhal.cps", "assets/machines/sienci-altmill-2x4.mch", false),
    ("assets/posts/tinyg.cps", "assets/machines/tinyg-3axis.mch", false),
    ("assets/posts/carvera.cps", "assets/machines/makera-carvera.mch", true),
];

/// The two traverse lines in the order the post writes them: the Z-only
/// rapid first, or (right after a Carvera tool change) X/Y first.
fn traverse<'a>(xy_first: bool, z: &'a str, xy: &'a str) -> [&'a str; 2] {
    if xy_first { [xy, z] } else { [z, xy] }
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn post(cps: &str, mch: &str, stream: &str) -> engine::PostOutput {
    let mut m = machine::mirror(&parse_mch(&read(mch)).unwrap_or_else(|e| panic!("{mch}: {e}")));
    if let Some(o) = m.as_object_mut() {
        o.entry("safeRetractZ").or_insert(serde_json::json!(0.0));
    }
    engine::run(&read(cps), &m, stream).unwrap_or_else(|e| panic!("{cps}: {e}"))
}

/// Two tools, each section's first move a rapid down to 2 mm at X20 Y10
/// (the #1614 shape: `G0 X20 Y10` then `G0 X20 Y10 Z2`).
fn two_tools(planes: &str) -> String {
    format!(
        "PARAM SpindleSpeed 6000
PARAM Coolant off
{planes}TOOL 1 6MM END MILL
SECTION
Rapid 20000 10000 2000
Feed 20000 10000 -1000 F500000
Feed 30000 10000 -1000 F500000
ENDSECTION
PARAM SpindleSpeed 6000
PARAM Coolant off
{planes}TOOL 2 3MM END MILL
SECTION
Rapid 20000 10000 2000
Feed 20000 10000 -1000 F500000
ENDSECTION
"
    )
}

fn is_motion(l: &str) -> bool {
    l.split_whitespace().any(|w| matches!(w, "G0" | "G1" | "G2" | "G3"))
}

/// The motion lines after the (first) tool change: the lifts around it
/// (`G53 G0 Z..`, machine coordinates; TinyG lifts again after its M0)
/// are the tool change's own business.
fn motions_after_tool_change(program: &str) -> Vec<&str> {
    program
        .lines()
        .skip_while(|l| !l.split_whitespace().any(|w| w == "M6"))
        .filter(|l| is_motion(l) && !l.starts_with("G53 "))
        .collect()
}

/// The first two motion lines after each tool change: the Z-only rapid
/// to `want_z` and X20 Y10 alone, in the post's order.
fn assert_clearance_first(cps: &str, xy_first: bool, program: &str, want_z: &str) {
    let lines: Vec<&str> = program.lines().collect();
    let changes: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.split_whitespace().any(|w| w == "M6"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changes.len(), 2, "{cps}: two tool changes\n{program}");
    for i in changes {
        let mut motions = lines[i..].iter().filter(|l| is_motion(l) && !l.starts_with("G53 "));
        let got: Vec<&str> = motions.by_ref().take(2).copied().collect();
        assert_eq!(
            got,
            traverse(xy_first, want_z, "G0 X20 Y10"),
            "{cps}: the stated clearance and X/Y alone, before any descent\n{program}"
        );
    }
}

#[test]
fn the_first_traverse_after_a_tool_change_states_the_clearance_z() {
    for (cps, mch, xy_first) in POSTS {
        let out = post(cps, mch, &two_tools("PARAM ClearancePlane 25.4\nPARAM RetractPlane 12.7\n"));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert_clearance_first(cps, *xy_first, &out.program, "G0 Z25.4");
    }
}

#[test]
fn the_retract_plane_serves_when_no_clearance_plane_is_stated() {
    for (cps, mch, xy_first) in POSTS {
        let out = post(cps, mch, &two_tools("PARAM RetractPlane 12.7\n"));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert_clearance_first(cps, *xy_first, &out.program, "G0 Z12.7");
    }
}

#[test]
fn a_first_traverse_with_no_stated_plane_is_refused_by_name() {
    for (cps, mch, _) in POSTS {
        let out = post(cps, mch, &two_tools(""));
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("first X/Y traverse") && w.contains("no clearance height")),
            "{cps}: {:?}\n{}",
            out.warnings,
            out.program
        );
        assert!(
            !out.program.lines().any(|l| l == "G0 X20 Y10"),
            "{cps}: no X/Y traverse at an unknown height\n{}",
            out.program
        );
    }
}

#[test]
fn the_traverse_is_never_below_the_moves_own_height() {
    // A first rapid to 50.8 mm, above the 25.4 mm clearance: the traverse
    // rides at the rapid's own height.
    let s = "PARAM SpindleSpeed 6000
PARAM Coolant off
PARAM ClearancePlane 25.4
TOOL 2 6MM END MILL
SECTION
Rapid 0 0 50800
Feed 10000 0 -1000 F500000
ENDSECTION
";
    for (cps, mch, xy_first) in POSTS {
        let out = post(cps, mch, s);
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let motions = motions_after_tool_change(&out.program);
        assert_eq!(motions.get(..2), Some(&traverse(*xy_first, "G0 Z50.8", "G0 X0 Y0")[..]), "{cps}\n{}", out.program);
    }
}

/// A feed as the first move: the traverse at the clearance, then a rapid
/// down to the stream's start height only when that is lower - never a
/// second `G0 Z` to the height the traverse already stated.
#[test]
fn a_feed_first_move_rapids_down_from_the_stated_clearance() {
    let stream = |start_z: f64| {
        format!(
            "PARAM SpindleSpeed 6000
PARAM Coolant off
PARAM ClearancePlane 10
TOOL 1 6MM END MILL
PARAM StartX 20
PARAM StartY 10
PARAM StartZ {start_z}
SECTION
Feed 20000 10000 -1000 F500000
ENDSECTION
"
        )
    };
    for (cps, mch, xy_first) in POSTS {
        let pair = traverse(*xy_first, "G0 Z10", "G0 X20 Y10");
        // Starting at the clearance itself: no second Z rapid.
        let out = post(cps, mch, &stream(10.0));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let motions = motions_after_tool_change(&out.program);
        assert_eq!(motions.get(..2), Some(&pair[..]), "{cps}\n{}", out.program);
        assert!(
            motions.get(2).is_some_and(|l| l.starts_with("G1 ")),
            "{cps}: no second Z rapid to the same height\n{}",
            out.program
        );
        // Starting lower: the rapid down to the start, then the feed.
        let out = post(cps, mch, &stream(5.0));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let motions = motions_after_tool_change(&out.program);
        assert_eq!(motions.get(..3), Some(&[pair[0], pair[1], "G0 Z5"][..]), "{cps}\n{}", out.program);
    }
}

/// At the program's start with no tool change, every post (the Carvera
/// too) states the clearance Z before the first X/Y traverse.
#[test]
fn at_the_program_start_with_no_tool_change_the_clearance_z_comes_first() {
    let s = "PARAM SpindleSpeed 6000
PARAM Coolant off
PARAM ClearancePlane 25.4
SECTION
Rapid 20000 10000 2000
Feed 20000 10000 -1000 F500000
ENDSECTION
";
    for (cps, mch, _) in POSTS {
        let out = post(cps, mch, s);
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert!(!out.program.lines().any(|l| l.split_whitespace().any(|w| w == "M6")), "{cps}: no tool change\n{}", out.program);
        let motions: Vec<&str> = out.program.lines().filter(|l| is_motion(l) && !l.starts_with("G53 ")).collect();
        assert_eq!(motions.get(..2), Some(&["G0 Z25.4", "G0 X20 Y10"][..]), "{cps}\n{}", out.program);
    }
}

/// The Carvera's X/Y traverse right after `M6` is still made at the
/// changer's exit height, which no source we read confirms (the #1614
/// hazard stays there, option b open). The script must say so where the
/// order is chosen, and must not claim that the hazard is gone. This pin
/// fails if the open-hazard note is removed while the order stays.
#[test]
fn the_carvera_script_names_the_open_hazard_after_its_tool_change() {
    let cps = read("assets/posts/carvera.cps");
    let note = cps.find("OPEN HAZARD (#1614)").unwrap_or_else(|| panic!("carvera.cps: the open-hazard note is missing"));
    let order = cps.find("if (changedAt)").unwrap_or_else(|| panic!("carvera.cps: the M6 order branch is missing"));
    assert!(note < order, "carvera.cps: the note must stand with the traverse comment, before the order branch");
    let block = &cps[note..order];
    assert!(block.contains("until a public source") && block.contains("M6 exit position"), "carvera.cps: the note must say what closes it\n{block}");
    assert!(!cps.contains("hazard is not added"), "carvera.cps: claims the hazard is gone");
}
