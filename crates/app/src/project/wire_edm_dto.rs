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
    /// No-core burnout's wall clearance, mm (#540); absent cuts the
    /// profile alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burnout_clearance: Option<f64>,
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
            },
            start: p.start,
            burnout_clearance: p.burnout,
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
            },
            start: self.start,
            burnout: self.burnout_clearance,
        })
    }
}
