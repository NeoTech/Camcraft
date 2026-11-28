//! #3642: an insert seat - the pocket of a turning holder, a boring
//! bar or a cutter body - and the ISO 5608 holder designation that
//! states one. Positions 1 to 9: clamping, insert shape, holder style
//! (the edge angle), insert clearance, hand, shank height, shank
//! width, tool length and insert size. Our own parser over the
//! position layout tool vendors publish; no standard text is copied.

use crate::insert_codes::{Clearance, FixingHole, Hand, InsertShape, CLEARANCE_LETTERS, SHAPE_LETTERS};
use crate::insert_iso1832::{normalize, two_digits, DesignationError, Thickness};

const STD: &str = "ISO 5608";

/// Position 1: how the holder clamps the insert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Clamping {
    /// A clamp from the top on a plain insert.
    C,
    /// A clamp from the top and a pin in the insert's hole.
    M,
    /// A lever that pulls the insert by its hole.
    P,
    /// A screw through the insert's countersunk hole.
    S,
    /// A rigid clamp that pulls the insert by its hole.
    D,
}

pub const CLAMPING_LETTERS: &str = "C M P S D";

impl Clamping {
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Some(match c {
            'C' => Self::C,
            'M' => Self::M,
            'P' => Self::P,
            'S' => Self::S,
            'D' => Self::D,
            _ => return None,
        })
    }

    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Self::C => 'C',
            Self::M => 'M',
            Self::P => 'P',
            Self::S => 'S',
            Self::D => 'D',
        }
    }

    /// The insert hole this clamping holds.
    #[must_use]
    pub fn takes(self) -> FixingHole {
        match self {
            Self::C => FixingHole::None,
            Self::M | Self::P | Self::D => FixingHole::Cylindrical,
            Self::S => FixingHole::Countersunk,
        }
    }
}

/// Position 3: the holder style letter, read as the cutting edge angle
/// (degrees, from the feed direction) it sets on the insert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HolderStyle(char);

pub const STYLE_LETTERS: &str = "A B C D E F G H J K L M N P Q R S T U V W Y";

impl HolderStyle {
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Self(c).approach_deg().map(|_| Self(c))
    }

    #[must_use]
    pub fn letter(self) -> char {
        self.0
    }

    /// The cutting edge angle in degrees.
    #[must_use]
    pub fn approach_deg(self) -> Option<f64> {
        Some(match self.0 {
            'A' | 'C' | 'F' | 'G' => 90.0,
            'B' | 'K' | 'R' => 75.0,
            'D' | 'S' => 45.0,
            'E' | 'T' | 'W' => 60.0,
            'H' | 'Q' => 107.5,
            'J' | 'U' => 93.0,
            'L' => 95.0,
            'M' => 50.0,
            'N' => 63.0,
            'P' => 117.5,
            'V' => 72.5,
            'Y' => 85.0,
            _ => return None,
        })
    }
}

/// One insert pocket: what an insert must be to sit in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InsertSeatSpec {
    pub clamping: Clamping,
    pub shape: InsertShape,
    pub style: HolderStyle,
    pub clearance: Clearance,
    pub hand: Hand,
    /// The insert size code (ISO 1832 position 5) the pocket takes.
    pub size: u8,
    /// The insert thickness the pocket (and its shim) takes; `None`
    /// when the holder does not state one.
    pub thickness: Option<Thickness>,
}

impl InsertSeatSpec {
    /// Refuse a seat built in code (its fields are public) that its
    /// holder code would refuse: an insert size off 01 to 99. The letter
    /// enums, [`HolderStyle`] and [`Thickness`] hold only valid values.
    pub fn check(&self) -> Result<(), DesignationError> {
        if (1..=99).contains(&self.size) {
            return Ok(());
        }
        Err(DesignationError::Position {
            standard: STD,
            pos: 9,
            name: "insert size",
            found: self.size.to_string(),
            expected: "two digits, 01 to 99 (mm)".to_owned(),
        })
    }
}

/// A turning holder as its ISO 5608 code states it: the seat, and the
/// rectangular shank that is its machine side.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HolderDesignation {
    pub seat: InsertSeatSpec,
    /// The shank height and width (mm).
    pub shank_h: u8,
    pub shank_b: u8,
    /// Position 8, the tool length letter.
    pub length: char,
    /// All that follows position 9, as written.
    pub tail: String,
}

impl HolderDesignation {
    /// Parse a metric holder code ("PCLNR2525M12"). Spaces are ignored
    /// and letters are read in upper case.
    pub fn parse(text: &str) -> Result<Self, DesignationError> {
        let core = normalize(STD, text, 12)?;
        let b = core.as_bytes();
        let at = |i: usize| char::from(b[i]);
        let pos = |pos: u8, name, found: String, expected: &str| DesignationError::Position {
            standard: STD,
            pos,
            name,
            found,
            expected: expected.to_owned(),
        };
        let clamping =
            Clamping::from_letter(at(0)).ok_or_else(|| pos(1, "clamping", at(0).into(), CLAMPING_LETTERS))?;
        let shape = InsertShape::from_letter(at(1)).ok_or_else(|| pos(2, "insert shape", at(1).into(), SHAPE_LETTERS))?;
        let style = HolderStyle::from_letter(at(2)).ok_or_else(|| pos(3, "holder style", at(2).into(), STYLE_LETTERS))?;
        let clearance =
            Clearance::from_letter(at(3)).ok_or_else(|| pos(4, "insert clearance", at(3).into(), CLEARANCE_LETTERS))?;
        let hand = Hand::from_letter(at(4)).ok_or_else(|| pos(5, "hand", at(4).into(), "R L N"))?;
        let mm = |p: u8, name, s: &str| {
            two_digits(s).filter(|&n| n > 0).ok_or_else(|| pos(p, name, s.to_owned(), "two digits, 01 to 99 (mm)"))
        };
        let shank_h = mm(6, "shank height", &core[5..7])?;
        let shank_b = mm(7, "shank width", &core[7..9])?;
        let length = at(9);
        if length_mm(length).is_none() && length != 'X' {
            return Err(pos(8, "tool length", length.into(), "A to Y without I, O and Z (X: special)"));
        }
        let size = mm(9, "insert size", &core[10..12])?;
        let tail = core[12..].to_owned();
        if let Some(bad) = tail.chars().find(|c| !(c.is_ascii_alphanumeric() || *c == '-')) {
            return Err(pos(10, "suffix", bad.into(), "letters, digits or '-'"));
        }
        let seat = InsertSeatSpec { clamping, shape, style, clearance, hand, size, thickness: None };
        Ok(Self { seat, shank_h, shank_b, length, tail })
    }

    /// The tool length (mm); `None` for a special length (X).
    #[must_use]
    pub fn length_mm(&self) -> Option<f64> {
        length_mm(self.length)
    }
}

impl core::fmt::Display for HolderDesignation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = &self.seat;
        write!(
            f,
            "{}{}{}{}{}{:02}{:02}{}{:02}{}",
            s.clamping.letter(),
            s.shape.letter(),
            s.style.letter(),
            s.clearance.letter(),
            s.hand.letter(),
            self.shank_h,
            self.shank_b,
            self.length,
            s.size,
            self.tail
        )
    }
}

/// Position 8: the tool length letters and their lengths (mm).
fn length_mm(c: char) -> Option<f64> {
    Some(match c {
        'A' => 32.0,
        'B' => 40.0,
        'C' => 50.0,
        'D' => 60.0,
        'E' => 70.0,
        'F' => 80.0,
        'G' => 90.0,
        'H' => 100.0,
        'J' => 110.0,
        'K' => 125.0,
        'L' => 140.0,
        'M' => 150.0,
        'N' => 160.0,
        'P' => 170.0,
        'Q' => 180.0,
        'R' => 200.0,
        'S' => 250.0,
        'T' => 300.0,
        'U' => 350.0,
        'V' => 400.0,
        'W' => 450.0,
        'Y' => 500.0,
        _ => return None,
    })
}
