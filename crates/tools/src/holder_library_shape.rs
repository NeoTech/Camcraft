//! #4338 (U12 of #3663): a user holder's shape as the holder form edits
//! it - the taper, the clamp, the shank range, the gauge length and the
//! nose and largest steel diameters - and the component it makes: its
//! machine and workpiece sides, and its outline. The outline is the holder
//! library's own (`HolderEntry::points`), so it keeps the library's law
//! "never smaller than the steel" for the values the user states: an ER
//! nut at its stated diameter over its whole length, a Weldon body
//! `WELDON_SCREW` wider, a shrink-fit nose and body `SHRINK_MARGIN` wider,
//! the flange as a full cylinder. A value that cannot describe steel is
//! refused by name. `HolderShape::of` reads the shape back from a
//! component and accepts it only when the outline it makes is the
//! component's outline, point for point.
//! Provenance: our own model over our own holder library (ISO 15488 ER
//! sizes, DIN 1835 Weldon bores, DIN 69882-8 shrink chucks as recorded in
//! docs/decisions/793-taper-holders.md and 792-er-collets.md); no HSMWorks
//! material involved.

use super::{library, Clamp, ErSize, HolderEntry, Taper, ER_SIZES, SHRINK_MARGIN, WELDON_SCREW};
use crate::component::ComponentKind;
use crate::component_geometry::ComponentGeometry;
use crate::connection_code::{ConnectionCode, MAX_D};

/// How a user holder grips the tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeClamp {
    /// An ER collet chuck of a standard size: the size states the collet
    /// range and the nut's length.
    Er(ErSize),
    /// A Weldon side-lock bore.
    Weldon,
    /// A shrink-fit bore.
    Shrink,
}

/// The shape of a user holder (mm). `bore` is the Weldon or shrink-fit
/// bore (an ER chuck reads its collet range from its size). `nose_d` is
/// the steel at the nose face - the ER nut, the Weldon body, the shrink
/// nose; `largest_d` is the widest steel of the body below the flange,
/// which an ER or Weldon body has at its nose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HolderShape {
    pub taper: Taper,
    pub clamp: ShapeClamp,
    pub bore: f64,
    pub gauge_len: f64,
    pub nose_d: f64,
    pub largest_d: f64,
}

/// Why a shape is refused. Each refusal names the value.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeError {
    /// A value that is not a positive, finite number.
    NotPositive(&'static str),
    /// The bore is larger than a connection code takes.
    BoreTooLarge(f64),
    /// The nose steel cannot surround what it clamps.
    NoseTooSmall { nose_d: f64, inner: f64 },
    /// A shrink-fit body narrower than its nose.
    BodyBelowNose { largest_d: f64, nose_d: f64 },
    /// An ER or Weldon body states a largest diameter that is not its nose.
    OneBody { largest_d: f64, nose_d: f64 },
    /// The taper has no connection code.
    Taper(String),
    /// The holder library or the component refuses the outline.
    Outline(String),
}

impl core::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPositive(what) => write!(f, "the {what} must be a positive number"),
            Self::BoreTooLarge(d) => write!(f, "the bore Ø{d} mm is larger than Ø{MAX_D} mm"),
            Self::NoseTooSmall { nose_d, inner } => {
                write!(f, "the nose diameter Ø{nose_d} mm must be larger than the Ø{inner} mm it clamps")
            }
            Self::BodyBelowNose { largest_d, nose_d } => write!(
                f,
                "the largest diameter Ø{largest_d} mm is smaller than the nose diameter Ø{nose_d} mm - a shrink-fit body widens from its nose"
            ),
            Self::OneBody { largest_d, nose_d } => write!(
                f,
                "the largest diameter Ø{largest_d} mm is not the nose diameter Ø{nose_d} mm - this clamp has one body diameter"
            ),
            Self::Taper(t) => write!(f, "the taper {t} has no connection code"),
            Self::Outline(why) => write!(f, "the outline is refused: {why}"),
        }
    }
}

impl std::error::Error for ShapeError {}

/// The tapers a user holder can have: each library taper that has a
/// connection code, in the library's order.
#[must_use]
pub fn tapers() -> Vec<Taper> {
    let mut out: Vec<Taper> = Vec::new();
    for t in library().into_iter().filter_map(|e| e.taper) {
        if !out.iter().any(|o| o.name == t.name) && ConnectionCode::parse(t.name).is_ok() {
            out.push(t);
        }
    }
    out
}

/// A value read back from an outline, on a micrometre grid: the form
/// compares it with what the user typed.
fn grid(v: f64) -> f64 {
    (v * 1e6).round() / 1e6
}

fn positive(v: f64, what: &'static str) -> Result<f64, ShapeError> {
    if v.is_finite() && v > 0.0 { Ok(v) } else { Err(ShapeError::NotPositive(what)) }
}

impl HolderShape {
    /// The shank diameters the holder clamps, from and to.
    #[must_use]
    pub fn clamps(&self) -> (f64, f64) {
        match self.clamp {
            ShapeClamp::Er(er) => (er.min_bore, er.max_bore),
            ShapeClamp::Weldon | ShapeClamp::Shrink => (self.bore, self.bore),
        }
    }

    /// The library entry this shape draws, or why the values cannot
    /// describe steel.
    pub fn entry(&self) -> Result<HolderEntry, ShapeError> {
        positive(self.gauge_len, "gauge length")?;
        let nose_d = positive(self.nose_d, "nose diameter")?;
        let largest_d = positive(self.largest_d, "largest diameter")?;
        let clamp = match self.clamp {
            ShapeClamp::Er(er) => {
                if nose_d <= er.max_bore {
                    return Err(ShapeError::NoseTooSmall { nose_d, inner: er.max_bore });
                }
                Clamp::Er(ErSize { nut_d: nose_d, ..er })
            }
            ShapeClamp::Weldon | ShapeClamp::Shrink => {
                let bore = positive(self.bore, "bore")?;
                if bore > MAX_D {
                    return Err(ShapeError::BoreTooLarge(bore));
                }
                if nose_d <= bore {
                    return Err(ShapeError::NoseTooSmall { nose_d, inner: bore });
                }
                if self.clamp == ShapeClamp::Weldon {
                    Clamp::Weldon { bore, body_d: nose_d }
                } else {
                    if largest_d < nose_d {
                        return Err(ShapeError::BodyBelowNose { largest_d, nose_d });
                    }
                    Clamp::Shrink { bore, nose_d, body_d: largest_d }
                }
            }
        };
        if !matches!(self.clamp, ShapeClamp::Shrink) && (largest_d - nose_d).abs() > 1e-9 {
            return Err(ShapeError::OneBody { largest_d, nose_d });
        }
        Ok(HolderEntry { taper: Some(self.taper), clamp, gauge_len: self.gauge_len, label: "" })
    }

    /// The machine side (the taper) and the workpiece side (the collet or
    /// the bore), as the built-in holder components state them.
    pub fn codes(&self) -> Result<(ConnectionCode, ConnectionCode), ShapeError> {
        let mcs = ConnectionCode::parse(self.taper.name).map_err(|_| ShapeError::Taper(self.taper.name.to_owned()))?;
        let csw = match self.clamp {
            ShapeClamp::Er(er) => ConnectionCode::parse(er.name),
            ShapeClamp::Weldon => ConnectionCode::parse(&format!("WELDON {}", self.bore)),
            ShapeClamp::Shrink => ConnectionCode::parse(&format!("CYL {}", self.bore)),
        }
        .map_err(|e| ShapeError::Outline(e.to_string()))?;
        Ok((mcs, csw))
    }

    /// The component kind (machine and workpiece side).
    pub fn kind(&self) -> Result<ComponentKind, ShapeError> {
        let (mcs, csw) = self.codes()?;
        Ok(ComponentKind::AdaptiveItem { mcs, csw })
    }

    /// The component geometry: the port `port` on the nose face, the body
    /// the library's outline of this shape.
    pub fn geometry(&self, port: &str) -> Result<ComponentGeometry, ShapeError> {
        let entry = self.entry()?;
        ComponentGeometry::holder(&entry, port, self.gauge_len).map_err(|e| ShapeError::Outline(e.to_string()))
    }

    /// The shape of a holder component, when its outline is the one this
    /// shape makes (a holder copied from the library, or one this form
    /// wrote). `None` for any other outline: the form cannot edit it.
    #[must_use]
    pub fn of(kind: &ComponentKind, geometry: &ComponentGeometry, port: &str) -> Option<Self> {
        let ComponentKind::AdaptiveItem { mcs, csw } = kind else {
            return None;
        };
        let taper = tapers().into_iter().find(|t| ConnectionCode::parse(t.name).is_ok_and(|c| &c == mcs))?;
        let (clamp, bore) = match csw {
            ConnectionCode::Er { size } => {
                let name = format!("ER{size}");
                (ShapeClamp::Er(*ER_SIZES.iter().find(|e| e.name == name)?), 0.0)
            }
            ConnectionCode::Weldon { d } => (ShapeClamp::Weldon, *d),
            ConnectionCode::Cylindrical { d } => (ShapeClamp::Shrink, *d),
            _ => return None,
        };
        let gauge_len = geometry.port(port).map(|f| grid(-f.origin()[2])).filter(|g| g.is_finite() && *g > 0.0)?;
        let points = geometry.body()?.profile().points();
        // Each radius of the outline is a candidate for the nose and the
        // body; the margins the library adds come off again.
        let mut radii: Vec<f64> = points.iter().map(|p| p.r).filter(|r| *r > 0.0).collect();
        radii.sort_by(f64::total_cmp);
        radii.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let margin = match clamp {
            ShapeClamp::Er(_) => 0.0,
            ShapeClamp::Weldon => WELDON_SCREW,
            ShapeClamp::Shrink => SHRINK_MARGIN,
        };
        let same = |s: &Self| {
            s.geometry(port).ok().and_then(|g| g.body().map(|b| b.profile().points().to_vec())).is_some_and(|p| {
                p.len() == points.len() && p.iter().zip(points).all(|(a, b)| (a.z - b.z).abs() < 1e-6 && (a.r - b.r).abs() < 1e-6)
            })
        };
        let base = Self { taper, clamp, bore, gauge_len, nose_d: 0.0, largest_d: 0.0 };
        for &rn in &radii {
            let nose_d = grid(2.0 * (rn - margin));
            if clamp == ShapeClamp::Shrink {
                for &rb in radii.iter().filter(|r| **r >= rn) {
                    let s = Self { nose_d, largest_d: grid(2.0 * (rb - margin)), ..base };
                    if same(&s) {
                        return Some(s);
                    }
                }
            } else {
                let s = Self { nose_d, largest_d: nose_d, ..base };
                if same(&s) {
                    return Some(s);
                }
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "holder_library_shape_tests.rs"]
mod tests;
