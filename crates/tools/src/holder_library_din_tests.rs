//! #3919: the DIN 69871 SK30 / SK40 and router ISO30 holders match the
//! recorded table (docs/decisions/3919-standard-holders.md) and draw
//! the flange whole, at the gauge line's standoff.

use super::super::{find, library, HolderEntry};
use super::*;
use crate::spun::SpunProfile;

fn entry(name: &str) -> HolderEntry {
    find(name).expect("a library entry")
}

fn close(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9)
}

fn names(taper: &str) -> Vec<String> {
    library().into_iter().filter(|e| e.taper.is_some_and(|t| t.name == taper)).map(|e| e.name()).collect()
}

/// The dimension table, as recorded (mm).
#[test]
fn the_din_69871_tapers_match_the_recorded_table() {
    assert_eq!((SK30.gauge_d, SK30.flange_d, SK30.standoff, SK30.flange_t, SK30.ring), (31.75, 50.0, 3.2, 15.9, "flange"));
    assert_eq!((SK40.gauge_d, SK40.flange_d, SK40.standoff, SK40.flange_t, SK40.ring), (44.45, 63.55, 3.2, 15.9, "flange"));
    assert!((SK30.front_face() - 19.1).abs() < 1e-9 && (SK40.front_face() - 19.1).abs() < 1e-9);
    assert_eq!((ISO30.name, ISO30.gauge_d, ISO30.flange_d, ISO30.standoff, ISO30.flange_t), ("ISO30", 31.75, 50.0, 3.2, 15.9));
}

/// Each taper lists its ER chucks, then its Weldon bores.
#[test]
fn the_din_entries_are_listed() {
    let thirty = |t: &str| {
        let mut want: Vec<String> = ["ER16", "ER20", "ER25", "ER32"].iter().map(|e| format!("{t} {e}")).collect();
        for b in ["6 mm", "8 mm", "10 mm", "12 mm", "16 mm", "1/4 in", "3/8 in", "1/2 in"] {
            want.push(format!("{t} Weldon {b}"));
        }
        want
    };
    assert_eq!(names("SK30"), thirty("SK30"));
    assert_eq!(names("ISO30"), thirty("ISO30"));
    let sk40 = names("SK40");
    assert_eq!(sk40.len(), 5 + 11);
    for want in ["SK40 ER40", "SK40 ER32", "SK40 Weldon 20 mm", "SK40 Weldon 3/4 in"] {
        assert!(sk40.iter().any(|n| n == want), "{want}");
    }
    assert_eq!(entry("SK40 ER40").gauge_len, 80.0);
    assert_eq!(entry("SK40 Weldon 12 mm").gauge_len, 63.0);
}

/// SK40 ER32, point by point: nut, body, 45-degree flare to the flange
/// front (19.1 below the gauge line), the Ø63.55 flange, the 3.2
/// standoff at the 7/24 cone's radius, the gauge line.
#[test]
fn the_sk40_er32_outline_is_the_table() {
    let e = entry("SK40 ER32");
    let zr: Vec<(f64, f64)> = e.points(40.0, 70.0).expect("draws").iter().map(|p| (p.z, p.r)).collect();
    let (front, fr) = (110.0 - 19.1, 63.55 / 2.0);
    let sr = (44.45 + 3.2 * 7.0 / 24.0) / 2.0;
    let want = [
        (40.0, 0.0),
        (40.0, 25.0),             // nut face, Ø50
        (62.5, 25.0),             // nut top, 22.5 long
        (front - (fr - 25.0), 25.0), // body up to the flare foot
        (front, fr),              // the flange front
        (110.0 - 3.2, fr),        // 15.9 thick
        (110.0 - 3.2, sr),        // the standoff
        (110.0, sr),
        (110.0, 0.0),             // the gauge line
    ];
    assert!(close(&zr, &want), "{zr:?}");
    let s = SpunProfile::new(e.points(40.0, 70.0).expect("draws")).expect("valid");
    assert!((s.max_radius() - fr).abs() < 1e-9);
}

/// ISO30 is the SK30 shape under the router name: the same outline for
/// the same clamp.
#[test]
fn an_iso30_holder_draws_its_sk30_twin() {
    for clamp in ["ER20", "ER32", "Weldon 12 mm", "Weldon 1/2 in"] {
        let (a, b) = (entry(&format!("ISO30 {clamp}")), entry(&format!("SK30 {clamp}")));
        assert_eq!(a.points(30.0, a.gauge_len), b.points(30.0, b.gauge_len), "{clamp}");
    }
    assert!(find("ISO30 ER40").is_err(), "an ER40 nut is wider than the Ø50 flange is built for");
}
