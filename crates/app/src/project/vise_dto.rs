//! A built-in vise's parameters as files hold them (#788): an optional
//! `vise` key on the fixture marker. Absent - every imported fixture and
//! every older file - reads as plain steel, and an older build ignores
//! the key and keeps the saved facets, so the file stays readable both
//! ways. Lengths are millimetres like every length in the file.
//!
//! Provenance: our own wire shape over `vise::ViseParams`.

use serde::{Deserialize, Serialize};

use crate::vise::ViseParams;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViseDto {
    pub size: String,
    pub jaw_width: f64,
    pub jaw_height: f64,
    pub jaw_thickness: f64,
    pub max_opening: f64,
    pub length: f64,
    pub width: f64,
    pub base_height: f64,
    pub housing_length: f64,
    pub housing_height: f64,
    pub opening: f64,
}

impl ViseDto {
    #[must_use]
    pub fn from_params(p: &ViseParams) -> Self {
        Self {
            size: p.size.clone(),
            jaw_width: p.jaw_width,
            jaw_height: p.jaw_height,
            jaw_thickness: p.jaw_thickness,
            max_opening: p.max_opening,
            length: p.length,
            width: p.width,
            base_height: p.base_height,
            housing_length: p.housing_length,
            housing_height: p.housing_height,
            opening: p.opening,
        }
    }

    /// The parameters, when they keep the vise law; a file whose vise
    /// key was hand-edited into nonsense keeps its facets as plain steel
    /// (logged), never a vise that would regenerate wrongly.
    #[must_use]
    pub fn to_params(&self, model: &str) -> Option<ViseParams> {
        let p = ViseParams {
            size: self.size.clone(),
            jaw_width: self.jaw_width,
            jaw_height: self.jaw_height,
            jaw_thickness: self.jaw_thickness,
            max_opening: self.max_opening,
            length: self.length,
            width: self.width,
            base_height: self.base_height,
            housing_length: self.housing_length,
            housing_height: self.housing_height,
            opening: self.opening,
        };
        match p.check() {
            Ok(()) => Some(p),
            Err(why) => {
                tracing::warn!(model, "vise parameters dropped on load, kept as plain steel: {why}");
                None
            }
        }
    }
}
