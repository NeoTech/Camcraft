//! #3919 (third slice): the DIN 69871 SK50 taper and the ER50 collet
//! chucks on the 40 and 50 tapers match the recorded table
//! (docs/decisions/3919-standard-holders-3.md); an ER50 nut wider than
//! its flange is drawn whole, down to the flange.

use super::super::{find, library, Clamp, HolderEntry, LibraryError, ER50};
use super::*;
use crate::spun::SpunProfile;

fn entry(name: &str) -> HolderEntry {
    find(name).expect("a library entry")
}

fn zr(e: &HolderEntry, nose: f64, gauge_len: f64) -> Vec<(f64, f64)> {
    e.points(nose, gauge_len).expect("draws").iter().map(|p| (p.z, p.r)).collect()
}

fn close(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (p.0 - q.0).abs() < 1e-9 && (p.1 - q.1).abs() < 1e-9)
}

/// The SK50 row and the ER50 nut, as recorded (mm).
#[test]
fn sk50_and_er50_match_the_recorded_table() {
    let row = |t: Taper| (t.name, t.gauge_d, t.flange_d, t.standoff, t.flange_t, t.ring);
    assert_eq!(row(SK50), ("SK50", 69.85, 97.5, 3.2, 15.9, "flange"));
    assert!((SK50.front_face() - 19.1).abs() < 1e-9, "the SK30 / SK40 flange front");
    assert_eq!((ER50.name, ER50.min_bore, ER50.max_bore, ER50.nut_d, ER50.nut_len), ("ER50", 5.0, 34.0, 78.0, 40.0));
}

/// SK50 lists ER16-ER50, then every Weldon bore, all at 100 mm; ER50
/// sits on each 40 and 50 taper and on no smaller one; there is no
/// standalone ER50 chuck.
#[test]
fn the_sk50_and_er50_entries_are_listed() {
    let lib = library();
    let names = |t: &str| -> Vec<String> {
        lib.iter().filter(|e| e.taper.is_some_and(|x| x.name == t) && !matches!(e.clamp, Clamp::Shrink { .. })).map(HolderEntry::name).collect()
    };
    let sk50 = names("SK50");
    assert_eq!(sk50.len(), 6 + 11);
    let ers: Vec<String> = ["ER16", "ER20", "ER25", "ER32", "ER40", "ER50"].iter().map(|e| format!("SK50 {e}")).collect();
    assert_eq!(sk50[..6], ers[..]);
    for want in ["SK50 Weldon 6 mm", "SK50 Weldon 20 mm", "SK50 Weldon 1/4 in", "SK50 Weldon 3/4 in"] {
        assert!(sk50.iter().any(|n| n == want), "{want}");
    }
    assert!(lib.iter().filter(|e| e.taper == Some(SK50)).all(|e| e.gauge_len == 100.0));
    for t in ["BT40", "CAT40", "SK40", "BT50", "CAT50", "SK50"] {
        assert_eq!(entry(&format!("{t} ER50")).gauge_len, 100.0, "{t}");
    }
    for t in ["BT30", "SK30", "ISO30", "TTS", "HSK-A63", "HSK-E40"] {
        assert!(find(&format!("{t} ER50")).is_err(), "{t}");
    }
    assert!(find("ER50 chuck").is_err());
    // SK50 sits right after SK40 in the editor's order.
    let mut tapers: Vec<&str> = lib.iter().filter_map(|e| e.taper.map(|t| t.name)).collect();
    tapers.dedup();
    let at = |n: &str| tapers.iter().position(|t| *t == n);
    assert_eq!(at("SK50"), at("SK40").map(|i| i + 1));
}

/// SK50 ER50, point by point: the Ø78 nut 40 long, the body, the
/// 45-degree flare to the Ø97.5 flange front (19.1 below the gauge
/// line), the 15.9 flange, the 3.2 standoff at the 7/24 cone's radius.
#[test]
fn the_sk50_er50_outline_is_the_table() {
    let e = entry("SK50 ER50");
    let (gauge, fr) = (140.0, 97.5 / 2.0);
    let front = gauge - 19.1;
    let sr = (69.85 + 3.2 * 7.0 / 24.0) / 2.0;
    let want = [
        (40.0, 0.0),
        (40.0, 39.0),                // nut face, Ø78
        (80.0, 39.0),                // nut top, 40 long
        (front - (fr - 39.0), 39.0), // body up to the flare foot
        (front, fr),                 // the flange front
        (gauge - 3.2, fr),           // 15.9 thick
        (gauge - 3.2, sr),           // the standoff
        (gauge, sr),
        (gauge, 0.0), // the gauge line
    ];
    let got = zr(&e, 40.0, 100.0);
    assert!(close(&got, &want), "{got:?}");
    let s = SpunProfile::new(e.points(40.0, 100.0).expect("draws")).expect("valid");
    assert!((s.max_radius() - fr).abs() < 1e-9);
}

/// BT40 ER50: the Ø78 nut is wider than the Ø63 flange, so the body
/// runs at nut width up to the flange front with no flare, and the
/// flange steps in from it - never smaller than the steel.
#[test]
fn a_bt40_er50_body_wider_than_its_flange_is_drawn_whole() {
    let e = entry("BT40 ER50");
    assert_eq!(e.flare_len(), 0.0);
    let sr = (44.45 + 2.0 * 7.0 / 24.0) / 2.0;
    let want = [
        (0.0, 0.0),
        (0.0, 39.0),  // nut face
        (40.0, 39.0), // nut top
        (73.0, 39.0), // body at nut width to the flange front (27 below)
        (73.0, 31.5), // the Ø63 flange
        (98.0, 31.5),
        (98.0, sr),
        (100.0, sr),
        (100.0, 0.0),
    ];
    let got = zr(&e, 0.0, 100.0);
    assert!(close(&got, &want), "{got:?}");
    let s = SpunProfile::new(e.points(0.0, 100.0).expect("draws")).expect("valid");
    assert!((s.max_radius() - 39.0).abs() < 1e-9);
    // The flange front, the 40 mm nut and 1 mm of body.
    assert!((e.min_gauge() - 68.0).abs() < 1e-9);
    assert!(matches!(e.points(0.0, 67.0), Err(LibraryError::TooShort { .. })));
}

/// An ER50 chuck takes Ø5-34 shanks and refuses the rest by name.
#[test]
fn an_er50_chuck_takes_its_collet_range() {
    let e = entry("SK50 ER50");
    assert!(e.fits_shank(5.0).is_ok() && e.fits_shank(34.0).is_ok());
    for bad in [4.0, 35.0, f64::NAN] {
        let err = e.fits_shank(bad).expect_err("refused");
        assert!(err.to_string().contains("ER50"), "{err}");
    }
}
