//! Sketch recipes (#3194): a rectangle, a circle, an obround or a regular polygon stated by its
//! dimensions, compiled to exact lines, arcs and circles in a [`SketchData`].
//!
//! Provenance: our own design for issue 3194. The shapes are plane geometry from machine-tool
//! handbooks (the across-flats and across-corners sizes of a regular polygon). No vendor
//! material, label or dialog was consulted.
//!
//! Contract. A dimension that is not finite, not positive, too small or too large is refused by
//! name: nothing is clamped. The outline is centred on the profile origin, closed and
//! counterclockwise, on the layer [`LAYER`]. The same recipe always gives the same primitives,
//! bit for bit, so a stored recipe can be checked against its stored outline.

use crate::dxf::{ImportSummary, LayeredPrimitive, Primitive, Unit, Units};
use crate::geo2d::Point2;
use crate::sketch_data::{SketchData, SketchError, SourceUnits};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// The layer of every recipe outline.
pub const LAYER: &str = "outline";
/// The smallest dimension and the shortest straight edge of a recipe, mm.
pub const MIN_SIZE_MM: f64 = 0.1;
/// The largest dimension of a recipe, mm.
pub const MAX_SIZE_MM: f64 = 100_000.0;
/// The most sides of a regular polygon.
pub const MAX_SIDES: u32 = 1000;

/// Which size a polygon's `diameter` states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonSize {
    /// The distance between two opposite flats (the wrench size of a hexagon). An odd polygon
    /// reads it as twice the distance from the centre to a flat.
    AcrossFlats,
    /// The diameter of the circle through the corners.
    AcrossCorners,
}

/// A shape stated by its dimensions, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SketchRecipe {
    /// `width` along X, `height` along Y; `corner_radius` 0 gives sharp corners.
    Rectangle { width: f64, height: f64, corner_radius: f64 },
    Circle { diameter: f64 },
    /// A slot shape: `length` overall along X, `width` across, with round ends.
    Obround { length: f64, width: f64 },
    /// A regular polygon with one flat at the bottom, parallel to X.
    Polygon { sides: u32, diameter: f64, size: PolygonSize },
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum RecipeError {
    #[error("the {shape} {field} must be a finite number")]
    NotFinite { shape: &'static str, field: &'static str },
    #[error("the {shape} {field} must be at least {MIN_SIZE_MM} mm, not {value} mm")]
    TooSmall { shape: &'static str, field: &'static str, value: f64 },
    #[error("the {shape} {field} must be at most {MAX_SIZE_MM} mm, not {value} mm")]
    TooLarge { shape: &'static str, field: &'static str, value: f64 },
    #[error("the rectangle corner radius must not be negative, not {value} mm")]
    NegativeRadius { value: f64 },
    #[error(
        "the rectangle corner radius {radius} mm is larger than half the smaller side ({limit} mm)"
    )]
    CornerRadius { radius: f64, limit: f64 },
    #[error("the obround width {width} mm is larger than its length {length} mm")]
    ObroundWidth { length: f64, width: f64 },
    #[error("a polygon needs 3 to {MAX_SIDES} sides, not {sides}")]
    Sides { sides: u32 },
    #[error(
        "the {shape} has a straight edge of {length} mm, shorter than {MIN_SIZE_MM} mm: \
         make the shape larger, or make its round ends meet exactly"
    )]
    ShortEdge { shape: &'static str, length: f64 },
    #[error(transparent)]
    Sketch(#[from] SketchError),
}

impl SketchRecipe {
    /// The shape's plain name.
    #[must_use]
    pub fn shape(&self) -> &'static str {
        match self {
            SketchRecipe::Rectangle { .. } => "rectangle",
            SketchRecipe::Circle { .. } => "circle",
            SketchRecipe::Obround { .. } => "obround",
            SketchRecipe::Polygon { .. } => "polygon",
        }
    }

    /// Check every dimension; refuse the first bad one by name.
    pub fn validate(&self) -> Result<(), RecipeError> {
        let shape = self.shape();
        match *self {
            SketchRecipe::Rectangle { width, height, corner_radius } => {
                size(shape, "width", width)?;
                size(shape, "height", height)?;
                if !corner_radius.is_finite() {
                    return Err(RecipeError::NotFinite { shape, field: "corner radius" });
                }
                if corner_radius < 0.0 {
                    return Err(RecipeError::NegativeRadius { value: corner_radius });
                }
                let limit = width.min(height) / 2.0;
                if corner_radius > limit {
                    return Err(RecipeError::CornerRadius { radius: corner_radius, limit });
                }
                short_edge(shape, width - 2.0 * corner_radius)?;
                short_edge(shape, height - 2.0 * corner_radius)
            }
            SketchRecipe::Circle { diameter } => size(shape, "diameter", diameter),
            SketchRecipe::Obround { length, width } => {
                size(shape, "length", length)?;
                size(shape, "width", width)?;
                if width > length {
                    return Err(RecipeError::ObroundWidth { length, width });
                }
                short_edge(shape, length - width)
            }
            SketchRecipe::Polygon { sides, diameter, size: which } => {
                if !(3..=MAX_SIDES).contains(&sides) {
                    return Err(RecipeError::Sides { sides });
                }
                size(shape, "diameter", diameter)?;
                let radius = polygon_radius(sides, diameter, which);
                size(shape, "corner diameter", 2.0 * radius)?;
                short_edge(shape, 2.0 * radius * (PI / f64::from(sides)).sin())
            }
        }
    }

    /// The outline: closed, counterclockwise, centred on the origin, on [`LAYER`].
    pub fn primitives(&self) -> Result<Vec<LayeredPrimitive>, RecipeError> {
        self.validate()?;
        let p = Point2::new;
        let out = match *self {
            SketchRecipe::Rectangle { width, height, corner_radius: r } => {
                let (x, y) = (width / 2.0, height / 2.0);
                if r == 0.0 {
                    let c = [p(-x, -y), p(x, -y), p(x, y), p(-x, y)];
                    (0..4).map(|i| line(c[i], c[(i + 1) % 4])).collect()
                } else {
                    let (ix, iy) = (x - r, y - r);
                    let mut out = Vec::with_capacity(8);
                    // Each corner arc, with the edge that leads into it.
                    let corners = [
                        (p(ix, -iy), -FRAC_PI_2, p(-ix, -y), p(ix, -y)),
                        (p(ix, iy), 0.0, p(x, -iy), p(x, iy)),
                        (p(-ix, iy), FRAC_PI_2, p(ix, y), p(-ix, y)),
                        (p(-ix, -iy), PI, p(-x, iy), p(-x, -iy)),
                    ];
                    for (center, start, a, b) in corners {
                        if a != b {
                            out.push(line(a, b));
                        }
                        out.push(arc(center, r, start, FRAC_PI_2));
                    }
                    out
                }
            }
            SketchRecipe::Circle { diameter } => {
                vec![layered(Primitive::Circle { center: p(0.0, 0.0), radius: diameter / 2.0 })]
            }
            SketchRecipe::Obround { length, width } => {
                let (r, s) = (width / 2.0, (length - width) / 2.0);
                if s == 0.0 {
                    vec![layered(Primitive::Circle { center: p(0.0, 0.0), radius: r })]
                } else {
                    vec![
                        line(p(-s, -r), p(s, -r)),
                        arc(p(s, 0.0), r, -FRAC_PI_2, PI),
                        line(p(s, r), p(-s, r)),
                        arc(p(-s, 0.0), r, FRAC_PI_2, PI),
                    ]
                }
            }
            SketchRecipe::Polygon { sides, diameter, size: which } => {
                let radius = polygon_radius(sides, diameter, which);
                let n = f64::from(sides);
                let corner = |k: u32| {
                    let a = -FRAC_PI_2 - PI / n + TAU * f64::from(k) / n;
                    p(radius * a.cos(), radius * a.sin())
                };
                let corners: Vec<Point2> = (0..sides).map(corner).collect();
                let len = corners.len();
                (0..len).map(|i| line(corners[i], corners[(i + 1) % len])).collect()
            }
        };
        Ok(out)
    }

    /// The recipe as a sketch: its outline in millimetres, chained.
    pub fn sketch(&self) -> Result<SketchData, RecipeError> {
        let units = SourceUnits { stated: Units::Known(Unit::Millimeters), applied: Unit::Millimeters };
        let primitives = self.primitives()?;
        Ok(SketchData::from_parts(primitives, vec![LAYER.to_owned()], units, None, ImportSummary::default())?)
    }
}

/// The corner radius of a regular polygon.
fn polygon_radius(sides: u32, diameter: f64, which: PolygonSize) -> f64 {
    match which {
        PolygonSize::AcrossCorners => diameter / 2.0,
        PolygonSize::AcrossFlats => diameter / (2.0 * (PI / f64::from(sides)).cos()),
    }
}

fn size(shape: &'static str, field: &'static str, value: f64) -> Result<(), RecipeError> {
    if !value.is_finite() {
        Err(RecipeError::NotFinite { shape, field })
    } else if value < MIN_SIZE_MM {
        Err(RecipeError::TooSmall { shape, field, value })
    } else if value > MAX_SIZE_MM {
        Err(RecipeError::TooLarge { shape, field, value })
    } else {
        Ok(())
    }
}

/// A straight edge is absent (0) or at least [`MIN_SIZE_MM`] long.
fn short_edge(shape: &'static str, length: f64) -> Result<(), RecipeError> {
    if length > 0.0 && length < MIN_SIZE_MM {
        Err(RecipeError::ShortEdge { shape, length })
    } else {
        Ok(())
    }
}

fn layered(primitive: Primitive) -> LayeredPrimitive {
    LayeredPrimitive { layer: LAYER.to_owned(), primitive }
}

fn line(start: Point2, end: Point2) -> LayeredPrimitive {
    layered(Primitive::Line { start, end })
}

fn arc(center: Point2, radius: f64, start_angle: f64, sweep_angle: f64) -> LayeredPrimitive {
    layered(Primitive::Arc { center, radius, start_angle, sweep_angle })
}
