//! The Sinker Burn's file mirror (#541): the op's parameters as plain
//! keys - an additive variant, no older file names it. The orbit
//! (#544) is an additive key inside it.
//! Provenance: our own file format (`project/mod.rs`); the names are ours.

use super::*;

use crate::operation::{SinkerBurnOp, SinkerDatum, SinkerOrbit, SinkerOrbitKind};

/// The orbit pattern's file word (#544) - the same words a machine's
/// `edm.orbits[].kind` states.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SinkerOrbitKindDto {
    Circular,
    Vector,
    Planetary,
}

fn four() -> u32 {
    4
}

fn one() -> u32 {
    1
}

/// The orbit (#544): additive - a file without it burns a plain plunge.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct SinkerOrbitDto {
    pub kind: SinkerOrbitKindDto,
    pub radius: f64,
    #[serde(default = "four")]
    pub vectors: u32,
    #[serde(default = "one")]
    pub turns: u32,
}

impl SinkerOrbitDto {
    fn from_orbit(o: &SinkerOrbit) -> Self {
        SinkerOrbitDto {
            kind: match o.kind {
                SinkerOrbitKind::Circular => SinkerOrbitKindDto::Circular,
                SinkerOrbitKind::Vector => SinkerOrbitKindDto::Vector,
                SinkerOrbitKind::Planetary => SinkerOrbitKindDto::Planetary,
            },
            radius: o.radius,
            vectors: o.vectors,
            turns: o.turns,
        }
    }

    fn to_orbit(self) -> SinkerOrbit {
        SinkerOrbit {
            kind: match self.kind {
                SinkerOrbitKindDto::Circular => SinkerOrbitKind::Circular,
                SinkerOrbitKindDto::Vector => SinkerOrbitKind::Vector,
                SinkerOrbitKindDto::Planetary => SinkerOrbitKind::Planetary,
            },
            radius: self.radius,
            vectors: self.vectors,
            turns: self.turns,
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orbit: Option<SinkerOrbitDto>,
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
            orbit: p.orbit.as_ref().map(SinkerOrbitDto::from_orbit),
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
            orbit: self.orbit.map(SinkerOrbitDto::to_orbit),
        })
    }
}
