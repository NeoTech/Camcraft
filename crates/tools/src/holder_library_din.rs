//! The DIN 69871 (ISO 7388-1 form A/AD) steep-taper holders of the
//! built-in holder library (#3919): SK30, SK40 and SK50, and the router
//! spindles' ISO30, which is the same 30 taper and flange. Specced from
//! the DIN ISO 7388-1 dimension table as tooling makers publish it
//! (the numbers and their sources are in docs/decisions/3919-standard-holders.md
//! and, for SK50, docs/decisions/3919-standard-holders-3.md);
//! the outlines are our own conservative bodies, drawn by the laws of
//! `holder_library.rs` (full flange cylinder, 45-degree flare, the
//! profile stops at the gauge line), not any vendor's model.

use super::{family, HolderEntry, Taper, ER16, ER20, ER25, ER32, ER40, ER50, INCH_BORES, METRIC_BORES};

/// DIN 69871 / ISO 7388-1 size 30: Ø31.75 gauge, Ø50 flange, 3.2 from
/// the gauge line to the flange's back face, a 15.9 thick flange (its
/// front face 19.1 from the gauge line).
pub const SK30: Taper = Taper { name: "SK30", ring: "flange", gauge_d: 31.75, flange_d: 50.0, standoff: 3.2, flange_t: 15.9 };
/// DIN 69871 / ISO 7388-1 size 40: Ø44.45 gauge, Ø63.55 flange, the
/// same 3.2 standoff and 15.9 flange as size 30.
pub const SK40: Taper = Taper { name: "SK40", ring: "flange", gauge_d: 44.45, flange_d: 63.55, standoff: 3.2, flange_t: 15.9 };
/// DIN 69871 / ISO 7388-1 size 50: Ø69.85 gauge, Ø97.5 flange, the
/// same 3.2 standoff and 15.9 flange as sizes 30 and 40.
pub const SK50: Taper = Taper { name: "SK50", ring: "flange", gauge_d: 69.85, flange_d: 97.5, standoff: 3.2, flange_t: 15.9 };
/// The router spindles' ISO30 (automatic tool change spindles): the
/// DIN 69871 size 30 taper and flange under the name router users know.
pub const ISO30: Taper = Taper { name: "ISO30", ..SK30 };

/// The SK30, SK40, SK50 and ISO30 entries, in the editor's order. The
/// ER sizes and Weldon bores are the ones each flange carries on the BT
/// tapers of the same size; the default gauge lengths are typical
/// catalog lengths, as the BT entries use.
pub(super) fn entries() -> Vec<HolderEntry> {
    let thirty = |t: Taper| {
        let ers = [(ER16, 60.0), (ER20, 60.0), (ER25, 60.0), (ER32, 60.0)];
        // The Ø50 flange: the 16 mm and 1/2 in bores are the largest a
        // 30-taper side-lock body is built for.
        let bores = METRIC_BORES.iter().filter(|b| b.1 <= 16.0).chain(INCH_BORES.iter().filter(|b| b.1 <= 12.7));
        family(t, &ers, bores, 60.0)
    };
    let mut v = thirty(SK30);
    let ers = [(ER16, 70.0), (ER20, 70.0), (ER25, 70.0), (ER32, 70.0), (ER40, 80.0), (ER50, 100.0)];
    v.extend(family(SK40, &ers, METRIC_BORES.iter().chain(INCH_BORES.iter()), 63.0));
    // SK50: the BT50 cut - ER16-ER50 and every Weldon bore, at 100.
    let ers = [(ER16, 100.0), (ER20, 100.0), (ER25, 100.0), (ER32, 100.0), (ER40, 100.0), (ER50, 100.0)];
    v.extend(family(SK50, &ers, METRIC_BORES.iter().chain(INCH_BORES.iter()), 100.0));
    v.extend(thirty(ISO30));
    v
}

#[cfg(test)]
#[path = "holder_library_din_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "holder_library_sk50_tests.rs"]
mod sk50_tests;
