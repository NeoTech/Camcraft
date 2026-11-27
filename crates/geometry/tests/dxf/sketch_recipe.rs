//! Provenance: our own expected values, computed by hand from plane geometry. Issue 3194.
//! Gate map: covers camcraft_geometry::sketch_recipe (in the dxf suite).

use camcraft_geometry::dxf::Primitive;
use camcraft_geometry::sketch_recipe::{LAYER, PolygonSize, RecipeError, SketchRecipe};
use std::f64::consts::PI;

fn closed_outline(recipe: SketchRecipe) -> (f64, (f64, f64, f64, f64)) {
    let data = recipe.sketch().unwrap_or_else(|e| panic!("{recipe:?}: {e}"));
    let chains: Vec<_> = data.chains().all().collect();
    assert_eq!(chains.len(), 1, "{recipe:?} gives one chain");
    assert!(chains[0].is_closed(), "{recipe:?} gives a closed chain");
    assert_eq!(chains[0].layer(), LAYER);
    let (lo, hi) = data.bounds().expect("bounds");
    (chains[0].length(), (lo.x, lo.y, hi.x, hi.y))
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn rectangle_is_closed_centred_and_its_perimeter_is_exact() {
    let (len, (x0, y0, x1, y1)) = closed_outline(SketchRecipe::Rectangle { width: 100.0, height: 50.0, corner_radius: 0.0 });
    assert!(near(len, 300.0), "perimeter {len}");
    assert!(near(x0, -50.0) && near(y0, -25.0) && near(x1, 50.0) && near(y1, 25.0));
}

#[test]
fn rounded_rectangle_uses_exact_arcs() {
    let recipe = SketchRecipe::Rectangle { width: 100.0, height: 50.0, corner_radius: 5.0 };
    let prims = recipe.primitives().expect("rounded rectangle");
    assert_eq!(prims.iter().filter(|p| matches!(p.primitive, Primitive::Arc { .. })).count(), 4);
    assert_eq!(prims.iter().filter(|p| matches!(p.primitive, Primitive::Line { .. })).count(), 4);
    let (len, (x0, _, x1, y1)) = closed_outline(recipe);
    assert!(near(len, 2.0 * (90.0 + 40.0) + 2.0 * PI * 5.0), "perimeter {len}");
    assert!(near(x0, -50.0) && near(x1, 50.0) && near(y1, 25.0));
}

#[test]
fn rectangle_with_full_round_ends_drops_the_empty_edges() {
    let recipe = SketchRecipe::Rectangle { width: 60.0, height: 20.0, corner_radius: 10.0 };
    let prims = recipe.primitives().expect("full round ends");
    assert_eq!(prims.iter().filter(|p| matches!(p.primitive, Primitive::Line { .. })).count(), 2);
    let (len, _) = closed_outline(recipe);
    assert!(near(len, 2.0 * 40.0 + 2.0 * PI * 10.0), "perimeter {len}");
}

#[test]
fn circle_is_one_exact_circle() {
    let recipe = SketchRecipe::Circle { diameter: 30.0 };
    let prims = recipe.primitives().expect("circle");
    assert!(matches!(prims[..], [ref p] if matches!(p.primitive, Primitive::Circle { radius, .. } if radius == 15.0)));
    let (len, (x0, _, x1, _)) = closed_outline(recipe);
    assert!(near(len, PI * 30.0) && near(x0, -15.0) && near(x1, 15.0));
}

#[test]
fn obround_has_two_lines_and_two_half_arcs() {
    let recipe = SketchRecipe::Obround { length: 50.0, width: 10.0 };
    let (len, (x0, y0, x1, y1)) = closed_outline(recipe);
    assert!(near(len, 2.0 * 40.0 + PI * 10.0), "perimeter {len}");
    assert!(near(x0, -25.0) && near(y0, -5.0) && near(x1, 25.0) && near(y1, 5.0));
}

#[test]
fn hexagon_across_flats_puts_a_flat_at_the_bottom() {
    let recipe = SketchRecipe::Polygon { sides: 6, diameter: 17.0, size: PolygonSize::AcrossFlats };
    let (len, (_, y0, _, y1)) = closed_outline(recipe);
    let corner_radius = 17.0 / (2.0 * (PI / 6.0).cos());
    assert!(near(len, 6.0 * corner_radius), "perimeter {len}");
    assert!(near(y0, -8.5) && near(y1, 8.5), "flats at y = +-8.5: {y0} {y1}");
}

#[test]
fn polygon_across_corners_reaches_the_stated_circle() {
    let recipe = SketchRecipe::Polygon { sides: 4, diameter: 20.0, size: PolygonSize::AcrossCorners };
    let (len, (x0, _, x1, _)) = closed_outline(recipe);
    assert!(near(len, 4.0 * 20.0 * (PI / 4.0).sin()), "perimeter {len}");
    assert!(near(x1 - x0, 20.0 * (PI / 4.0).sin()), "a square on its flat");
}

#[test]
fn bad_dimensions_are_refused_by_name_and_never_clamped() {
    let cases: Vec<(SketchRecipe, &str)> = vec![
        (SketchRecipe::Circle { diameter: f64::NAN }, "diameter must be a finite number"),
        (SketchRecipe::Circle { diameter: 0.0 }, "diameter must be at least"),
        (SketchRecipe::Circle { diameter: -5.0 }, "diameter must be at least"),
        (SketchRecipe::Circle { diameter: 1.0e9 }, "must be at most"),
        (SketchRecipe::Rectangle { width: 10.0, height: f64::INFINITY, corner_radius: 0.0 }, "height must be a finite"),
        (SketchRecipe::Rectangle { width: 10.0, height: 4.0, corner_radius: 2.5 }, "larger than half the smaller side"),
        (SketchRecipe::Rectangle { width: 10.0, height: 4.0, corner_radius: -1.0 }, "must not be negative"),
        (SketchRecipe::Rectangle { width: 10.0, height: 4.0, corner_radius: 1.99 }, "shorter than"),
        (SketchRecipe::Obround { length: 5.0, width: 10.0 }, "larger than its length"),
        (SketchRecipe::Polygon { sides: 2, diameter: 10.0, size: PolygonSize::AcrossFlats }, "3 to 1000 sides, not 2"),
        (SketchRecipe::Polygon { sides: 1000, diameter: 1.0, size: PolygonSize::AcrossCorners }, "shorter than"),
    ];
    for (recipe, words) in cases {
        let err = recipe.sketch().expect_err("refused");
        assert!(err.to_string().contains(words), "{recipe:?}: '{err}' should say '{words}'");
        assert!(!matches!(err, RecipeError::Sketch(_)), "{recipe:?} is refused before chaining");
    }
}

#[test]
fn the_same_recipe_gives_the_same_outline_bit_for_bit() {
    let recipe = SketchRecipe::Polygon { sides: 7, diameter: 33.3, size: PolygonSize::AcrossFlats };
    let a = recipe.sketch().expect("polygon");
    let b = recipe.sketch().expect("polygon");
    assert_eq!(a.primitive_signature(), b.primitive_signature());
}
