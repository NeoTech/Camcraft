//! #1616 (the #1614 hazard in the other mill posts): grblHAL, TinyG and
//! the Carvera state the height of the first X/Y traverse after a tool change
//! (and at the program's start) - a Z-only `G0 Z` to the section's
//! clearance plane, else its retract plane - instead of rapiding X/Y at
//! whatever height the tool change left the tip. A long tool on a high
//! work offset can stand below the part top there. A stream stating
//! neither plane is refused by name.
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
/// traverse.
const POSTS: &[(&str, &str)] = &[
    ("assets/posts/grblhal.cps", "assets/machines/sienci-altmill-2x4.mch"),
    ("assets/posts/tinyg.cps", "assets/machines/tinyg-3axis.mch"),
    ("assets/posts/carvera.cps", "assets/machines/makera-carvera.mch"),
];

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
/// to `want_z` and then X20 Y10 alone.
fn assert_clearance_first(cps: &str, program: &str, want_z: &str) {
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
        let (first, second) = (motions.next(), motions.next());
        assert_eq!(
            first,
            Some(&want_z),
            "{cps}: the stated clearance first after the tool change\n{program}"
        );
        assert_eq!(second, Some(&"G0 X20 Y10"), "{cps}: then X/Y alone\n{program}");
    }
}

#[test]
fn the_first_traverse_after_a_tool_change_states_the_clearance_z_first() {
    for (cps, mch) in POSTS {
        let out = post(cps, mch, &two_tools("PARAM ClearancePlane 25.4\nPARAM RetractPlane 12.7\n"));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert_clearance_first(cps, &out.program, "G0 Z25.4");
    }
}

#[test]
fn the_retract_plane_serves_when_no_clearance_plane_is_stated() {
    for (cps, mch) in POSTS {
        let out = post(cps, mch, &two_tools("PARAM RetractPlane 12.7\n"));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert_clearance_first(cps, &out.program, "G0 Z12.7");
    }
}

#[test]
fn a_first_traverse_with_no_stated_plane_is_refused_by_name() {
    for (cps, mch) in POSTS {
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
    for (cps, mch) in POSTS {
        let out = post(cps, mch, s);
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let lines: Vec<&str> = out.program.lines().collect();
        let xy = lines
            .iter()
            .position(|l| *l == "G0 X0 Y0")
            .unwrap_or_else(|| panic!("{cps}: traverse\n{}", out.program));
        assert_eq!(lines.get(xy - 1), Some(&"G0 Z50.8"), "{cps}\n{}", out.program);
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
    for (cps, mch) in POSTS {
        // Starting at the clearance itself: no second Z rapid.
        let out = post(cps, mch, &stream(10.0));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let motions = motions_after_tool_change(&out.program);
        assert_eq!(motions.first(), Some(&"G0 Z10"), "{cps}\n{}", out.program);
        assert_eq!(motions.get(1), Some(&"G0 X20 Y10"), "{cps}\n{}", out.program);
        assert!(
            motions.get(2).is_some_and(|l| l.starts_with("G1 ")),
            "{cps}: no second Z rapid to the same height\n{}",
            out.program
        );
        // Starting lower: the rapid down to the start, then the feed.
        let out = post(cps, mch, &stream(5.0));
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        let motions = motions_after_tool_change(&out.program);
        assert_eq!(motions.get(..3), Some(&["G0 Z10", "G0 X20 Y10", "G0 Z5"][..]), "{cps}\n{}", out.program);
    }
}
