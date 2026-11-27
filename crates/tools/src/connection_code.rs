//! #3642: the connection code of one side of a tool component - the
//! machine side (ISO 13399 property ADINTMS) or the workpiece side
//! (ADINTWS). One normalized type for both, so a parent's workpiece
//! side and a child's machine side compare directly. The families and
//! their size series are from the published standards: steep tapers
//! (JIS B 6339 BT, ASME B5.50 CAT, ISO 7388-1 SK), HSK (ISO 12164),
//! polygon taper (ISO 26623), KM (ISO 26622), ER collets (ISO 15488),
//! Weldon and plain shanks (DIN 1835), TTS, rectangular shanks
//! (ISO 5608), VDI (ISO 10889), BMT and shell mill arbors. The text
//! form is our own.

use crate::holder_library::BORE_SLACK;

/// The steep taper families.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SteepTaper {
    /// JIS B 6339 / MAS 403.
    Bt,
    /// ASME B5.50.
    Cat,
    /// ISO 7388-1 / DIN 69871.
    Sk,
}

/// One normalized connection.
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionCode {
    SteepTaper { family: SteepTaper, size: u16 },
    /// HSK, form A to F, by its flange size.
    Hsk { form: char, size: u16 },
    /// The polygon taper of ISO 26623, by its flange size.
    Psc { size: u16 },
    /// ISO 26622, by its flange size.
    Km { size: u16 },
    /// An ER collet seat (ISO 15488).
    Er { size: u16 },
    /// A shank with a Weldon flat (DIN 1835 form B), diameter in mm.
    Weldon { d: f64 },
    /// A plain cylindrical shank or bore, diameter in mm.
    Cylindrical { d: f64 },
    /// The Tormach tooling system's 3/4 in straight shank.
    Tts,
    /// A rectangular turning shank (ISO 5608), height x width in mm.
    RectShank { h: u16, b: u16 },
    /// A VDI turret shank (ISO 10889), diameter in mm.
    Vdi { d: u16 },
    /// A bolted turret mount, by its size.
    Bmt { size: u16 },
    /// A shell mill arbor, pilot diameter in mm.
    ShellMillArbor { d: f64 },
    /// Any other connection, by its own name. Matches only the same
    /// name.
    Custom(String),
}

const BT: &[u16] = &[30, 40, 50];
const CAT_SK: &[u16] = &[30, 40, 45, 50, 60];
const HSK: &[u16] = &[25, 32, 40, 50, 63, 80, 100, 125, 160];
const PSC_KM: &[u16] = &[32, 40, 50, 63, 80, 100];
const ER: &[u16] = &[8, 11, 16, 20, 25, 32, 40, 50];
const VDI: &[u16] = &[16, 20, 25, 30, 40, 50, 60, 80];
const BMT: &[u16] = &[40, 45, 55, 65, 75, 85];
/// The largest shank or arbor diameter (mm) a code takes.
pub const MAX_D: f64 = 200.0;

/// Why a connection code text is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionError {
    pub text: String,
    pub why: String,
}

impl core::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "the connection code '{}' is refused: {}", self.text, self.why)
    }
}

impl std::error::Error for ConnectionError {}

impl ConnectionCode {
    /// Parse the text form that [`Display`](core::fmt::Display)
    /// writes. Case and spaces are ignored, except in a custom name.
    pub fn parse(text: &str) -> Result<Self, ConnectionError> {
        let t = text.trim();
        let err = |why: &str| ConnectionError { text: t.to_owned(), why: why.to_owned() };
        if t.is_empty() {
            return Err(err("it is empty"));
        }
        if !t.is_ascii() {
            return Err(err("it has a character that is not ASCII"));
        }
        if t.len() > 7 && t[..7].eq_ignore_ascii_case("CUSTOM:") {
            let name = t[7..].trim();
            return if name.is_empty() { Err(err("a custom code needs a name")) } else { Ok(Self::Custom(name.to_owned())) };
        }
        let u: String = t.chars().filter(|c| !c.is_ascii_whitespace()).map(|c| c.to_ascii_uppercase()).collect();
        let series = |rest: &str, set: &[u16], what: &str| -> Result<u16, ConnectionError> {
            rest.parse::<u16>().ok().filter(|n| set.contains(n)).ok_or_else(|| err(&format!("{what} takes {set:?}")))
        };
        let diameter = |rest: &str, what: &str| -> Result<f64, ConnectionError> {
            rest.parse::<f64>()
                .ok()
                .filter(|d| d.is_finite() && *d > 0.0 && *d <= MAX_D)
                .ok_or_else(|| err(&format!("{what} takes a diameter above 0 and up to {MAX_D} mm")))
        };
        // Longer prefixes first, so "CAT" is not read as a custom code
        // and "HSK" not as "H".
        let code = if u == "TTS" {
            Self::Tts
        } else if let Some(r) = u.strip_prefix("HSK-") {
            let mut c = r.chars();
            let form = c.next().filter(|f| ('A'..='F').contains(f)).ok_or_else(|| err("HSK takes a form A to F"))?;
            Self::Hsk { form, size: series(c.as_str(), HSK, "HSK")? }
        } else if let Some(r) = u.strip_prefix("BT") {
            Self::SteepTaper { family: SteepTaper::Bt, size: series(r, BT, "BT")? }
        } else if let Some(r) = u.strip_prefix("CAT") {
            Self::SteepTaper { family: SteepTaper::Cat, size: series(r, CAT_SK, "CAT")? }
        } else if let Some(r) = u.strip_prefix("SK") {
            Self::SteepTaper { family: SteepTaper::Sk, size: series(r, CAT_SK, "SK")? }
        } else if let Some(r) = u.strip_prefix("PSC") {
            Self::Psc { size: series(r, PSC_KM, "PSC")? }
        } else if let Some(r) = u.strip_prefix("KM") {
            Self::Km { size: series(r, PSC_KM, "KM")? }
        } else if let Some(r) = u.strip_prefix("ER") {
            Self::Er { size: series(r, ER, "ER")? }
        } else if let Some(r) = u.strip_prefix("VDI") {
            Self::Vdi { d: series(r, VDI, "VDI")? }
        } else if let Some(r) = u.strip_prefix("BMT") {
            Self::Bmt { size: series(r, BMT, "BMT")? }
        } else if let Some(r) = u.strip_prefix("WELDON") {
            Self::Weldon { d: diameter(r, "a Weldon shank")? }
        } else if let Some(r) = u.strip_prefix("CYL") {
            Self::Cylindrical { d: diameter(r, "a cylindrical shank")? }
        } else if let Some(r) = u.strip_prefix("ARBOR") {
            Self::ShellMillArbor { d: diameter(r, "a shell mill arbor")? }
        } else if let Some(r) = u.strip_prefix("RECT") {
            let (h, b) = r.split_once('X').ok_or_else(|| err("a rectangular shank is RECT <h>x<b>"))?;
            let mm = |s: &str| s.parse::<u16>().ok().filter(|n| (1..=100).contains(n));
            match (mm(h), mm(b)) {
                (Some(h), Some(b)) => Self::RectShank { h, b },
                _ => return Err(err("a rectangular shank takes a height and a width of 1 to 100 mm")),
            }
        } else {
            return Err(err("it names no known connection (write CUSTOM:<name> for another one)"));
        };
        Ok(code)
    }

    /// The canonical form of a code built in code (not parsed): the
    /// value that its text parses to. An HSK form in lower case reads as
    /// upper case, and a custom name loses its outer spaces. A code that
    /// its text form refuses (a size off its series, a non-finite
    /// diameter, an HSK form outside A to F) is refused here.
    pub fn canonical(&self) -> Result<Self, ConnectionError> {
        Self::parse(&self.to_string())
    }

    /// Refuse a code that [`canonical`](Self::canonical) refuses.
    pub fn check(&self) -> Result<(), ConnectionError> {
        self.canonical().map(|_| ())
    }

    /// Does a child whose machine side is `child` mate with this
    /// workpiece side? Both sides compare in their canonical form, and
    /// an invalid code mates nothing. The same code mates (diameters
    /// within [`BORE_SLACK`]); a plain bore or collet also takes a
    /// Weldon shank of its size, a Weldon bore takes only a Weldon
    /// shank (its screw needs the flat).
    #[must_use]
    pub fn takes(&self, child: &Self) -> bool {
        match (self.canonical(), child.canonical()) {
            (Ok(parent), Ok(child)) => parent.takes_canonical(&child),
            _ => false,
        }
    }

    fn takes_canonical(&self, child: &Self) -> bool {
        let same_d = |a: f64, b: f64| (a - b).abs() <= BORE_SLACK;
        match (self, child) {
            (Self::Weldon { d: a }, Self::Weldon { d: b })
            | (Self::Cylindrical { d: a }, Self::Cylindrical { d: b } | Self::Weldon { d: b })
            | (Self::ShellMillArbor { d: a }, Self::ShellMillArbor { d: b }) => same_d(*a, *b),
            (Self::Custom(a), Self::Custom(b)) => a == b,
            _ => self == child,
        }
    }
}

impl core::fmt::Display for ConnectionCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SteepTaper { family, size } => {
                let name = match family {
                    SteepTaper::Bt => "BT",
                    SteepTaper::Cat => "CAT",
                    SteepTaper::Sk => "SK",
                };
                write!(f, "{name}{size}")
            }
            Self::Hsk { form, size } => write!(f, "HSK-{form}{size}"),
            Self::Psc { size } => write!(f, "PSC{size}"),
            Self::Km { size } => write!(f, "KM{size}"),
            Self::Er { size } => write!(f, "ER{size}"),
            Self::Weldon { d } => write!(f, "WELDON {d}"),
            Self::Cylindrical { d } => write!(f, "CYL {d}"),
            Self::Tts => write!(f, "TTS"),
            Self::RectShank { h, b } => write!(f, "RECT {h}x{b}"),
            Self::Vdi { d } => write!(f, "VDI{d}"),
            Self::Bmt { size } => write!(f, "BMT{size}"),
            Self::ShellMillArbor { d } => write!(f, "ARBOR {d}"),
            Self::Custom(name) => write!(f, "CUSTOM:{name}"),
        }
    }
}
