//! #3919: the BT50 and CAT50 holders match the recorded table
//! (docs/decisions/3919-standard-holders-2.md) and draw the flange
//! whole, at the gauge line's standoff.

use super::super::{find, library, Clamp, HolderEntry};
use super::*;
use crate::spun::SpunProfile;

fn entry(name: &str) -> HolderEntry {
    find(name).expect("a library entry")
}

fn close(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9)
}

/// The taper's ER and Weldon entries (its shrink-fit chucks are pinned
/// in holder_library_shrink_tests.rs).
fn names(taper: &str) -> Vec<String> {
    library()
        .into_iter()
        .filter(|e| e.taper.is_some_and(|t| t.name == taper) && !matches!(e.clamp, Clamp::Shrink { .. }))
        .map(|e| e.name())
        .collect()
}

/// The dimension table, as recorded (mm).
#[test]
fn the_50_tapers_match_the_recorded_table() {
    let row = |t: Taper| (t.name, t.gauge_d, t.flange_d, t.standoff, t.flange_t, t.ring);
    assert_eq!(row(BT50), ("BT50", 69.85, 100.0, 3.0, 35.0, "flange"));
    assert_eq!(row(CAT50), ("CAT50", 69.85, 98.425, 3.175, 15.875, "flange"));
    assert!((BT50.front_face() - 38.0).abs() < 1e-9);
    assert!((CAT50.front_face() - 19.05).abs() < 1e-9, "0.750 in");
}

/// Each taper lists ER16-ER50, then every Weldon bore, at 100 mm.
#[test]
fn the_50_taper_entries_are_listed() {
    for t in ["BT50", "CAT50"] {
        let n = names(t);
        assert_eq!(n.len(), 6 + 11, "{t}");
        let ers: Vec<String> = ["ER16", "ER20", "ER25", "ER32", "ER40", "ER50"].iter().map(|e| format!("{t} {e}")).collect();
        assert_eq!(n[..6], ers[..], "{t}");
        for want in ["Weldon 6 mm", "Weldon 20 mm", "Weldon 1/4 in", "Weldon 3/4 in"] {
            assert!(n.iter().any(|x| *x == format!("{t} {want}")), "{t} {want}");
        }
        assert!(library().iter().filter(|e| e.taper.is_some_and(|x| x.name == t)).all(|e| e.gauge_len >= 100.0), "{t}");
    }
    assert!(find("BT50 ER11").is_err() && find("CAT50 ER11").is_err());
}

/// BT50 ER32, point by point: nut, body, 45-degree flare to the flange
/// front (38 below the gauge line), the Ø100 flange, the 3 standoff at
/// the 7/24 cone's radius, the gauge line.
#[test]
fn the_bt50_er32_outline_is_the_table() {
    let e = entry("BT50 ER32");
    let zr: Vec<(f64, f64)> = e.points(40.0, 100.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    let (gauge, front, fr) = (140.0, 140.0 - 38.0, 50.0);
    let sr = (69.85 + 3.0 * 7.0 / 24.0) / 2.0;
    let want = [
        (40.0, 0.0),
        (40.0, 25.0),                // nut face, Ø50
        (62.5, 25.0),                // nut top, 22.5 long
        (front - (fr - 25.0), 25.0), // body up to the flare foot
        (front, fr),                 // the flange front
        (gauge - 3.0, fr),           // 35 thick
        (gauge - 3.0, sr),           // the standoff
        (gauge, sr),
        (gauge, 0.0), // the gauge line
    ];
    assert!(close(&zr, &want), "{zr:?}");
    let s = SpunProfile::new(e.points(40.0, 100.0).expect("draws")).expect("valid");
    assert!((s.max_radius() - fr).abs() < 1e-9);
}

/// CAT50 Weldon 1/2 in: the CAT40 standoff and flange on the 50 taper -
/// the Ø98.425 flange front 19.05 below the gauge line.
#[test]
fn the_cat50_weldon_flange_sits_at_the_cat_standoff() {
    let e = entry("CAT50 Weldon 1/2 in");
    let pts: Vec<(f64, f64)> = e.points(0.0, 100.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    let fr = 98.425 / 2.0;
    assert!(pts.iter().any(|p| (p.0 - (100.0 - 19.05)).abs() < 1e-9 && (p.1 - fr).abs() < 1e-9), "{pts:?}");
    assert!(pts.iter().any(|p| (p.0 - (100.0 - 3.175)).abs() < 1e-9 && (p.1 - fr).abs() < 1e-9), "{pts:?}");
}
