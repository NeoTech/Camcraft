//! The Sinker Burn's file mirror (#541): the op's parameters as plain
//! keys - an additive variant, no older file names it.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{SinkerBurnOp, SinkerDatum, SinkerPass};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SinkerDatumDto {
    WorkTop,
    CavityFloor,
    ElectrodeTip,
}

/// Which pass of the cavity (#543). A file of #541 names none: single.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SinkerPassDto {
    #[default]
    Single,
    Rough,
    Finish,
}

impl SinkerPassDto {
    fn is_single(&self) -> bool {
        *self == SinkerPassDto::Single
    }
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SinkerDto {
    pub clearance_z: HeightDto,
    pub top_z: HeightDto,
    pub floor_z: HeightDto,
    pub datum: SinkerDatumDto,
    pub depth: f64,
    pub x: f64,
    pub y: f64,
    pub feed: f64,
    #[serde(default)]
    pub condition: String,
    /// #543: additive keys, written only for a rough or finish burn.
    #[serde(default, skip_serializing_if = "SinkerPassDto::is_single")]
    pub pass: SinkerPassDto,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cavity_x: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cavity_y: f64,
}

impl SinkerDto {
    pub(super) fn from_op(p: &SinkerBurnOp) -> Self {
        SinkerDto {
            clearance_z: p.clearance_z.into(),
            top_z: p.top_z.into(),
            floor_z: p.floor_z.into(),
            datum: match p.datum {
                SinkerDatum::WorkTop => SinkerDatumDto::WorkTop,
                SinkerDatum::CavityFloor => SinkerDatumDto::CavityFloor,
                SinkerDatum::ElectrodeTip => SinkerDatumDto::ElectrodeTip,
            },
            depth: p.depth,
            x: p.x,
            y: p.y,
            feed: p.feed,
            condition: p.condition.clone(),
            pass: match p.pass {
                SinkerPass::Single => SinkerPassDto::Single,
                SinkerPass::Rough => SinkerPassDto::Rough,
                SinkerPass::Finish => SinkerPassDto::Finish,
            },
            cavity_x: p.cavity_x,
            cavity_y: p.cavity_y,
        }
    }

    pub(super) fn to_params(&self) -> OpParams {
        OpParams::SinkerBurn(SinkerBurnOp {
            clearance_z: self.clearance_z.into(),
            top_z: self.top_z.into(),
            floor_z: self.floor_z.into(),
            datum: match self.datum {
                SinkerDatumDto::WorkTop => SinkerDatum::WorkTop,
                SinkerDatumDto::CavityFloor => SinkerDatum::CavityFloor,
                SinkerDatumDto::ElectrodeTip => SinkerDatum::ElectrodeTip,
            },
            depth: self.depth,
            x: self.x,
            y: self.y,
            feed: self.feed,
            condition: self.condition.clone(),
            pass: match self.pass {
                SinkerPassDto::Single => SinkerPass::Single,
                SinkerPassDto::Rough => SinkerPass::Rough,
                SinkerPassDto::Finish => SinkerPass::Finish,
            },
            cavity_x: self.cavity_x,
            cavity_y: self.cavity_y,
        })
    }
}
