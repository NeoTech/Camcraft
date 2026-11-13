//! The wire contour's file mirror (#532): the op's parameters as plain
//! keys - the two workpiece planes, the machine condition's id, the
//! side and the start hole. `side` has no default on purpose: a file
//! (or an MCP patch) that leaves it out is refused, never read as a
//! die - the side is stated, not inferred (#290 D8). The wire and the
//! gap are not here: they are the machine's condition table's.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{WireContourOp, WireSide};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WireSideDto {
    Die,
    Punch,
    /// An open run's kerf, left of travel (#1109).
    Left,
    /// An open run's kerf, right of travel (#1109).
    Right,
}

fn default_clearance() -> f64 {
    crate::operation::DEFAULT_CLEARANCE
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde's skip predicate takes a reference
fn is_default_clearance(c: &f64) -> bool {
    c.to_bits() == default_clearance().to_bits()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WireContourDto {
    pub top_z: HeightDto,
    pub bottom_z: HeightDto,
    /// The machine's EDM condition id; absent is none chosen.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub condition: String,
    pub side: WireSideDto,
    /// The start hole / threading point, `[x, y]` mm; absent is not
    /// stated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<[f64; 2]>,
    /// An open run's run-out clearance, mm (#1109); absent is the
    /// seed's, so a closed cut's file reads and writes as before.
    #[serde(default = "default_clearance", skip_serializing_if = "is_default_clearance")]
    pub clearance: f64,
}

impl WireContourDto {
    pub(super) fn from_op(p: &WireContourOp) -> Self {
        WireContourDto {
            top_z: p.top_z.into(),
            bottom_z: p.bottom_z.into(),
            condition: p.condition.clone(),
            side: match p.side {
                WireSide::Die => WireSideDto::Die,
                WireSide::Punch => WireSideDto::Punch,
                WireSide::Left => WireSideDto::Left,
                WireSide::Right => WireSideDto::Right,
            },
            start: p.start,
            clearance: p.clearance,
        }
    }

    pub(super) fn to_params(&self) -> OpParams {
        OpParams::WireContour(WireContourOp {
            top_z: self.top_z.into(),
            bottom_z: self.bottom_z.into(),
            condition: self.condition.clone(),
            side: match self.side {
                WireSideDto::Die => WireSide::Die,
                WireSideDto::Punch => WireSide::Punch,
                WireSideDto::Left => WireSide::Left,
                WireSideDto::Right => WireSide::Right,
            },
            start: self.start,
            clearance: self.clearance,
        })
    }
}
