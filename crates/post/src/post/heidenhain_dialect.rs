//! The HEIDENHAIN dialect resolver (#4095): `controller.identity` (model,
//! NC software, installed options) to the capability flags that the two
//! HEIDENHAIN posts read, one table for both.
//!
//! Provenance: HEIDENHAIN's public user manuals and "new and modified
//! software functions" lists, as collected in `docs/research/heidenhain/`
//! (`dialects.md` sections 1 and 3, `differences.md` B2/B3). Each flag
//! value and its source is in `docs/decisions/4095-heidenhain-dialect-resolver.md`.
//! The flag names and the table form are ours.
//!
//! The machine file states facts only. The resolver never guesses: a model
//! or an NC software identifier that the table does not name is refused by
//! name, because one cycle (247) changes its meaning between versions.

use crate::machine::ControllerIdentity;
use serde::Serialize;
use serde_json::{json, Value};

pub use super::heidenhain_profiles::{C200, Cycle247, Dwell, Flags, Tri};
use super::heidenhain_profiles::{Ids, Rels, Row, PARAXIAL, ROWS};

/// A canonical NC software identifier: 6 digits, `-`, 2 release digits,
/// and an optional ` SP<n>` (`dialects.md` section 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NcSoftware {
    /// The 6-digit identifier. The first five digits are the family, the
    /// sixth is the variant (for example 34049 + 0..4 on the iTNC 530).
    pub id: u32,
    /// The release, `01` to `99`. It orders versions inside one family.
    pub release: u8,
    /// The service pack, when stated (` SP8`).
    pub service_pack: Option<u16>,
}

impl NcSoftware {
    pub fn family(&self) -> u32 {
        self.id / 10
    }
    pub fn variant(&self) -> u32 {
        self.id % 10
    }
}

/// The resolved dialect of one stated control.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dialect {
    /// The profile name of `dialects.md` section 3 (`itnc-49x`, `tnc640-c`, ...).
    pub profile: &'static str,
    pub nc: NcSoftware,
    /// The installed option numbers. `None`: the machine file does not
    /// state them (unknown). An empty list: no options installed.
    pub options: Option<Vec<u16>>,
    #[serde(flatten)]
    pub flags: Flags,
}

/// Parse a canonical NC software identifier. Each other form (a space in
/// the identifier, a third release digit, release `00`, a lower-case `sp`)
/// is refused by name, never matched by its prefix.
pub fn parse_nc(text: &str) -> Result<NcSoftware, String> {
    let bad = || {
        format!(
            "NC software \"{text}\" is not a canonical HEIDENHAIN identifier \
             (6 digits, \"-\", 2 release digits, optional \" SP<n>\", for example \"340490-03\" or \"606425-04 SP8\")"
        )
    };
    let b = text.as_bytes();
    let digits = |r: std::ops::Range<usize>| b.get(r).is_some_and(|s| s.iter().all(u8::is_ascii_digit));
    if b.len() < 9 || !digits(0..6) || b[6] != b'-' || !digits(7..9) {
        return Err(bad());
    }
    let id: u32 = text[0..6].parse().map_err(|_| bad())?;
    let release: u8 = text[7..9].parse().map_err(|_| bad())?;
    if release == 0 {
        return Err(format!("NC software \"{text}\": release 00 does not exist"));
    }
    let service_pack = match &text[9..] {
        "" => None,
        rest => {
            let n = rest.strip_prefix(" SP").ok_or_else(bad)?;
            if n.is_empty() || n.len() > 3 || n.starts_with('0') || !n.bytes().all(|c| c.is_ascii_digit()) {
                return Err(bad());
            }
            Some(n.parse().map_err(|_| bad())?)
        }
    };
    Ok(NcSoftware { id, release, service_pack })
}

/// Parse the installed options: HEIDENHAIN option numbers as decimal
/// strings with no leading zero (`"0"`, `"8"`, `"19"`). Absent: unknown.
pub fn parse_options(options: Option<&[String]>) -> Result<Option<Vec<u16>>, String> {
    let Some(options) = options else { return Ok(None) };
    let mut out = Vec::with_capacity(options.len());
    for o in options {
        let canonical = !o.is_empty()
            && o.len() <= 3
            && o.bytes().all(|c| c.is_ascii_digit())
            && (o == "0" || !o.starts_with('0'));
        let n = o.parse::<u16>().ok().filter(|_| canonical).ok_or_else(|| {
            format!("installed option \"{o}\" is not a HEIDENHAIN option number (digits, no leading zero, for example \"8\" or \"19\")")
        })?;
        out.push(n);
    }
    Ok(Some(out))
}

fn ids_match(ids: &Ids, nc: &NcSoftware) -> bool {
    match *ids {
        Ids::Exact(list) => list.contains(&nc.id),
        Ids::Family { fam, vmin, vmax } => nc.family() == fam && (vmin..=vmax).contains(&nc.variant()),
    }
}

fn rels_match(rels: &Rels, release: u8) -> bool {
    match *rels {
        Rels::Any => true,
        Rels::Range(lo, hi) => (lo..=hi).contains(&release),
        Rels::List(list) => list.contains(&release),
    }
}

fn row_for(model: &str, nc: &NcSoftware) -> Option<&'static Row> {
    ROWS.iter()
        .find(|r| r.models.contains(&model) && ids_match(&r.ids, nc) && rels_match(&r.rels, nc.release))
}

/// Resolve one stated control. The error names the model and the NC
/// software, and says why the table cannot serve them.
pub fn resolve(id: &ControllerIdentity) -> Result<Dialect, String> {
    let model = id.model.as_str();
    let who = format!("{model} NC {}", id.nc_software);
    if PARAXIAL.contains(&model) {
        return Err(format!(
            "{model} is a paraxial positioning control (one axis in each block, no arcs); \
             no CamCraft HEIDENHAIN post writes its dialect"
        ));
    }
    if !ROWS.iter().any(|r| r.models.contains(&model)) {
        let mut known: Vec<&str> = ROWS.iter().flat_map(|r| r.models.iter().copied()).collect();
        known.sort_unstable();
        known.dedup();
        return Err(format!(
            "controller model \"{model}\" is not a HEIDENHAIN control in the dialect table (known: {})",
            known.join(", ")
        ));
    }
    let nc = parse_nc(&id.nc_software)?;
    let options = parse_options(id.installed_options.as_deref())?;
    let row = row_for(model, &nc).ok_or_else(|| {
        format!(
            "{who} is not in the HEIDENHAIN dialect table: the research names no profile \
             for this model, NC software family and release"
        )
    })?;
    Ok(Dialect { profile: row.profile, nc, options, flags: row.flags })
}

/// The `machine.heidenhainDialect` value of a post script: the dialect, or
/// `{ "error": why }`. A post decides if it serves the control.
pub fn script_value(identity: &Value) -> Value {
    let resolved = serde_json::from_value::<ControllerIdentity>(identity.clone())
        .map_err(|e| format!("controller identity: {e}"))
        .and_then(|id| resolve(&id));
    match resolved.and_then(|d| serde_json::to_value(d).map_err(|e| e.to_string())) {
        Ok(v) => v,
        Err(why) => json!({ "error": why }),
    }
}

#[cfg(test)]
#[path = "heidenhain_dialect_tests.rs"]
mod tests;
