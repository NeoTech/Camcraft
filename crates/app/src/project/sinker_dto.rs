//! The Sinker Burn's file mirror (#541): the op's parameters as plain
//! keys - an additive variant, no older file names it.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{SinkerBurnOp, SinkerDatum};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SinkerDatumDto {
    WorkTop,
    CavityFloor,
    ElectrodeTip,
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
    /// #546: additive keys - an older file states none (one layer, no
    /// measured wear) and loads unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear_lost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear_over: Option<f64>,
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
            layer: p.layer,
            wear_lost: p.wear_lost,
            wear_over: p.wear_over,
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
            layer: self.layer,
            wear_lost: self.wear_lost,
            wear_over: self.wear_over,
        })
    }
}
