//! The world's write revision (#2201): one number that moves on each
//! component add, set or remove in a world, so a display cache can key
//! on "nothing in the document changed" without a list of writers that
//! can go out of date. Provenance: our own frame-cost law
//! (docs/decisions/2201-status-cache.md) over the flecs observer API
//! (flecs_ecs 0.2.2, MIT); no vendor material consulted.
//!
//! ## The laws here
//!
//! - **Every write moves it.** A flecs observer on the wildcard id sees
//!   each `OnAdd`, `OnSet` and `OnRemove` of each component of each
//!   entity: a command, its undo and redo, a generation result, a project
//!   load and an entity delete all land through these events. A write
//!   in place (`get::<&mut T>`) sends no event, so no code may write a
//!   component that way (`world_rev_tests::no_write_in_place`).
//! - **Unique across worlds.** Each move takes a fresh number from one
//!   process counter, so two worlds never show the same revision, and a
//!   replaced document never matches a key of the old one.
//! - **Memo writes do not count.** `GeoSig` and `GeomPrintMemo` are
//!   derived memos that a read writes; they are not inputs of anything.
//! - **Display only.** No verdict, verify, collision check or post reads
//!   it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use flecs_ecs::prelude::*;

use crate::schema::{GeoSig, GeomPrintMemo, WorldRevision};

/// The process counter: each revision move takes the next number.
static NEXT: AtomicU64 = AtomicU64::new(1);

fn fresh() -> u64 {
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Give a new world its revision and the observer that moves it. Called
/// once from `schema::register_schema`.
pub(crate) fn install(world: &World) {
    let rev = Arc::new(AtomicU64::new(fresh()));
    world.set(WorldRevision(Arc::clone(&rev)));
    // Ids of the memo components and of the revision itself: their
    // writes say nothing about the document.
    let skip: [u64; 3] = [
        *world.component::<GeoSig>().id(),
        *world.component::<GeomPrintMemo>().id(),
        *world.component::<WorldRevision>().id(),
    ];
    // Plain components, then pairs: no document state rides a pair
    // today, but a relation added later moves the revision too.
    let plain = Arc::clone(&rev);
    world
        .observer::<flecs::OnSet, ()>()
        .with(flecs::Wildcard::ID)
        .set_inout_none()
        .add_event(flecs::OnAdd::ID)
        .add_event(flecs::OnRemove::ID)
        .each_iter(move |it, _, ()| {
            let id = *it.event_id().id();
            if !skip.contains(&id) {
                plain.store(fresh(), Ordering::Relaxed);
            }
        });
    world
        .observer::<flecs::OnSet, ()>()
        .with((flecs::Wildcard::ID, flecs::Wildcard::ID))
        .set_inout_none()
        .add_event(flecs::OnAdd::ID)
        .add_event(flecs::OnRemove::ID)
        .each_iter(move |_, _, ()| {
            rev.store(fresh(), Ordering::Relaxed);
        });
}

/// The world's write revision now. `None` for a world that was not made
/// by `world::new_world` - a caller then computes fresh each time.
#[must_use]
pub fn of(world: &World) -> Option<u64> {
    world.try_get::<&WorldRevision>(|r| r.0.load(Ordering::Relaxed))
}
