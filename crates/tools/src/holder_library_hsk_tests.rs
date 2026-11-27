//! #3919: the HSK holders match the recorded table
//! (docs/decisions/3919-standard-holders.md), draw the collar whole at
//! the flange diameter, and top out at the plane face - the gauge line.

use super::super::{find, library, HolderEntry, WELDON_SCREW};
use super::*;
use crate::spun::SpunProfile;

fn entry(name: &str) -> HolderEntry {
    find(name).expect("a library entry")
}

fn close(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9)
}

fn hsk() -> Vec<HolderEntry> {
    library().into_iter().filter(|e| e.taper.is_some_and(|t| t.name.starts_with("HSK-"))).collect()
}

/// The dimension table, as recorded (mm): d1, f1, d2; no standoff.
#[test]
fn the_hsk_tapers_match_the_recorded_table() {
    let row = |t: Taper| (t.name, t.flange_d, t.flange_t, t.gauge_d, t.standoff, t.ring);
    assert_eq!(row(HSK_A63), ("HSK-A63", 63.0, 26.0, 48.0, 0.0, "flange"));
    assert_eq!(row(HSK_F63), ("HSK-F63", 63.0, 26.0, 38.0, 0.0, "flange"));
    assert_eq!(row(HSK_E40), ("HSK-E40", 40.0, 20.0, 30.0, 0.0, "flange"));
    assert_eq!(row(HSK_E32), ("HSK-E32", 32.0, 20.0, 24.0, 0.0, "flange"));
}

/// Each HSK size lists the ER chucks and Weldon bores its flange
/// carries, at the recorded default gauge lengths.
#[test]
fn the_hsk_entries_are_listed() {
    let count = |t: &str| hsk().iter().filter(|e| e.taper.is_some_and(|x| x.name == t)).count();
    assert_eq!((count("HSK-A63"), count("HSK-F63"), count("HSK-E40"), count("HSK-E32")), (5 + 11, 5 + 11, 4 + 7, 3 + 3));
    for (name, gauge) in [
        ("HSK-A63 ER32", 100.0),
        ("HSK-A63 ER40", 100.0),
        ("HSK-A63 Weldon 20 mm", 80.0),
        ("HSK-F63 ER16", 100.0),
        ("HSK-F63 Weldon 3/4 in", 80.0),
        ("HSK-E40 ER11", 70.0),
        ("HSK-E40 ER25", 70.0),
        ("HSK-E40 Weldon 12 mm", 65.0),
        ("HSK-E40 Weldon 1/2 in", 65.0),
        ("HSK-E32 ER20", 60.0),
        ("HSK-E32 Weldon 8 mm", 60.0),
        ("HSK-E32 Weldon 1/4 in", 60.0),
    ] {
        assert_eq!(entry(name).gauge_len, gauge, "{name}");
    }
    assert!(find("HSK-E32 ER25").is_err() && find("HSK-E40 ER32").is_err());
}

/// HSK-A63 ER32, point by point: nut, body, 45-degree flare to the
/// collar's front face (26 below the plane face), the Ø63 collar up to
/// the plane face - the gauge line - and nothing above it.
#[test]
fn the_hsk_a63_er32_outline_is_the_table() {
    let e = entry("HSK-A63 ER32");
    let zr: Vec<(f64, f64)> = e.points(40.0, 100.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    let want = [
        (40.0, 0.0),
        (40.0, 25.0),   // nut face, Ø50
        (62.5, 25.0),   // nut top, 22.5 long
        (107.5, 25.0),  // body up to the flare foot: 6.5 below the collar
        (114.0, 31.5),  // the collar's front face, Ø63
        (140.0, 31.5),  // the plane face: the gauge line
        (140.0, 0.0),
    ];
    assert!(close(&zr, &want), "{zr:?}");
    assert!((e.min_gauge() - (26.0 + 22.5 + 6.5)).abs() < 1e-9);
}

/// A nut or body wider than a small HSK flange steps in to the collar,
/// never under it: HSK-E32 ER20 (Ø35 nut on a Ø32 collar) and HSK-E40
/// Weldon 12 mm (Ø42 body plus the screw allowance on a Ø40 collar).
#[test]
fn a_nose_wider_than_the_collar_steps_in_to_it() {
    let zr: Vec<(f64, f64)> = entry("HSK-E32 ER20").points(20.0, 60.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    let want = [(20.0, 0.0), (20.0, 17.5), (39.5, 17.5), (60.0, 17.5), (60.0, 16.0), (80.0, 16.0), (80.0, 0.0)];
    assert!(close(&zr, &want), "{zr:?}");
    let r = 21.0 + WELDON_SCREW;
    let zr: Vec<(f64, f64)> = entry("HSK-E40 Weldon 12 mm").points(0.0, 65.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    assert!(close(&zr, &[(0.0, 0.0), (0.0, r), (45.0, r), (45.0, 20.0), (65.0, 20.0), (65.0, 0.0)]), "{zr:?}");
}

/// Every HSK entry is a closed, valid spun profile whose top is the
/// plane face at the collar's radius, at its default and its shortest
/// gauge length.
#[test]
fn every_hsk_profile_tops_out_at_the_plane_face() {
    for e in hsk() {
        let t = e.taper.expect("filtered");
        for (nose, gauge) in [(25.0, e.gauge_len), (0.0, e.min_gauge())] {
            let pts = e.points(nose, gauge).unwrap_or_else(|err| panic!("{}: {err}", e.name()));
            let s = SpunProfile::new(pts.clone()).unwrap_or_else(|err| panic!("{}: {err}", e.name()));
            let collar = pts.iter().rev().nth(1).map(|p| (p.z, p.r));
            assert_eq!(collar, Some((nose + gauge, t.flange_d * 0.5)), "{}: the collar reaches the plane face", e.name());
            assert!(s.max_radius() >= t.flange_d * 0.5, "{}", e.name());
        }
    }
}
