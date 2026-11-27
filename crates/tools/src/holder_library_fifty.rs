//! The 50-size steep-taper holders of the built-in holder library
//! (#3919): BT50 (JIS B 6339 / MAS 403 BT) and CAT50 (ASME B5.50), in
//! ER collet chuck and Weldon side-lock versions. Specced from the
//! taper tables as tooling makers publish them (the numbers and their
//! sources are in docs/decisions/3919-standard-holders-2.md); the
//! outlines are our own conservative bodies, drawn by the laws of
//! `holder_library.rs` (full flange cylinder, 45-degree flare, the
//! profile stops at the gauge line), not any vendor's model.

use super::{family, HolderEntry, Taper, ER16, ER20, ER25, ER32, ER40, ER50, INCH_BORES, METRIC_BORES};

/// JIS B 6339 / MAS 403 BT50: Ø69.85 gauge, Ø100 flange, 3 from the
/// gauge line to the flange's back face, a 35 thick flange (its front
/// face 38 from the gauge line).
pub const BT50: Taper = Taper { name: "BT50", ring: "flange", gauge_d: 69.85, flange_d: 100.0, standoff: 3.0, flange_t: 35.0 };
/// ASME B5.50 size 50 (CAT50, V-flange): 2.750 in gauge, 3.875 in
/// flange, 0.125 in standoff, 0.625 in flange (front face 0.750 in from
/// the gauge line) - the CAT40 standoff and flange on the 50 taper.
pub const CAT50: Taper = Taper { name: "CAT50", ring: "flange", gauge_d: 69.85, flange_d: 98.425, standoff: 3.175, flange_t: 15.875 };

/// The default gauge length (mm, gauge line to nose face) of every
/// 50-taper entry: a typical catalog length for the size.
const GAUGE: f64 = 100.0;

/// The BT50 and CAT50 entries, in the editor's order: each taper's ER
/// chucks (ER16-ER50, the whole ER table but ER11), then every Weldon
/// bore - a 50-taper body carries the largest of them.
pub(super) fn entries() -> Vec<HolderEntry> {
    let ers = [(ER16, GAUGE), (ER20, GAUGE), (ER25, GAUGE), (ER32, GAUGE), (ER40, GAUGE), (ER50, GAUGE)];
    let mut v = family(BT50, &ers, METRIC_BORES.iter().chain(INCH_BORES.iter()), GAUGE);
    v.extend(family(CAT50, &ers, INCH_BORES.iter().chain(METRIC_BORES.iter()), GAUGE));
    v
}

#[cfg(test)]
#[path = "holder_library_fifty_tests.rs"]
mod tests;
