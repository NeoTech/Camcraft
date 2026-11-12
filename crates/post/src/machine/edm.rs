//! EDM machine data (#530, part of #290): the `.mch` file's `edm`
//! section for the `wire_edm` and `sinker_edm` kinds. Wire: the lower
//! and upper guide planes, the maximum taper and the wire diameters the
//! machine takes, AWT. Sinker: the orbit patterns the control offers
//! (with the control's own word for each), the maximum orbit radius and
//! the pickup method. The U/V and Z strokes are ordinary kinematics
//! axes. Every number is optional by the unstated-limits law - absent is
//! "not stated", never zero - and the condition table, codes and
//! generator codes are separate sub-issues (#533, #542); the condition
//! table (#531) holds only the geometry a toolpath needs.
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

/// What a generator condition is for (#531): the rough cut, a skim
/// pass, or the finish. The word is the file's; this is CamCraft's.
pub const CONDITION_ROLES: [&str; 3] = ["rough", "skim", "finish"];

/// One generator condition the user states from their machine's manual
/// (#531, #290 D4). CamCraft reads only the geometry it needs - the
/// spark gap per side and the wire diameter (wire) or the electrode
/// undersize per side (sinker); power, flushing and the rest stay the
/// control's, named by `id` (the word the control selects it by).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Condition {
    /// The control's own selector for this condition.
    pub id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Spark gap per side, mm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
    /// Wire: the wire diameter this condition burns with, mm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wire_diameter: Option<f64>,
    /// Sinker: the electrode undersize per side this condition wants, mm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undersize: Option<f64>,
    /// Sinker: the electrode's axial wear rate under this condition
    /// (#546) - electrode length lost per unit of depth burned, a ratio
    /// in [0, 1). Unstated = no rate: the burn is not wear-compensated
    /// from the table (no fixed ratio is ever assumed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wear: Option<f64>,
    /// One of [`CONDITION_ROLES`], or empty: not stated.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

/// A field an operation needs from a condition (#531).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionField {
    Gap,
    WireDiameter,
    Undersize,
}

impl ConditionField {
    fn word(self) -> &'static str {
        match self {
            ConditionField::Gap => "gap",
            ConditionField::WireDiameter => "wire_diameter",
            ConditionField::Undersize => "undersize",
        }
    }
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
    /// The generator conditions the user states (#531). Empty = none
    /// stated, and every EDM op refuses by name.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
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
            && self.conditions.iter().all(condition_ok)
            && unique_ids(&self.conditions)
    }

    /// The condition `id` with every field in `need` stated, or the
    /// refusal that names what is missing - the door an EDM op's command
    /// calls at creation (#531), so a missing gap is refused before any
    /// generator runs.
    ///
    /// # Errors
    /// No condition `id` on this machine, or one of `need` not stated.
    pub fn condition(&self, machine: &str, id: &str, need: &[ConditionField]) -> Result<&Condition, String> {
        let c = self
            .conditions
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("machine {machine} states no EDM condition {id:?}"))?;
        for f in need {
            let stated = match f {
                ConditionField::Gap => c.gap.is_some(),
                ConditionField::WireDiameter => c.wire_diameter.is_some(),
                ConditionField::Undersize => c.undersize.is_some(),
            };
            if !stated {
                return Err(format!("machine {machine}: EDM condition {id:?} states no {}", f.word()));
            }
        }
        Ok(c)
    }
}

fn condition_ok(c: &Condition) -> bool {
    let len = |r: Option<f64>| r.is_none_or(|x| x.is_finite() && x > 0.0);
    !c.id.trim().is_empty()
        && c.gap.is_none_or(|g| g.is_finite() && g >= 0.0)
        && len(c.wire_diameter)
        && c.undersize.is_none_or(|u| u.is_finite() && u >= 0.0)
        && c.wear.is_none_or(wear_rate_ok)
        && (c.role.is_empty() || CONDITION_ROLES.contains(&c.role.as_str()))
}

/// The axial wear rate law (#546): a finite ratio from 0 up to, but not
/// including, 1 - the electrode cannot lose more length than the burn
/// gains depth.
#[must_use]
pub fn wear_rate_ok(r: f64) -> bool {
    r.is_finite() && (0.0..1.0).contains(&r)
}

fn unique_ids(cs: &[Condition]) -> bool {
    cs.iter().enumerate().all(|(i, c)| cs.iter().skip(i + 1).all(|d| d.id != c.id))
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
    e.conditions = parse_conditions(v.get("conditions"), f)?;
    Ok(Some(e))
}

/// `edm.conditions[]` (#531): each entry's id is required and unique;
/// lengths scale with the file's unit; a garbled entry refuses the
/// file by its id.
fn parse_conditions(v: Option<&serde_json::Value>, f: f64) -> Result<Vec<Condition>, String> {
    let list = match v {
        None | Some(serde_json::Value::Null) => return Ok(Vec::new()),
        Some(serde_json::Value::Array(a)) => a,
        Some(x) => return Err(format!("machine file: edm.conditions must be a list, not {x}")),
    };
    let mut out: Vec<Condition> = Vec::new();
    for c in list {
        let id = match c.get("id") {
            Some(serde_json::Value::String(s)) if !s.trim().is_empty() => s.clone(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            _ => return Err(format!("machine file: an edm condition states no id: {c}")),
        };
        if out.iter().any(|o| o.id == id) {
            return Err(format!("machine file: edm condition {id:?} is stated twice"));
        }
        let word = |k: &str| -> Result<String, String> {
            match c.get(k) {
                None | Some(serde_json::Value::Null) => Ok(String::new()),
                Some(serde_json::Value::String(s)) => Ok(s.clone()),
                Some(x) => Err(format!("machine file: edm condition {id:?} {k} must be a word, not {x}")),
            }
        };
        let num_at = |k: &str, zero_ok: bool| -> Result<Option<f64>, String> {
            let n = num(c, k, f).map_err(|_| format!("machine file: edm condition {id:?} {k} must be a number"))?;
            match n {
                Some(x) if x < 0.0 || (!zero_ok && x == 0.0) => {
                    Err(format!("machine file: edm condition {id:?} {k} must be positive, not {x}"))
                }
                other => Ok(other),
            }
        };
        let cond = Condition {
            name: word("name")?,
            gap: num_at("gap", true)?,
            wire_diameter: num_at("wire_diameter", false)?,
            undersize: num_at("undersize", true)?,
            wear: match num(c, "wear", 1.0) {
                Err(_) => return Err(format!("machine file: edm condition {id:?} wear must be a number")),
                Ok(Some(w)) if !wear_rate_ok(w) => {
                    return Err(format!(
                        "machine file: edm condition {id:?} wear {w} is not an axial wear rate - state a ratio from 0 up to (not including) 1: the electrode cannot wear more than the burn deepens"
                    ));
                }
                Ok(w) => w,
            },
            role: word("role")?,
            notes: word("notes")?,
            id,
        };
        if !(cond.role.is_empty() || CONDITION_ROLES.contains(&cond.role.as_str())) {
            return Err(format!(
                "machine file: edm condition {:?} role {:?} is not one of {}",
                cond.id,
                cond.role,
                CONDITION_ROLES.join(", ")
            ));
        }
        out.push(cond);
    }
    Ok(out)
}
