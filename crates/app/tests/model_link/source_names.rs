//! Shared fixtures of the #3915 pins: the area suite's block at two widths,
//! with source face records of our own, and operations that hold picks on
//! it. The record lands through the import command
//! (`ImportModel::source_faces`), as a `.sc` import lands it; the `.sc`
//! reader has its own pins.
//! Provenance: our own fixtures; no vendor material consulted.

#![allow(dead_code)]

use std::sync::Arc;

use flecs_ecs::prelude::World;

use camcraft_app::commands::update_model_follow::FollowReport;
use camcraft_app::commands::{History, ImportModel, UpdateModel};
use camcraft_app::geometry::source_faces::{FaceName, FacetRange, SourceFaces};
use camcraft_app::geometry::{Mesh, Point3, cube};
use camcraft_app::operation::{FeatureSel, GeoSel, SrcRef};
use camcraft_app::schema::Operation;
use camcraft_app::{model, source_faces};

pub const PART: &str = "part";
/// The six faces of `cube()`, in its facet order (two facets each).
pub const CUBE: [&str; 6] = ["bottom", "top", "front", "back", "right", "left"];
pub const TOP: usize = 1;
pub const FRONT: usize = 2;
pub const BACK: usize = 3;
pub const RIGHT: usize = 4;
pub const LEFT: usize = 5;

/// The area suite's block (`block_world` is `block(40.0)`): 40 deep and
/// 10 high, `width` along +x from x = -20. A wider block keeps the x = -20
/// face (carried) and moves the other five.
pub fn block(width: f64) -> Mesh {
    cube(
        Point3::new(-20.0, -20.0, 0.0),
        Point3::new(-20.0 + width, 20.0, 10.0),
    )
}

pub fn name(n: &str) -> FaceName {
    FaceName {
        name: n.into(),
        verified: true,
        former: None,
    }
}

pub fn names(list: &[&str]) -> Vec<FaceName> {
    list.iter().map(|n| name(n)).collect()
}

/// A record over a block: cube face `k` is record face `slot[k]`.
pub fn record(mesh: &Mesh, names: Vec<FaceName>, slot: [usize; 6]) -> Arc<SourceFaces> {
    let mut ranges: Vec<FacetRange> = Vec::new();
    for t in 0..mesh.facets.len() {
        let face = u32::try_from(slot[t / 2]).expect("a small index");
        match ranges.last_mut() {
            Some(r) if r.face == face && r.end == t => r.end = t + 1,
            _ => ranges.push(FacetRange {
                start: t,
                end: t + 1,
                face,
            }),
        }
    }
    Arc::new(SourceFaces { names, ranges })
}

/// The names in [`CUBE`] order.
pub fn plain(mesh: &Mesh) -> Arc<SourceFaces> {
    record(mesh, names(&CUBE), [0, 1, 2, 3, 4, 5])
}

/// The same names in the reversed record order, with cube face `k`'s
/// name replaced by `with` when given.
pub fn reversed(mesh: &Mesh, with: Option<(usize, FaceName)>) -> Arc<SourceFaces> {
    let mut list = CUBE;
    list.reverse();
    let mut n = names(&list);
    if let Some((k, f)) = with {
        n[5 - k] = f;
    }
    record(mesh, n, [5, 4, 3, 2, 1, 0])
}

/// The pick of cube face `k` of the live part.
pub fn face(world: &World, k: usize) -> FeatureSel {
    let e = model::find_by_name(world, PART).expect("the part");
    let f = model::features_of(world, &e).expect("features");
    FeatureSel::Face(f.faces[f.facet_face[2 * k] as usize].id)
}

pub fn on_part(f: FeatureSel) -> GeoSel {
    GeoSel {
        entity: PART.into(),
        feature: f,
    }
}

/// The area suite's block world (`area_support::block_world`), with the
/// part imported carrying `rec`, the way a `.sc` import lands its record
/// (`ImportModel::source_faces`).
pub fn part_world(rec: Option<Arc<SourceFaces>>) -> World {
    let mut import = ImportModel::from_mesh(PART, block(40.0));
    let named = rec.is_some();
    import.source_faces = rec;
    let world = super::area_support::world_with(Some(import), 0.0, 10.0);
    let e = model::find_by_name(&world, PART).expect("the part");
    assert_eq!(
        source_faces::of(&world, &e).is_some(),
        named,
        "the record tiles the part"
    );
    world
}

/// An operation entity landed as it is: the follow reads picks wherever
/// they are stored, so these pins skip the pick-time gates.
pub fn land_op(world: &World, key: &str, op: Operation) {
    let e = model::spawn(world, key);
    e.entity_view(world).set(op);
}

/// Import the part again from `mesh` with `rec`, as `.sc` re-import does;
/// the report and the window's phrase, read before the command runs.
pub fn reimport(
    world: &World,
    h: &mut History,
    mesh: Mesh,
    rec: Option<Arc<SourceFaces>>,
) -> (FollowReport, String) {
    let upd = UpdateModel::new(world, PART, mesh)
        .expect("an update")
        .with_source_faces(rec);
    let out = (upd.follow_report().clone(), upd.summary());
    h.execute(world, upd);
    out
}

pub fn op_of(world: &World, key: &str) -> Operation {
    let e = model::find_by_name(world, key).expect("the operation");
    model::operation_of(world, &e).expect("an operation")
}

pub fn src_of(world: &World, key: &str) -> Vec<GeoSel> {
    match op_of(world, key).src {
        SrcRef::Selection(items) => items,
        _ => Vec::new(),
    }
}

pub fn review_of(world: &World, key: &str) -> Vec<String> {
    let e = model::find_by_name(world, key).expect("the operation");
    model::operation_meta_of(world, &e).needs_review
}
