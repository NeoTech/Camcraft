//! EDM machine data (#530, part of #290): the `.mch` file's `edm`
//! section for the `wire_edm` and `sinker_edm` kinds. Wire: the lower
//! and upper guide planes, the maximum taper and the wire diameters the
//! machine takes, AWT. Sinker: the orbit patterns the control offers
//! (with the control's own word for each), the maximum orbit radius and
//! the pickup method. The U/V and Z strokes are ordinary kinematics
//! axes. Every number is optional by the unstated-limits law - absent is
//! "not stated", never zero - and the condition table, codes and
//! generator data are separate sub-issues (#531, #533, #542).
//!
//! Provenance: the umbrella's design notes (#290, D1-D10) over public
//! EDM terminology (machine builders' published tutorials and the
//! EDM review literature it cites); no vendor machine file, manual
//! table or post was copied.

use serde::{Deserialize, Serialize};

/// The orbit pattern geometries CamCraft can generate (#290 D6). A
/// machine lists which it offers; the word is the control's.
pub const ORBIT_KINDS: [&str; 3] = ["circular", "vector", "planetary"];

/// One orbit pattern a sinker control offers: CamCraft's geometry name
/// (one of [`ORBIT_KINDS`]) and the control's word for it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrbitWord {
    pub kind: String,
    pub word: String,
}

/// The `edm` section. Lengths are mm (the parser scales the file's
/// unit on the way in); the taper is degrees from vertical.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Edm {
    /// Wire: the lower guide's plane, machine Z, mm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_guide_z: Option<f64>,
    /// Wire: the upper guide's plane, machine Z, mm (above the lower).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_guide_z: Option<f64>,
    /// Wire: the largest taper angle the machine can cut, degrees from
    /// vertical (0 < angle < 90).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_taper_deg: Option<f64>,
    /// Wire: the wire diameters the machine takes, mm.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub wire_diameters: Vec<f64>,
    /// Wire: automatic wire threading. Unstated = not claimed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awt: Option<bool>,
    /// Sinker: the orbit patterns the control offers.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub orbits: Vec<OrbitWord>,
    /// Sinker: the largest orbit radius, mm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_orbit_radius: Option<f64>,
    /// Sinker: how the electrode is picked up / referenced (free text
    /// from the machine's manual: a chuck or holder system's name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pickup: Option<String>,
}

impl Edm {
    /// The same law `parse_edm` enforces at the front door, for a
    /// hand-edited project file's machine (the project reader's gate).
    #[must_use]
    pub fn is_sane(&self) -> bool {
        let pos = |r: Option<f64>| r.is_none_or(|x| x.is_finite() && x > 0.0);
        let finite = |r: Option<f64>| r.is_none_or(f64::is_finite);
        finite(self.lower_guide_z)
            && finite(self.upper_guide_z)
            && match (self.lower_guide_z, self.upper_guide_z) {
                (Some(lo), Some(hi)) => hi > lo,
                _ => true,
            }
            && self.max_taper_deg.is_none_or(|a| a.is_finite() && a > 0.0 && a < 90.0)
            && self.wire_diameters.iter().all(|d| d.is_finite() && *d > 0.0)
            && pos(self.max_orbit_radius)
            && self.orbits.iter().all(|o| ORBIT_KINDS.contains(&o.kind.as_str()) && !o.word.trim().is_empty())
    }
}

fn num(v: &serde_json::Value, key: &str, f: f64) -> Result<Option<f64>, String> {
    match v.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(x) => match x.as_f64() {
            Some(n) if n.is_finite() => Ok(Some(n * f)),
            _ => Err(format!("machine file: edm.{key} must be a number, not {x}")),
        },
    }
}

/// Read the `edm` section (`None` when the file has none). Lengths are
/// scaled by `f` (the file's mm factor); a garbled value refuses the
/// file by name, never reads as unstated.
///
/// # Errors
/// A non-number where a number is due, a non-positive length, a taper
/// outside (0, 90) degrees, an upper guide not above the lower, or an
/// orbit kind CamCraft does not generate.
pub(super) fn parse_edm(v: Option<&serde_json::Value>, f: f64) -> Result<Option<Edm>, String> {
    let Some(v) = v.filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    if !v.is_object() {
        return Err("machine file: edm must be an object".to_owned());
    }
    let mut e = Edm {
        lower_guide_z: num(v, "lower_guide_z", f)?,
        upper_guide_z: num(v, "upper_guide_z", f)?,
        max_taper_deg: num(v, "max_taper_deg", 1.0)?,
        max_orbit_radius: num(v, "max_orbit_radius", f)?,
        ..Edm::default()
    };
    if let (Some(lo), Some(hi)) = (e.lower_guide_z, e.upper_guide_z)
        && hi <= lo
    {
        return Err("machine file: edm.upper_guide_z must be above edm.lower_guide_z".to_owned());
    }
    if let Some(a) = e.max_taper_deg
        && !(a > 0.0 && a < 90.0)
    {
        return Err(format!("machine file: edm.max_taper_deg must be between 0 and 90 degrees, not {a}"));
    }
    if let Some(r) = e.max_orbit_radius
        && r <= 0.0
    {
        return Err(format!("machine file: edm.max_orbit_radius must be positive, not {r}"));
    }
    match v.get("wire_diameters") {
        None | Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Array(ds)) => {
            for d in ds {
                match d.as_f64() {
                    Some(n) if n.is_finite() && n > 0.0 => e.wire_diameters.push(n * f),
                    _ => return Err(format!("machine file: edm.wire_diameters holds {d}, not a positive number")),
                }
            }
        }
        Some(x) => return Err(format!("machine file: edm.wire_diameters must be a list, not {x}")),
    }
    match v.get("awt") {
        None | Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Bool(b)) => e.awt = Some(*b),
        Some(x) => return Err(format!("machine file: edm.awt must be true or false, not {x}")),
    }
    match v.get("pickup") {
        None | Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::String(s)) if !s.trim().is_empty() => e.pickup = Some(s.clone()),
        Some(x) => return Err(format!("machine file: edm.pickup must be a word, not {x}")),
    }
    match v.get("orbits") {
        None | Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Array(os)) => {
            for o in os {
                let kind = o.get("kind").and_then(|k| k.as_str()).unwrap_or_default();
                let word = o.get("word").and_then(|w| w.as_str()).unwrap_or_default();
                if !ORBIT_KINDS.contains(&kind) {
                    return Err(format!(
                        "machine file: edm orbit kind {kind:?} is not one of {}",
                        ORBIT_KINDS.join(", ")
                    ));
                }
                if word.trim().is_empty() {
                    return Err(format!("machine file: edm orbit {kind} states no control word"));
                }
                e.orbits.push(OrbitWord { kind: kind.to_owned(), word: word.to_owned() });
            }
        }
        Some(x) => return Err(format!("machine file: edm.orbits must be a list, not {x}")),
    }
    Ok(Some(e))
}
