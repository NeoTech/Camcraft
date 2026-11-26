//! #3642: an indexable insert's designation code (ISO 1832, metric
//! form) parsed into a typed [`InsertSpec`]: positions 1 to 7 are
//! shape, clearance, tolerance class, fixing, size, thickness and the
//! corner; what follows (edge condition, hand, the maker's chip
//! breaker) is kept as written. Our own parser over the position
//! layout that tool vendors publish with their catalogs; no standard
//! text is copied.
//!
//! The law: every malformed code is refused by name - the standard,
//! the position, what was found and what that position takes. An
//! empty, short, non-ASCII or inch-form code is a clean error, never a
//! panic.

use crate::insert_codes::{
    Clearance, Fixing, Hand, InsertShape, CLEARANCE_LETTERS, FIXING_LETTERS, SHAPE_LETTERS, TOLERANCE_LETTERS,
};

/// Why a designation code is refused. Each refusal names the standard
/// and, where it can, the position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignationError {
    Empty { standard: &'static str },
    NonAscii { standard: &'static str, text: String },
    TooShort { standard: &'static str, text: String, len: usize, need: usize },
    Position { standard: &'static str, pos: u8, name: &'static str, found: String, expected: String },
    /// A value built in code whose text reads back as a different value.
    NotCanonical { standard: &'static str, text: String },
}

impl core::fmt::Display for DesignationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty { standard } => write!(f, "the {standard} designation is empty"),
            Self::NonAscii { standard, text } => {
                write!(f, "the {standard} designation '{text}' has a character that is not ASCII")
            }
            Self::TooShort { standard, text, len, need } => write!(
                f,
                "the {standard} designation '{text}' has {len} characters - it needs {need} at least"
            ),
            Self::Position { standard, pos, name, found, expected } => {
                write!(f, "{standard} position {pos} ({name}): '{found}' is not valid - it takes {expected}")
            }
            Self::NotCanonical { standard, text } => {
                write!(f, "the {standard} value '{text}' does not read back as the same value")
            }
        }
    }
}

impl std::error::Error for DesignationError {}

const STD: &str = "ISO 1832";

/// Position 6: the thickness code, two characters. Digits are the
/// thickness in whole mm; a `T` and a digit is a size between two
/// whole mm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Thickness([u8; 2]);

impl Thickness {
    /// The code for its two characters; `None` if they are not one.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let b = code.as_bytes();
        let &[a, d] = b else { return None };
        let ok = (a.is_ascii_digit() || a == b'T') && d.is_ascii_digit() && !(a == b'0' && d == b'0');
        ok.then_some(Self([a, d]))
    }

    #[must_use]
    pub fn code(self) -> String {
        self.0.iter().map(|&b| char::from(b)).collect()
    }

    /// The nominal thickness (mm) of the common codes; `None` for a
    /// code this table does not hold.
    #[must_use]
    pub fn nominal_mm(self) -> Option<f64> {
        Some(match &self.0 {
            b"01" => 1.59,
            b"T1" => 1.98,
            b"02" => 2.38,
            b"T2" => 2.78,
            b"03" => 3.18,
            b"T3" => 3.97,
            b"04" => 4.76,
            b"05" => 5.56,
            b"06" => 6.35,
            b"07" => 7.94,
            b"09" => 9.52,
            _ => return None,
        })
    }
}

/// Position 7: the corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Corner {
    /// A nose radius in tenths of a mm ("08" is 0.8 mm).
    Radius(u8),
    /// A sharp corner ("00" on an insert that is not round).
    Sharp,
    /// A round insert ("M0" or "00").
    Round,
    /// A milling insert's wiper facet: the letter of its edge angle
    /// (A, D, E, F, P or Z) and its clearance letter.
    Facet { angle: char, clearance: Clearance },
}

impl Corner {
    /// The nose radius in mm; `None` for a round insert or a facet.
    #[must_use]
    pub fn nose_radius_mm(self) -> Option<f64> {
        match self {
            Self::Radius(t) => Some(f64::from(t) / 10.0),
            Self::Sharp => Some(0.0),
            Self::Round | Self::Facet { .. } => None,
        }
    }

    fn code(self) -> String {
        match self {
            Self::Radius(t) => format!("{t:02}"),
            Self::Sharp => "00".to_owned(),
            Self::Round => "M0".to_owned(),
            Self::Facet { angle, clearance } => format!("{angle}{}", clearance.letter()),
        }
    }
}

/// The inscribed-circle diameters (mm) of the common metric and inch
/// insert sizes, smallest first.
pub const IC_SERIES: [f64; 11] = [3.97, 4.76, 5.56, 6.35, 7.94, 9.525, 12.7, 15.875, 19.05, 25.4, 31.75];

/// The edge length is cut to whole mm after this much (mm) is added:
/// a 6.35 mm triangle's edge works out to 10.998 mm, and its code is 11.
const EDGE_SLACK: f64 = 0.02;

/// What position 7 takes, for a refusal of a corner built in code.
const CORNER_TAKES: &str =
    "a radius of 00 to 99 (0.1 mm, not on shape R), a round corner (shape R only), or a facet: A D E F P Z and a clearance letter";

/// One insert, as its designation code states it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InsertSpec {
    pub shape: InsertShape,
    pub clearance: Clearance,
    /// Position 3, kept as its letter.
    pub tolerance: char,
    pub fixing: Fixing,
    /// Position 5: the edge length in whole mm (the diameter of a
    /// round insert).
    pub size: u8,
    pub thickness: Thickness,
    pub corner: Corner,
    /// The cutting direction, when the code states one after
    /// position 7; `None` when it does not.
    pub hand: Option<Hand>,
    /// All that follows position 7, as written (upper case, no
    /// spaces): the edge condition, the hand, the maker's chip breaker.
    pub tail: String,
}

impl InsertSpec {
    /// Parse a metric designation. Spaces are ignored and letters are
    /// read in upper case: "cnmg 12 04 08-pm" is "CNMG120408-PM".
    pub fn parse(text: &str) -> Result<Self, DesignationError> {
        let core = normalize(STD, text, 10)?;
        let b = core.as_bytes();
        let at = |i: usize| char::from(b[i]);
        let pos = |pos: u8, name, found: String, expected: &str| DesignationError::Position {
            standard: STD,
            pos,
            name,
            found,
            expected: expected.to_owned(),
        };
        let shape = InsertShape::from_letter(at(0)).ok_or_else(|| pos(1, "shape", at(0).into(), SHAPE_LETTERS))?;
        let clearance =
            Clearance::from_letter(at(1)).ok_or_else(|| pos(2, "clearance", at(1).into(), CLEARANCE_LETTERS))?;
        let tolerance = at(2);
        if !(tolerance.is_ascii_uppercase() && TOLERANCE_LETTERS.contains(tolerance)) {
            return Err(pos(3, "tolerance class", tolerance.into(), TOLERANCE_LETTERS));
        }
        let fixing = Fixing::from_letter(at(3)).ok_or_else(|| pos(4, "fixing", at(3).into(), FIXING_LETTERS))?;
        let size = two_digits(&core[4..6])
            .filter(|&n| n > 0)
            .ok_or_else(|| pos(5, "size", core[4..6].to_owned(), "two digits, 01 to 99 (an inch code is not read)"))?;
        let thickness = Thickness::from_code(&core[6..8])
            .ok_or_else(|| pos(6, "thickness", core[6..8].to_owned(), "two digits, or T and a digit"))?;
        let corner = parse_corner(shape, &core[8..10]).ok_or_else(|| {
            let expected = if shape == InsertShape::R {
                "M0 or 00 on a round insert"
            } else {
                "two digits (radius in 0.1 mm), or a facet: A D E F P Z and a clearance letter"
            };
            pos(7, "corner", core[8..10].to_owned(), expected)
        })?;
        let tail = core[10..].to_owned();
        if let Some(bad) = tail.chars().find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '/'))) {
            return Err(pos(8, "suffix", bad.into(), "letters, digits, '-', '.' or '/'"));
        }
        Ok(Self { shape, clearance, tolerance, fixing, size, thickness, corner, hand: tail_hand(&tail), tail })
    }

    /// Refuse a spec built in code (its fields are public) that its own
    /// text would refuse or read back as a different value: a tolerance
    /// letter off the table or in lower case, a size off 01 to 99, a
    /// corner the shape cannot have, a suffix character the parser
    /// changes or refuses, a hand the suffix does not state. Each
    /// refusal names the position. The letter enums and [`Thickness`]
    /// hold only valid values, so they need no check.
    pub fn check(&self) -> Result<(), DesignationError> {
        let pos = |pos: u8, name, found: String, expected: &str| DesignationError::Position {
            standard: STD,
            pos,
            name,
            found,
            expected: expected.to_owned(),
        };
        let t = self.tolerance;
        if !(t.is_ascii_uppercase() && TOLERANCE_LETTERS.contains(t)) {
            return Err(pos(3, "tolerance class", t.into(), TOLERANCE_LETTERS));
        }
        if !(1..=99).contains(&self.size) {
            return Err(pos(5, "size", self.size.to_string(), "01 to 99"));
        }
        if parse_corner(self.shape, &self.corner.code()) != Some(self.corner) {
            return Err(pos(7, "corner", format!("{:?}", self.corner), CORNER_TAKES));
        }
        let allowed = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '/');
        if let Some(bad) = self.tail.chars().find(|&c| !allowed(c)) {
            return Err(pos(8, "suffix", bad.into(), "upper-case letters, digits, '-', '.' or '/'"));
        }
        if self.hand != tail_hand(&self.tail) {
            let found = self.hand.map_or_else(|| "none".to_owned(), |h| h.letter().to_string());
            return Err(pos(8, "hand", found, "the hand that the suffix states"));
        }
        // The guard for the whole class: the value is the parse of its text.
        let text = self.to_string();
        if Self::parse(&text).as_ref() != Ok(self) {
            return Err(DesignationError::NotCanonical { standard: STD, text });
        }
        Ok(())
    }

    /// The inscribed circle (mm): the series size whose edge, cut to
    /// whole mm (after [`EDGE_SLACK`]), is the size code. `None` for a parallelogram, a
    /// rectangle or a round insert, and for a size off the series.
    #[must_use]
    pub fn inscribed_circle_mm(&self) -> Option<f64> {
        let k = self.shape.edge_per_ic()?;
        IC_SERIES.iter().copied().find(|ic| (ic * k + EDGE_SLACK).floor() == f64::from(self.size))
    }
}

impl core::fmt::Display for InsertSpec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{}{}{:02}{}{}{}",
            self.shape.letter(),
            self.clearance.letter(),
            self.tolerance,
            self.fixing.letter(),
            self.size,
            self.thickness.code(),
            self.corner.code(),
            self.tail
        )
    }
}

/// Trim, refuse empty and non-ASCII text, drop the spaces, read upper
/// case, and refuse a code shorter than `need`.
pub(crate) fn normalize(standard: &'static str, text: &str, need: usize) -> Result<String, DesignationError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(DesignationError::Empty { standard });
    }
    if !trimmed.is_ascii() {
        return Err(DesignationError::NonAscii { standard, text: trimmed.to_owned() });
    }
    let core: String = trimmed.chars().filter(|c| !c.is_ascii_whitespace()).map(|c| c.to_ascii_uppercase()).collect();
    if core.len() < need {
        return Err(DesignationError::TooShort { standard, text: trimmed.to_owned(), len: core.len(), need });
    }
    Ok(core)
}

/// Two ASCII digits as a number.
pub(crate) fn two_digits(s: &str) -> Option<u8> {
    match s.as_bytes() {
        &[a, b] if a.is_ascii_digit() && b.is_ascii_digit() => Some((a - b'0') * 10 + (b - b'0')),
        _ => None,
    }
}

fn parse_corner(shape: InsertShape, s: &str) -> Option<Corner> {
    let round = shape == InsertShape::R;
    if s == "M0" || (round && s == "00") {
        return round.then_some(Corner::Round);
    }
    if round {
        return None;
    }
    if let Some(t) = two_digits(s) {
        return Some(if t == 0 { Corner::Sharp } else { Corner::Radius(t) });
    }
    let mut c = s.chars();
    let (angle, clr) = (c.next()?, c.next()?);
    let clearance = Clearance::from_letter(clr)?;
    "ADEFPZ".contains(angle).then_some(Corner::Facet { angle, clearance })
}

/// The hand a code states after position 7: one letter (R, L, N), or
/// an edge condition letter (F, E, T, S, K, P) and then the hand.
/// Anything else (a maker's chip breaker) states no hand.
fn tail_hand(tail: &str) -> Option<Hand> {
    let head = tail.split('-').next().unwrap_or("");
    let b = head.as_bytes();
    match *b {
        [h] => Hand::from_letter(char::from(h)),
        [e, h] if b"FETSKP".contains(&e) => Hand::from_letter(char::from(h)),
        _ => None,
    }
}
