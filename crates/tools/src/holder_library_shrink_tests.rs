//! #3919: the shrink-fit chucks match the recorded DIN 69882-8 rows
//! (docs/decisions/3919-standard-holders-2.md), draw one envelope of
//! the 4.5-degree nose cone and the 45-degree flare, never under the
//! table's steel, and group into one picker row for each taper.

use super::super::{find, library, picker_rows, resolve, HolderEntry, LibraryError, PickerRow};
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

/// The outline's radius at height `z` (the larger one at a step).
fn radius_at(pts: &[(f64, f64)], z: f64) -> f64 {
    pts.windows(2)
        .filter(|w| w[0].0 <= z + 1e-9 && z <= w[1].0 + 1e-9)
        .map(|w| if (w[1].0 - w[0].0).abs() < 1e-12 { w[0].1.max(w[1].1) } else { w[0].1 + (w[1].1 - w[0].1) * (z - w[0].0) / (w[1].0 - w[0].0) })
        .fold(0.0, f64::max)
}

/// The recorded rows (mm): bore, nose, body and default gauge length.
#[test]
fn the_shrink_rows_match_the_recorded_table() {
    assert!((SHRINK_TAPER - 4.5f64.to_radians().tan()).abs() < 1e-15);
    assert_eq!(SHRINK_MARGIN, 2.0);
    let row = |n: &str| {
        let e = entry(n);
        let Clamp::Shrink { bore, nose_d, body_d } = e.clamp else { panic!("{n} is a shrink-fit chuck") };
        (bore, nose_d, body_d, e.gauge_len)
    };
    assert_eq!(row("BT30 Shrink 6 mm"), (6.0, 21.0, 27.0, 80.0));
    assert_eq!(row("BT40 Shrink 12 mm"), (12.0, 24.0, 32.0, 90.0));
    assert_eq!(row("SK40 Shrink 16 mm"), (16.0, 27.0, 34.0, 80.0));
    assert_eq!(row("CAT40 Shrink 20 mm"), (20.0, 33.0, 42.0, 80.0));
    assert_eq!(row("HSK-A63 Shrink 32 mm"), (32.0, 44.0, 53.0, 120.0));
    assert_eq!(row("HSK-E40 Shrink 3 mm"), (3.0, 11.0, 15.0, 60.0));
    // HSK-E32 has its own body column from 6 to 12 mm.
    assert_eq!(row("HSK-E32 Shrink 10 mm"), (10.0, 24.0, 26.0, 70.0));
    assert_eq!(row("HSK-E32 Shrink 14 mm"), (14.0, 27.0, 34.0, 90.0));
    assert_eq!(row("BT50 Shrink 25 mm"), (25.0, 44.0, 53.0, 120.0));
    assert_eq!(row("CAT50 Shrink 18 mm"), (18.0, 33.0, 42.0, 100.0));
}

/// Each taper but TTS lists its shrink-fit bores, after every ER and
/// Weldon entry of the library.
#[test]
fn the_shrink_entries_are_listed() {
    let lib = library();
    let names = |t: &str| -> Vec<String> {
        lib.iter().filter(|e| e.taper.is_some_and(|x| x.name == t) && matches!(e.clamp, Clamp::Shrink { .. })).map(HolderEntry::name).collect()
    };
    let bt30: Vec<String> = ["3", "4", "5", "6", "8", "10", "12", "14", "16", "20"].iter().map(|b| format!("BT30 Shrink {b} mm")).collect();
    assert_eq!(names("BT30"), bt30);
    assert_eq!(names("SK30").len(), 10);
    assert_eq!(names("ISO30").len(), 10);
    assert_eq!(names("BT40").len(), 13);
    assert_eq!(names("SK40").len(), 13);
    assert_eq!(names("CAT40").len(), 8);
    assert_eq!(names("HSK-A63").len(), 13);
    assert_eq!(names("HSK-F63").len(), 12);
    assert_eq!(names("HSK-E40").len(), 11);
    assert_eq!(names("HSK-E32").len(), 9);
    assert_eq!(names("BT50").len(), 8);
    assert_eq!(names("CAT50").len(), 10);
    assert!(names("TTS").is_empty());
    // No published SK50 shrink-fit table was found (3919-standard-holders-3.md).
    assert!(names("SK50").is_empty());
    let first = lib.iter().position(|e| matches!(e.clamp, Clamp::Shrink { .. })).expect("shrink entries");
    assert!(lib[first..].iter().all(|e| matches!(e.clamp, Clamp::Shrink { .. })));
}

/// SK40 Shrink 12 mm, point by point: the nose face at the table's Ø24
/// plus the margin, the 4.5-degree cone up to where the flare from the
/// flange front (19.1 below the gauge line) overtakes it, the Ø63.55
/// flange, the standoff, the gauge line.
#[test]
fn the_sk40_shrink_12_outline_is_the_table() {
    let t = SHRINK_TAPER;
    let (nose, gauge) = (40.0, 120.0);
    let (front, fr, r0) = (gauge - 19.1, 63.55 / 2.0, 12.0 + SHRINK_MARGIN);
    // The cone (toward Ø32 + margin) meets the flare before the body.
    let z = (fr - front - r0 + nose * t) / (t - 1.0);
    let rz = r0 + (z - nose) * t;
    assert!((rz - (fr - (front - z))).abs() < 1e-9 && rz < 16.0 + SHRINK_MARGIN);
    let sr = (44.45 + 3.2 * 7.0 / 24.0) / 2.0;
    let want = [(nose, 0.0), (nose, r0), (z, rz), (front, fr), (gauge - 3.2, fr), (gauge - 3.2, sr), (gauge, sr), (gauge, 0.0)];
    let got = zr(&entry("SK40 Shrink 12 mm"), nose, gauge - nose);
    assert!(close(&got, &want), "{got:?}");
}

/// A long chuck reaches its body before the flare: BT40 Shrink 6 mm at
/// a 160 gauge length runs the cone to Ø27 + margin, the body, then the
/// flare into the flange front 27 below the gauge line.
#[test]
fn a_long_chuck_runs_its_body_to_the_flare() {
    let (nose, gauge) = (40.0, 200.0);
    let (r0, br, fr, front) = (10.5 + SHRINK_MARGIN, 13.5 + SHRINK_MARGIN, 31.5, gauge - 27.0);
    let zc = nose + (br - r0) / SHRINK_TAPER;
    let foot = front - (fr - br);
    let sr = (44.45 + 2.0 * 7.0 / 24.0) / 2.0;
    let want = [(nose, 0.0), (nose, r0), (zc, br), (foot, br), (front, fr), (gauge - 2.0, fr), (gauge - 2.0, sr), (gauge, sr), (gauge, 0.0)];
    let got = zr(&entry("BT40 Shrink 6 mm"), nose, gauge - nose);
    assert!(close(&got, &want), "{got:?}");
}

/// Every shrink-fit entry, at its default gauge length and at its
/// shortest, draws a valid profile that is never under the table's
/// steel plus the margin between the nose face and the flange.
#[test]
fn the_outline_is_never_under_the_table_steel() {
    for e in library().into_iter().filter(|e| matches!(e.clamp, Clamp::Shrink { .. })) {
        let Clamp::Shrink { nose_d, body_d, .. } = e.clamp else { continue };
        let Some(taper) = e.taper else { panic!("{}: a shrink-fit chuck sits in a taper", e.name()) };
        for g in [e.gauge_len, e.min_gauge()] {
            let nose = 30.0;
            let pts = zr(&e, nose, g);
            SpunProfile::new(e.points(nose, g).expect("draws")).unwrap_or_else(|err| panic!("{}: {err}", e.name()));
            let front = nose + g - taper.front_face();
            for i in 0..=200 {
                let z = nose + (front - nose) * f64::from(i) / 200.0;
                let steel = (nose_d * 0.5 + (z - nose) * SHRINK_TAPER).min(body_d * 0.5) + SHRINK_MARGIN;
                assert!(radius_at(&pts, z) >= steel - 1e-9, "{} at gauge {g}: z {z}", e.name());
            }
        }
        assert!(matches!(e.points(30.0, e.min_gauge() - 0.1), Err(LibraryError::TooShort { .. })), "{}", e.name());
    }
}

/// A wider maker's chuck of the same size stays inside: an SK30 16 mm
/// chuck catalogued with a Ø30 nose and a Ø37.06 body reached 45.9 above
/// the nose face (decision file) is under the drawn outline.
#[test]
fn a_wider_makers_chuck_stays_inside() {
    let e = entry("SK30 Shrink 16 mm");
    let nose = 30.0;
    let pts = zr(&e, nose, 65.0);
    for i in 0..=100 {
        let dz = 45.9 * f64::from(i) / 100.0;
        let theirs = (15.0 + dz * SHRINK_TAPER).min(37.06 / 2.0);
        assert!(radius_at(&pts, nose + dz) >= theirs - 1e-9, "dz {dz}");
    }
    assert!(radius_at(&pts, nose + 45.9) >= 37.06 / 2.0);
}

/// The picker shows one shrink-fit row for each taper; a pick resolves
/// to the bore that takes the tool's shank, or is refused by name.
#[test]
fn the_picker_groups_the_shrink_bores() {
    let rows: Vec<String> = picker_rows().into_iter().filter(|r| matches!(r, PickerRow::Shrink { .. })).map(|r| r.name()).collect();
    let want = ["BT30", "BT40", "CAT40", "SK30", "SK40", "ISO30", "HSK-A63", "HSK-F63", "HSK-E40", "HSK-E32", "BT50", "CAT50"];
    assert_eq!(rows, want.map(|t| format!("{t} Shrink")));
    assert!(picker_rows().iter().all(|r| !matches!(r, PickerRow::Entry(e) if matches!(e.clamp, Clamp::Shrink { .. }))));
    assert_eq!(resolve("bt40  shrink", 12.0), find("BT40 Shrink 12 mm"));
    let err = resolve("HSK-E32 Shrink", 7.0).expect_err("no 7 mm bore");
    assert_eq!(
        err.to_string(),
        "a Ø7.000 mm shank does not fit - the holder takes a HSK-E32 Shrink bore: \
         Ø3.000, Ø4.000, Ø5.000, Ø6.000, Ø8.000, Ø10.000, Ø12.000, Ø14.000, Ø16.000 mm"
    );
    assert_eq!(resolve("TTS Shrink", 12.0), Err(LibraryError::Unknown("TTS Shrink".into())));
    // A shrink-fit bore takes its own shank only.
    let e = entry("CAT40 Shrink 12 mm");
    assert!(e.fits_shank(12.0).is_ok() && e.fits_shank(12.7).is_err() && e.fits_shank(f64::NAN).is_err());
}

/// With no flange the body runs to the given face and closes there; a
/// flare already above the nose takes the nose face whole.
#[test]
fn the_envelope_handles_its_edge_cases() {
    let pts: Vec<(f64, f64)> = envelope(0.0, 10.0, 12.0, 100.0, 0.0).iter().map(|p| (p.z, p.r)).collect();
    let zc = 2.0 / SHRINK_TAPER;
    assert!(close(&pts, &[(0.0, 0.0), (0.0, 10.0), (zc, 12.0), (100.0, 12.0), (100.0, 0.0)]), "{pts:?}");
    let pts: Vec<(f64, f64)> = envelope(0.0, 10.0, 12.0, 5.0, 30.0).iter().map(|p| (p.z, p.r)).collect();
    assert!(close(&pts, &[(0.0, 0.0), (0.0, 25.0), (5.0, 30.0)]), "{pts:?}");
}
