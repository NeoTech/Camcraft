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

    // Each name permanently identifies the material, wire and full quality tuple.
    // A changed definition needs a new name, even if it requests the same Ra.
    pub(super) fn definition(self) -> (&'static str, &'static str) {
        match self {
            Self::FineRa030 => ("CCB030", "0.30,1.50,1.00,F,F,F"),
            Self::FinePlusRa035 => ("CCB035", "0.35,1.50,1.00,F,F,F"),
        }
    }

    pub(super) fn working_offsets(self, height: f64) -> Result<&'static [u16], String> {
        // Exact stock thickness (mm) and wire-centre offsets (µm) in working-step order.
        let rows: &[(u16, &[u16])] = match self {
            Self::FineRa030 => &[
                (15, &[245, 155, 140, 134]),
                (20, &[245, 155, 140, 134]),
                (40, &[260, 170, 140, 130]),
                (80, &[266, 176, 143, 133]),
            ],
            Self::FinePlusRa035 => &[(20, &[245, 155, 140, 134])],
        };
        if let Some((_, offsets)) = rows.iter().find(|(mm, _)| f64::from(*mm) == height) {
            return Ok(offsets);
        }
        let supported = rows
            .iter()
            .map(|(mm, _)| mm.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!(
            "The selected Agie quality has no qualified offset for stock thickness {height} mm. Supported thicknesses in mm are {supported}"
        ))
    }

    /// The values to check on the loaded work before releasing its steps.
    pub fn controller_review(self, height: f64) -> Result<String, String> {
        let (name, tuple) = self.definition();
        let offsets = self
            .working_offsets(height)?
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!(
            "Agie Control must show zero working steps on import. On each work verify 0001, Berco Cut 0.25 and {height} mm. The {name} request is {tuple} (Ra, Te, Tkm in µm and three flags). Verify the displayed targets and both visible options. Assign the matching TECCUT row on this work. Required working-step offsets in µm are {offsets}."
        ))
    }
}
