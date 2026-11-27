//! The HSK hollow-taper holders of the built-in holder library (#3919):
//! HSK-A63 and HSK-E40/E32 (DIN 69893-1 / -5, ISO 12164-1) and HSK-F63
//! (DIN 69893-6). Specced from the DIN 69893 dimension tables as
//! tooling makers publish them (the numbers and their sources are in
//! docs/decisions/3919-standard-holders.md); the outlines are our own
//! conservative bodies, drawn by the laws of `holder_library.rs`, not
//! any vendor's model.
//!
//! An HSK holder seats with its flange's plane face on the spindle
//! face, so that face is the gauge line: no standoff. The collar (f1:
//! the plane face to the collar's tool-side face) is drawn as a full
//! cylinder at the flange diameter (d1) - the gripper groove, the
//! orientation notch and the drive slots in it are enveloped, never cut
//! away. The hollow taper above the gauge line is inside the spindle.

use super::{family, HolderEntry, Taper, ER11, ER16, ER20, ER25, ER32, ER40, INCH_BORES, METRIC_BORES};

/// DIN 69893-1 HSK-A63: Ø63 flange (d1), 26 collar (f1), Ø48 taper
/// gauge (d2).
pub const HSK_A63: Taper = Taper { name: "HSK-A63", ring: "flange", gauge_d: 48.0, flange_d: 63.0, standoff: 0.0, flange_t: 26.0 };
/// DIN 69893-6 HSK-F63: the Ø63 flange and 26 collar of A63 on the
/// smaller Ø38 taper (d2) of size 50.
pub const HSK_F63: Taper = Taper { name: "HSK-F63", ring: "flange", gauge_d: 38.0, flange_d: 63.0, standoff: 0.0, flange_t: 26.0 };
/// DIN 69893-5 HSK-E40: Ø40 flange (d1), 20 collar (f1), Ø30 taper
/// gauge (d2).
pub const HSK_E40: Taper = Taper { name: "HSK-E40", ring: "flange", gauge_d: 30.0, flange_d: 40.0, standoff: 0.0, flange_t: 20.0 };
/// DIN 69893-5 HSK-E32: Ø32 flange (d1), 20 collar (f1), Ø24 taper
/// gauge (d2).
pub const HSK_E32: Taper = Taper { name: "HSK-E32", ring: "flange", gauge_d: 24.0, flange_d: 32.0, standoff: 0.0, flange_t: 20.0 };

/// The HSK entries, in the editor's order: A63, F63, E40, E32. The ER
/// sizes and Weldon bores are the ones each flange carries; the default
/// gauge lengths (plane face to nose face) are typical catalog lengths.
pub(super) fn entries() -> Vec<HolderEntry> {
    let sixty_three = |t: Taper| {
        let ers = [(ER16, 100.0), (ER20, 100.0), (ER25, 100.0), (ER32, 100.0), (ER40, 100.0)];
        family(t, &ers, METRIC_BORES.iter().chain(INCH_BORES.iter()), 80.0)
    };
    let mut v = sixty_three(HSK_A63);
    v.extend(sixty_three(HSK_F63));
    let ers = [(ER11, 70.0), (ER16, 70.0), (ER20, 70.0), (ER25, 70.0)];
    let bores = METRIC_BORES.iter().filter(|b| b.1 <= 12.0).chain(INCH_BORES.iter().filter(|b| b.1 <= 12.7));
    v.extend(family(HSK_E40, &ers, bores, 65.0));
    let ers = [(ER11, 60.0), (ER16, 60.0), (ER20, 60.0)];
    let bores = METRIC_BORES.iter().filter(|b| b.1 <= 8.0).chain(INCH_BORES.iter().filter(|b| b.1 <= 6.35));
    v.extend(family(HSK_E32, &ers, bores, 60.0));
    v
}

#[cfg(test)]
#[path = "holder_library_hsk_tests.rs"]
mod tests;
