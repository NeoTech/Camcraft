//! Resident quality requests from the MIT Agie Processor presets and our offline control measurements.

/// A supported quality for cold die steel 0001 and Berco Cut 0.25 mm wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgieQuality {
    FineRa030,
    FinePlusRa035,
}

impl AgieQuality {
    pub fn from_condition(id: &str) -> Option<Self> {
        match id {
            "AGIE_QN2" => Some(Self::FineRa030),
            "AGIE_BERCO_RA035" => Some(Self::FinePlusRa035),
            _ => None,
        }
    }

    pub(super) fn parameters(self) -> &'static str {
        // Requested Ra, Te, Tkm (µm) and three control options.
        match self {
            Self::FineRa030 => "0.30,1.50,1.00,F,F,F",
            Self::FinePlusRa035 => "0.35,1.50,1.00,F,F,F",
        }
    }

    pub(super) fn qualified_offsets(self) -> &'static [(u16, u32)] {
        // Exact stock thickness (mm), greatest measured wire-centre offset (µm).
        match self {
            Self::FineRa030 => &[(15, 245), (20, 245), (40, 260), (80, 266)],
            Self::FinePlusRa035 => &[(20, 245)],
        }
    }
}
