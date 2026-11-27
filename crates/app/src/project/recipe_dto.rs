//! The file and MCP mirror of a sketch recipe (#3194, `sketch_recipe::SketchRecipe`).
//! Provenance: our own project format.
//!
//! The object is tagged by `shape`: `rectangle` (`width`, `height`, `corner_radius`),
//! `circle` (`diameter`), `obround` (`length`, `width`) or `polygon` (`sides`, `diameter`,
//! `across`: `flats` or `corners`). All sizes are millimetres. An unknown key is refused,
//! so a misspelt dimension never falls back to a default.

use serde::{Deserialize, Serialize};

use crate::sketch_recipe::{PolygonSize, SketchRecipe};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecipeDto {
    Rectangle {
        width: f64,
        height: f64,
        #[serde(default)]
        corner_radius: f64,
    },
    Circle {
        diameter: f64,
    },
    Obround {
        length: f64,
        width: f64,
    },
    Polygon {
        sides: u32,
        diameter: f64,
        across: Across,
    },
}

/// Which size a polygon's `diameter` states.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Across {
    Flats,
    Corners,
}

impl From<RecipeDto> for SketchRecipe {
    fn from(dto: RecipeDto) -> Self {
        match dto {
            RecipeDto::Rectangle { width, height, corner_radius } => SketchRecipe::Rectangle { width, height, corner_radius },
            RecipeDto::Circle { diameter } => SketchRecipe::Circle { diameter },
            RecipeDto::Obround { length, width } => SketchRecipe::Obround { length, width },
            RecipeDto::Polygon { sides, diameter, across } => SketchRecipe::Polygon {
                sides,
                diameter,
                size: match across {
                    Across::Flats => PolygonSize::AcrossFlats,
                    Across::Corners => PolygonSize::AcrossCorners,
                },
            },
        }
    }
}

impl From<SketchRecipe> for RecipeDto {
    fn from(recipe: SketchRecipe) -> Self {
        match recipe {
            SketchRecipe::Rectangle { width, height, corner_radius } => RecipeDto::Rectangle { width, height, corner_radius },
            SketchRecipe::Circle { diameter } => RecipeDto::Circle { diameter },
            SketchRecipe::Obround { length, width } => RecipeDto::Obround { length, width },
            SketchRecipe::Polygon { sides, diameter, size } => RecipeDto::Polygon {
                sides,
                diameter,
                across: match size {
                    PolygonSize::AcrossFlats => Across::Flats,
                    PolygonSize::AcrossCorners => Across::Corners,
                },
            },
        }
    }
}
