//! Part-in-spindle mode on a mill (#3406): the mill spindle holds the
//! bar in a collet and turns it, and stationary lathe tools sit on the
//! table, each at its own work offset. A setup on a `Kind::Mill` machine
//! states the mode ([`PartInSpindle`]: the stock's length-offset register,
//! the collet's stick-out and outer diameter) and its table stations
//! (each tool's 1-based work offset, approach side and spindle
//! direction). `None`, the default and every earlier file, states none.
//!
//! ## What the mode does
//!
//! - **The family door** ([`door`]): the setup admits the turning family
//!   and an axial Drill (the spindle's own hole), and refuses milling
//!   work and live-tool cross holes by name.
//! - **Generation** ([`work_kind`]): the setup's work is judged as a
//!   lathe's - the part turns about the spindle, the tool stands still.
//! - **The collet's rating**: `SetupDef::workholding_max_rpm` is
//!   required, and the work-spindle limit applies as on a lathe
//!   (`setup_workholding`).
//! - **The post** ([`program_problems`]): no post section voices the
//!   table stations yet, so a program with work in the setup is refused
//!   by name - never posted through a milling post as milling.
//!
//! Provenance: our own design (#2580's research: a part held in a mill
//! spindle, lathe tools on the table, one work offset for each tool);
//! the practice is general machining knowledge. No vendor material.

use flecs_ecs::prelude::World;
use serde::{Deserialize, Serialize};

use crate::machine::Kind;
use crate::model;
use crate::operation::{Family, OpParams};
use crate::setup;

/// The highest length-offset register a setup may state: a typo guard.
pub const MAX_H_REGISTER: u32 = 9999;
/// The longest stick-out or widest collet a setup may state, mm.
pub const MAX_SIZE_MM: f64 = 1000.0;

/// How the spindle holds the stock: the length-offset register of the
/// stock (`G43 H<h_register>`), and the collet's stick-out and outer
/// diameter, mm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    pub h_register: u32,
    pub stick_out: f64,
    pub collet_od: f64,
}

/// The side of the part a table tool approaches from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApproachSide {
    #[serde(rename = "-X")]
    MinusX,
    #[serde(rename = "+X")]
    PlusX,
}

impl ApproachSide {
    pub const ALL: [ApproachSide; 2] = [ApproachSide::MinusX, ApproachSide::PlusX];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            ApproachSide::MinusX => "-X",
            ApproachSide::PlusX => "+X",
        }
    }

    #[must_use]
    pub fn from_word(w: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.word().eq_ignore_ascii_case(w.trim()))
    }
}

/// The direction the mill spindle turns the part for a station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpindleDir {
    M3,
    M4,
}

impl SpindleDir {
    pub const ALL: [SpindleDir; 2] = [SpindleDir::M3, SpindleDir::M4];

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            SpindleDir::M3 => "M3",
            SpindleDir::M4 => "M4",
        }
    }

    #[must_use]
    pub fn from_word(w: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.word().eq_ignore_ascii_case(w.trim()))
    }
}

/// One tool on the table: its 1-based work offset, the side it
/// approaches from, and the spindle direction it cuts with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableStation {
    pub tool: String,
    pub work_offset: usize,
    pub side: ApproachSide,
    pub spindle: SpindleDir,
}

/// A setup's part-in-spindle record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartInSpindle {
    pub holding: Holding,
    #[serde(default)]
    pub stations: Vec<TableStation>,
}

fn size_refusal(what: &str, v: f64) -> Option<String> {
    (!(v.is_finite() && v > 0.0 && v <= MAX_SIZE_MM))
        .then(|| format!("a collet {what} must be more than 0 and at most {MAX_SIZE_MM} mm, got {v}"))
}

/// Why `h` cannot be a holding; `None` when it can.
#[must_use]
pub fn invalid_holding(h: &Holding) -> Option<String> {
    if h.h_register == 0 || h.h_register > MAX_H_REGISTER {
        return Some(format!(
            "the stock's length offset register must be 1 to {MAX_H_REGISTER}, got {}",
            h.h_register
        ));
    }
    size_refusal("stick-out", h.stick_out).or_else(|| size_refusal("outer diameter", h.collet_od))
}

/// Why `record` cannot be stated; `None` when it can: a bad holding, a
/// work offset out of range, a tool with two stations, or two tools that
/// share one work offset.
#[must_use]
pub fn invalid(record: &PartInSpindle) -> Option<String> {
    if let Some(why) = invalid_holding(&record.holding) {
        return Some(why);
    }
    let top = crate::setup_offsets::MAX_SLOT.saturating_add(1);
    for (i, s) in record.stations.iter().enumerate() {
        if s.tool.trim().is_empty() {
            return Some("a table station names no tool".into());
        }
        if s.work_offset == 0 || s.work_offset > top {
            return Some(format!("tool {}'s work offset must be 1 to {top}, got {}", s.tool, s.work_offset));
        }
        for other in record.stations.iter().take(i) {
            if other.tool == s.tool {
                return Some(format!("tool {} has two table stations", s.tool));
            }
            if other.work_offset == s.work_offset {
                return Some(format!(
                    "tools {} and {} share work offset {} - each table tool needs its own work offset",
                    other.tool, s.tool, s.work_offset
                ));
            }
        }
    }
    None
}

/// The record of a declared setup context, whatever its machine; `None`
/// when it states none or the document has no setups.
#[must_use]
pub fn of(world: &World, declared: Option<&str>) -> Option<PartInSpindle> {
    let name = setup::effective(world, declared)?;
    setup::def_of(world, &name)?.part_in_spindle
}

/// Whether the declared setup context holds the part in a mill spindle:
/// it states the mode and its machine is a mill.
#[must_use]
pub fn active(world: &World, declared: Option<&str>) -> bool {
    of(world, declared).is_some() && crate::machine::kind_of_setup(world, declared) == Kind::Mill
}

/// The machine kind the setup's work is judged as: a lathe in an active
/// part-in-spindle setup (the part turns, the tool stands still), else
/// the kind of the machine it mounts.
#[must_use]
pub fn work_kind(world: &World, declared: Option<&str>) -> Kind {
    if active(world, declared) { Kind::Lathe } else { crate::machine::kind_of_setup(world, declared) }
}

/// Whether an active part-in-spindle setup runs `params`: the turning
/// family, and a Drill with no cross holes.
pub(crate) fn runs(params: &OpParams) -> bool {
    params.kind().info().family() == Family::Turn || matches!(params, OpParams::Drill { cross, .. } if cross.is_empty())
}

/// The family door of an active part-in-spindle setup: `None` when the
/// context is not one (the machine's own door judges), else the verdict.
///
/// # Errors
/// The refusal of a milling kind or a cross-hole Drill, by name.
pub(crate) fn door(world: &World, context: Option<&str>, params: &OpParams) -> Option<Result<(), String>> {
    let name = setup::effective(world, context)?;
    if !active(world, Some(&name)) {
        return None;
    }
    if runs(params) {
        return Some(Ok(()));
    }
    let work = match params {
        OpParams::Drill { .. } => "a Drill operation with cross holes needs live tooling".to_owned(),
        _ => format!("a {} operation is {} work", params.kind(), params.kind().info().family().work()),
    };
    Some(Err(format!(
        "{work}; setup '{name}' holds the part in the mill spindle (part-in-spindle mode), where only turning operations and an on-centre Drill run - use another setup, or clear the mode"
    )))
}

/// The operations whose effective setup is `name`, in program order.
pub(crate) fn ops_of(world: &World, name: &str) -> Vec<(String, crate::schema::Operation)> {
    setup::operation_order(world)
        .into_iter()
        .filter_map(|(n, e)| model::operation_of(world, &e).map(|op| (n, op)))
        .filter(|(_, op)| setup::effective(world, op.setup.as_deref()).as_deref() == Some(name))
        .collect()
}

/// The program-level clause: each setup that states the mode, refused
/// by name where its machine is not a mill, where it states no collet
/// rating, and where it holds work no post section voices yet.
#[must_use]
pub fn program_problems(world: &World) -> Vec<String> {
    let mut out = Vec::new();
    for (name, _) in setup::setups(world) {
        let Some(def) = setup::def_of(world, &name) else { continue };
        if def.part_in_spindle.is_none() {
            continue;
        }
        let kind = crate::machine::kind_of_setup(world, Some(&name));
        if kind != Kind::Mill {
            out.push(format!(
                "setup '{name}' states part-in-spindle mode, but its machine is a {} - the mode is for a mill; clear it",
                kind.word()
            ));
            continue;
        }
        if def.workholding_max_rpm.is_none() {
            out.push(format!(
                "setup '{name}' holds the part in the mill spindle (part-in-spindle mode) but states no workholding maximum rpm for its collet - state it before posting"
            ));
        }
        if ops_of(world, &name).iter().any(|(_, op)| !op.suppressed) {
            out.push(format!(
                "setup '{name}' holds the part in the mill spindle (part-in-spindle mode): no post section voices its table stations yet (the stock's G43 H, one work offset for each tool) - nothing is posted"
            ));
        }
    }
    out
}
