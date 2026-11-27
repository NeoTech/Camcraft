//! The laser profile's file mirror (#4081): the op's parameters as
//! plain keys (blank options absent, no tabs absent), on the waterjet's
//! lead, side and tab keys. Additive - no older file names the kind, so
//! absent keys never need a default.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{JetLead, JetSide, JetTabsOp, LaserOp};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LaserDto {
    pub clearance_z: HeightDto,
    pub top_z: HeightDto,
    pub bottom_z: HeightDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kerf: Option<f64>,
    #[serde(default)]
    pub control_comp: bool,
    #[serde(default)]
    pub reversed: bool,
    pub lead: JetLeadDto,
    pub lead_in: f64,
    pub lead_out: f64,
    pub open_side: JetSideDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tabs: Option<JetTabsDto>,
    pub feed: f64,
    pub power: f64,
    pub pierce_time: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linearize: Option<f64>,
    #[serde(default, skip_serializing_if = "PathPicksDto::is_empty")]
    pub picks: PathPicksDto,
}

impl LaserDto {
    pub(super) fn from_op(p: &LaserOp) -> Self {
        LaserDto {
            clearance_z: p.clearance_z.into(),
            top_z: p.top_z.into(),
            bottom_z: p.bottom_z.into(),
            kerf: p.kerf,
            control_comp: p.control_comp,
            reversed: p.reversed,
            lead: match p.lead {
                JetLead::Arc => JetLeadDto::Arc,
                JetLead::Straight => JetLeadDto::Straight,
            },
            lead_in: p.lead_in,
            lead_out: p.lead_out,
            open_side: match p.open_side {
                JetSide::Center => JetSideDto::Center,
                JetSide::Left => JetSideDto::Left,
                JetSide::Right => JetSideDto::Right,
            },
            tabs: p.tabs.as_ref().map(|t| JetTabsDto { count: t.count, width: t.width, at: t.at.clone() }),
            feed: p.feed,
            power: p.power,
            pierce_time: p.pierce_time,
            linearize: p.linearize,
            picks: PathPicksDto::from_picks(&p.picks),
        }
    }

    pub(super) fn to_params(&self) -> OpParams {
        OpParams::LaserProfile(LaserOp {
            clearance_z: self.clearance_z.into(),
            top_z: self.top_z.into(),
            bottom_z: self.bottom_z.into(),
            kerf: self.kerf,
            control_comp: self.control_comp,
            reversed: self.reversed,
            lead: match self.lead {
                JetLeadDto::Arc => JetLead::Arc,
                JetLeadDto::Straight => JetLead::Straight,
            },
            lead_in: self.lead_in,
            lead_out: self.lead_out,
            open_side: match self.open_side {
                JetSideDto::Center => JetSide::Center,
                JetSideDto::Left => JetSide::Left,
                JetSideDto::Right => JetSide::Right,
            },
            tabs: self.tabs.as_ref().map(|t| JetTabsOp { count: t.count, width: t.width, at: t.at.clone() }),
            feed: self.feed,
            power: self.power,
            pierce_time: self.pierce_time,
            linearize: self.linearize,
            picks: self.picks.to_picks(),
        })
    }
}
