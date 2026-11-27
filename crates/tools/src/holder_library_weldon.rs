//! #2018: one Weldon pick for each taper ("a weldon is a weldon"). The
//! picker lists "BT40 Weldon" once; the bore is the library's own
//! standard bore for the tool's shank (DIN 1835 form B / DIN 6359, the
//! inch bores, the TTS set-screw bores), and with it that bore's body
//! diameter. The data keeps one entry for each bore: a real side-lock
//! holder has a fixed bore, and its body diameter is a collision shape,
//! so a grouped pick always resolves to an existing entry and draws
//! exactly its outline. #3919 groups the shrink-fit bores the same way
//! ("BT40 Shrink"). Our own lookup over the library's tables.

use super::{find, library, Clamp, HolderEntry, LibraryError, Taper};

/// A Weldon bore: its label, bore and typical body diameter (mm).
pub(super) type Bore = (&'static str, f64, f64);

/// Metric Weldon bores (DIN 1835 form B shanks) with the body
/// diameters typical of DIN 6359 side-lock holders.
pub(super) const METRIC_BORES: [Bore; 6] =
    [("6 mm", 6.0, 25.0), ("8 mm", 8.0, 28.0), ("10 mm", 10.0, 35.0), ("12 mm", 12.0, 42.0), ("16 mm", 16.0, 48.0), ("20 mm", 20.0, 52.0)];
/// Inch Weldon bores with the body diameters typical of inch end-mill
/// holders (1.00, 1.25, 1.375, 1.75, 1.75 in).
pub(super) const INCH_BORES: [Bore; 5] = [
    ("1/4 in", 6.35, 25.4),
    ("3/8 in", 9.525, 31.75),
    ("1/2 in", 12.7, 34.925),
    ("5/8 in", 15.875, 44.45),
    ("3/4 in", 19.05, 44.45),
];

/// One row of the editor's holder picker: a library entry as it is, or
/// one Weldon (or shrink-fit, #3919) row standing for every bore of
/// that kind on a taper.
#[derive(Clone, Debug, PartialEq)]
pub enum PickerRow {
    Entry(HolderEntry),
    Weldon { taper: Taper, bores: Vec<f64> },
    Shrink { taper: Taper, bores: Vec<f64> },
}

impl PickerRow {
    /// The name the picker lists and [`resolve`] takes back: an entry's
    /// own name, "BT40 Weldon" or "BT40 Shrink".
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Entry(e) => e.name(),
            Self::Weldon { taper, .. } => weldon_name(taper),
            Self::Shrink { taper, .. } => shrink_name(taper),
        }
    }
}

/// The grouped Weldon pick's name for `taper`: "BT40 Weldon".
#[must_use]
pub fn weldon_name(taper: &Taper) -> String {
    format!("{} Weldon", taper.name)
}

/// The grouped shrink-fit pick's name for `taper`: "BT40 Shrink".
#[must_use]
pub fn shrink_name(taper: &Taper) -> String {
    format!("{} Shrink", taper.name)
}

/// The two grouped kinds: Weldon side lock and shrink fit, each a
/// fixed bore for one shank.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Weldon,
    Shrink,
}

impl Kind {
    fn word(self) -> &'static str {
        match self {
            Self::Weldon => "Weldon",
            Self::Shrink => "Shrink",
        }
    }

    /// The entry's bore when it is of this kind.
    fn bore(self, c: Clamp) -> Option<f64> {
        match (self, c) {
            (Self::Weldon, Clamp::Weldon { bore, .. }) | (Self::Shrink, Clamp::Shrink { bore, .. }) => Some(bore),
            _ => None,
        }
    }
}

/// The holder picker's rows, in the library's order: every ER chuck as
/// it is, and each taper's Weldon bores and shrink-fit bores as one row
/// each, where the first of them stands.
#[must_use]
pub fn picker_rows() -> Vec<PickerRow> {
    let mut rows: Vec<PickerRow> = Vec::new();
    for e in library() {
        let grouped = [Kind::Weldon, Kind::Shrink].into_iter().find_map(|k| k.bore(e.clamp).map(|b| (k, b)));
        let (Some(t), Some((kind, bore))) = (e.taper, grouped) else {
            rows.push(PickerRow::Entry(e));
            continue;
        };
        let at = rows.iter().position(|r| match (kind, r) {
            (Kind::Weldon, PickerRow::Weldon { taper, .. }) | (Kind::Shrink, PickerRow::Shrink { taper, .. }) => {
                taper.name == t.name
            }
            _ => false,
        });
        match (at.and_then(|i| rows.get_mut(i)), kind) {
            (Some(PickerRow::Weldon { bores, .. } | PickerRow::Shrink { bores, .. }), _) => bores.push(bore),
            // `position` only finds a grouped row.
            (Some(PickerRow::Entry(_)), _) => {}
            (None, Kind::Weldon) => rows.push(PickerRow::Weldon { taper: t, bores: vec![bore] }),
            (None, Kind::Shrink) => rows.push(PickerRow::Shrink { taper: t, bores: vec![bore] }),
        }
    }
    rows
}

/// The `taper`'s Weldon entry whose bore takes a shank of `shank_d`.
/// A shank no standard bore of the taper takes is refused by name with
/// the fit check's own text, listing the bores there are.
pub fn weldon_for(taper: &str, shank_d: f64) -> Result<HolderEntry, LibraryError> {
    bore_for(taper, shank_d, Kind::Weldon)
}

/// The `taper`'s shrink-fit entry whose bore takes a shank of
/// `shank_d`, refused by name as [`weldon_for`] refuses.
pub fn shrink_for(taper: &str, shank_d: f64) -> Result<HolderEntry, LibraryError> {
    bore_for(taper, shank_d, Kind::Shrink)
}

fn bore_for(taper: &str, shank_d: f64, kind: Kind) -> Result<HolderEntry, LibraryError> {
    let all: Vec<HolderEntry> = library()
        .into_iter()
        .filter(|e| kind.bore(e.clamp).is_some() && e.taper.is_some_and(|t| t.name.eq_ignore_ascii_case(taper)))
        .collect();
    if all.is_empty() {
        return Err(LibraryError::Unknown(format!("{taper} {}", kind.word())));
    }
    if let Some(e) = all.iter().find(|e| e.fits_shank(shank_d).is_ok()) {
        return Ok(*e);
    }
    let bores: Vec<String> = all.iter().filter_map(|e| kind.bore(e.clamp)).map(|b| format!("Ø{b:.3}")).collect();
    let name = all.first().and_then(|e| e.taper).map_or_else(|| taper.to_owned(), |t| t.name.to_owned());
    Err(LibraryError::Shank { shank_d, takes: format!("a {name} {} bore: {} mm", kind.word(), bores.join(", ")) })
}

/// The library entry a pick names, for a tool whose shank is `shank_d`:
/// a grouped Weldon or shrink-fit pick ("BT40 Weldon", "BT40 Shrink",
/// case and spacing ignored) is the taper's bore for that shank; any
/// other name is [`find`]'s entry.
pub fn resolve(name: &str, shank_d: f64) -> Result<HolderEntry, LibraryError> {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.as_slice() {
        [taper, w] if w.eq_ignore_ascii_case("weldon") => weldon_for(taper, shank_d),
        [taper, w] if w.eq_ignore_ascii_case("shrink") => shrink_for(taper, shank_d),
        _ => find(name),
    }
}

#[cfg(test)]
#[path = "holder_library_weldon_tests.rs"]
mod tests;
