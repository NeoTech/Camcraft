//! Why a pick from the built-in holder library is refused (#793): each
//! refusal names itself. Our own refusal texts; split from
//! `holder_library.rs` to keep that file under the 400-line law.

use super::MAX_GAUGE;

/// Why a library pick is refused. Each refusal names itself.
#[derive(Clone, Debug, PartialEq)]
pub enum LibraryError {
    Unknown(String),
    NonFinite,
    BelowTip,
    TooShort { min: f64 },
    TooLong,
    Shank { shank_d: f64, takes: String },
}

impl core::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unknown(n) => write!(f, "there is no holder '{n}' in the holder library"),
            Self::NonFinite => write!(f, "the gauge length and the projection must be finite numbers"),
            Self::BelowTip => write!(f, "the holder's nose cannot sit below the tool tip (projection < 0)"),
            Self::TooShort { min } => {
                write!(f, "the gauge length is shorter than the flange and nose need ({min:.2} mm at least)")
            }
            Self::TooLong => write!(f, "the gauge length is longer than {MAX_GAUGE} mm"),
            Self::Shank { shank_d, takes } => write!(f, "a Ø{shank_d:.3} mm shank does not fit - the holder takes {takes}"),
        }
    }
}

impl std::error::Error for LibraryError {}
