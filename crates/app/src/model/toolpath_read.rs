//! Per-frame readers of an operation's cached path (#2201): read the
//! stored [`Toolpath`] in place, so a display that asks each frame (the
//! overlay lines, the browser's warning count, the status badge) does
//! not clone every path of the program on every frame.
//! Provenance: our own frame-cost law (docs/decisions/2238-frame-cost-
//! rule.md); no vendor material consulted.

use flecs_ecs::core::EntityViewGet; // not re-exported in prelude (0.2.2)
use flecs_ecs::prelude::{Entity, World};

use crate::schema::Toolpath;

/// Run `f` on the cached path in place - [`super::toolpath_of`] without
/// the clone. `None` when the entity has no cached path.
pub fn with_toolpath<R>(world: &World, entity: &Entity, f: impl FnOnce(&Toolpath) -> R) -> Option<R> {
    entity.entity_view(world).try_get::<&Toolpath>(f)
}

/// The count of verify warnings on the cached path (0 when none is
/// cached).
#[must_use]
pub fn toolpath_warning_count(world: &World, entity: &Entity) -> usize {
    with_toolpath(world, entity, |t| t.warnings.len()).unwrap_or(0)
}

/// Is there a cached path with at least one move.
#[must_use]
pub fn toolpath_nonempty(world: &World, entity: &Entity) -> bool {
    with_toolpath(world, entity, |t| !t.path.is_empty()).unwrap_or(false)
}
