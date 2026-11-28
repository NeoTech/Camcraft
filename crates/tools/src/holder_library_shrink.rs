//! The shrink-fit chucks of the built-in holder library (#3919), on
//! each steep taper and HSK taper of the library but TTS. Specced from
//! the DIN 69882-8 form of the shrink chuck (4.5-degree outer taper;
//! nose diameter d2 and body diameter d3 for each clamping diameter d1)
//! as tooling makers publish its table (the numbers and their sources
//! are in docs/decisions/3919-standard-holders-2.md); the outline is our
//! own conservative envelope, not any vendor's model.
//!
//! The laws here, on top of `holder_library.rs`'s:
//! - **Never smaller than the steel.** The nose and the body are drawn
//!   [`SHRINK_MARGIN`] wider in radius than the table, so that the
//!   wider makers' chucks of the same size stay inside.
//! - **One envelope from nose to flange.** The nose cone rises at
//!   [`SHRINK_TAPER`] from the nose face to the body; the 45-degree
//!   flare envelope ([`FLARE`]) runs down from the flange's front face.
//!   The outline is the larger of the two at each height, so a short
//!   chuck whose cone runs into its flare is drawn whole, never refused.

use super::{HolderEntry, Taper, BT30, BT40, CAT40, FLARE, BT50, CAT50, HSK_A63, HSK_E32, HSK_E40, HSK_F63, ISO30, SK30, SK40};
use super::Clamp;
use crate::spun::SpunPoint;

/// The radius (mm) the nose and the body are drawn wider than the
/// table: one maker's chuck of the same size is up to 1.53 wider (an
/// SK30 16 mm chuck with a Ø30 nose and a Ø37.06 body against the
/// table's Ø27 and Ø34), rounded up.
pub const SHRINK_MARGIN: f64 = 2.0;
/// The nose cone's slope: tan(4.5 degrees), the DIN 69882-8 outer taper.
pub const SHRINK_TAPER: f64 = 0.078_701_706_824_618_45;

/// One clamping size: its label, d1 (the bore), d2 (the nose) and d3
/// (the body), mm.
type Size = (&'static str, f64, f64, f64);

/// The DIN 69882-8 sizes as the makers' tables state them.
const SIZES: [Size; 13] = [
    ("3 mm", 3.0, 11.0, 15.0),
    ("4 mm", 4.0, 14.0, 22.0),
    ("5 mm", 5.0, 16.0, 22.0),
    ("6 mm", 6.0, 21.0, 27.0),
    ("8 mm", 8.0, 21.0, 27.0),
    ("10 mm", 10.0, 24.0, 32.0),
    ("12 mm", 12.0, 24.0, 32.0),
    ("14 mm", 14.0, 27.0, 34.0),
    ("16 mm", 16.0, 27.0, 34.0),
    ("18 mm", 18.0, 33.0, 42.0),
    ("20 mm", 20.0, 33.0, 42.0),
    ("25 mm", 25.0, 44.0, 53.0),
    ("32 mm", 32.0, 44.0, 53.0),
];

/// HSK-E32's own body column: the Ø32 flange limits d3 to 26 from 6 to
/// 12 mm.
const E32_SIZES: [Size; 4] = [("6 mm", 6.0, 21.0, 26.0), ("8 mm", 8.0, 21.0, 26.0), ("10 mm", 10.0, 24.0, 26.0), ("12 mm", 12.0, 24.0, 26.0)];

/// Each taper's bores with their default gauge lengths (mm, gauge line
/// to nose face): the shortest length the table lists for that bore.
const BORES: [(Taper, &[(f64, f64)]); 12] = [
    (BT30, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 80.0), (14.0, 80.0), (16.0, 80.0), (20.0, 80.0)]),
    (BT40, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 90.0), (8.0, 90.0), (10.0, 90.0), (12.0, 90.0), (14.0, 90.0), (16.0, 90.0), (18.0, 90.0), (20.0, 90.0), (25.0, 100.0), (32.0, 100.0)]),
    (CAT40, &[(6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 80.0), (16.0, 80.0), (20.0, 80.0), (25.0, 100.0), (32.0, 100.0)]),
    (SK30, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 80.0), (14.0, 80.0), (16.0, 80.0), (20.0, 80.0)]),
    (SK40, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 80.0), (14.0, 80.0), (16.0, 80.0), (18.0, 80.0), (20.0, 80.0), (25.0, 100.0), (32.0, 100.0)]),
    (ISO30, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 80.0), (14.0, 80.0), (16.0, 80.0), (20.0, 80.0)]),
    (HSK_A63, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 70.0), (8.0, 70.0), (10.0, 70.0), (12.0, 70.0), (14.0, 75.0), (16.0, 75.0), (18.0, 95.0), (20.0, 100.0), (25.0, 115.0), (32.0, 120.0)]),
    (HSK_F63, &[(3.0, 80.0), (4.0, 80.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 85.0), (12.0, 90.0), (14.0, 90.0), (16.0, 95.0), (20.0, 80.0), (25.0, 115.0), (32.0, 120.0)]),
    (HSK_E40, &[(3.0, 60.0), (4.0, 70.0), (5.0, 80.0), (6.0, 80.0), (8.0, 80.0), (10.0, 80.0), (12.0, 90.0), (14.0, 90.0), (16.0, 90.0), (18.0, 90.0), (20.0, 90.0)]),
    (HSK_E32, &[(3.0, 60.0), (4.0, 60.0), (5.0, 60.0), (6.0, 70.0), (8.0, 70.0), (10.0, 70.0), (12.0, 80.0), (14.0, 90.0), (16.0, 90.0)]),
    (BT50, &[(6.0, 100.0), (8.0, 100.0), (10.0, 100.0), (12.0, 100.0), (16.0, 100.0), (20.0, 100.0), (25.0, 120.0), (32.0, 120.0)]),
    (CAT50, &[(6.0, 100.0), (8.0, 100.0), (10.0, 100.0), (12.0, 100.0), (14.0, 100.0), (16.0, 100.0), (18.0, 100.0), (20.0, 100.0), (25.0, 100.0), (32.0, 100.0)]),
];

/// The table row of `bore` on `taper` (HSK-E32 reads its own body
/// column where it has one).
fn size(taper: &Taper, bore: f64) -> Option<Size> {
    let own: &[Size] = if taper.name == HSK_E32.name { &E32_SIZES } else { &[] };
    own.iter().chain(SIZES.iter()).find(|s| (s.1 - bore).abs() < 1e-9).copied()
}

/// Every shrink-fit entry, taper by taper in [`BORES`] order.
pub(super) fn entries() -> Vec<HolderEntry> {
    BORES
        .iter()
        .flat_map(|(t, bores)| {
            bores.iter().filter_map(move |&(bore, gauge_len)| {
                let (label, bore, nose_d, body_d) = size(t, bore)?;
                Some(HolderEntry { taper: Some(*t), clamp: Clamp::Shrink { bore, nose_d, body_d }, gauge_len, label })
            })
        })
        .collect()
}

/// The outline from the nose face (`nose`, radius `r0`) to the flange's
/// front face (`front`, radius `fr`): the nose cone at [`SHRINK_TAPER`]
/// up to the body radius `body_r`, under the 45-degree flare that runs
/// down from the flange front, the larger of the two at each height.
/// The flare rises faster than the cone and the body, so the two cross
/// at most once. With no flange (`fr` 0) the body runs to `front` and
/// closes to the axis there.
pub(super) fn envelope(nose: f64, r0: f64, body_r: f64, front: f64, fr: f64) -> Vec<SpunPoint> {
    let p = SpunPoint::line;
    let t = SHRINK_TAPER;
    let body_r = body_r.max(r0);
    // Where the cone reaches the body.
    let zc = nose + (body_r - r0) / t;
    let body = |z: f64| (r0 + (z - nose) * t).min(body_r);
    let flare = |z: f64| fr - (front - z) * FLARE;
    let mut pts = vec![p(nose, 0.0)];
    if flare(nose) >= r0 {
        // The flare is above the cone from the nose face on.
        pts.extend([p(nose, flare(nose)), p(front, fr)]);
        return pts;
    }
    pts.push(p(nose, r0));
    if flare(front) <= body(front) {
        // The body is at least as wide as the flange: no flare.
        if zc < front {
            pts.push(p(zc, body_r));
        }
        pts.extend([p(front, body(front)), p(front, fr)]);
        return pts;
    }
    // The one crossing: on the cone, or on the body's cylinder.
    let on_cone = (fr - front * FLARE - r0 + nose * t) / (t - FLARE);
    let z = if on_cone <= zc { on_cone } else { front - (fr - body_r) / FLARE };
    if zc < z {
        pts.push(p(zc, body_r));
    }
    pts.extend([p(z, body(z)), p(front, fr)]);
    pts
}

#[cfg(test)]
#[path = "holder_library_shrink_tests.rs"]
mod tests;
