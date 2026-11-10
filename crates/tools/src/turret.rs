//! The turret side a lathe's tool orientation numbers are read in
//! (#650): the 1-9 chart is a drawing, and a rear-turret lathe reads
//! the same drawing mirrored in X.
//!
//! The chart ([`crate::orientation`]) is drawn for a FRONT tool post -
//! X+ toward the operator, the tool in front of the spindle. On a
//! REAR turret (the tool behind the spindle, X+ away from the
//! operator) the same physical insert stands on the other side of the
//! drawing: the standard right-hand OD tool, whose tip stands at
//! (-X, -Z) from its nose centre on either machine, is drawn in the
//! chart's position 3 instead of 2. The numbering mirrors in X -
//! 1 <-> 4, 2 <-> 3, 6 <-> 8 - and 5, 7, 9 (on the Z line) do not
//! move. The physical offset in the control's own X (radius) and Z is
//! identical on both machines; only the NUMBER the operator types
//! into the control's tool table changes.
//!
//! The law here: a tool stores its orientation in the chart's own
//! (front) frame - the CANONICAL code, a physical fact about the
//! insert - and every place a number meets a person (the tool editor,
//! a refusal, the post's tool note) shows it in the mounted machine's
//! frame. A front machine shows the stored code unchanged, so every
//! project and library written before #650 reads exactly as it did.
//!
//! Provenance: LinuxCNC Lathe User Information (tool orientation
//! chart, public docs) for the chart; the rear-turret mirror is
//! confirmed against the behaviour of Tormach PathPilot's lathe
//! conversational (rear turret: a standard OD turning tool is
//! orientation 3, an OD threading tool 8) - read as a behaviour
//! reference only, no text or output copied. Our own design otherwise.

use crate::orientation::{self, Contact};

/// Which side of the spindle a lathe's tools stand on - the frame its
/// tool table numbers orientations in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turret {
    /// In front of the spindle, X+ toward the operator - the frame the
    /// chart is drawn in. Every machine that states no side.
    #[default]
    Front,
    /// Behind the spindle, X+ away from the operator (a slant-bed or
    /// gang-tool rear turret): the chart's numbers mirror in X.
    Rear,
}

impl Turret {
    /// The `.mch` word (`general.turret`).
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Turret::Front => "front",
            Turret::Rear => "rear",
        }
    }

    /// Read a `.mch` word; None for anything but `front` / `rear`.
    #[must_use]
    pub fn from_word(w: &str) -> Option<Self> {
        match w.trim().to_ascii_lowercase().as_str() {
            "front" => Some(Turret::Front),
            "rear" => Some(Turret::Rear),
            _ => None,
        }
    }

    /// The number this machine's tool table gives a canonical (chart,
    /// front-frame) code. Out-of-range codes pass through untouched -
    /// they are refused by name elsewhere, never renumbered.
    #[must_use]
    pub fn shown(self, canonical: u8) -> u8 {
        match self {
            Turret::Front => canonical,
            Turret::Rear => mirror(canonical),
        }
    }

    /// The canonical code for a number typed in this machine's frame
    /// (the mirror is its own inverse).
    #[must_use]
    pub fn canonical(self, shown: u8) -> u8 {
        self.shown(shown)
    }

    /// The orientations 1-8, NUMBERED IN THIS FRAME and sorted, whose
    /// nose presents every contact in `need` - the list a refusal names.
    #[must_use]
    pub fn fitting(self, need: &[Contact]) -> Vec<u8> {
        let mut v: Vec<u8> = orientation::fitting(need).into_iter().map(|q| self.shown(q)).collect();
        v.sort_unstable();
        v
    }

    /// The codes an OD blade can be touched off as, numbered in this
    /// frame and sorted (#641's `[1, 2, 6]` on a front machine).
    #[must_use]
    pub fn blade_fitting(self) -> Vec<u8> {
        let mut v: Vec<u8> = orientation::BLADE_FITTING.iter().map(|q| self.shown(*q)).collect();
        v.sort_unstable();
        v
    }

    /// The editor's label for a canonical code, numbered in this frame:
    /// the number changes, the physical description does not.
    #[must_use]
    pub fn label(self, canonical: u8) -> String {
        format!("{} - {}", self.shown(canonical), orientation::describe(canonical))
    }

    /// The editor's blade label for a canonical code, in this frame.
    #[must_use]
    pub fn blade_label(self, canonical: u8) -> String {
        format!("{} - {}", self.shown(canonical), orientation::blade_describe(canonical))
    }

    /// The canonical codes in the order this frame's numbers run, for
    /// the editor's list (1..9 as the machine numbers them).
    #[must_use]
    pub fn codes_in_order(self) -> [u8; 9] {
        let mut out = [0u8; 9];
        for (slot, shown) in out.iter_mut().zip(1..=9u8) {
            *slot = self.canonical(shown);
        }
        out
    }

    /// The editor's caption for the frame.
    #[must_use]
    pub fn caption(self) -> &'static str {
        match self {
            Turret::Front => "front-turret numbering (X+ toward the operator)",
            Turret::Rear => "rear-turret numbering (X+ away from the operator): the chart mirrored in X",
        }
    }
}

/// The chart mirrored in X: 1 <-> 4, 2 <-> 3, 6 <-> 8; 5, 7, 9 and
/// every out-of-range code unchanged.
#[must_use]
pub fn mirror(q: u8) -> u8 {
    match q {
        1 => 4,
        4 => 1,
        2 => 3,
        3 => 2,
        6 => 8,
        8 => 6,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rear_frame_mirrors_the_chart_in_x() {
        for (f, r) in [(1, 4), (2, 3), (3, 2), (4, 1), (5, 5), (6, 8), (7, 7), (8, 6), (9, 9)] {
            assert_eq!(Turret::Rear.shown(f), r, "front {f}");
            assert_eq!(Turret::Rear.canonical(r), f, "rear {r}");
            assert_eq!(Turret::Front.shown(f), f);
            // The physical offset is the mirror's X sign flipped: the
            // same tool, drawn on the other side of the spindle.
            let (Some((fx, fz)), Some((rx, rz))) = (orientation::tip_offset(f), orientation::tip_offset(r)) else {
                panic!("chart code {f}");
            };
            assert_eq!((fx, fz), (-rx, rz));
        }
        assert_eq!(mirror(0), 0);
        assert_eq!(mirror(12), 12);
    }

    #[test]
    fn the_pathpilot_numbers_read_as_the_standard_tools() {
        // A rear-turret OD turning tool is 3, an OD thread tip 8: the
        // front chart's 2 and 6.
        assert_eq!(Turret::Rear.canonical(3), 2);
        assert_eq!(Turret::Rear.canonical(8), 6);
        // Defaults show in the frame: OD insert 3, boring bar 2.
        assert_eq!(Turret::Rear.label(2), "3 - OD, cuts toward the chuck");
        assert_eq!(Turret::Rear.label(3), "2 - ID, cuts toward the chuck");
        assert_eq!(Turret::Front.label(2), orientation::label(2));
    }

    #[test]
    fn refusal_lists_are_numbered_in_the_frame() {
        // The PathPilot chamfer listing, [1, 5, 6] and [2, 6, 7] in
        // rear numbers, is the bore-mouth chamfer pair (+X with +Z or
        // -Z) - the front chart's [4, 5, 8] / [3, 7, 8].
        assert_eq!(Turret::Rear.fitting(&[Contact::PlusX, Contact::PlusZ]), vec![1, 5, 6]);
        assert_eq!(Turret::Rear.fitting(&[Contact::PlusX, Contact::MinusZ]), vec![2, 6, 7]);
        assert_eq!(Turret::Front.fitting(&[Contact::PlusX, Contact::PlusZ]), vec![4, 5, 8]);
        // Facing, in each frame.
        assert_eq!(Turret::Front.fitting(&[Contact::MinusX, Contact::MinusZ]), vec![2, 6, 7]);
        assert_eq!(Turret::Rear.fitting(&[Contact::MinusX, Contact::MinusZ]), vec![3, 7, 8]);
        assert_eq!(Turret::Front.blade_fitting(), vec![1, 2, 6]);
        assert_eq!(Turret::Rear.blade_fitting(), vec![3, 4, 8]);
    }

    #[test]
    fn words_and_order() {
        assert_eq!(Turret::from_word("Rear"), Some(Turret::Rear));
        assert_eq!(Turret::from_word("front"), Some(Turret::Front));
        assert_eq!(Turret::from_word("left"), None);
        assert_eq!(Turret::default(), Turret::Front);
        assert_eq!(Turret::Rear.codes_in_order(), [4, 3, 2, 1, 5, 8, 7, 6, 9]);
        assert_eq!(Turret::Front.codes_in_order(), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
