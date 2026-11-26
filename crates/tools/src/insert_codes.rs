//! #3642: the letter codes an indexable insert designation is built
//! from - shape, clearance, tolerance class and fixing - as typed
//! values with their letters. Our own tables of the code meanings that
//! the ISO 1832 designation system publishes and that tool vendors
//! print in their catalogs; no standard text is copied.

/// Position 1: the insert's shape, read by its corner angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InsertShape {
    /// Parallelogram, 85 degrees.
    A,
    /// Parallelogram, 82 degrees.
    B,
    /// Rhombic, 80 degrees.
    C,
    /// Rhombic, 55 degrees.
    D,
    /// Rhombic, 75 degrees.
    E,
    /// Hexagon, 120 degrees.
    H,
    /// Parallelogram, 55 degrees.
    K,
    /// Rectangle, 90 degrees.
    L,
    /// Rhombic, 86 degrees.
    M,
    /// Octagon, 135 degrees.
    O,
    /// Pentagon, 108 degrees.
    P,
    /// Round.
    R,
    /// Square, 90 degrees.
    S,
    /// Triangle, 60 degrees.
    T,
    /// Rhombic, 35 degrees.
    V,
    /// Trigon, 80 degrees.
    W,
}

/// The shape letters in one place, for the parsers' refusals.
pub const SHAPE_LETTERS: &str = "A B C D E H K L M O P R S T V W";

impl InsertShape {
    /// The shape for its letter; `None` for a letter that is not one.
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Some(match c {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            'D' => Self::D,
            'E' => Self::E,
            'H' => Self::H,
            'K' => Self::K,
            'L' => Self::L,
            'M' => Self::M,
            'O' => Self::O,
            'P' => Self::P,
            'R' => Self::R,
            'S' => Self::S,
            'T' => Self::T,
            'V' => Self::V,
            'W' => Self::W,
            _ => return None,
        })
    }

    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::H => 'H',
            Self::K => 'K',
            Self::L => 'L',
            Self::M => 'M',
            Self::O => 'O',
            Self::P => 'P',
            Self::R => 'R',
            Self::S => 'S',
            Self::T => 'T',
            Self::V => 'V',
            Self::W => 'W',
        }
    }

    /// The corner angle in degrees; `None` for a round insert.
    #[must_use]
    pub fn corner_deg(self) -> Option<f64> {
        Some(match self {
            Self::A => 85.0,
            Self::B => 82.0,
            Self::C | Self::W => 80.0,
            Self::D | Self::K => 55.0,
            Self::E => 75.0,
            Self::H => 120.0,
            Self::L | Self::S => 90.0,
            Self::M => 86.0,
            Self::O => 135.0,
            Self::P => 108.0,
            Self::T => 60.0,
            Self::V => 35.0,
            Self::R => return None,
        })
    }

    /// The edge length over the inscribed circle's diameter, for the
    /// shapes whose every edge touches that circle. Each edge is the
    /// two tangent lengths from its end corners, `r / tan(a / 2)`. A
    /// trigon alternates 80 and 160 degree corners. `None` for a
    /// parallelogram or a rectangle (no circle touches all four
    /// edges) and for a round insert (its size is its diameter).
    #[must_use]
    pub fn edge_per_ic(self) -> Option<f64> {
        let cot_half = |deg: f64| 1.0 / (deg.to_radians() * 0.5).tan();
        // The edge in units of the radius, halved to units of the diameter.
        let per_r = match self {
            Self::C | Self::D | Self::E | Self::M | Self::V => {
                let a = self.corner_deg()?;
                cot_half(a) + cot_half(180.0 - a)
            }
            Self::S | Self::T | Self::H | Self::O | Self::P => 2.0 * cot_half(self.corner_deg()?),
            Self::W => cot_half(80.0) + cot_half(160.0),
            Self::A | Self::B | Self::K | Self::L | Self::R => return None,
        };
        Some(per_r * 0.5)
    }
}

/// Position 2 (and position 4 of a holder): the clearance angle under
/// the cutting edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Clearance {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    N,
    P,
    /// Another clearance, stated in the catalog.
    O,
}

pub const CLEARANCE_LETTERS: &str = "A B C D E F G N P O";

impl Clearance {
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Some(match c {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            'D' => Self::D,
            'E' => Self::E,
            'F' => Self::F,
            'G' => Self::G,
            'N' => Self::N,
            'P' => Self::P,
            'O' => Self::O,
            _ => return None,
        })
    }

    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::N => 'N',
            Self::P => 'P',
            Self::O => 'O',
        }
    }

    /// The angle in degrees; `None` for "another clearance".
    #[must_use]
    pub fn degrees(self) -> Option<f64> {
        Some(match self {
            Self::A => 3.0,
            Self::B => 5.0,
            Self::C => 7.0,
            Self::D => 15.0,
            Self::E => 20.0,
            Self::F => 25.0,
            Self::G => 30.0,
            Self::N => 0.0,
            Self::P => 11.0,
            Self::O => return None,
        })
    }
}

/// Position 3: the tolerance class letters. Kept as its letter: no
/// fit check reads it.
pub const TOLERANCE_LETTERS: &str = "A C E F G H J K L M N U";

/// Position 4: how the insert is held and whether it has a chip
/// breaker - the fixing hole is what a holder's clamping must match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fixing {
    /// No hole, no chip breaker.
    N,
    /// No hole, chip breaker on one face.
    R,
    /// No hole, chip breakers on both faces.
    F,
    /// Cylindrical hole, no chip breaker.
    A,
    /// Cylindrical hole, chip breaker on one face.
    M,
    /// Cylindrical hole, chip breakers on both faces.
    G,
    /// Hole countersunk on one face (40-60 degrees), no chip breaker.
    W,
    /// Hole countersunk on one face (40-60 degrees), chip breaker.
    T,
    /// Hole countersunk on both faces (40-60 degrees), no chip breaker.
    Q,
    /// Hole countersunk on both faces (40-60 degrees), chip breakers.
    U,
    /// Hole countersunk on one face (70-90 degrees), no chip breaker.
    B,
    /// Hole countersunk on one face (70-90 degrees), chip breaker.
    H,
    /// Hole countersunk on both faces (70-90 degrees), no chip breaker.
    C,
    /// Hole countersunk on both faces (70-90 degrees), chip breakers.
    J,
    /// A special form, stated in the catalog.
    X,
}

pub const FIXING_LETTERS: &str = "N R F A M G W T Q U B H C J X";

/// What holds the insert in place, as a clamp reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FixingHole {
    /// A plain insert: a clamp from the top holds it.
    None,
    /// A straight hole: a pin or lever pulls it into the pocket.
    Cylindrical,
    /// A countersunk hole: a screw holds it.
    Countersunk,
    /// A special form: no standard clamp takes it.
    Special,
}

impl Fixing {
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Some(match c {
            'N' => Self::N,
            'R' => Self::R,
            'F' => Self::F,
            'A' => Self::A,
            'M' => Self::M,
            'G' => Self::G,
            'W' => Self::W,
            'T' => Self::T,
            'Q' => Self::Q,
            'U' => Self::U,
            'B' => Self::B,
            'H' => Self::H,
            'C' => Self::C,
            'J' => Self::J,
            'X' => Self::X,
            _ => return None,
        })
    }

    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Self::N => 'N',
            Self::R => 'R',
            Self::F => 'F',
            Self::A => 'A',
            Self::M => 'M',
            Self::G => 'G',
            Self::W => 'W',
            Self::T => 'T',
            Self::Q => 'Q',
            Self::U => 'U',
            Self::B => 'B',
            Self::H => 'H',
            Self::C => 'C',
            Self::J => 'J',
            Self::X => 'X',
        }
    }

    /// The hole that the clamp must match.
    #[must_use]
    pub fn hole(self) -> FixingHole {
        match self {
            Self::N | Self::R | Self::F => FixingHole::None,
            Self::A | Self::M | Self::G => FixingHole::Cylindrical,
            Self::W | Self::T | Self::Q | Self::U | Self::B | Self::H | Self::C | Self::J => FixingHole::Countersunk,
            Self::X => FixingHole::Special,
        }
    }
}

/// The cutting direction of an insert or a holder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    R,
    L,
    /// Neutral: cuts in both directions.
    N,
}

impl Hand {
    #[must_use]
    pub fn from_letter(c: char) -> Option<Self> {
        Some(match c {
            'R' => Self::R,
            'L' => Self::L,
            'N' => Self::N,
            _ => return None,
        })
    }

    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Self::R => 'R',
            Self::L => 'L',
            Self::N => 'N',
        }
    }
}
