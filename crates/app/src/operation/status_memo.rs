//! The status the tree, the status bar and the operation page SHOW
//! (#2201): [`status_of`] memoized on the world's write revision
//! (`world_rev`), so a still frame computes no stamp. Our own frame-cost
//! law (docs/decisions/2201-status-cache.md); no vendor material
//! consulted.
//!
//! ## The laws here
//!
//! - **Display only.** Generation, verify, the collision checks and the
//!   post call [`status_of`] - the uncached truth - and never this.
//! - **The key is the whole world.** Each status is computed from world
//!   components only, and each add, set or remove of a component moves
//!   the revision, so a hit answers what a fresh [`status_of`] answers.
//! - **A moving world is not cached.** When the revision moves during
//!   the compute, the answer is returned but not kept.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use flecs_ecs::prelude::{Entity, World};

use super::{OpStatus, stale_reason, status_of};

/// One revision and the answers given at it, per entity.
#[derive(Default)]
struct Memo {
    rev: u64,
    status: HashMap<Entity, OpStatus>,
    reason: HashMap<Entity, Option<String>>,
}

thread_local! {
    static MEMO: RefCell<Memo> = RefCell::new(Memo::default());
    /// Real [`status_of`] computes made for the display on this thread -
    /// the counter the #2201 pins read.
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// Look up `key` at revision `rev`; a new revision forgets all answers.
fn hit<T: Clone>(rev: u64, pick: impl Fn(&Memo) -> Option<T>) -> Option<T> {
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        if m.rev != rev {
            *m = Memo { rev, ..Memo::default() };
        }
        pick(&m)
    })
}

fn keep(rev: u64, put: impl FnOnce(&mut Memo)) {
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        if m.rev == rev {
            put(&mut m);
        }
    });
}

/// [`status_of`] for the DISPLAY: the answer of the last compute while
/// the world's revision has not moved. Never call it from a verdict.
#[must_use]
pub fn status_shown(world: &World, entity: &Entity) -> OpStatus {
    let id = *entity;
    let Some(rev) = crate::world_rev::of(world) else {
        BUILDS.with(|n| n.set(n.get().saturating_add(1)));
        return status_of(world, entity);
    };
    if let Some(s) = hit(rev, |m| m.status.get(&id).copied()) {
        return s;
    }
    BUILDS.with(|n| n.set(n.get().saturating_add(1)));
    let s = status_of(world, entity);
    if crate::world_rev::of(world) == Some(rev) {
        keep(rev, |m| {
            m.status.insert(id, s);
        });
    }
    s
}

/// [`stale_reason`] for the DISPLAY, on the same key as
/// [`status_shown`]. Never call it from a verdict.
#[must_use]
pub fn stale_reason_shown(world: &World, entity: &Entity) -> Option<String> {
    let id = *entity;
    let Some(rev) = crate::world_rev::of(world) else {
        return stale_reason(world, entity);
    };
    if status_shown(world, entity) != OpStatus::Stale {
        return None;
    }
    if let Some(r) = hit(rev, |m| m.reason.get(&id).cloned()) {
        return r;
    }
    let r = stale_reason(world, entity);
    if crate::world_rev::of(world) == Some(rev) {
        keep(rev, |m| {
            m.reason.insert(id, r.clone());
        });
    }
    r
}

/// How many real status computes [`status_shown`] made on this thread.
#[must_use]
pub fn status_builds() -> u64 {
    BUILDS.with(Cell::get)
}
