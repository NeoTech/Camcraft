//! The wire contour's file mirror (#532): the op's parameters as plain
//! keys - the two workpiece planes, the machine condition's id, the
//! side and the start hole. `side` has no default on purpose: a file
//! (or an MCP patch) that leaves it out is refused, never read as a
//! die - the side is stated, not inferred (#290 D8); the 4-axis block's
//! pairing rule (#537) likewise. The wire and the
//! gap are not here: they are the machine's condition table's.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{WireContourOp, WireFourAxis, WirePairingRule, WireSide, WireSyncPair, FOUR_AXIS_TOLERANCE};

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
    /// The constant taper, degrees (#536); absent is a vertical cut, so
    /// every file written before it reads unchanged.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub taper_deg: f64,
    /// The 4-axis cut (#537); absent is the one-profile cut, so every
    /// file written before it reads unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub four_axis: Option<WireFourAxisDto>,
}

/// The pairing rule's words. No default: a 4-axis block that leaves the
/// rule out is refused, never read as arc length (#537: stated).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WirePairingDto {
    ArcLength,
    SyncPoints,
}

/// One sync pair, `[x, y]` mm each; a side left out is unpaired (kept,
/// and refused by name at generation).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct WireSyncDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lower: Option<[f64; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WireFourAxisDto {
    pub pairing: WirePairingDto,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sync: Vec<WireSyncDto>,
    /// The twist tolerance, mm; absent is the default.
    #[serde(default = "four_axis_tolerance")]
    pub tolerance: f64,
}

fn four_axis_tolerance() -> f64 {
    FOUR_AXIS_TOLERANCE
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    *v == 0.0
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
            taper_deg: p.taper_deg,
            four_axis: p.four_axis.as_ref().map(|f| WireFourAxisDto {
                pairing: match f.pairing {
                    WirePairingRule::ArcLength => WirePairingDto::ArcLength,
                    WirePairingRule::SyncPoints => WirePairingDto::SyncPoints,
                },
                sync: f.sync.iter().map(|s| WireSyncDto { upper: s.upper, lower: s.lower }).collect(),
                tolerance: f.tolerance,
            }),
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
            taper_deg: self.taper_deg,
            four_axis: self.four_axis.as_ref().map(|f| WireFourAxis {
                pairing: match f.pairing {
                    WirePairingDto::ArcLength => WirePairingRule::ArcLength,
                    WirePairingDto::SyncPoints => WirePairingRule::SyncPoints,
                },
                sync: f.sync.iter().map(|s| WireSyncPair { upper: s.upper, lower: s.lower }).collect(),
                tolerance: f.tolerance,
            }),
        })
    }
}
