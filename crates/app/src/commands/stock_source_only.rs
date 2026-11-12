//! SetStockSourceOnly (#1094): say whether the active setup's from-model
//! stock's source model is raw stock only (a casting or blank you cut
//! into) - then the program verdict does not judge it as the finished
//! part. Off is the default and every older document's state.
//!
//! Provenance: our own design (a CAM choice is a user parameter with a
//! safe default); the command shape follows `SetStock` - validate and
//! snapshot at construction, pure world writes in apply/invert.

use flecs_ecs::prelude::*;

use crate::commands::{Command, CommandError};
use crate::model;
use crate::schema::StockSourceOnly;
use crate::stock::StockShape;

pub struct SetStockSourceOnly {
    /// The stock entity, resolved from the active setup at construction
    /// (undo replays against this name, as `SetStock` does).
    stock_name: String,
    next: Option<StockSourceOnly>,
    prev: Option<StockSourceOnly>,
}

impl SetStockSourceOnly {
    /// `on` marks the current source model raw stock only; off clears the
    /// mark. Turning it on needs a from-model stock - any other stock has
    /// no source model, and the refusal says so.
    pub fn new(world: &World, on: bool) -> Result<Self, CommandError> {
        let active = crate::setup::active(world);
        let stock_name = crate::setup::stock_entity_name(world, active.as_deref());
        let entity = model::find_by_name(world, &stock_name)
            .filter(|e| model::is_stock(world, e))
            .ok_or_else(|| CommandError::Invalid(format!("no stock '{stock_name}' - set the stock first")))?;
        let prev = entity
            .entity_view(world)
            .try_get::<&StockSourceOnly>(|f| f.clone());
        let next = if on {
            match model::stock_def_of(world, &entity) {
                Some(StockShape::FromModel { source }) => Some(StockSourceOnly { source }),
                _ => {
                    return Err(CommandError::Invalid(format!(
                        "'{stock_name}' is not made from a model - only a from-model stock's source can be raw stock only"
                    )));
                }
            }
        } else {
            None
        };
        Ok(SetStockSourceOnly { stock_name, next, prev })
    }
}

fn land(world: &World, stock_name: &str, value: &Option<StockSourceOnly>) {
    let Some(entity) = model::find_by_name(world, stock_name) else {
        return; // the stock is gone (idempotent replay)
    };
    let view = entity.entity_view(world);
    match value {
        Some(f) => {
            view.set(f.clone());
        }
        None => {
            view.remove(world.component::<StockSourceOnly>());
        }
    }
}

impl Command for SetStockSourceOnly {
    fn apply(&self, world: &World) {
        land(world, &self.stock_name, &self.next);
    }

    fn invert(&self, world: &World) {
        land(world, &self.stock_name, &self.prev);
    }
}
