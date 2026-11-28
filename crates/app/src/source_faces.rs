//! The source faces of a model (#3915): for a model imported from a
//! Solidcraft design, SolidCraft's name of each face of the body and the
//! facets each face became (`geometry::source_faces`). `UpdateModel` reads
//! it to follow face picks by name (`commands::update_model_follow`).
//!
//! Provenance: our own ECS helper in the `mesh_solids` and `model_source`
//! pattern (#917, #2463). No HSMWorks material consulted.
//!
//! ## The law here
//!
//! - The project file saves it beside the facets (`source_faces`, no
//!   version step: `project/source_faces_dto.rs`), so a reopened project
//!   still follows picks by name at the next re-import.
//! - Only the commands that land facets on a model write it (the import,
//!   `UpdateModel` with `with_source_faces`, the undo of a delete), after
//!   the facets and their revision are landed.
//! - It is read only at the revision and the facet count it was landed
//!   with. Any other landing (a repair, an update from a file that is not
//!   `.sc`, a rebuilt vise) makes it stale, and [`of`] then gives none: a
//!   record never speaks for other facets.
//! - A record that does not tile the facets is never stored: it is dropped
//!   with a logged note.

use std::sync::Arc;

use flecs_ecs::core::EntityViewGet;
use flecs_ecs::prelude::{Entity, World};

use crate::geometry::source_faces::SourceFaces as Record;
use crate::model;
use crate::schema::SourceFaces;

/// Land or clear the source faces of `entity`'s facets. Call it after
/// `model::set_facets` and any `model::set_rev`: the record keeps the
/// revision and the facet count that stand now. A record that does not
/// tile the facets is dropped with a logged note; `None` clears it.
pub fn set(world: &World, entity: &Entity, record: Option<&Arc<Record>>) {
    let view = entity.entity_view(world);
    let len = model::facets_of(world, entity).map_or(0, |f| f.len());
    match record {
        Some(r) if r.tiles(len) => {
            view.set(SourceFaces {
                rev: model::rev_of(world, entity),
                len,
                record: Arc::clone(r),
            });
        }
        Some(r) => {
            tracing::warn!(
                "model source faces: {} ranges do not tile its {len} facets; the face names are dropped",
                r.ranges.len()
            );
            view.remove(world.component::<SourceFaces>());
        }
        None => {
            view.remove(world.component::<SourceFaces>());
        }
    }
}

/// The source faces of `entity`'s facets as they stand now, or none: none
/// were landed, or other facets landed since (the revision or the facet
/// count moved).
#[must_use]
pub fn of(world: &World, entity: &Entity) -> Option<Arc<Record>> {
    let rev = model::rev_of(world, entity);
    let len = model::facets_of(world, entity).map_or(0, |f| f.len());
    entity
        .entity_view(world)
        .try_get::<&SourceFaces>(|s| {
            (s.rev == rev && s.len == len && s.record.tiles(len)).then(|| Arc::clone(&s.record))
        })
        .flatten()
}
