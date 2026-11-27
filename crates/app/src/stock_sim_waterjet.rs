//! Provenance: our own refusal to judge waterjet removal with a milling stock model (#3637).

use crate::{model, operation::Family, schema::Operation};

/// Waterjet and laser removal (#4081) have no validated stock model or
/// sheet laws yet.
#[must_use]
pub fn skips(op: &Operation) -> bool {
    matches!(op.params.kind().info().family(), Family::Waterjet | Family::Laser)
}

pub(crate) fn refuse_waterjet_cuts(world: &flecs_ecs::prelude::World) -> Result<(), String> {
    for (name, entity) in crate::posting::program_ops(world)? {
        let Some(op) = model::operation_of(world, &entity) else { continue };
        if skips(&op) {
            let w = if op.params.kind().info().family() == Family::Laser { "laser" } else { "waterjet" };
            return Err(format!("{name}: {w} stock removal is not judged: the stock sim has no {w} model"));
        }
    }
    Ok(())
}
