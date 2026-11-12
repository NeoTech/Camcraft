//! The EDM condition table (#531, part of #290): `edm.conditions[]`,
//! the generator conditions the user states from their machine's
//! manual. Moved out of `edm.rs` unchanged when #551's strict reader
//! landed (merge resolution); the #560 review's rules follow.
//!
//! Provenance: the umbrella's design notes (#290, D4: no vendor
//! condition data ships) over public EDM terminology; no vendor manual
//! table was copied.

use serde::{Deserialize, Serialize};

use super::edm::Edm;

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

impl Edm {
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

pub(super) fn condition_ok(c: &Condition) -> bool {
    let len = |r: Option<f64>| r.is_none_or(|x| x.is_finite() && x > 0.0);
    !c.id.trim().is_empty()
        && c.gap.is_none_or(|g| g.is_finite() && g >= 0.0)
        && len(c.wire_diameter)
        && c.undersize.is_none_or(|u| u.is_finite() && u >= 0.0)
        && (c.role.is_empty() || CONDITION_ROLES.contains(&c.role.as_str()))
}

pub(super) fn unique_ids(cs: &[Condition]) -> bool {
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

/// `edm.conditions[]` (#531): each entry's id is required and unique;
/// lengths scale with the file's unit; a garbled entry refuses the
/// file by its id.
pub(super) fn parse_conditions(v: Option<&serde_json::Value>, f: f64) -> Result<Vec<Condition>, String> {
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
