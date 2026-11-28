//! Sketch recipes (#3194): `CreateSketchRecipe` places a shape stated by
//! its dimensions as a profile on the top face of a setup's stock,
//! `EditSketchRecipe` changes those dimensions in place, and `AddQuickOp`
//! places a recipe profile and a 2D Contour on it as one undo step.
//!
//! Provenance: our own commands over `camcraft_geometry::sketch_recipe`;
//! the pattern is `InsertText`'s. Plain machining terms only; no vendor
//! material, label or dialog flow consulted.
//!
//! ## The laws these commands keep
//!
//! - **Everything is refused at construction.** A bad dimension (by
//!   name, never clamped), a placement that is not finite, an unknown
//!   setup, a setup without a stock, a name in use, a profile that has
//!   no recipe, and an operation its own gates refuse never reach the
//!   history.
//! - **A recipe profile is a profile.** It is the same `Sketch` entity a
//!   DXF drawing gives, so `MoveSketch`, `DeleteEntity`, the operations
//!   and the project file treat it as one.
//! - **An edit is a new outline.** `EditSketchRecipe` writes the
//!   recipe's new outline; the operations on the profile go stale through
//!   their stamps, which hash the outline.
//! - **`apply` and `invert` are pure writes**, keyed by name.

use std::sync::Arc;

use flecs_ecs::prelude::World;

use crate::commands::{BatchCommand, Command, CommandError, CreateOperation};
use crate::model;
use crate::operation::{OpParams, SrcRef};
use crate::schema::{LEGACY_WCS_NAMES, Location, STOCK_NAME, Sketch};
use crate::sketch;
use crate::sketch_recipe::SketchRecipe;

/// Where a recipe profile lies on its stock face, and its name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecipePlacement {
    /// The shape's centre on the stock's top face, mm from the stock origin.
    pub x: f64,
    pub y: f64,
    /// The turn about the face normal, degrees, counterclockwise seen
    /// from above.
    pub angle_deg: f64,
    /// The profile's name; `None` is the shape's name, made unique.
    pub name: Option<String>,
}

pub struct CreateSketchRecipe {
    name: String,
    at: Location,
    sketch: Sketch,
}

impl CreateSketchRecipe {
    /// Check `recipe` and `place` and compile the outline now, for `setup`
    /// (`None`: the active setup).
    pub fn new(world: &World, setup: Option<&str>, recipe: SketchRecipe, place: &RecipePlacement) -> Result<Self, CommandError> {
        let setup = match setup {
            Some(name) if !crate::setup::exists(world, name) => return Err(CommandError::NotFound(name.to_owned())),
            Some(name) => Some(name.to_owned()),
            None => crate::setup::active(world),
        };
        match sketch::stock_face(world, setup.as_deref()) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(CommandError::Invalid(format!(
                    "{} has no stock - make a stock first, then place the {} on it",
                    setup.as_deref().map_or_else(|| "the document".to_owned(), |s| format!("setup '{s}'")),
                    recipe.shape()
                )));
            }
            Err(why) => return Err(CommandError::Invalid(why.into())),
        }
        let RecipePlacement { x, y, angle_deg, .. } = *place;
        if !(x.is_finite() && y.is_finite() && angle_deg.is_finite()) {
            return Err(CommandError::Invalid(format!("the {} placement must be finite", recipe.shape())));
        }
        if x.abs() > sketch::MAX_OFFSET || y.abs() > sketch::MAX_OFFSET {
            return Err(CommandError::Invalid(format!(
                "the {} placement is more than {} mm from the stock origin",
                recipe.shape(),
                sketch::MAX_OFFSET
            )));
        }
        let data = recipe.sketch().map_err(|e| CommandError::Invalid(e.to_string()))?;
        let name = recipe_name(world, recipe, place.name.as_deref())?;
        // One turn is every turn: keep the angle in (-180, 180], as `MoveSketch` does.
        let angle = 180.0 - (180.0 - angle_deg).rem_euclid(360.0);
        let sketch = Sketch { setup, data: Arc::new(data), recipe: Some(recipe) };
        Ok(CreateSketchRecipe { name, at: sketch::on_face(x, y, angle), sketch })
    }

    /// The name the profile lands under.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The setup the profile joins (`None` in a document without setups).
    #[must_use]
    pub fn setup(&self) -> Option<&str> {
        self.sketch.setup.as_deref()
    }
}

/// A stated name must be free and must not be a setup-furniture name. The
/// shape's own name is made unique (`Rectangle`, `Rectangle (2)`).
fn recipe_name(world: &World, recipe: SketchRecipe, stated: Option<&str>) -> Result<String, CommandError> {
    let reserved = |n: &str| n.eq_ignore_ascii_case(STOCK_NAME) || LEGACY_WCS_NAMES.contains(&n) || crate::setup::is_furniture_shape(n);
    if let Some(name) = stated {
        if name.trim().is_empty() || reserved(name) {
            return Err(CommandError::Invalid(format!("'{name}' is not a usable name for a profile")));
        }
        if model::find_by_name(world, name).is_some() {
            return Err(CommandError::NameConflict(name.to_owned()));
        }
        return Ok(name.to_owned());
    }
    let shape = recipe.shape();
    let stem: String = shape[..1].to_uppercase() + &shape[1..];
    sketch::free_name(world, &stem).ok_or(CommandError::NameConflict(stem))
}

impl Command for CreateSketchRecipe {
    fn apply(&self, world: &World) {
        if model::find_by_name(world, &self.name).is_some() {
            return; // already there (an idempotent redo path)
        }
        let entity = model::spawn(world, &self.name);
        model::set_transform(world, &entity, self.at);
        entity.entity_view(world).set(self.sketch.clone());
    }

    fn invert(&self, world: &World) {
        // Name-keyed, and only ever a profile: finding nothing, or
        // something else under the name, must no-op, not delete it.
        if let Some((entity, _)) = sketch::find(world, &self.name) {
            model::remove(world, &entity);
        }
    }
}

pub struct EditSketchRecipe {
    name: String,
    prev: Sketch,
    next: Sketch,
}

impl EditSketchRecipe {
    /// Give the recipe profile `name` the dimensions of `recipe`. The
    /// shape may change too (a rectangle may become an obround).
    pub fn new(world: &World, name: &str, recipe: SketchRecipe) -> Result<Self, CommandError> {
        let entity = model::find_by_name(world, name).ok_or_else(|| CommandError::NotFound(name.to_owned()))?;
        let prev = sketch::sketch_of(world, &entity).ok_or_else(|| CommandError::Invalid(format!("'{name}' is not a profile")))?;
        if prev.recipe.is_none() {
            return Err(CommandError::Invalid(format!(
                "'{name}' was not made from dimensions (it is a drawing or a text), so it has no recipe to edit"
            )));
        }
        let data = recipe.sketch().map_err(|e| CommandError::Invalid(e.to_string()))?;
        let next = Sketch { setup: prev.setup.clone(), data: Arc::new(data), recipe: Some(recipe) };
        Ok(EditSketchRecipe { name: name.to_owned(), prev, next })
    }

    fn write(&self, world: &World, sketch: &Sketch) {
        if let Some((entity, _)) = sketch::find(world, &self.name) {
            entity.entity_view(world).set(sketch.clone());
        }
    }
}

impl Command for EditSketchRecipe {
    fn apply(&self, world: &World) {
        self.write(world, &self.next);
    }

    fn invert(&self, world: &World) {
        self.write(world, &self.prev);
    }
}

/// A recipe profile on the active setup's stock and a 2D Contour on it,
/// as one undo step.
pub struct AddQuickOp {
    profile: String,
    operation: String,
    batch: BatchCommand,
}

impl AddQuickOp {
    /// `params` must be a 2D Contour's. The operation goes through every
    /// gate `CreateOperation` keeps, run against the placed profile: the
    /// profile is placed, the operation is checked, and the profile is
    /// removed again before this returns. Nothing of it stays in the
    /// world or the history on a refusal.
    pub fn new(
        world: &World,
        recipe: SketchRecipe,
        place: &RecipePlacement,
        operation: &str,
        tool: &str,
        params: OpParams,
    ) -> Result<Self, CommandError> {
        if !matches!(params, OpParams::Contour { .. }) {
            return Err(CommandError::Invalid("a quick op makes a 2D Contour; give 2D Contour parameters".to_owned()));
        }
        if model::find_by_name(world, operation).is_some() {
            return Err(CommandError::NameConflict(operation.to_owned()));
        }
        let profile = CreateSketchRecipe::new(world, None, recipe, place)?;
        let name = profile.name().to_owned();
        profile.apply(world);
        let op = CreateOperation::new(world, operation, tool, SrcRef::Entity(name.clone()), params);
        profile.invert(world);
        let op = op?;
        let label = format!("Quick op: {} {name}", recipe.shape());
        let batch = BatchCommand::new(label, vec![Box::new(profile), Box::new(op)]);
        Ok(AddQuickOp { profile: name, operation: operation.to_owned(), batch })
    }

    /// The profile's name.
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// The operation's name.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
}

impl Command for AddQuickOp {
    fn apply(&self, world: &World) {
        self.batch.apply(world);
    }

    fn invert(&self, world: &World) {
        self.batch.invert(world);
    }
}
