//! The world box of the document's solids, made once per input (#2201):
//! `model::solid_world_aabb` transforms every vertex of every model, and
//! the viewport asked it several times on every frame (the gizmo length
//! behind the triad, the axis labels and the pick lines, the chevron
//! arm of the toolpath overlay, the height-plane reach). The box is a
//! pure function of the key below; a still frame reads it.
//!
//! The key holds every input of `solid_world_aabb`, for each model in
//! the order the world lists them:
//! - the model entity;
//! - its facet list by allocation: a `Weak` of the `Arc<[Triangle]>` that
//!   a geometry write replaces (the `Weak` keeps the address from reuse);
//! - its pose (`Location`), by value.
//!
//! Display only: the verify check, the collision checks and the post
//! gate never read this memo; they call `model::solid_world_aabb`.
//! Provenance: our own frame-cost law (docs/decisions/2238-frame-cost-
//! rule.md); no vendor material consulted.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

use camcraft_app::geometry::{Aabb, Triangle};
use camcraft_app::model;
use camcraft_app::schema::Location;
use flecs_ecs::prelude::{Entity, World};

/// One model's inputs.
struct Part {
    entity: Entity,
    facets: Option<Weak<[Triangle]>>,
    pose: Option<Location>,
}

impl PartialEq for Part {
    fn eq(&self, o: &Self) -> bool {
        let same_facets = match (&self.facets, &o.facets) {
            (None, None) => true,
            (Some(a), Some(b)) => a.ptr_eq(b),
            _ => false,
        };
        self.entity == o.entity && same_facets && self.pose == o.pose
    }
}

thread_local! {
    static MEMO: RefCell<Option<(Vec<Part>, Option<Aabb>)>> = const { RefCell::new(None) };
    /// Real `solid_world_aabb` builds made through [`solid_world_aabb`].
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

fn key(world: &World) -> Vec<Part> {
    model::model_entities(world)
        .into_iter()
        .map(|(_, entity)| Part {
            entity,
            facets: model::facets_of(world, &entity).map(|t| Arc::downgrade(&t)),
            pose: model::transform_of(world, &entity),
        })
        .collect()
}

/// [`model::solid_world_aabb`], built again only when a model, its facet
/// list or its pose changed since the last call.
#[must_use]
pub fn solid_world_aabb(world: &World) -> Option<Aabb> {
    let now = key(world);
    let hit = MEMO.with(|m| m.borrow().as_ref().and_then(|(k, b)| (*k == now).then_some(*b)));
    if let Some(b) = hit {
        return b;
    }
    BUILDS.with(|n| n.set(n.get().saturating_add(1)));
    let b = crate::frame_cost::time("scene/solid_aabb.build", || model::solid_world_aabb(world));
    MEMO.with(|m| *m.borrow_mut() = Some((now, b)));
    b
}

/// How many real box builds [`solid_world_aabb`] made on this thread.
#[must_use]
pub fn solid_aabb_builds() -> u64 {
    BUILDS.with(Cell::get)
}

#[cfg(test)]
mod tests {
    //! The rule's two pins (docs/decisions/2238-frame-cost-rule.md rule
    //! 5): still calls build once, and each input of the key builds
    //! again with the answer of a fresh `model::solid_world_aabb`.
    use camcraft_app::geometry::Point3;

    use super::*;

    fn tri(s: f64) -> Arc<[Triangle]> {
        let p = |x, y, z| Point3::new(x * s, y * s, z * s);
        Arc::from(vec![Triangle::new(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0), p(0.0, 1.0, 1.0))])
    }

    /// One build since `*n`, equal to the fresh box.
    fn rebuilt(w: &World, n: &mut u64, what: &str) {
        let b = solid_world_aabb(w);
        assert_eq!(solid_aabb_builds() - *n, 1, "{what}: one new build");
        assert_eq!(b, model::solid_world_aabb(w), "{what}: the fresh box");
        *n = solid_aabb_builds();
    }

    #[test]
    fn still_calls_build_the_box_once() {
        let w = camcraft_app::world::new_world();
        let a = model::spawn(&w, "a");
        model::set_facets(&w, &a, tri(10.0));
        let before = solid_aabb_builds();
        let first = solid_world_aabb(&w);
        for _ in 0..5 {
            assert_eq!(solid_world_aabb(&w), first);
        }
        assert_eq!(solid_aabb_builds() - before, 1, "six calls, one build");
        assert_eq!(first, model::solid_world_aabb(&w));
        assert!(first.is_some());
    }

    #[test]
    fn each_input_of_the_key_builds_again() {
        let w = camcraft_app::world::new_world();
        let a = model::spawn(&w, "a");
        model::set_facets(&w, &a, tri(10.0));
        let _ = solid_world_aabb(&w);
        let mut n = solid_aabb_builds();

        // The pose: a move, then a turn.
        model::set_location(&w, &a, Point3::new(5.0, 0.0, 0.0));
        rebuilt(&w, &mut n, "location");
        let mut loc = model::transform_of(&w, &a).unwrap_or_default();
        loc.rot = [0.0, 0.0, std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2];
        model::set_transform(&w, &a, loc);
        rebuilt(&w, &mut n, "rotation");

        // The facets: a new list of the same triangles, then other ones.
        model::set_facets(&w, &a, tri(10.0));
        rebuilt(&w, &mut n, "same facets, new list");
        model::set_facets(&w, &a, tri(30.0));
        rebuilt(&w, &mut n, "other facets");

        // The model set: one more part, then that part gone.
        let b = model::spawn(&w, "b");
        model::set_facets(&w, &b, tri(-50.0));
        rebuilt(&w, &mut n, "a second part");
        model::remove(&w, &b);
        rebuilt(&w, &mut n, "the second part removed");

        // Still again: no build.
        let _ = solid_world_aabb(&w);
        assert_eq!(solid_aabb_builds(), n, "a still call after the changes builds nothing");
    }
}
